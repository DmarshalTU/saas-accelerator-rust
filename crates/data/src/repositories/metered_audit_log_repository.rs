use async_trait::async_trait;
use crate::models::MeteredAuditLog;
use crate::pool::DbPool;

#[async_trait]
pub trait MeteredAuditLogRepository: Send + Sync {
    async fn get_by_subscription_id(
        &self,
        subscription_id: i32,
    ) -> Result<Vec<MeteredAuditLog>, sqlx::Error>;
}

pub struct PostgresMeteredAuditLogRepository {
    pool: DbPool,
}

impl PostgresMeteredAuditLogRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl MeteredAuditLogRepository for PostgresMeteredAuditLogRepository {
    async fn get_by_subscription_id(
        &self,
        subscription_id: i32,
    ) -> Result<Vec<MeteredAuditLog>, sqlx::Error> {
        sqlx::query_as::<_, MeteredAuditLog>(
            "SELECT id, subscription_id, request_json, response_json, status_code, created_date, subscription_usage_date, run_by
             FROM metered_audit_logs
             WHERE subscription_id = $1
             ORDER BY created_date DESC",
        )
        .bind(subscription_id)
        .fetch_all(&{self.pool.get()})
        .await
    }
}
