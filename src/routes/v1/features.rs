use crate::extractors::Json;
use crate::features;
use axum::{http::StatusCode, response::IntoResponse};
use serde_json::json;

pub async fn get_features() -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(json!({
            "signup": *features::SIGNUPS_ENABLED,
            "totp": *features::TOTP_ENABLED,
        })),
    )
}
