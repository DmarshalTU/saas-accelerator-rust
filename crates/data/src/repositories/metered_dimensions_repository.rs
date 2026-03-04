use async_trait::async_trait;
use crate::models::MeteredDimension;
use crate::pool::DbPool;

#[async_trait]
pub trait MeteredDimensionsRepository: Send + Sync {
    async fn get_by_plan_id(&self, plan_id: i32) -> Result<Vec<MeteredDimension>, sqlx::Error>;
}

pub struct PostgresMeteredDimensionsRepository {
    pool: DbPool,
}

impl PostgresMeteredDimensionsRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl MeteredDimensionsRepository for PostgresMeteredDimensionsRepository {
    async fn get_by_plan_id(&self, plan_id: i32) -> Result<Vec<MeteredDimension>, sqlx::Error> {
        sqlx::query_as::<_, MeteredDimension>(
            "SELECT id, plan_id, dimension, description, created_date FROM metered_dimensions WHERE plan_id = $1 ORDER BY dimension",
        )
        .bind(plan_id)
        .fetch_all(&self.pool)
        .await
    }
}
