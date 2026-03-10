use async_trait::async_trait;
use crate::pool::DbPool;
use uuid::Uuid;

#[async_trait]
pub trait WebhookOperationRepository: Send + Sync {
    async fn is_processed(&self, operation_id: Uuid) -> Result<bool, sqlx::Error>;
    async fn mark_processed(&self, operation_id: Uuid) -> Result<(), sqlx::Error>;
}

pub struct PostgresWebhookOperationRepository {
    pool: DbPool,
}

impl PostgresWebhookOperationRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl WebhookOperationRepository for PostgresWebhookOperationRepository {
    async fn is_processed(&self, operation_id: Uuid) -> Result<bool, sqlx::Error> {
        let row: (bool,) = sqlx::query_as(
            "SELECT EXISTS(SELECT 1 FROM webhook_processed_operations WHERE operation_id = $1)",
        )
        .bind(operation_id)
        .fetch_one(&{self.pool.get()})
        .await?;
        Ok(row.0)
    }

    async fn mark_processed(&self, operation_id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO webhook_processed_operations (operation_id) VALUES ($1) ON CONFLICT (operation_id) DO NOTHING",
        )
        .bind(operation_id)
        .execute(&{self.pool.get()})
        .await?;
        Ok(())
    }
}
