// src/api/profiles.rs
use anyhow::{Error, Result};
use reqwest::header::{HeaderMap, HeaderValue, COOKIE};

use super::models::Account;

pub struct SteamProfileRequester {
    account: Account,
}

impl SteamProfileRequester {
    pub fn new(account: Account) -> Self {
        SteamProfileRequester { account }
    }

    pub async fn report_account(
        &self,
        steam_id: u64,
        reason: String,
        app_id: String,
    ) -> Result<()> {
        let client = reqwest::Client::new();
        let url = format!("https://steamcommunity.com/actions/ReportAbuse/sessionid={}&json=1&abuseID={}&eAbuseType=10&abuseDescription={}&ingameAppID={}", self.account.session_id, steam_id, reason, app_id);

        let mut headers = HeaderMap::new();
        headers.insert(
            "Accept",
            HeaderValue::from_static("text/javascript, text/html, application/xml, text/xml, */*"),
        );
        headers.insert(
            "Content-Type",
            HeaderValue::from_static("application/x-www-form-urlencoded; charset=UTF-8"),
        );
        headers.insert("Content-Length", HeaderValue::from_static("0"));

        let cookie_value = format!(
            "sessionid={}; steamLoginSecure={}",
            self.account.session_id, self.account.steam_login_secure
        );
        headers.insert(COOKIE, HeaderValue::from_str(&cookie_value)?);

        let response = client.post(url).headers(headers).send().await?;

        if response.status().is_success() {
            if response.text().await?.contains("1") {
                return Ok(());
            } else {
                return Err(Error::msg("Failed to report account"));
            }
        } else {
            Err(Error::msg(format!(
                "Failed to report account: {}",
                response.status()
            )))
        }
    }
}
