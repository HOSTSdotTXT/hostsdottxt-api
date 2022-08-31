use axum::extract::Extension;
use axum::{
    routing::{get, post, put},
    Router, Server,
};
use dotenvy::dotenv;
use log::info;
use sqlx::postgres::PgPoolOptions;
use std::env;
use std::net::SocketAddr;
use std::sync::Arc;
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;

mod db;
mod extractors;
mod features;
mod routes;

#[tokio::main]
async fn main() {
    // Load configuration from .env
    dotenv().ok();

    if std::env::args().nth(1) == Some("--version".to_string()) {
        println!(
            "{}",
            option_env!("CARGO_PKG_VERSION").unwrap_or_else(|| "unknown")
        );
        return;
    }

    println!("SIGNUPS_ENABLED = {}", *features::SIGNUPS_ENABLED);
    println!("TOTP_ENABLED = {}", *features::TOTP_ENABLED);

    // Set logging levels if not already set
    if env::var_os("RUST_LOG").is_none() {
        env::set_var("RUST_LOG", "hdt_api=debug,tower_http=debug");
    }

    // Initialize tracing with previously set logging levels
    tracing_subscriber::fmt::init();

    // Connect to Postgres
    let pg_pool = Arc::new(
        PgPoolOptions::new()
            .max_connections(5)
            .connect(&env::var("DATABASE_URL").unwrap())
            .await
            .unwrap(),
    );
    info!("Postgres pool initialized");
    let metrics_pool = match env::var("METRICS_URL") {
        Ok(url) => Some(Arc::new(
            PgPoolOptions::new()
                .max_connections(5)
                .connect(&url)
                .await
                .unwrap(),
        )),
        Err(_) => None,
    };
    info!("Metrics pool (possibly) initialized");

    // Create our WhoIs client
    let whois_client = whois_rust::WhoIs::from_string(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/data/whois-servers.json"
    )))
    .unwrap();

    let app = Router::new()
        .nest(
            "/api",
            Router::new().nest(
                "/v1",
                Router::new()
                    .route("/features", get(routes::v1::features::get_features))
                    .route("/metrics", get(routes::v1::metrics::get_metrics))
                    .nest(
                        "/users",
                        Router::new()
                            .route("/", post(routes::v1::users::create_user))
                            .route("/", get(routes::v1::users::get_all_users))
                            .route("/totp", get(routes::v1::users::needs_totp))
                            .route("/login", post(routes::v1::users::login))
                            .route("/whoami", get(routes::v1::users::whoami)),
                    )
                    .nest(
                        "/zones",
                        Router::new()
                            .route("/", get(routes::v1::zones::list_zones))
                            .route("/root", get(routes::v1::zones::get_root_domain))
                            .route(
                                "/:zone_id",
                                get(routes::v1::records::get_records)
                                    .post(routes::v1::zones::create_zone)
                                    .put(routes::v1::records::create_record),
                            )
                            .route(
                                "/:zone_id/:record_id",
                                put(routes::v1::records::update_record)
                                    .delete(routes::v1::records::delete_record),
                            ),
                    ),
            ),
        )
        .layer(ServiceBuilder::new().layer(TraceLayer::new_for_http()))
        .layer(Extension(pg_pool))
        .layer(Extension(metrics_pool))
        .layer(Extension(whois_client));

    let addr = SocketAddr::from(([0, 0, 0, 0], 8000));
    info!("Binding to {addr}");

    Server::bind(&addr)
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
        .await
        .unwrap();
}
