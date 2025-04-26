// src/cli/input.rs
use std::io::{self, Write};

use anyhow::Result;
use colored::Colorize;

pub fn get_action_choice() -> Result<String> {
    let mut action_choice = String::new();
    print!("{}", "➡️  Enter your choice (1, 2, 3): ".blue().bold());
    io::stdout().flush()?;
    io::stdin().read_line(&mut action_choice)?;
    Ok(action_choice.trim().to_string())
}

pub fn get_target_profiles() -> Result<String> {
    let mut profiles_str = String::new();
    print!("{}", "🎯 Enter target profile ID(s) / Nickname(s) / Link(s) (comma-separated, use 'search:term!>limit' for search): ".blue().bold());
    io::stdout().flush()?;
    io::stdin().read_line(&mut profiles_str)?;
    Ok(profiles_str)
}

pub fn get_comment_filters() -> Result<String> {
    let mut filters_str = String::new();
    print!("{}", "⚙️  Enter keywords to filter comments (comma-separated, or press Enter to use autofilters): ".blue().bold());
    io::stdout().flush()?;
    io::stdin().read_line(&mut filters_str)?;
    Ok(filters_str)
}

pub fn get_report_reason() -> Result<String> {
    let mut reason = String::new();
    print!(
        "{}",
        "📝 Enter reason for reporting (default: 'aim + wh'): "
            .blue()
            .bold()
    );
    io::stdout().flush()?;
    io::stdin().read_line(&mut reason)?;
    Ok(reason)
}

pub fn get_app_id() -> Result<String> {
    let mut app_id_str = String::new();
    print!(
        "{}",
        "🎮 Enter in-game App ID (default: 730 'CS2'): "
            .blue()
            .bold()
    );
    io::stdout().flush()?;
    io::stdin().read_line(&mut app_id_str)?;
    Ok(app_id_str)
}

pub fn get_comment_text() -> Result<String, io::Error> {
    print!("{}", "Enter the comment text to post: ".bold().blue());
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input.trim().to_string())
}
