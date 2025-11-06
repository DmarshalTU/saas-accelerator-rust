use async_trait::async_trait;
use crate::models::SubscriptionAuditLog;
use crate::pool::DbPool;
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[async_trait]
pub trait SubscriptionAuditLogRepository: Send + Sync {
    async fn create(&self, log: &SubscriptionAuditLog) -> Result<SubscriptionAuditLog, sqlx::Error>;
    async fn save(&self, log: &SubscriptionAuditLog) -> Result<i32, sqlx::Error>;
    async fn get_by_subscription_id(&self, subscription_id: i32) -> Result<Vec<SubscriptionAuditLog>, sqlx::Error>;
    async fn get_subscription_by_subscription_id(&self, subscription_id: uuid::Uuid) -> Result<Vec<SubscriptionAuditLog>, sqlx::Error>;
    async fn log_status_during_provisioning(
        &self,
        subscription_id: uuid::Uuid,
        error_description: &str,
        subscription_status: &str,
    ) -> Result<(), sqlx::Error>;
}

pub struct PostgresSubscriptionAuditLogRepository {
    pool: DbPool,
}

impl PostgresSubscriptionAuditLogRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl SubscriptionAuditLogRepository for PostgresSubscriptionAuditLogRepository {
    async fn create(&self, log: &SubscriptionAuditLog) -> Result<SubscriptionAuditLog, sqlx::Error> {
        let result = sqlx::query_as::<_, SubscriptionAuditLog>(
            r#"
            INSERT INTO subscription_audit_logs 
            (subscription_id, attribute, old_value, new_value, create_date, create_by)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id, subscription_id, attribute, old_value, new_value, create_date, create_by
            "#
        )
        .bind(log.subscription_id)
        .bind(&log.attribute)
        .bind(&log.old_value)
        .bind(&log.new_value)
        .bind(log.create_date.unwrap_or_else(|| Utc::now()))
        .bind(log.create_by)
        .fetch_one(&self.pool)
        .await?;

        Ok(result)
    }

    async fn save(&self, log: &SubscriptionAuditLog) -> Result<i32, sqlx::Error> {
        let result = self.create(log).await?;
        Ok(result.id)
    }

    async fn get_by_subscription_id(&self, subscription_id: i32) -> Result<Vec<SubscriptionAuditLog>, sqlx::Error> {
        sqlx::query_as::<_, SubscriptionAuditLog>(
            "SELECT id, subscription_id, attribute, old_value, new_value, create_date, create_by 
             FROM subscription_audit_logs 
             WHERE subscription_id = $1 
             ORDER BY create_date DESC"
        )
        .bind(subscription_id)
        .fetch_all(&self.pool)
        .await
    }

    async fn get_subscription_by_subscription_id(&self, subscription_id: Uuid) -> Result<Vec<SubscriptionAuditLog>, sqlx::Error> {
        sqlx::query_as::<_, SubscriptionAuditLog>(
            "SELECT sal.id, sal.subscription_id, sal.attribute, sal.old_value, sal.new_value, sal.create_date, sal.create_by 
             FROM subscription_audit_logs sal
             INNER JOIN subscriptions s ON sal.subscription_id = s.id
             WHERE s.amp_subscription_id = $1 
             ORDER BY sal.create_date DESC"
        )
        .bind(subscription_id)
        .fetch_all(&self.pool)
        .await
    }

    async fn log_status_during_provisioning(
        &self,
        subscription_id: Uuid,
        error_description: &str,
        subscription_status: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO web_job_subscription_status (subscription_id, subscription_status, description, insert_date)
             SELECT s.amp_subscription_id, $2, $3, $4
             FROM subscriptions s
             WHERE s.amp_subscription_id = $1"
        )
        .bind(subscription_id)
        .bind(subscription_status)
        .bind(error_description)
        .bind(Some(chrono::Utc::now()))
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}

