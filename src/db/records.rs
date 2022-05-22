use crate::db::models::Record;
use crate::db::strings;
use sqlx::types::Uuid;
use sqlx::{Pool, Postgres};

pub async fn create_record(
    pool: &Pool<Postgres>,
    zone_id: &str,
    name: &str,
    record_type: &str,
    content: &str,
    ttl: u32,
) -> Result<Record, sqlx::Error> {
    let mut transaction = pool.begin().await?;
    let zone = sqlx::query_as::<_, Record>(&strings::CREATE_RECORD)
        .bind(zone_id)
        .bind(name)
        .bind(record_type)
        .bind(content)
        .bind(ttl)
        .fetch_one(&mut transaction)
        .await?;
    transaction.commit().await?;
    Ok(zone)
}

pub async fn update_record(
    pool: &Pool<Postgres>,
    record_id: &Uuid,
    name: &str,
    record_type: &str,
    content: &str,
    ttl: u32,
) -> Result<Record, sqlx::Error> {
    let mut transaction = pool.begin().await?;
    let record = sqlx::query_as::<_, Record>(&strings::UPDATE_RECORD)
        .bind(name)
        .bind(record_type)
        .bind(content)
        .bind(ttl)
        .bind(record_id)
        .fetch_one(&mut transaction)
        .await?;
    transaction.commit().await?;
    Ok(record)
}

pub async fn get_records(pool: &Pool<Postgres>, zone_id: &str) -> Result<Vec<Record>, sqlx::Error> {
    let records = sqlx::query_as::<_, Record>(&strings::GET_RECORDS)
        .bind(zone_id)
        .fetch_all(pool)
        .await?;
    Ok(records)
}
