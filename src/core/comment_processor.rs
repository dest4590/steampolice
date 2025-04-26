// src/core/comment_processor.rs
use std::{
    collections::HashSet,
    sync::{Arc, Mutex},
    time::Duration,
};

use anyhow::Result;
use colored::Colorize;
use tokio::time::sleep;

use crate::{
    api::{comments::SteamCommentRequester, models::Account},
    config::files::write_reported_profiles_to_json,
};

pub async fn process_account(
    account: Account,
    profile_id: u64,
    filters: Arc<Vec<String>>,
    reported_profiles: Arc<Mutex<HashSet<u64>>>,
) -> Result<()> {
    let requester = SteamCommentRequester::new(account.clone());

    let already_reported_profile = reported_profiles.lock().unwrap().contains(&profile_id);
    if already_reported_profile {
        println!(
            "{}",
            format!(
                "[{}] ℹ️  Skipping profile {}, already processed.",
                account.name.cyan().bold(),
                profile_id.to_string().bold()
            )
            .blue()
        );
        return Ok(());
    }

    match requester.get_comments_html(profile_id).await {
        Ok(document) => {
            let comment_selector = scraper::Selector::parse(".commentthread_comment").unwrap();
            let mut comment_iterator = document.select(&comment_selector).peekable().rev();

            while let Some(comment_element) = comment_iterator.next() {
                if let Some(id_attr) = comment_element.value().attr("id") {
                    if let Some(comment_id_str) = id_attr.strip_prefix("comment_") {
                        if let Ok(comment_id) = comment_id_str.parse::<u64>() {
                            let text_selector =
                                scraper::Selector::parse(".commentthread_comment_text").unwrap();
                            if let Some(text_element) =
                                comment_element.select(&text_selector).next()
                            {
                                let comment_text = text_element
                                    .text()
                                    .collect::<String>()
                                    .trim()
                                    .to_lowercase();
                                for filter in filters.iter() {
                                    if comment_text.contains(filter) {
                                        match requester
                                            .hide_and_report_comment(profile_id.clone(), comment_id)
                                            .await
                                        {
                                            Ok(_) => {
                                                println!(
                                                    "{}",
                                                    format!(
                                                        "[{}] 🚨  Reported malicious comment {} from profile {} (Account: {})",
                                                        account.name.cyan().bold(),
                                                        comment_id.to_string().bold(),
                                                        profile_id.to_string().bold(),
                                                        &account.session_id[..10].green()
                                                    )
                                                    .green(),
                                                );
                                            }
                                            Err(e) => {
                                                println!(
                                                    "{}",
                                                    format!(
                                                        "[{}] ⚠️  Failed to report comment {}: {}",
                                                        account.name.cyan().bold(),
                                                        comment_id.to_string().bold(),
                                                        e
                                                    )
                                                    .yellow(),
                                                );
                                            }
                                        }

                                        sleep(Duration::from_millis(500)).await;
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        Err(err) => {
            println!(
                "{}",
                format!(
                    "[{}] ⚠️  Error getting comments for profile {}: {}",
                    account.name.cyan().bold(),
                    profile_id.to_string().bold(),
                    err
                )
                .yellow()
            );
        }
    }
    reported_profiles.lock().unwrap().insert(profile_id);
    if let Err(e) = write_reported_profiles_to_json(&reported_profiles.lock().unwrap()) {
        eprintln!(
            "{}",
            format!("❌  Error saving reported profiles: {}", e)
                .red()
                .bold()
        );
    }
    Ok(())
}
