use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::types::Uuid;
use sqlx::FromRow;

#[derive(Serialize, Deserialize, FromRow, Debug)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    #[serde(skip_serializing)]
    pub password: String,
    pub display_name: Option<String>,
    pub created_at: DateTime<Utc>,
    pub modified_at: DateTime<Utc>,
    pub admin: bool,
    pub enabled: bool,
    #[serde(skip_serializing)]
    pub totp_secret: Option<String>,
}

#[derive(Serialize, Deserialize, FromRow, Debug)]
pub struct Zone {
    pub id: String,
    pub owner_uuid: Uuid,
    pub created_at: DateTime<Utc>,
    pub modified_at: DateTime<Utc>,
    // pub enabled: bool,
}

#[derive(Serialize, Deserialize, FromRow, Debug)]
pub struct Record {
    pub id: Uuid,
    pub zone_id: String,
    pub name: String,
    #[serde(rename = "type")]
    #[sqlx(rename = "type")]
    pub record_type: String,
    pub content: String,
    pub ttl: i32,
    pub created_at: DateTime<Utc>,
    pub modified_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, FromRow, Debug)]
pub struct Metrics {
    pub p50: f64,
    pub p90: f64,
    pub p95: f64,
    pub p99: f64,
    pub avg: f64,
    pub count: i64,
}
