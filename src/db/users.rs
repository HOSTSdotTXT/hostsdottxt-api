use sqlx::{Pool, Postgres};

use crate::db::models::User;
use crate::db::strings;

#[cfg(debug_assertions)]
const BCRYPT_COST: u32 = 8;
#[cfg(not(debug_assertions))]
const BCRYPT_COST: u32 = 14;

pub async fn get_user(pool: &Pool<Postgres>, email: &str) -> Result<User, sqlx::Error> {
    let user = sqlx::query_as::<_, User>(&strings::GET_USER)
        .bind(email)
        .fetch_one(pool)
        .await?;
    Ok(user)
}

pub async fn get_all_users(pool: &Pool<Postgres>) -> Result<Vec<User>, sqlx::Error> {
    let users = sqlx::query_as::<_, User>(&strings::GET_ALL_USERS)
        .fetch_all(pool)
        .await?;
    Ok(users)
}

pub async fn create_user(
    pool: &Pool<Postgres>,
    email: &str,
    password: &str,
    display_name: &Option<String>,
) -> Result<User, sqlx::Error> {
    let mut transaction = pool.begin().await?;
    let user = sqlx::query_as::<_, User>(&strings::CREATE_USER)
        .bind(email)
        .bind(bcrypt::hash(password, BCRYPT_COST).unwrap())
        .bind(display_name)
        .fetch_one(&mut transaction)
        .await?;
    transaction.commit().await?;
    Ok(user)
}

pub async fn get_user_from_api_key(
    pool: &Pool<Postgres>,
    token_hash: &str,
) -> Result<User, sqlx::Error> {
    let user = sqlx::query_as::<_, User>(&strings::GET_USER_FROM_API_KEY)
        .bind(token_hash)
        .fetch_one(pool)
        .await?;
    Ok(user)
}
