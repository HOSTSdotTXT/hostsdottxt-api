use crate::db::models::Zone;
use crate::db::strings;
use sqlx::types::Uuid;
use sqlx::{Pool, Postgres};

pub async fn create_zone(
    pool: &Pool<Postgres>,
    id: &str,
    owner_uuid: Uuid,
) -> Result<Zone, sqlx::Error> {
    let mut transaction = pool.begin().await?;
    let zone = sqlx::query_as::<_, Zone>(&strings::CREATE_ZONE)
        .bind(id)
        .bind(owner_uuid)
        .fetch_one(&mut transaction)
        .await?;
    transaction.commit().await?;
    Ok(zone)
}

pub async fn get_zones(pool: &Pool<Postgres>, owner_uuid: Uuid) -> Result<Vec<Zone>, sqlx::Error> {
    let zones = sqlx::query_as::<_, Zone>(&strings::GET_ZONES)
        .bind(owner_uuid)
        .fetch_all(pool)
        .await?;
    Ok(zones)
}

pub async fn get_zone(pool: &Pool<Postgres>, id: &str) -> Result<Zone, sqlx::Error> {
    let zone = sqlx::query_as::<_, Zone>(&strings::GET_ZONE)
        .bind(id)
        .fetch_one(pool)
        .await?;
    Ok(zone)
}
