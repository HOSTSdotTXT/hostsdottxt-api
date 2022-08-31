use crate::extractors::Json;
use crate::{db, features};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Extension;
use serde_json::json;
use sqlx::{Pool, Postgres};
use std::sync::Arc;

pub async fn get_metrics(
    Extension(metrics_pool): Extension<Option<Arc<Pool<Postgres>>>>,
) -> impl IntoResponse {
    if !*features::METRICS_ENABLED {
        return (
            StatusCode::NOT_FOUND,
            Json(json!({"error": "Metrics not enabled"})),
        );
    }

    let metrics = db::metrics::get_metrics(&metrics_pool.unwrap()).await;
    match metrics {
        Ok(metrics) => (StatusCode::OK, Json(json!(metrics))),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": err.to_string()})),
        ),
    }
}
