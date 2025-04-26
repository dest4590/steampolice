// generated/cookie.rs
use std::{fs, path::Path};

use colored::Colorize;
use serde::{Deserialize, Serialize};
use serde_json::from_reader;

#[derive(Serialize, Deserialize, PartialEq)]
struct Account {
    name: String,
    session_id: String,
    steam_login_secure: String,
}

fn main() {
    let path = Path::new("generated/");
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries {
            if let Ok(entry) = entry {
                let file_name = entry.file_name();
                let file_name_str = file_name.to_string_lossy();
                if file_name_str.starts_with("generated") {
                    let username = file_name_str
                        .split('-')
                        .nth(1)
                        .unwrap()
                        .trim_end_matches(".json")
                        .to_string();
                    let file = fs::read_to_string(entry.path());
                    if let Ok(file) = file {
                        let session_id = file.split("|||").nth(0).unwrap();
                        let steam_login_secure =
                            file.split("|||").nth(1).unwrap().replace("%7C", "|");
                        let accounts = fs::read_to_string("accounts.json");
                        if let Ok(accounts) = accounts {
                            let mut accounts: Vec<Account> =
                                from_reader(accounts.as_bytes()).unwrap();
                            let new_account = Account {
                                name: username.clone(),
                                session_id: session_id.to_string(),
                                steam_login_secure: steam_login_secure.to_string(),
                            };
                            if !accounts.contains(&new_account) {
                                accounts.push(new_account);
                                println!("Added account: {}", username.yellow());
                                if let Err(e) = fs::write(
                                    "accounts.json",
                                    serde_json::to_string_pretty(&accounts).unwrap(),
                                ) {
                                    eprintln!("Failed to write to accounts.json: {}", e);
                                }

                                if let Err(e) = fs::remove_file(entry.path()) {
                                    eprintln!("Failed to remove file: {}", e);
                                }
                            } else {
                                println!("Account already exists: {}", username.red());
                            }
                        }
                    }
                }
            }
        }
    } else {
        println!("Could not read the directory");
    }
}
