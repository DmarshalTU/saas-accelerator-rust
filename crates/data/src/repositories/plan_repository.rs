use async_trait::async_trait;
use crate::models::Plan;
use crate::pool::DbPool;
use uuid::Uuid;

#[async_trait]
pub trait PlanRepository: Send + Sync {
    async fn get_by_id(&self, id: i32) -> Result<Option<Plan>, sqlx::Error>;
    async fn get_by_plan_id(&self, plan_id: &str) -> Result<Option<Plan>, sqlx::Error>;
    async fn get_by_offer_id(&self, offer_id: Uuid) -> Result<Vec<Plan>, sqlx::Error>;
    async fn get_by_internal_reference(&self, plan_guid: Uuid) -> Result<Option<Plan>, sqlx::Error>;
    async fn get_plans_by_user(&self) -> Result<Vec<Plan>, sqlx::Error>;
    async fn get_all(&self) -> Result<Vec<Plan>, sqlx::Error>;
}

pub struct PostgresPlanRepository {
    pool: DbPool,
}

impl PostgresPlanRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl PlanRepository for PostgresPlanRepository {
    async fn get_by_id(&self, id: i32) -> Result<Option<Plan>, sqlx::Error> {
        sqlx::query_as::<_, Plan>(
            "SELECT id, plan_id, description, display_name, is_metering_supported, 
             is_per_user, plan_guid, offer_id 
             FROM plans WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&{self.pool.get()})
        .await
    }

    async fn get_by_plan_id(&self, plan_id: &str) -> Result<Option<Plan>, sqlx::Error> {
        sqlx::query_as::<_, Plan>(
            "SELECT id, plan_id, description, display_name, is_metering_supported, 
             is_per_user, plan_guid, offer_id 
             FROM plans WHERE plan_id = $1",
        )
        .bind(plan_id)
        .fetch_optional(&{self.pool.get()})
        .await
    }

    async fn get_by_offer_id(&self, offer_id: Uuid) -> Result<Vec<Plan>, sqlx::Error> {
        sqlx::query_as::<_, Plan>(
            "SELECT id, plan_id, description, display_name, is_metering_supported, 
             is_per_user, plan_guid, offer_id 
             FROM plans WHERE offer_id = $1",
        )
        .bind(offer_id)
        .fetch_all(&{self.pool.get()})
        .await
    }

    async fn get_by_internal_reference(&self, plan_guid: Uuid) -> Result<Option<Plan>, sqlx::Error> {
        sqlx::query_as::<_, Plan>(
            "SELECT id, plan_id, description, display_name, is_metering_supported, 
             is_per_user, plan_guid, offer_id 
             FROM plans WHERE plan_guid = $1",
        )
        .bind(plan_guid)
        .fetch_optional(&{self.pool.get()})
        .await
    }

    async fn get_plans_by_user(&self) -> Result<Vec<Plan>, sqlx::Error> {
        self.get_all().await
    }

    async fn get_all(&self) -> Result<Vec<Plan>, sqlx::Error> {
        sqlx::query_as::<_, Plan>(
            "SELECT id, plan_id, description, display_name, is_metering_supported, 
             is_per_user, plan_guid, offer_id 
             FROM plans",
        )
        .fetch_all(&{self.pool.get()})
        .await
    }
}

