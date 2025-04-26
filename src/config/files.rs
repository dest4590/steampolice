// src/config/files.rs
use std::{
    collections::HashSet,
    fs::{self, File},
    io::{BufReader, Read},
    path::Path,
};

use anyhow::{Context, Result};
use colored::Colorize;
use serde::{Deserialize, Serialize};
use serde_json;

const ACCOUNTS_FILENAME: &str = "accounts.json";
const FILTERS_FILENAME: &str = "filters.json";
const REPORTED_PROFILES_FILENAME: &str = "reported_profiles.json";
const WORDS_FILENAME: &str = "words.json";

#[derive(Serialize, Deserialize)]
pub struct DefaultAccount {
    pub name: String,
    pub session_id: String,
    pub steam_login_secure: String,
}

pub fn create_default_json_files() -> Result<bool> {
    if !Path::new(WORDS_FILENAME).exists() {
        write_words_to_json(&Vec::new())?;
    }

    if !Path::new(ACCOUNTS_FILENAME).exists() {
        println!(
            "{}",
            "🛠️  Creating default 'accounts.json' file...".blue().bold()
        );
        let default_accounts = vec![DefaultAccount {
            name: "account1".to_string(),
            session_id: "YOUR_SESSION_ID".to_string(),
            steam_login_secure: "YOUR_STEAM_LOGIN_SECURE".to_string(),
        }];
        let file = File::create(ACCOUNTS_FILENAME)?;
        serde_json::to_writer_pretty(&file, &default_accounts)
            .context(format!("Failed to write default {}", ACCOUNTS_FILENAME))?;
        println!(
            "{}",
            format!(
                "👉 Please fill in your account details in '{}'.",
                ACCOUNTS_FILENAME
            )
            .blue()
        );
        return Ok(false);
    }

    if !Path::new(FILTERS_FILENAME).exists() {
        println!(
            "{}",
            "🛠️  Creating default 'filters.json' file...".blue().bold()
        );
        let default_filters: Vec<String> = vec!["example_filter".to_string()];
        let file = File::create(FILTERS_FILENAME)?;
        serde_json::to_writer_pretty(&file, &default_filters)
            .context(format!("Failed to write default {}", FILTERS_FILENAME))?;
        println!(
            "{}",
            format!(
                "💡 Consider adding keywords to filter in '{}'.",
                FILTERS_FILENAME
            )
            .blue()
        );
        return Ok(false);
    }

    if !Path::new(REPORTED_PROFILES_FILENAME).exists() {
        println!(
            "{}",
            "🛠️  Creating default 'reported_profiles.json' file..."
                .blue()
                .bold()
        );
        let default_reported_profiles: Vec<u64> = Vec::new();
        let file = File::create(REPORTED_PROFILES_FILENAME)?;
        serde_json::to_writer_pretty(&file, &default_reported_profiles).context(format!(
            "Failed to write default {}",
            REPORTED_PROFILES_FILENAME
        ))?;
    }

    Ok(true)
}

pub fn read_accounts_from_json() -> Result<Vec<super::super::api::models::Account>> {
    let mut file = fs::File::open(ACCOUNTS_FILENAME)
        .with_context(|| format!("Failed to open {}", ACCOUNTS_FILENAME))?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .with_context(|| format!("Failed to read {}", ACCOUNTS_FILENAME))?;
    let accounts = serde_json::from_str(&contents).with_context(|| {
        format!(
            "⚠️  Failed to parse {} - Invalid JSON format",
            ACCOUNTS_FILENAME
        )
        .yellow()
    })?;
    Ok(accounts)
}

pub fn read_filters_from_json_file() -> Result<Vec<String>> {
    let file = File::open(FILTERS_FILENAME)
        .with_context(|| format!("Failed to open {}", FILTERS_FILENAME))?;
    let reader = BufReader::new(file);
    let filters = serde_json::from_reader(reader).with_context(|| {
        format!(
            "⚠️  Failed to parse {} - Invalid JSON format (expected array of strings)",
            FILTERS_FILENAME
        )
        .yellow()
    })?;
    Ok(filters)
}

pub fn read_reported_profiles_from_json() -> Result<HashSet<u64>> {
    if !Path::new(REPORTED_PROFILES_FILENAME).exists() {
        return Ok(HashSet::new());
    }
    let file = File::open(REPORTED_PROFILES_FILENAME)
        .with_context(|| format!("Failed to open {}", REPORTED_PROFILES_FILENAME))?;
    let reader = BufReader::new(file);
    let reported_profiles: Vec<u64> = serde_json::from_reader(reader).with_context(|| {
        format!(
            "⚠️  Failed to parse {} - Invalid JSON format (expected array of numbers)",
            REPORTED_PROFILES_FILENAME
        )
        .yellow()
    })?;
    Ok(reported_profiles.into_iter().collect())
}

pub fn write_reported_profiles_to_json(reported_profiles: &HashSet<u64>) -> Result<()> {
    let file = File::create(REPORTED_PROFILES_FILENAME)?;
    serde_json::to_writer_pretty(&file, &reported_profiles.iter().collect::<Vec<_>>())
        .context(format!("Failed to write to {}", REPORTED_PROFILES_FILENAME))?;
    Ok(())
}

pub fn read_words_from_json() -> Result<Vec<String>> {
    const WORDS_FILENAME: &str = "words.json";
    let file =
        File::open(WORDS_FILENAME).with_context(|| format!("Failed to open {}", WORDS_FILENAME))?;
    let reader = BufReader::new(file);
    let words: Vec<String> = serde_json::from_reader(reader).with_context(|| {
        format!(
            "⚠️  Failed to parse {} - Invalid JSON format (expected array of strings)",
            WORDS_FILENAME
        )
    })?;
    Ok(words)
}

pub fn write_words_to_json(words: &Vec<String>) -> Result<()> {
    const WORDS_FILENAME: &str = "words.json";
    let file = File::create(WORDS_FILENAME)?;
    serde_json::to_writer_pretty(&file, words)
        .context(format!("Failed to write to {}", WORDS_FILENAME))?;
    Ok(())
}
