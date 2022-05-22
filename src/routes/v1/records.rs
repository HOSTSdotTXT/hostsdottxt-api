use crate::db;
use crate::extractors::{Json, Jwt};
use crate::routes::v1::{requests, zones};
use axum::extract::Path;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Extension;
use serde_json::json;
use sqlx::{Pool, Postgres};
use uuid::Uuid;
use std::sync::Arc;

pub async fn get_records(
    Path(id): Path<String>,
    Jwt(user): Jwt,
    Extension(pool): Extension<Arc<Pool<Postgres>>>,
) -> impl IntoResponse {
    let domain = zones::ensure_trailing_dot(&id);

    let zone = db::zones::get_zone(&pool, &domain).await;
    if zone.is_err() {
        return (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": format!("Zone {domain} not found") })),
        );
    }
    let zone = zone.unwrap();

    if zone.owner_uuid != user.sub {
        return (
            StatusCode::FORBIDDEN,
            Json(json!({ "error": "You do have permissions to access this zone" })),
        );
    }

    let records = db::records::get_records(&pool, &zone.id).await.unwrap();

    (StatusCode::OK, Json(json!(records)))
}

pub async fn create_record(
    Path(zone_id): Path<String>,
    Jwt(user): Jwt,
    Json(data): Json<requests::Record>,
    Extension(pool): Extension<Arc<Pool<Postgres>>>,
) -> impl IntoResponse {
    let zone = db::zones::get_zone(&pool, &zone_id).await;

    if zone.is_err() {
        return (
            StatusCode::NOT_FOUND,
            Json(json!({"error": "Zone not found"})),
        );
    }
    let zone = zone.unwrap();

    if zone.owner_uuid != user.sub {
        return (
            StatusCode::FORBIDDEN,
            Json(json!({"error": "You do not have permissions to access this zone"})),
        );
    }

    if !data.name.ends_with(&zone.id) {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "Record name must be fully qualified"})),
        );
    }

    let record = db::records::create_record(
        &pool,
        &zone.id,
        &data.name,
        &data.record_type,
        &data.content,
        data.ttl,
    )
    .await;
    if record.is_err() {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": record.unwrap_err().to_string()})),
        );
    }
    let record = record.unwrap();

    (StatusCode::OK, Json(json!(record)))
}

pub async fn update_record(
    Path((zone_id, record_id)): Path<(String, String)>,
    Jwt(user): Jwt,
    Json(data): Json<requests::Record>,
    Extension(pool): Extension<Arc<Pool<Postgres>>>,
) -> impl IntoResponse {
    let zone = db::zones::get_zone(&pool, &zone_id).await;

    if zone.is_err() {
        return (
            StatusCode::NOT_FOUND,
            Json(json!({"error": "Zone not found"})),
        );
    }
    let zone = zone.unwrap();

    if zone.owner_uuid != user.sub {
        return (
            StatusCode::FORBIDDEN,
            Json(json!({"error": "You do not have permissions to access this zone"})),
        );
    }

    if !data.name.ends_with(&zone.id) {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "Record name must be fully qualified"})),
        );
    }

    let record = db::records::update_record(
        &pool,
        &Uuid::parse_str(&record_id).unwrap(),
        &data.name,
        &data.record_type,
        &data.content,
        data.ttl,
    )
    .await;
    if record.is_err() {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": record.unwrap_err().to_string()})),
        );
    }
    let record = record.unwrap();

    (StatusCode::OK, Json(json!(record)))
}
