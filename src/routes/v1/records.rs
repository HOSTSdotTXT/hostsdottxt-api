use crate::db;
use crate::extractors::{Json, Jwt};
use crate::routes::v1::{requests, zones};
use axum::extract::Path;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Extension;
use serde_json::json;
use sqlx::{Pool, Postgres};
use std::net::{Ipv4Addr, Ipv6Addr};
use std::str::FromStr;
use std::sync::Arc;
use trust_dns_proto::rr::{Name, RecordType};
use uuid::Uuid;

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

    if let Err(e) = validate_record(&data.record_type, &data.content) {
        return (StatusCode::BAD_REQUEST, Json(json!({ "error": e })));
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

    // TODO: Check to make sure record is within zone

    if !data.name.ends_with(&zone.id) {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "Record name must be fully qualified"})),
        );
    }

    if let Err(e) = validate_record(&data.record_type, &data.content) {
        return (StatusCode::BAD_REQUEST, Json(json!({ "error": e })));
    }

    let record = db::records::update_record(
        &pool,
        &zone_id,
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

pub async fn delete_record(
    Path((zone_id, record_id)): Path<(String, String)>,
    Jwt(user): Jwt,
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

    // TODO: Make sure record exists
    // TODO: Check to make sure record is within zone

    let result =
        db::records::delete_record(&pool, &zone_id, &Uuid::parse_str(&record_id).unwrap()).await;
    if result.is_err() {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": result.unwrap_err().to_string()})),
        );
    }

    (
        StatusCode::OK,
        Json(json!({
            "message": format!("Record {} deleted", record_id)
        })),
    )
}

fn validate_record(rtype: &str, content: &str) -> Result<(), String> {
    match RecordType::from_str(rtype) {
        Ok(rtype) => match rtype {
            RecordType::A => content
                .parse::<Ipv4Addr>()
                .map(|_| ())
                .map_err(|_| String::from("Invalid IPv4 address")),
            RecordType::AAAA => content
                .parse::<Ipv6Addr>()
                .map(|_| ())
                .map_err(|_| String::from("Invalid IPv6 address")),
            RecordType::CNAME => content
                .parse::<Name>()
                .map(|_| ())
                .map_err(|_| String::from("Invalid CNAME")),
            RecordType::MX => content
                .parse::<Name>()
                .map(|_| ())
                .map_err(|_| String::from("Invalid MX record")),
            RecordType::TXT => Ok(()),
            _ => Err(String::from("Unknown record type")),
        },
        _ => Err(String::from("Unknown record type")),
    }
}
