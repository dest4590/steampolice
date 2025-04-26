// src/api/search.rs
use anyhow::{Error, Result};
use reqwest::header::{HeaderMap, HeaderValue, COOKIE};
use scraper::Html;
use serde::Deserialize;
use serde_json::{self};

use super::models::Account;

#[derive(Deserialize, Debug, Clone)]
#[allow(dead_code)]
pub struct SteamProfilesResponse {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub link: String,
}

#[derive(Deserialize, Debug)]
struct SearchResult {
    html: String,
    #[serde(rename = "search_result_count")]
    search_result_count: u32,
}

pub struct SteamSearchRequester {
    account: Account,
}

impl SteamSearchRequester {
    pub fn new(account: Account) -> Self {
        SteamSearchRequester { account }
    }

    async fn get_search_result(&self, text: &str, page: u32) -> Result<SearchResult> {
        let client = reqwest::Client::new();
        let steam_login_parts: Vec<&str> = self.account.steam_login_secure.split("||").collect();
        let url = format!("https://steamcommunity.com/search/SearchCommunityAjax?text={}&filter=users&sessionid={}&steamid_user={}&page={}",
            text, self.account.session_id, steam_login_parts[0], page);
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

        let response = client.get(&url).headers(headers).send().await?;

        if response.status().is_success() {
            let text = response.text().await?;
            let search_result: SearchResult = serde_json::from_str(&text)?;
            Ok(search_result)
        } else {
            Err(Error::msg(format!(
                "Failed to get search results: {}",
                response.status()
            )))
        }
    }

    pub async fn get_profiles_from_page(
        &self,
        text: &str,
        page: u32,
    ) -> Result<(Vec<SteamProfilesResponse>, u32)> {
        let search_result = self.get_search_result(text, page).await?;
        let document = Html::parse_document(&search_result.html);
        let profiles = self.parse_profiles(&document)?;
        Ok((profiles, search_result.search_result_count))
    }

    pub fn parse_profiles(&self, document: &Html) -> Result<Vec<SteamProfilesResponse>> {
        let mut profiles = vec![];
        for element in document.select(&scraper::Selector::parse(".searchPersonaName").unwrap()) {
            if let Some(link) = element.value().attr("href") {
                let name = element.text().collect::<String>();
                profiles.push(SteamProfilesResponse {
                    name,
                    link: link.to_string(),
                });
            }
        }
        Ok(profiles)
    }
}
