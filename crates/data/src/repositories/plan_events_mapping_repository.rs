use async_trait::async_trait;
use crate::models::PlanEventsMapping;
use crate::pool::DbPool;
use uuid::Uuid;

#[async_trait]
pub trait PlanEventsMappingRepository: Send + Sync {
    async fn get_plan_event(
        &self,
        plan_id: Uuid,
        event_id: i32,
    ) -> Result<Option<PlanEventsMapping>, sqlx::Error>;
}

pub struct PostgresPlanEventsMappingRepository {
    pool: DbPool,
}

impl PostgresPlanEventsMappingRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl PlanEventsMappingRepository for PostgresPlanEventsMappingRepository {
    async fn get_plan_event(
        &self,
        plan_id: Uuid,
        event_id: i32,
    ) -> Result<Option<PlanEventsMapping>, sqlx::Error> {
        sqlx::query_as::<_, PlanEventsMapping>(
            "SELECT pem.id, pem.plan_id, pem.event_id, pem.success_state_emails, pem.failure_state_emails, pem.create_date, pem.copy_to_customer
             FROM plan_events_mapping pem
             INNER JOIN plans p ON pem.plan_id = p.id
             WHERE p.plan_guid = $1 AND pem.event_id = $2
             LIMIT 1",
        )
        .bind(plan_id)
        .bind(event_id)
        .fetch_optional(&self.pool)
        .await
    }
}

