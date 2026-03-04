use async_trait::async_trait;
use crate::models::Subscription;
use crate::pool::DbPool;
use uuid::Uuid;

#[async_trait]
pub trait SubscriptionRepository: Send + Sync {
    async fn get_by_id(&self, id: i32) -> Result<Option<Subscription>, sqlx::Error>;
    async fn get_by_amp_subscription_id(
        &self,
        amp_subscription_id: Uuid,
    ) -> Result<Option<Subscription>, sqlx::Error>;
    async fn get_by_amp_subscription_id_with_deactivated(
        &self,
        amp_subscription_id: Uuid,
        include_deactivated: bool,
    ) -> Result<Option<Subscription>, sqlx::Error>;
    async fn get_all(&self) -> Result<Vec<Subscription>, sqlx::Error>;
    async fn get_by_user_id(&self, user_id: i32) -> Result<Vec<Subscription>, sqlx::Error>;
    async fn get_subscriptions_by_email_address(
        &self,
        email_address: &str,
        subscription_id: Option<Uuid>,
        include_deactivated: bool,
    ) -> Result<Vec<Subscription>, sqlx::Error>;
    async fn create(&self, subscription: &Subscription) -> Result<Subscription, sqlx::Error>;
    async fn save(&self, subscription: &Subscription) -> Result<i32, sqlx::Error>;
    async fn update(&self, subscription: &Subscription) -> Result<Subscription, sqlx::Error>;
    async fn update_status_for_subscription(
        &self,
        subscription_id: Uuid,
        status: &str,
        is_active: bool,
    ) -> Result<(), sqlx::Error>;
    async fn update_plan_for_subscription(
        &self,
        subscription_id: Uuid,
        plan_id: &str,
    ) -> Result<(), sqlx::Error>;
    async fn update_quantity_for_subscription(
        &self,
        subscription_id: Uuid,
        quantity: i32,
    ) -> Result<(), sqlx::Error>;
    async fn delete(&self, id: i32) -> Result<(), sqlx::Error>;
}

pub struct PostgresSubscriptionRepository {
    pool: DbPool,
}

impl PostgresSubscriptionRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl SubscriptionRepository for PostgresSubscriptionRepository {
    async fn get_by_id(&self, id: i32) -> Result<Option<Subscription>, sqlx::Error> {
        sqlx::query_as::<_, Subscription>(
            "SELECT id, amp_subscription_id, subscription_status, amp_plan_id, amp_offer_id, 
             is_active, create_by, create_date, modify_date, user_id, name, amp_quantity, 
             purchaser_email, purchaser_tenant_id, term, start_date, end_date 
             FROM subscriptions WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
    }

    async fn get_by_amp_subscription_id(
        &self,
        amp_subscription_id: Uuid,
    ) -> Result<Option<Subscription>, sqlx::Error> {
        sqlx::query_as::<_, Subscription>(
            "SELECT id, amp_subscription_id, subscription_status, amp_plan_id, amp_offer_id, 
             is_active, create_by, create_date, modify_date, user_id, name, amp_quantity, 
             purchaser_email, purchaser_tenant_id, term, start_date, end_date 
             FROM subscriptions WHERE amp_subscription_id = $1",
        )
        .bind(amp_subscription_id)
        .fetch_optional(&self.pool)
        .await
    }

    async fn get_all(&self) -> Result<Vec<Subscription>, sqlx::Error> {
        sqlx::query_as::<_, Subscription>(
            "SELECT id, amp_subscription_id, subscription_status, amp_plan_id, amp_offer_id, 
             is_active, create_by, create_date, modify_date, user_id, name, amp_quantity, 
             purchaser_email, purchaser_tenant_id, term, start_date, end_date 
             FROM subscriptions ORDER BY create_date DESC",
        )
        .fetch_all(&self.pool)
        .await
    }

    async fn get_by_user_id(&self, user_id: i32) -> Result<Vec<Subscription>, sqlx::Error> {
        sqlx::query_as::<_, Subscription>(
            "SELECT id, amp_subscription_id, subscription_status, amp_plan_id, amp_offer_id, 
             is_active, create_by, create_date, modify_date, user_id, name, amp_quantity, 
             purchaser_email, purchaser_tenant_id, term, start_date, end_date 
             FROM subscriptions WHERE user_id = $1 ORDER BY create_date DESC",
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
    }

    async fn create(&self, subscription: &Subscription) -> Result<Subscription, sqlx::Error> {
        let result = sqlx::query_as::<_, Subscription>(
            "INSERT INTO subscriptions (
                amp_subscription_id, subscription_status, amp_plan_id, amp_offer_id, 
                is_active, create_by, create_date, user_id, name, amp_quantity, 
                purchaser_email, purchaser_tenant_id, term, start_date, end_date
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
            RETURNING id, amp_subscription_id, subscription_status, amp_plan_id, amp_offer_id, 
                      is_active, create_by, create_date, modify_date, user_id, name, amp_quantity, 
                      purchaser_email, purchaser_tenant_id, term, start_date, end_date",
        )
        .bind(subscription.amp_subscription_id)
        .bind(&subscription.subscription_status)
        .bind(&subscription.amp_plan_id)
        .bind(&subscription.amp_offer_id)
        .bind(subscription.is_active)
        .bind(subscription.create_by)
        .bind(subscription.create_date)
        .bind(subscription.user_id)
        .bind(&subscription.name)
        .bind(subscription.amp_quantity)
        .bind(&subscription.purchaser_email)
        .bind(subscription.purchaser_tenant_id)
        .bind(&subscription.term)
        .bind(subscription.start_date)
        .bind(subscription.end_date)
        .fetch_one(&self.pool)
        .await?;

        Ok(result)
    }

    async fn update(&self, subscription: &Subscription) -> Result<Subscription, sqlx::Error> {
        sqlx::query_as::<_, Subscription>(
            "UPDATE subscriptions SET 
                subscription_status = $2, amp_plan_id = $3, amp_offer_id = $4, 
                is_active = $5, modify_date = $6, user_id = $7, name = $8, 
                amp_quantity = $9, purchaser_email = $10, purchaser_tenant_id = $11, 
                term = $12, start_date = $13, end_date = $14
            WHERE id = $1
            RETURNING id, amp_subscription_id, subscription_status, amp_plan_id, amp_offer_id, 
                      is_active, create_by, create_date, modify_date, user_id, name, amp_quantity, 
                      purchaser_email, purchaser_tenant_id, term, start_date, end_date",
        )
        .bind(subscription.id)
        .bind(&subscription.subscription_status)
        .bind(&subscription.amp_plan_id)
        .bind(&subscription.amp_offer_id)
        .bind(subscription.is_active)
        .bind(Some(chrono::Utc::now()))
        .bind(subscription.user_id)
        .bind(&subscription.name)
        .bind(subscription.amp_quantity)
        .bind(&subscription.purchaser_email)
        .bind(subscription.purchaser_tenant_id)
        .bind(&subscription.term)
        .bind(subscription.start_date)
        .bind(subscription.end_date)
        .fetch_one(&self.pool)
        .await
    }

    async fn get_by_amp_subscription_id_with_deactivated(
        &self,
        amp_subscription_id: Uuid,
        _include_deactivated: bool,
    ) -> Result<Option<Subscription>, sqlx::Error> {
        self.get_by_amp_subscription_id(amp_subscription_id).await
    }

    async fn get_subscriptions_by_email_address(
        &self,
        email_address: &str,
        subscription_id: Option<Uuid>,
        _include_deactivated: bool,
    ) -> Result<Vec<Subscription>, sqlx::Error> {
        let query = subscription_id.map_or_else(
            || {
                sqlx::query_as::<_, Subscription>(
                    "SELECT s.id, s.amp_subscription_id, s.subscription_status, s.amp_plan_id, s.amp_offer_id, 
                     s.is_active, s.create_by, s.create_date, s.modify_date, s.user_id, s.name, s.amp_quantity, 
                     s.purchaser_email, s.purchaser_tenant_id, s.term, s.start_date, s.end_date 
                     FROM subscriptions s
                     INNER JOIN users u ON s.user_id = u.user_id
                     WHERE u.email_address = $1
                     ORDER BY s.create_date DESC",
                )
                .bind(email_address)
            },
            |sub_id| {
                sqlx::query_as::<_, Subscription>(
                    "SELECT s.id, s.amp_subscription_id, s.subscription_status, s.amp_plan_id, s.amp_offer_id, 
                     s.is_active, s.create_by, s.create_date, s.modify_date, s.user_id, s.name, s.amp_quantity, 
                     s.purchaser_email, s.purchaser_tenant_id, s.term, s.start_date, s.end_date 
                     FROM subscriptions s
                     INNER JOIN users u ON s.user_id = u.user_id
                     WHERE u.email_address = $1 AND s.amp_subscription_id = $2
                     ORDER BY s.create_date DESC",
                )
                .bind(email_address)
                .bind(sub_id)
            },
        );
        query.fetch_all(&self.pool).await
    }

    async fn save(&self, subscription: &Subscription) -> Result<i32, sqlx::Error> {
        let existing = self.get_by_amp_subscription_id(subscription.amp_subscription_id).await?;
        if let Some(mut existing_sub) = existing {
            existing_sub.subscription_status = subscription.subscription_status.clone();
            existing_sub.amp_plan_id = subscription.amp_plan_id.clone();
            existing_sub.amp_quantity = subscription.amp_quantity;
            existing_sub.amp_offer_id = subscription.amp_offer_id.clone();
            existing_sub.term = subscription.term.clone();
            existing_sub.start_date = subscription.start_date;
            existing_sub.end_date = subscription.end_date;
            existing_sub.modify_date = Some(chrono::Utc::now());
            let updated = self.update(&existing_sub).await?;
            Ok(updated.id)
        } else {
            let created = self.create(subscription).await?;
            Ok(created.id)
        }
    }

    async fn update_status_for_subscription(
        &self,
        subscription_id: Uuid,
        status: &str,
        is_active: bool,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE subscriptions SET 
                subscription_status = $2, is_active = $3, modify_date = $4
            WHERE amp_subscription_id = $1",
        )
        .bind(subscription_id)
        .bind(status)
        .bind(is_active)
        .bind(Some(chrono::Utc::now()))
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn update_plan_for_subscription(
        &self,
        subscription_id: Uuid,
        plan_id: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE subscriptions SET 
                amp_plan_id = $2, modify_date = $3
            WHERE amp_subscription_id = $1",
        )
        .bind(subscription_id)
        .bind(plan_id)
        .bind(Some(chrono::Utc::now()))
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn update_quantity_for_subscription(
        &self,
        subscription_id: Uuid,
        quantity: i32,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE subscriptions SET 
                amp_quantity = $2, modify_date = $3
            WHERE amp_subscription_id = $1",
        )
        .bind(subscription_id)
        .bind(quantity)
        .bind(Some(chrono::Utc::now()))
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn delete(&self, id: i32) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM subscriptions WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}

