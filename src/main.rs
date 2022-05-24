use axum::extract::Extension;
use axum::{
    response::IntoResponse,
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
mod routes;

#[tokio::main]
async fn main() {
    // Load configuration from .env
    dotenv().ok();

    if std::env::args().nth(1) == Some("--version".to_string()) {
        println!("{}", option_env!("CARGO_PKG_VERSION").unwrap_or_else(|| "unknown"));
        return;
    }

    // Set logging levels if not already set
    if env::var_os("RUST_LOG").is_none() {
        env::set_var("RUST_LOG", "fdns_api=debug,tower_http=debug");
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

    let app = Router::new()
        .nest(
            "/api",
            Router::new().nest(
                "/v1",
                Router::new()
                    .route("/", get(root))
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
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(Extension(pg_pool)),
        );

    let addr = SocketAddr::from(([0, 0, 0, 0], 8000));
    info!("Binding to {addr}");

    Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn root() -> impl IntoResponse {}
