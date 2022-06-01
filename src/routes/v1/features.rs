use axum::{response::IntoResponse, http::StatusCode};
use crate::extractors::Json;
use serde_json::json;

pub async fn get_features() -> impl IntoResponse {
    (StatusCode::OK, Json(json!({
        "totp": option_env!("FDNS_TOTP_ENABLED").unwrap_or("false").parse::<bool>().unwrap_or(false),
        "signup": option_env!("FDNS_SIGNUP_ENABLED").unwrap_or("false").parse::<bool>().unwrap_or(false),
    })))
}
