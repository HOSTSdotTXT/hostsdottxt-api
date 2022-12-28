use crate::db;
use crate::db::models::User;
use crate::extractors::{Json, Jwt};
use crate::routes::v1::requests;
use axum::extract::Query;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Extension;
use hmac::{Hmac, Mac};
use jwt::SignWithKey;
use lazy_static::lazy_static;
use serde_json::json;
use sha2::Sha256;
use sqlx::{Error, Pool, Postgres};
use std::collections::{BTreeMap, HashMap};
use std::env;
use std::sync::Arc;

lazy_static! {
    static ref JWT_SECRET: String = env::var("JWT_SECRET").unwrap();
}

pub async fn create_user(
    Json(signup): Json<requests::Signup>,
    Extension(pool): Extension<Arc<Pool<Postgres>>>,
) -> impl IntoResponse {
    if !(*crate::features::SIGNUPS_ENABLED) {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Signups are not enabled" })),
        );
    }
    // TODO: Potentially more checks for password strength
    if signup.password.len() < 12 {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "Password must be at least 12 characters"})),
        );
    }
    let user = db::users::create_user(&pool, &signup.email, &signup.password).await;
    match user {
        Ok(user) => (StatusCode::OK, Json(json!({ "token": issue_jwt(user) }))),
        Err(err) => match err {
            Error::Database(e) if e.code().unwrap_or(std::borrow::Cow::Borrowed("")) == "23505" => {
                (
                    StatusCode::BAD_REQUEST,
                    Json(json!({"error": "A user with that email already exists"})),
                )
            }
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": format!("{:?}", err) })),
            ),
        },
    }
}

pub async fn get_all_users(
    Jwt(user): Jwt,
    Extension(pool): Extension<Arc<Pool<Postgres>>>,
) -> impl IntoResponse {
    if !user.admin {
        return (
            StatusCode::FORBIDDEN,
            Json(json!({"error": "You do not have permission to perform this action"})),
        );
    }

    let users = db::users::get_all_users(&pool).await;
    match users {
        Ok(users) => (StatusCode::OK, Json(json!(users))),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": err.to_string()})),
        ),
    }
}

pub async fn whoami(Jwt(user): Jwt) -> impl IntoResponse {
    (StatusCode::OK, Json(json!(user)))
}

pub async fn needs_totp(
    Query(params): Query<HashMap<String, String>>,
    Extension(pool): Extension<Arc<Pool<Postgres>>>,
) -> impl IntoResponse {
    if !(*crate::features::TOTP_ENABLED) {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "TOTP is not enabled" })),
        );
    }
    match params.get("email") {
        Some(email) => match db::users::get_user(&pool, email).await {
            Ok(user) => (
                StatusCode::OK,
                Json(json!({"totp": user.totp_secret.is_some()})),
            ),
            Err(_) => (StatusCode::OK, Json(json!({"totp": false}))),
        },
        None => (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "Missing query parameter `email`"})),
        ),
    }
}

pub async fn login(
    Json(login_req): Json<requests::Login>,
    Extension(pool): Extension<Arc<Pool<Postgres>>>,
) -> impl IntoResponse {
    let user = db::users::get_user(&pool, &login_req.email).await;
    let user = match user {
        Ok(user) => user,
        Err(err) => match err {
            Error::RowNotFound => {
                return (
                    StatusCode::UNAUTHORIZED,
                    Json(json!({"error": "Invalid email or password"})),
                );
            }
            _ => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "error": format!("{:?}", err) })),
                )
            }
        },
    };

    if !bcrypt::verify(&login_req.password, &user.password).unwrap_or(false) {
        return (
            StatusCode::UNAUTHORIZED,
            Json(json!({"error": "Invalid email or password"})),
        );
    }
    if !user.enabled {
        return (
            StatusCode::UNAUTHORIZED,
            Json(json!({"error": "Invalid email or password"})),
        );
    }

    let token = issue_jwt(user);
    (StatusCode::OK, Json(json!({ "token": token })))
}

fn issue_jwt(user: User) -> String {
    let key: Hmac<Sha256> = Hmac::new_from_slice((*JWT_SECRET).as_bytes()).unwrap();
    let mut claims = BTreeMap::new();

    let iat = chrono::Utc::now().timestamp().to_string();
    let exp = (chrono::Utc::now() + chrono::Duration::hours(24))
        .timestamp()
        .to_string();
    let dn = user.email.clone();
    let admin = user.admin.to_string();
    let sub = user.id.to_string();

    // https://www.iana.org/assignments/jwt/jwt.xhtml
    claims.insert("iss", "hostsdottxt");
    claims.insert("sub", &sub);
    claims.insert("iat", &iat);
    claims.insert("exp", &exp);
    claims.insert("dn", &dn);
    claims.insert("email", &user.email);
    claims.insert("admin", &admin);

    claims.sign_with_key(&key).unwrap()
}
