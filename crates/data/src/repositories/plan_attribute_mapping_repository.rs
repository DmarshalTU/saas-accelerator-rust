use async_trait::async_trait;
use crate::models::PlanAttributeMapping;
use crate::pool::DbPool;

#[async_trait]
pub trait PlanAttributeMappingRepository: Send + Sync {
    async fn get_by_plan_id(&self, plan_id: i32) -> Result<Vec<PlanAttributeMapping>, sqlx::Error>;
    async fn replace_for_plan(&self, plan_id: i32, offer_attribute_ids: &[i32]) -> Result<(), sqlx::Error>;
}

pub struct PostgresPlanAttributeMappingRepository {
    pool: DbPool,
}

impl PostgresPlanAttributeMappingRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl PlanAttributeMappingRepository for PostgresPlanAttributeMappingRepository {
    async fn get_by_plan_id(&self, plan_id: i32) -> Result<Vec<PlanAttributeMapping>, sqlx::Error> {
        sqlx::query_as::<_, PlanAttributeMapping>(
            "SELECT plan_attribute_id, plan_id, offer_attribute_id, create_date
             FROM plan_attribute_mapping WHERE plan_id = $1 ORDER BY plan_attribute_id",
        )
        .bind(plan_id)
        .fetch_all(&self.pool)
        .await
    }

    async fn replace_for_plan(&self, plan_id: i32, offer_attribute_ids: &[i32]) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM plan_attribute_mapping WHERE plan_id = $1")
            .bind(plan_id)
            .execute(&self.pool)
            .await?;
        for &offer_attribute_id in offer_attribute_ids {
            sqlx::query(
                "INSERT INTO plan_attribute_mapping (plan_id, offer_attribute_id) VALUES ($1, $2)",
            )
            .bind(plan_id)
            .bind(offer_attribute_id)
            .execute(&self.pool)
            .await?;
        }
        Ok(())
    }
}
