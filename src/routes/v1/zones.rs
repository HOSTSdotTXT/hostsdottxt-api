use crate::db;
use crate::extractors::Json;
use crate::extractors::Jwt;
use axum::extract::Path;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Extension;
use lazy_static::lazy_static;
use serde_json::json;
use sqlx::{Error, Pool, Postgres};
use std::sync::Arc;

lazy_static! {
    static ref NAMESERVERS: Vec<String> =
        Vec::from([String::from("ns1.fdns.dev."), String::from("ns2.fdns.dev."), String::from("ns3.fdns.dev."), String::from("ns4.fdns.dev.")]);
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
    Path(id): Path<String>,
    Jwt(user): Jwt,
    Extension(pool): Extension<Arc<Pool<Postgres>>>,
) -> impl IntoResponse {
    let domain = ensure_trailing_dot(&id);
    let domain = addr::parse_domain_name(&domain).unwrap();
    if !domain.has_known_suffix() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "Invalid domain"})),
        );
    }

    let zone_id = domain.root().unwrap().to_owned();
    if zone_id != domain.as_str() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "Invalid domain",
                "message": "Domain must be a root domain"
            })),
        );
    }

    let zone = db::zones::create_zone(&pool, &zone_id, user.sub).await;
    match zone {
        Ok(zone) => (StatusCode::OK, Json(json!(zone))),
        Err(err) => match err {
            Error::Database(e) if e.code().unwrap_or(std::borrow::Cow::Borrowed("")) == "23505" => {
                (
                    StatusCode::BAD_REQUEST,
                    Json(json!({"error": "That zone already exists"})),
                )
            }
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": format!("{:?}", err) })),
            ),
        },
    }
}

pub async fn get_zone(
    Path(id): Path<String>,
    Jwt(user): Jwt,
    Extension(pool): Extension<Arc<Pool<Postgres>>>,
) -> impl IntoResponse {
    let domain = ensure_trailing_dot(&id);

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
            Json(json!({ "error": "You do have permissions to view this zone" })),
        );
    }

    (StatusCode::OK, Json(json!(zone)))
}

fn ensure_trailing_dot(domain: &str) -> String {
    if domain.ends_with('.') {
        return domain.to_string();
    }
    format!("{domain}.")
}
