use async_trait::async_trait;
use crate::models::ApplicationLog;
use crate::pool::DbPool;

#[async_trait]
pub trait ApplicationLogRepository: Send + Sync {
    async fn add_log(&self, log_detail: &ApplicationLog) -> Result<i32, sqlx::Error>;
    async fn update_log(&self, log_detail: &ApplicationLog) -> Result<i32, sqlx::Error>;
    async fn get_logs(&self) -> Result<Vec<ApplicationLog>, sqlx::Error>;
}

pub struct PostgresApplicationLogRepository {
    pool: DbPool,
}

impl PostgresApplicationLogRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ApplicationLogRepository for PostgresApplicationLogRepository {
    async fn add_log(&self, log_detail: &ApplicationLog) -> Result<i32, sqlx::Error> {
        let result = sqlx::query_scalar::<_, i32>(
            "INSERT INTO application_log (action_time, log_detail) 
             VALUES ($1, $2)
             RETURNING id",
        )
        .bind(log_detail.action_time)
        .bind(&log_detail.log_detail)
        .fetch_one(&self.pool)
        .await?;

        Ok(result)
    }

    async fn update_log(&self, log_detail: &ApplicationLog) -> Result<i32, sqlx::Error> {
        sqlx::query(
            "UPDATE application_log SET action_time = $2, log_detail = $3 WHERE id = $1",
        )
        .bind(log_detail.id)
        .bind(log_detail.action_time)
        .bind(&log_detail.log_detail)
        .execute(&self.pool)
        .await?;

        Ok(log_detail.id)
    }

    async fn get_logs(&self) -> Result<Vec<ApplicationLog>, sqlx::Error> {
        sqlx::query_as::<_, ApplicationLog>(
            "SELECT id, action_time, log_detail FROM application_log ORDER BY action_time DESC",
        )
        .fetch_all(&self.pool)
        .await
    }
}

