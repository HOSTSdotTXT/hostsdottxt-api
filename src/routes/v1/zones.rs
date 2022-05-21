use crate::db;
use crate::extractors::Json;
use crate::extractors::Jwt;
use axum::extract::Path;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Extension;
use serde_json::json;
use sqlx::{Pool, Postgres};
use std::sync::Arc;
use lazy_static::lazy_static;

lazy_static!{
    static ref NAMESERVERS: Vec<String> = Vec::from([String::from("ns1.fdns.dev."), String::from("ns2.fdns.dev.")]);
}

pub async fn list_zones(
    Jwt(user): Jwt,
    Extension(pool): Extension<Arc<Pool<Postgres>>>,
) -> impl IntoResponse {
    let zones = db::zones::get_zones(&pool, user.sub).await;
    match zones {
        Ok(zones) => (StatusCode::OK, Json(json!(zones))),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": err.to_string()})),
        ),
    }
}

pub async fn create_zone(
    Path(_id): Path<String>,
    Jwt(_user): Jwt,
    Extension(_pool): Extension<Arc<Pool<Postgres>>>,
) -> impl IntoResponse {
    (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": "Not implemented"})));
}

pub async fn get_zone(
    Path(_id): Path<String>,
    Jwt(_user): Jwt,
    Extension(_pool): Extension<Arc<Pool<Postgres>>>,
) -> impl IntoResponse {
    (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": "Not implemented"})));
}
