// main.rs
extern crate dotenv_codegen;

use std::{
    collections::HashSet,
    process::exit,
    sync::{Arc, Mutex},
};

use anyhow::Result;
use api::{comments::SteamCommentRequester, profiles::SteamProfileRequester};
use colored::Colorize;
use config::files::read_words_from_json;
use dotenv::dotenv;
use futures::future::join_all;
use rand::Rng;
use regex::Regex;
use tokio::task::JoinHandle;

mod api;
mod cli;
mod config;
mod core;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();

    println!("{}", "🚀 Welcome to the SteamPolice!".bold().cyan());

    if !config::files::create_default_json_files()? {
        exit(0);
    }

    let accounts = config::files::read_accounts_from_json()?;
    if accounts.is_empty() {
        eprintln!(
            "{}",
            format!(
                "❌  No accounts found in '{}'. Please add your account details.",
                "accounts.json"
            )
            .red()
            .bold()
        );
        return Ok(());
    }

    println!(
        "{}",
        "---------------- Account Information ----------------"
            .bold()
            .green()
    );

    println!(
        "{}",
        format!("✅  Loaded {} account(s):", accounts.len()).green()
    );

    for (i, account) in accounts.iter().enumerate() {
        println!(
            "{}",
            format!(
                "  {}) Name: {}, Session ID (partial): {}, Steam ID: {}",
                i + 1,
                account.name.bold().cyan(),
                &account.session_id[..10].green(),
                account
                    .steam_login_secure
                    .split("||")
                    .nth(0)
                    .unwrap()
                    .green()
            )
        );
    }

    println!(
        "{}",
        "-----------------------------------------------------"
            .bold()
            .green()
    );

    println!("{}", "🎯 What action do you want to perform?".blue().bold());
    println!("{}", "1. Report comments on profiles".blue());
    println!("{}", "2. Report profiles".blue());
    println!("{}", "3. Post comments on profiles".blue());

    let action_choice = cli::input::get_action_choice()?;

    let profiles_str = cli::input::get_target_profiles()?;

    let profiles_vec: Vec<String> = profiles_str
        .trim()
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    if profiles_vec.is_empty() {
        eprintln!("{}", "❌  No target profiles provided.".red().bold());
        return Ok(());
    }

    let mut target_profile_ids: Vec<u64> = Vec::new();
    for profile_str in profiles_vec {
        let trimmed_profile_str = profile_str.trim();
        if trimmed_profile_str.starts_with("search:") {
            let re = Regex::new(r"^search:(?P<term>[^!>]+)(!>(?P<limit>\d+))?$").unwrap();
            if let Some(captures) = re.captures(trimmed_profile_str) {
                let search_term = captures.name("term").unwrap().as_str();
                let max_pages = captures
                    .name("limit")
                    .map(|m| m.as_str().parse::<u32>().unwrap_or(0));

                if let Some(account) = accounts.first() {
                    match core::profile_reporter::handle_search_prefix(
                        account.clone(),
                        search_term,
                        max_pages,
                    )
                    .await
                    {
                        Ok(profile_links) => {
                            for link in profile_links {
                                if let Some(profile_id) = link
                                    .trim_start_matches("https://steamcommunity.com/profiles/")
                                    .parse::<u64>()
                                    .ok()
                                {
                                    target_profile_ids.push(profile_id);
                                }
                            }
                        }
                        Err(err) => {
                            eprintln!(
                                "{}",
                                format!("❌  Error during search for '{}': {}", search_term, err)
                                    .red()
                                    .bold()
                            );
                        }
                    }
                } else {
                    eprintln!(
                        "{}",
                        "❌  No accounts available to perform search.".red().bold()
                    );
                }
            } else {
                eprintln!(
                    "{}",
                    format!("❌  Invalid search format: {}", trimmed_profile_str)
                        .red()
                        .bold()
                );
            }
            continue;
        }

        let resolved_profile_id =
            if trimmed_profile_str.starts_with("https://steamcommunity.com/id/") {
                trimmed_profile_str
                    .trim_start_matches("https://steamcommunity.com/id/")
                    .to_string()
            } else if trimmed_profile_str.starts_with("https://steamcommunity.com/profiles/") {
                trimmed_profile_str
                    .trim_start_matches("https://steamcommunity.com/profiles/")
                    .trim_end_matches('/')
                    .to_string()
            } else {
                trimmed_profile_str.to_string()
            };

        let profile_id = if resolved_profile_id.len() < 17
            || !resolved_profile_id.chars().all(|c| c.is_digit(10))
            || resolved_profile_id.len() > 17
        {
            match SteamCommentRequester::get_user_id(resolved_profile_id.clone()).await {
                Ok(id) => {
                    println!(
                        "{}",
                        format!(
                            "🔗 Resolved '{}' to profile ID: {}",
                            resolved_profile_id.bold(),
                            id.bold()
                        )
                        .green()
                    );
                    id.parse::<u64>().unwrap_or_default()
                }
                Err(err) => {
                    eprintln!(
                        "{}",
                        format!(
                            "❌  Error resolving user ID for '{}': {}",
                            resolved_profile_id, err
                        )
                        .red()
                        .bold()
                    );
                    continue;
                }
            }
        } else {
            resolved_profile_id.parse::<u64>().unwrap_or_default()
        };
        target_profile_ids.push(profile_id);
    }

    if target_profile_ids.is_empty() {
        eprintln!("{}", "❌  No valid target profile IDs found.".red().bold());
        return Ok(());
    }

    println!(
        "{}",
        format!(
            "🎯 Targeting {} profile(s): {:?}",
            target_profile_ids.len(),
            &target_profile_ids
        )
        .bold()
        .cyan()
    );

    if action_choice == "1" {
        let filters_str = cli::input::get_comment_filters()?;

        let filters_vec: Vec<String> = if filters_str.trim().is_empty()
            || filters_str.trim() == "autofilter"
        {
            println!(
                "{}",
                format!("📁  Loading filters from '{}'...", "filters.json").blue()
            );
            match config::files::read_filters_from_json_file() {
                Ok(filters) => {
                    if filters.is_empty() {
                        println!(
                            "{}",
                            format!(
                                "⚠️  No filters found in '{}'. No filtering will be applied.",
                                "filters.json"
                            )
                            .yellow()
                        );
                        Vec::new()
                    } else {
                        println!(
                            "{}",
                            format!("✅  Loaded {} filter(s) from file.", filters.len()).green()
                        );
                        println!(
                            "{}",
                            format!("   Using filters: {:?}", filters).green().dimmed()
                        );
                        filters
                    }
                }
                Err(e) => {
                    eprintln!(
                        "{}",
                        format!("❌  Error reading filters from file: {}", e)
                            .red()
                            .bold()
                    );
                    return Err(e.into());
                }
            }
        } else {
            let manual_filters: Vec<String> = filters_str
                .trim()
                .split(',')
                .map(|s| s.trim().to_lowercase())
                .filter(|s| !s.is_empty())
                .collect();
            if manual_filters.is_empty() {
                println!(
                    "{}",
                    "⚠️  No filters provided. No filtering will be applied.".yellow()
                );
                Vec::new()
            } else {
                println!(
                    "{}",
                    format!("🔍  Using manual filters: {:?}", manual_filters).green()
                );
                manual_filters
            }
        };

        let filters = Arc::new(filters_vec);

        println!(
            "{}",
            format!(
                "📒  Loading previously processed profiles from '{}'...",
                "reported_profiles.json"
            )
            .blue()
        );
        let reported_profiles = match config::files::read_reported_profiles_from_json() {
            Ok(profiles) => {
                println!(
                    "{}",
                    format!(
                        "✅  Loaded {} previously processed profiles.",
                        profiles.len()
                    )
                    .green()
                );
                Arc::new(Mutex::new(profiles))
            }
            Err(e) => {
                eprintln!(
                    "{}",
                    format!("❌  Error reading processed profiles: {}", e)
                        .red()
                        .bold()
                );
                Arc::new(Mutex::new(HashSet::new()))
            }
        };

        println!("{}", "🚀 Starting comment processing...".bold().green());
        println!(
            "{}",
            "-----------------------------------------------------"
                .bold()
                .green()
        );

        for profile_id in target_profile_ids {
            println!(
                "{}",
                format!(
                    "🎯 Processing profile ID: {}",
                    profile_id.to_string().bold()
                )
                .cyan()
            );

            let mut handles = vec![];
            for account in accounts.clone() {
                let filters_clone = Arc::clone(&filters);
                let reported_profiles_clone = Arc::clone(&reported_profiles);
                let handle: JoinHandle<Result<()>> = tokio::task::spawn_blocking(move || {
                    tokio::runtime::Handle::current().block_on(
                        core::comment_processor::process_account(
                            account,
                            profile_id,
                            filters_clone,
                            reported_profiles_clone,
                        ),
                    )
                });
                handles.push(handle);
            }

            let results = join_all(handles).await;

            for result in results {
                if let Err(e) = result {
                    eprintln!(
                        "{}",
                        format!("⚠️  Task execution error: {}", e).red().bold()
                    );
                }
            }
            println!(
                "{}",
                "-----------------------------------------------------"
                    .bold()
                    .green()
            );
        }
    } else if action_choice == "2" {
        println!("{}", "🚀 Starting profile reporting...".bold().green());
        println!(
            "{}",
            "-----------------------------------------------------"
                .bold()
                .green()
        );
        let provided_reason = cli::input::get_report_reason()?;
        let app_id_str = cli::input::get_app_id()?;

        for profile_id in target_profile_ids {
            println!(
                "{}",
                format!(
                    "🎯 Processing profile ID: {}",
                    profile_id.to_string().bold()
                )
                .cyan()
            );

            let mut handles = Vec::new();
            for account in accounts.clone() {
                let account_clone = account.clone();
                let app_id_str_clone = app_id_str.clone();
                let profile_id_clone = profile_id;

                // Get reason for this specific account
                let reason = if provided_reason.trim().is_empty() {
                    let reasons = read_words_from_json()?;
                    if reasons.is_empty() {
                        eprintln!("{}", "❌  No reasons found in 'reasons.json'.".red().bold());
                        return Ok(());
                    }
                    let mut rng = rand::thread_rng();
                    reasons[rng.gen_range(0..reasons.len())].clone()
                } else {
                    provided_reason.clone()
                };

                let reason_clone = reason.clone();

                let handle: JoinHandle<Result<()>> = tokio::task::spawn_blocking(move || {
                    let requester = SteamProfileRequester::new(account_clone.clone());
                    match tokio::runtime::Handle::current().block_on(requester.report_account(
                        profile_id_clone,
                        reason_clone.clone(),
                        app_id_str_clone.clone(),
                    )) {
                        Ok(_) => {
                            println!(
                                "{}",
                                format!(
                                    "[{}] 🚨  Reported profile {} for reason '{}'",
                                    account_clone.name.cyan().bold(),
                                    profile_id_clone.to_string().bold(),
                                    reason_clone
                                )
                                .green()
                            );
                        }
                        Err(e) => {
                            println!(
                                "{}",
                                format!(
                                    "[{}] ⚠️  Failed to report profile {}: {}",
                                    account_clone.name.cyan().bold(),
                                    profile_id_clone.to_string().bold(),
                                    e
                                )
                                .yellow()
                            );
                        }
                    }
                    Ok(())
                });
                handles.push(handle);
            }

            let results = join_all(handles).await;
            for result in results {
                if let Err(e) = result {
                    eprintln!(
                        "{}",
                        format!("⚠️  Task execution error: {}", e).red().bold()
                    );
                }
            }

            println!(
                "{}",
                "-----------------------------------------------------"
                    .bold()
                    .green()
            );
        }
    } else if action_choice == "3" {
        println!("{}", "✍️  Starting comment posting...".bold().green());
        println!(
            "{}",
            "-----------------------------------------------------"
                .bold()
                .green()
        );

        for profile_id in target_profile_ids {
            println!(
                "{}",
                format!(
                    "🎯 Preparing to post comment on profile ID: {}",
                    profile_id.to_string().bold()
                )
                .cyan()
            );

            let comment_text = cli::input::get_comment_text()?;

            let mut handles = Vec::new();
            for account in accounts.clone() {
                let account_clone = account.clone();
                let profile_id_clone = profile_id;
                let comment_text_clone = comment_text.clone();

                let handle: JoinHandle<Result<()>> = tokio::task::spawn_blocking(move || {
                    let requester = SteamCommentRequester::new(account_clone.clone());
                    match tokio::runtime::Handle::current().block_on(
                        requester.post_comment(profile_id_clone, comment_text_clone.clone()),
                    ) {
                        Ok(_) => {
                            println!(
                                "{}",
                                format!(
                                    "[{}] 💬  Successfully posted comment on profile {}",
                                    account_clone.name.cyan().bold(),
                                    profile_id_clone.to_string().bold(),
                                )
                                .green()
                            );
                        }
                        Err(e) => {
                            println!(
                                "{}",
                                format!(
                                    "[{}] ⚠️  Failed to post comment on profile {}: {}",
                                    account_clone.name.cyan().bold(),
                                    profile_id_clone.to_string().bold(),
                                    e
                                )
                                .yellow()
                            );
                        }
                    }
                    Ok(())
                });
                handles.push(handle);

                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            }

            let results = join_all(handles).await;
            for result in results {
                if let Err(e) = result {
                    eprintln!(
                        "{}",
                        format!("⚠️  Task execution error: {}", e).red().bold()
                    );
                }
            }

            println!(
                "{}",
                "-----------------------------------------------------"
                    .bold()
                    .green()
            );
        }
    } else {
        eprintln!("{}", "❌  Invalid action choice.".red().bold());
    }

    println!(
        "{}",
        "==================== Processing Complete ===================="
            .bold()
            .green()
    );
    println!(
        "{}",
        "✨ Thank you for using the Steam Comment Reporter! ✨"
            .bold()
            .cyan()
    );
    println!(
        "{}",
        "============================================================"
            .bold()
            .green()
    );

    Ok(())
}
