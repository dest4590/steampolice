// src/api/comments.rs
use anyhow::{Error, Result};
use reqwest::header::{HeaderMap, HeaderValue, COOKIE};
use scraper::Html;
use serde::Deserialize;
use serde_json::{self, Value};

use super::models::Account;

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
pub struct SteamCommentResponse {
    pub success: bool,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub start: i32,
    #[serde(default)]
    pub pagesize: i32,
    #[serde(default)]
    pub total_count: i32,
    #[serde(default)]
    pub upvotes: i32,
    #[serde(default)]
    pub has_upvoted: i32,
    #[serde(default)]
    pub comments_html: String,
    #[serde(default)]
    pub timelastpost: i64,
}

pub struct SteamCommentRequester {
    account: Account,
}

impl SteamCommentRequester {
    pub fn new(account: Account) -> Self {
        SteamCommentRequester { account }
    }

    pub async fn get_user_id(nickname: String) -> Result<String> {
        let client = reqwest::Client::new();
        if dotenv::dotenv().is_err() {
            eprintln!("❌ .env file not found! Please create a .env file with your Steam API key.");
            std::process::exit(1);
        }

        let steam_api_key = std::env::var("STEAM_API")
            .expect("STEAM_API environment variable not set. Please rename .env.example to .env and set your Steam API key.");

        let url = format!(
            "http://api.steampowered.com/ISteamUser/ResolveVanityURL/v0001/?key={}&vanityurl={}",
            steam_api_key, nickname
        );
        let response = client.get(&url).send().await?;
        if response.status().is_success() {
            let text = response.text().await?;
            let json: Value = serde_json::from_str(&text)?;
            if let Some(response) = json.get("response") {
                if let Some(steam_id) = response.get("steamid").and_then(Value::as_str) {
                    Ok(steam_id.to_string())
                } else {
                    Err(Error::msg("Failed to extract steamid from response"))
                }
            } else {
                Err(Error::msg("Failed to extract response from response"))
            }
        } else {
            Err(Error::msg(format!(
                "Failed to get user id: {}",
                response.status()
            )))
        }
    }

    pub async fn get_comments_html(&self, profile_id: u64) -> Result<Html> {
        let client = reqwest::Client::new();
        let url = format!(
            "https://steamcommunity.com/comment/Profile/render/{}/-1/?start=0&count=5000&feature2=-1",
            profile_id
        );
        let response = client.get(&url).send().await?;
        if response.status().is_success() {
            let text = response.text().await?;
            let json: Value = serde_json::from_str(&text)?;
            if let Some(comments_html) = json.get("comments_html").and_then(Value::as_str) {
                let document = scraper::Html::parse_document(&comments_html);
                Ok(document)
            } else {
                Err(Error::msg("Failed to extract comments_html from response"))
            }
        } else {
            Err(Error::msg(format!(
                "Failed to get comments: {}",
                response.status()
            )))
        }
    }

    pub async fn post_comment(&self, profile_id: u64, comment: String) -> Result<()> {
        let client = reqwest::Client::new();
        let url = format!(
            "https://steamcommunity.com/comment/Profile/post/{}/-1/",
            profile_id
        );
        let mut headers = HeaderMap::new();
        headers.insert(
            "Accept",
            HeaderValue::from_static("text/javascript, text/html, application/xml, text/xml, */*"),
        );
        headers.insert(
            "Content-Type",
            HeaderValue::from_static("application/x-www-form-urlencoded; charset=UTF-8"),
        );

        let cookie_value = format!(
            "sessionid={}; steamLoginSecure={}",
            self.account.session_id, self.account.steam_login_secure
        );
        headers.insert(COOKIE, HeaderValue::from_str(&cookie_value)?);

        let body = format!(
            "comment={}&count=6&sessionid={}&feature2=-1",
            comment, self.account.session_id
        );

        let response = client.post(url).headers(headers).body(body).send().await?;

        if response.status().is_success() {
            let text = response.text().await?;
            let json: Value = serde_json::from_str(&text)?;

            if let Some(success) = json.get("success").and_then(Value::as_bool) {
                if success {
                    Ok(())
                } else {
                    Err(Error::msg(format!("Request indicates failure: {}", text)))
                }
            } else {
                Err(Error::msg(format!(
                    "Could not determine success from response: {}",
                    text
                )))
            }
        } else {
            Err(Error::msg(format!(
                "Request failed with status: {}",
                response.status()
            )))
        }
    }

    pub async fn hide_and_report_comment(
        &self,
        profile_id: u64,
        comment_id: u64,
    ) -> Result<SteamCommentResponse> {
        let client = reqwest::Client::new();

        let mut headers = HeaderMap::new();
        headers.insert(
            "Accept",
            HeaderValue::from_static("text/javascript, text/html, application/xml, text/xml, */*"),
        );
        headers.insert(
            "Content-Type",
            HeaderValue::from_static("application/x-www-form-urlencoded; charset=UTF-8"),
        );

        let cookie_value = format!(
            "sessionid={}; steamLoginSecure={}",
            self.account.session_id, self.account.steam_login_secure
        );
        headers.insert(COOKIE, HeaderValue::from_str(&cookie_value)?);

        let body = format!(
            "gidcomment={}&hide=1&sessionid={}&feature2=-1",
            comment_id, self.account.session_id
        );

        let response = client
            .post(format!(
                "https://steamcommunity.com/comment/Profile/hideandreport/{}/-1/",
                profile_id
            ))
            .headers(headers)
            .body(body)
            .send()
            .await?;

        if response.status().is_success() {
            let text = response.text().await?;
            let json: Value = serde_json::from_str(&text)?;

            if let Some(success) = json.get("success").and_then(Value::as_bool) {
                if success {
                    let parsed_response = serde_json::from_str(&text)?;
                    Ok(parsed_response)
                } else {
                    Err(Error::msg(format!("Request indicates failure: {}", text)))
                }
            } else {
                Err(Error::msg(format!(
                    "Could not determine success from response: {}",
                    text
                )))
            }
        } else {
            Err(Error::msg(format!(
                "Request failed with status: {}",
                response.status()
            )))
        }
    }
}
