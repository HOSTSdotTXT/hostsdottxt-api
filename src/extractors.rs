use axum::async_trait;
use axum::extract::ConnectInfo;
use axum::extract::FromRequest;
use axum::extract::{rejection::JsonRejection, RequestParts};
use axum::http::header::{self, HeaderValue};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::BoxError;
use hmac::{Hmac, Mac};
use jwt::VerifyWithKey;
use lazy_static::lazy_static;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::json;
use serde_json::Value;
use sha2::{Digest, Sha256};
use sqlx::types::Uuid;
use sqlx::{Pool, Postgres};
use std::collections::BTreeMap;
use std::env;
use std::error::Error;
use std::net::IpAddr;
use std::net::SocketAddr;
use std::sync::Arc;

lazy_static! {
    static ref JWT_SECRET: String = env::var("JWT_SECRET").unwrap();
}

pub struct Json<T>(pub T);

#[derive(Debug, Serialize, Deserialize)]
pub struct Token {
    pub iss: String,
    pub sub: Uuid,
    pub iat: i64,
    pub exp: i64,
    pub dn: String,
    pub email: String,
    pub admin: bool,
}
pub struct Jwt(pub Token);

#[async_trait]
impl<B> FromRequest<B> for Jwt
where
    B: axum::body::HttpBody + Send,
    B::Data: Send,
    B::Error: Into<BoxError>,
{
    type Rejection = (axum::http::StatusCode, Json<serde_json::Value>);

    async fn from_request(
        req: &mut axum::extract::RequestParts<B>,
    ) -> Result<Self, Self::Rejection> {
        // Grab the "Authorization" header from the request
        let auth_header = req
            .headers()
            .get(axum::http::header::AUTHORIZATION)
            .and_then(|value| value.to_str().ok());

        match auth_header {
            Some(header) => {
                let key: Hmac<Sha256> = Hmac::new_from_slice((*JWT_SECRET).as_bytes()).unwrap();
                let token = header.replace("Bearer ", "");
                if token.starts_with("hdt_") {
                    let mut hasher = Sha256::new();
                    hasher.update(token.as_bytes());
                    let digest = hasher.finalize();
                    let hash = hex::encode(digest);

                    let db_pool = req.extensions().get::<Arc<Pool<Postgres>>>().unwrap();

                    let user = match crate::db::users::get_user_from_api_key(db_pool, &hash).await {
                        Ok(user) => user,
                        Err(err) => {
                            return Err((
                                StatusCode::INTERNAL_SERVER_ERROR,
                                Json(json!({"error": err.to_string()})),
                            ))
                        }
                    };
                    let token = Token {
                        iss: "hostsdottxt".to_owned(),
                        sub: user.id,
                        iat: 0,
                        exp: 0,
                        dn: user.email.to_owned(),
                        email: user.email.to_owned(),
                        admin: user.admin,
                    };
                    return Ok(Self(token));
                }
                let claims: BTreeMap<String, String> = match token.verify_with_key(&key) {
                    Ok(claims) => claims,
                    Err(_) => {
                        return Err((
                            StatusCode::UNAUTHORIZED,
                            Json(json!({ "error": "Invalid token" })),
                        ))
                    }
                };

                let token = Token {
                    iss: claims.get("iss").unwrap().to_string(),
                    sub: Uuid::parse_str(claims.get("sub").unwrap()).unwrap(),
                    iat: claims.get("iat").unwrap().parse().unwrap(),
                    exp: claims.get("exp").unwrap().parse().unwrap(),
                    dn: claims.get("dn").unwrap().to_string(),
                    email: claims.get("email").unwrap().to_string(),
                    admin: claims.get("admin").unwrap().parse().unwrap(),
                };

                let now = chrono::Utc::now().timestamp();
                if token.iat > now || token.exp < now {
                    return Err((
                        StatusCode::UNAUTHORIZED,
                        Json(json!({"error": "Invalid token"})),
                    ));
                }

                return Ok(Self(token));
            }
            None => {
                return Err((
                    StatusCode::UNAUTHORIZED,
                    Json(json!({"error": "missing auth header"})),
                ))
            }
        }
    }
}

#[async_trait]
impl<B, T> FromRequest<B> for Json<T>
where
    T: DeserializeOwned,
    B: axum::body::HttpBody + Send,
    B::Data: Send,
    B::Error: Into<BoxError>,
{
    type Rejection = (StatusCode, axum::Json<Value>);

    async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        match axum::Json::<T>::from_request(req).await {
            Ok(value) => Ok(Self(value.0)),
            Err(rejection) => {
                let (status, body): (_, Value) = match rejection {
                    JsonRejection::JsonDataError(err) => (
                        StatusCode::BAD_REQUEST,
                        json!({
                            "error": format!("Invalid JSON request: {}", err),
                            "reason": format!("{}", find_error_source::<serde_json::Error>(&err).unwrap())
                        }),
                    ),
                    JsonRejection::MissingJsonContentType(err) => {
                        (StatusCode::BAD_REQUEST, json!({"error": err.to_string()}))
                    }
                    err => (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        json!({ "error": format!("Unknown internal error: {:?}", err) }),
                    ),
                };

                Err((status, axum::Json(body)))
            }
        }
    }
}

impl<T> IntoResponse for Json<T>
where
    T: Serialize,
{
    fn into_response(self) -> Response {
        match serde_json::to_vec(&self.0) {
            Ok(bytes) => (
                [(
                    header::CONTENT_TYPE,
                    HeaderValue::from_static(mime::APPLICATION_JSON.as_ref()),
                )],
                bytes,
            )
                .into_response(),
            Err(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                [(
                    header::CONTENT_TYPE,
                    HeaderValue::from_static(mime::APPLICATION_JSON.as_ref()),
                )],
                err.to_string(),
            )
                .into_response(),
        }
    }
}

// https://docs.rs/axum/latest/axum/extract/index.html#accessing-inner-errors
fn find_error_source<'a, T>(err: &'a (dyn Error + 'static)) -> Option<&'a T>
where
    T: Error + 'static,
{
    if let Some(err) = err.downcast_ref::<T>() {
        Some(err)
    } else if let Some(source) = err.source() {
        find_error_source(source)
    } else {
        None
    }
}

pub struct ClientIp(pub Option<IpAddr>);

#[async_trait]
impl<B> FromRequest<B> for ClientIp
where
    B: Send,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        let headers = req.headers();
        Ok(ClientIp(
            maybe_x_forwarded_for(headers)
                .or_else(|| maybe_x_real_ip(headers))
                .or_else(|| maybe_connect_info(req)),
        ))
    }
}

fn maybe_x_forwarded_for(headers: &HeaderMap) -> Option<IpAddr> {
    headers
        .get("X-Forwarded-For")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.split(',').next())
        .and_then(|value| value.trim().parse().ok())
}

fn maybe_x_real_ip(headers: &HeaderMap) -> Option<IpAddr> {
    headers
        .get("X-Real-Ip")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.parse().ok())
}

fn maybe_connect_info<B: Send>(req: &RequestParts<B>) -> Option<IpAddr> {
    req.extensions()
        .get::<ConnectInfo<SocketAddr>>()
        .map(|ConnectInfo(addr)| addr.ip())
}
