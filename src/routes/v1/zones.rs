use crate::db;
use crate::extractors::Json;
use crate::extractors::Jwt;
use axum::extract::Path;
use axum::extract::Query;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Extension;
use lazy_static::lazy_static;
use serde::Deserialize;
use serde_json::json;
use sqlx::{Error, Pool, Postgres};
use std::sync::Arc;
use whois_rust::{WhoIs, WhoIsLookupOptions};

lazy_static! {
    static ref NAMESERVERS: Vec<String> = Vec::from([
        String::from("ns1.hostsdottxt.net."),
        String::from("ns2.hostsdottxt.net."),
        String::from("ns3.hostsdottxt.net."),
        String::from("ns4.hostsdottxt.net.")
    ]);
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

#[derive(Deserialize)]
pub struct RootDomainQuery {
    domain: String,
}
pub async fn get_root_domain(
    Query(query): Query<RootDomainQuery>,
    Jwt(user): Jwt,
    Extension(pool): Extension<Arc<Pool<Postgres>>>,
) -> impl IntoResponse {
    let zones = db::zones::get_zones(&pool, user.sub).await;
    let domain = match query.domain.ends_with('.') {
        true => query.domain,
        false => format!("{}.", query.domain),
    };

    match zones {
        Ok(zones) => {
            let longest_zone = zones
                .iter()
                .filter(|zone| domain.ends_with(&format!(".{}", zone.id)))
                .max_by(|x, y| x.id.len().cmp(&y.id.len()));
            (StatusCode::OK, longest_zone.unwrap().id.clone())
        }
        Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()),
    }
}

pub async fn create_zone(
    Path(zone_id): Path<String>,
    Jwt(user): Jwt,
    Extension(pool): Extension<Arc<Pool<Postgres>>>,
    Extension(whois_client): Extension<WhoIs>,
) -> impl IntoResponse {
    let domain = ensure_trailing_dot(&zone_id);
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

    let lookup = match WhoIsLookupOptions::from_string(zone_id.trim_end_matches('.')) {
        Ok(lookup) => lookup,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "error": "Invalid domain",
                    "message": format!("Could not parse WhoIs Lookup ({})", e)
                })),
            )
        }
    };
    let whois_res = match whois_client.lookup(lookup) {
        Ok(res) => res,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "error": "Invalid domain",
                    "message": format!("{}", e)
                })),
            )
        }
    };

    let whois_info = whoisthere::parse_info(zone_id.trim_end_matches('.'), &whois_res);
    if !whois_info.is_registered || whois_info.is_under_grace_period {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "Invalid domain",
                "message": "Domain is not registered or expired"
            })),
        );
    }

    let zone = db::zones::create_zone(&pool, &zone_id, user.sub).await;
    if let Err(err) = zone {
        match err {
            Error::Database(e) if e.code().unwrap_or(std::borrow::Cow::Borrowed("")) == "23505" => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(json!({"error": "That zone already exists"})),
                )
            }
            _ => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "error": format!("{:?}", err) })),
                )
            }
        }
    }
    let zone = zone.unwrap();

    for ns in NAMESERVERS.iter() {
        if let Err(e) = db::records::create_record(&pool, &zone.id, &zone_id, "NS", ns, 3600).await
        {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": format!("{:?}", e) })),
            );
        }
    }

    (StatusCode::OK, Json(json!(zone)))
}

pub(crate) fn ensure_trailing_dot(domain: &str) -> String {
    if domain.ends_with('.') {
        return domain.to_string();
    }
    format!("{domain}.")
}
