use async_trait::async_trait;
use crate::models::MeteredPlanSchedulerManagement;
use crate::pool::DbPool;
use chrono::{DateTime, Utc};

#[async_trait]
pub trait MeteredPlanSchedulerRepository: Send + Sync {
    async fn get_all(&self) -> Result<Vec<MeteredPlanSchedulerManagement>, sqlx::Error>;
    async fn get_by_id(&self, id: i32) -> Result<Option<MeteredPlanSchedulerManagement>, sqlx::Error>;
    async fn update_next_run_time(&self, id: i32, next_run_time: DateTime<Utc>) -> Result<(), sqlx::Error>;
}

pub struct PostgresMeteredPlanSchedulerRepository {
    pool: DbPool,
}

impl PostgresMeteredPlanSchedulerRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl MeteredPlanSchedulerRepository for PostgresMeteredPlanSchedulerRepository {
    async fn get_all(&self) -> Result<Vec<MeteredPlanSchedulerManagement>, sqlx::Error> {
        sqlx::query_as::<_, MeteredPlanSchedulerManagement>(
            "SELECT id, scheduler_name, subscription_id, plan_id, dimension_id, frequency_id, quantity, start_date, next_run_time 
             FROM metered_plan_scheduler_management"
        )
        .fetch_all(&self.pool)
        .await
    }

    async fn get_by_id(&self, id: i32) -> Result<Option<MeteredPlanSchedulerManagement>, sqlx::Error> {
        sqlx::query_as::<_, MeteredPlanSchedulerManagement>(
            "SELECT id, scheduler_name, subscription_id, plan_id, dimension_id, frequency_id, quantity, start_date, next_run_time 
             FROM metered_plan_scheduler_management WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
    }

    async fn update_next_run_time(&self, id: i32, next_run_time: DateTime<Utc>) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE metered_plan_scheduler_management SET next_run_time = $1 WHERE id = $2"
        )
        .bind(next_run_time)
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}

