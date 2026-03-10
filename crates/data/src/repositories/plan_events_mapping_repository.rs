use async_trait::async_trait;
use crate::models::PlanEventsMapping;
use crate::pool::DbPool;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct PlanEventsMappingInsert {
    pub id: i32,
    pub plan_id: i32,
    pub event_id: i32,
    pub success_state_emails: Option<String>,
    pub failure_state_emails: Option<String>,
    pub copy_to_customer: Option<bool>,
}

#[async_trait]
pub trait PlanEventsMappingRepository: Send + Sync {
    async fn get_plan_event(
        &self,
        plan_id: Uuid,
        event_id: i32,
    ) -> Result<Option<PlanEventsMapping>, sqlx::Error>;
    async fn get_all_by_plan_id(&self, plan_id: i32) -> Result<Vec<PlanEventsMapping>, sqlx::Error>;
    async fn upsert(&self, row: &PlanEventsMappingInsert) -> Result<i32, sqlx::Error>;
    /// Delete all rows for `plan_id` whose `id` is NOT in `keep_ids`.
    /// Pass an empty slice to delete all rows for the plan.
    async fn delete_not_in(&self, plan_id: i32, keep_ids: &[i32]) -> Result<(), sqlx::Error>;
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
        .fetch_optional(&{self.pool.get()})
        .await
    }

    async fn get_all_by_plan_id(&self, plan_id: i32) -> Result<Vec<PlanEventsMapping>, sqlx::Error> {
        sqlx::query_as::<_, PlanEventsMapping>(
            "SELECT id, plan_id, event_id, success_state_emails, failure_state_emails, create_date, copy_to_customer
             FROM plan_events_mapping WHERE plan_id = $1 ORDER BY event_id",
        )
        .bind(plan_id)
        .fetch_all(&{self.pool.get()})
        .await
    }

    async fn delete_not_in(&self, plan_id: i32, keep_ids: &[i32]) -> Result<(), sqlx::Error> {
        if keep_ids.is_empty() {
            sqlx::query("DELETE FROM plan_events_mapping WHERE plan_id = $1")
                .bind(plan_id)
                .execute(&{self.pool.get()})
                .await?;
        } else {
            // Build parameterised IN-clause: $2, $3, …
            let placeholders: String = keep_ids
                .iter()
                .enumerate()
                .map(|(i, _)| format!("${}", i + 2))
                .collect::<Vec<_>>()
                .join(", ");
            let sql = format!(
                "DELETE FROM plan_events_mapping WHERE plan_id = $1 AND id NOT IN ({placeholders})"
            );
            let mut q = sqlx::query(&sql).bind(plan_id);
            for id in keep_ids {
                q = q.bind(id);
            }
            q.execute(&{self.pool.get()}).await?;
        }
        Ok(())
    }

    async fn upsert(&self, row: &PlanEventsMappingInsert) -> Result<i32, sqlx::Error> {
        if row.id > 0 {
            let updated = sqlx::query_scalar::<_, i32>(
                "UPDATE plan_events_mapping SET event_id = $2, success_state_emails = $3, failure_state_emails = $4, copy_to_customer = $5
                 WHERE id = $1 RETURNING id",
            )
            .bind(row.id)
            .bind(row.event_id)
            .bind(&row.success_state_emails)
            .bind(&row.failure_state_emails)
            .bind(row.copy_to_customer)
            .fetch_optional(&{self.pool.get()})
            .await?;
            if let Some(id) = updated {
                return Ok(id);
            }
        }
        let id = sqlx::query_scalar::<_, i32>(
            "INSERT INTO plan_events_mapping (plan_id, event_id, success_state_emails, failure_state_emails, copy_to_customer)
             VALUES ($1, $2, $3, $4, $5) RETURNING id",
        )
        .bind(row.plan_id)
        .bind(row.event_id)
        .bind(&row.success_state_emails)
        .bind(&row.failure_state_emails)
        .bind(row.copy_to_customer)
        .fetch_one(&{self.pool.get()})
        .await?;
        Ok(id)
    }
}

