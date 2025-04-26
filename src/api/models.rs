// src/api/models.rs
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct Account {
    pub name: String,
    pub session_id: String,
    pub steam_login_secure: String,
}
