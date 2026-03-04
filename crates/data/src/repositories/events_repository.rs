use async_trait::async_trait;
use crate::models::Events;
use crate::pool::DbPool;

#[async_trait]
pub trait EventsRepository: Send + Sync {
    async fn get_by_name(&self, name: &str) -> Result<Option<Events>, sqlx::Error>;
    async fn get_all(&self) -> Result<Vec<Events>, sqlx::Error>;
}

pub struct PostgresEventsRepository {
    pool: DbPool,
}

impl PostgresEventsRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl EventsRepository for PostgresEventsRepository {
    async fn get_by_name(&self, name: &str) -> Result<Option<Events>, sqlx::Error> {
        sqlx::query_as::<_, Events>(
            "SELECT id, events_name, is_active, create_date FROM events WHERE events_name = $1 LIMIT 1",
        )
        .bind(name)
        .fetch_optional(&self.pool)
        .await
    }

    async fn get_all(&self) -> Result<Vec<Events>, sqlx::Error> {
        sqlx::query_as::<_, Events>(
            "SELECT id, events_name, is_active, create_date FROM events ORDER BY id",
        )
        .fetch_all(&self.pool)
        .await
    }
}

