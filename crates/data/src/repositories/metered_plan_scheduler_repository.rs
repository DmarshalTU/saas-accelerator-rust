use async_trait::async_trait;
use crate::models::MeteredPlanSchedulerManagement;
use crate::pool::DbPool;
use chrono::{DateTime, Utc};

pub struct MeteredPlanSchedulerInsert {
    pub scheduler_name: String,
    pub subscription_id: i32,
    pub plan_id: i32,
    pub dimension_id: i32,
    pub frequency_id: i32,
    pub quantity: f64,
    pub start_date: DateTime<Utc>,
}

#[async_trait]
pub trait MeteredPlanSchedulerRepository: Send + Sync {
    async fn get_all(&self) -> Result<Vec<MeteredPlanSchedulerManagement>, sqlx::Error>;
    async fn get_by_id(&self, id: i32) -> Result<Option<MeteredPlanSchedulerManagement>, sqlx::Error>;
    async fn insert(&self, row: &MeteredPlanSchedulerInsert) -> Result<i32, sqlx::Error>;
    async fn delete_by_id(&self, id: i32) -> Result<(), sqlx::Error>;
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

    async fn insert(&self, row: &MeteredPlanSchedulerInsert) -> Result<i32, sqlx::Error> {
        let id = sqlx::query_scalar::<_, i32>(
            "INSERT INTO metered_plan_scheduler_management (scheduler_name, subscription_id, plan_id, dimension_id, frequency_id, quantity, start_date)
             VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING id",
        )
        .bind(&row.scheduler_name)
        .bind(row.subscription_id)
        .bind(row.plan_id)
        .bind(row.dimension_id)
        .bind(row.frequency_id)
        .bind(row.quantity)
        .bind(row.start_date)
        .fetch_one(&self.pool)
        .await?;
        Ok(id)
    }

    async fn delete_by_id(&self, id: i32) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM metered_plan_scheduler_management WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
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

