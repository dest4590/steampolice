// src/core/profile_reporter.rs
use std::{
    io::{self, Write},
    time::Duration,
};

use anyhow::Result;
use colored::Colorize;
use tokio::time::sleep;

use crate::api::{models::Account, search::SteamSearchRequester};

pub async fn handle_search_prefix(
    account: Account,
    search_term: &str,
    max_pages: Option<u32>,
) -> Result<Vec<String>> {
    println!(
        "{}",
        format!(
            "🔍  Searching for profiles matching: '{}'",
            search_term.bold()
        )
        .blue()
    );

    let searcher = SteamSearchRequester::new(account);
    let mut all_profiles = Vec::new();
    let mut page = 1;
    let mut total_results = 0;

    loop {
        if let Some(max) = max_pages {
            if page > max {
                println!(
                    "{}",
                    format!("ℹ️  Reached maximum search page limit: {}", max).blue()
                );
                break;
            }
        }

        print!(
            "{}",
            format!("🌐  Fetching search results page {}...", page).blue()
        );
        io::stdout().flush()?;
        match searcher.get_profiles_from_page(search_term, page).await {
            Ok((profiles, search_result_count)) => {
                println!("{}", " Done.".green());
                all_profiles.extend(profiles.clone());
                if page == 1 {
                    total_results = search_result_count;
                    println!(
                        "{}",
                        format!("ℹ️  Found {} profiles matching your search.", total_results)
                            .blue()
                    );
                }
                if all_profiles.len() >= total_results as usize && total_results > 0 {
                    println!("{}", "✅  All search results collected.".green().bold());
                    break;
                }
                if profiles.is_empty() {
                    println!("{}", "ℹ️  No more profiles found on this page.".blue());
                    break;
                }
                page += 1;

                sleep(Duration::from_millis(500)).await;
            }
            Err(err) => {
                println!("{}", " Failed.".red());
                eprintln!(
                    "{}",
                    format!("❌  Error getting search page {}: {}", page, err)
                        .red()
                        .bold()
                );
                break;
            }
        }
    }

    println!(
        "{}",
        format!(
            "✅  Search for '{}' complete. Found {} profiles.",
            search_term.bold(),
            all_profiles.len()
        )
        .green()
    );

    Ok(all_profiles
        .into_iter()
        .map(|profile| profile.link)
        .collect())
}
