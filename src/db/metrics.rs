use crate::db::models::Metrics;
use crate::db::strings;
use sqlx::{Pool, Postgres};

pub async fn get_metrics(pool: &Pool<Postgres>) -> Result<Metrics, sqlx::Error> {
    let mut transaction = pool.begin().await?;
    let metrics = sqlx::query_as::<_, Metrics>(&strings::GET_METRICS)
        .fetch_one(&mut transaction)
        .await?;
    transaction.commit().await?;
    Ok(metrics)
}
