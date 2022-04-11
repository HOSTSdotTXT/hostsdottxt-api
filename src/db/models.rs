use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Serialize, Deserialize, FromRow, Debug)]
pub struct User {
    pub email: String,
    #[serde(skip_serializing)]
    pub password: String,
    pub display_name: Option<String>,
    pub created_at: DateTime<Utc>,
    pub admin: bool,
    pub enabled: bool,
    #[serde(skip_serializing)]
    pub totp_secret: Option<String>,
}
