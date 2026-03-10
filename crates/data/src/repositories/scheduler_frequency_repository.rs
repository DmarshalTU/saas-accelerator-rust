use async_trait::async_trait;
use crate::models::SchedulerFrequency;
use crate::pool::DbPool;

#[async_trait]
pub trait SchedulerFrequencyRepository: Send + Sync {
    async fn get_all(&self) -> Result<Vec<SchedulerFrequency>, sqlx::Error>;
}

pub struct PostgresSchedulerFrequencyRepository {
    pool: DbPool,
}

impl PostgresSchedulerFrequencyRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl SchedulerFrequencyRepository for PostgresSchedulerFrequencyRepository {
    async fn get_all(&self) -> Result<Vec<SchedulerFrequency>, sqlx::Error> {
        sqlx::query_as::<_, SchedulerFrequency>(
            "SELECT id, frequency FROM scheduler_frequency ORDER BY id",
        )
        .fetch_all(&{self.pool.get()})
        .await
    }
}
