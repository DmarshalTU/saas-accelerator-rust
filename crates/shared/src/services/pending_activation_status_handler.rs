use async_trait::async_trait;
use uuid::Uuid;
use std::sync::Arc;
use tracing::info;
use super::status_handlers::{
    AbstractSubscriptionStatusHandler, SubscriptionLogAttributes, SubscriptionStatusHandler,
};

/// Pending Activation Status Handler
pub struct PendingActivationStatusHandler {
    base: AbstractSubscriptionStatusHandler,
    fulfillment_api_service: Arc<dyn FulfillmentApiServiceHelper>,
    subscription_log_repo: Arc<dyn SubscriptionLogRepositoryHelper>,
}

/// Helper trait for fulfillment API service
#[async_trait]
pub trait FulfillmentApiServiceHelper: Send + Sync {
    async fn activate_subscription(&self, subscription_id: Uuid, plan_id: &str) -> Result<(), String>;
    async fn delete_subscription(&self, subscription_id: Uuid, plan_id: &str) -> Result<(), String>;
}

/// Subscription audit log data for status handlers
#[derive(Debug, Clone)]
pub struct SubscriptionAuditLogData {
    pub id: i32,
    pub subscription_id: i32,
    pub attribute: String,
    pub old_value: String,
    pub new_value: String,
    pub create_date: chrono::DateTime<chrono::Utc>,
    pub create_by: Option<i32>,
}

/// Helper trait for subscription log repository
#[async_trait]
pub trait SubscriptionLogRepositoryHelper: Send + Sync {
    async fn save(&self, log: &SubscriptionAuditLogData) -> Result<i32, String>;
    async fn log_status_during_provisioning(
        &self,
        subscription_id: Uuid,
        error_description: &str,
        subscription_status: &str,
    ) -> Result<(), String>;
}

impl PendingActivationStatusHandler {
    pub fn new(
        base: AbstractSubscriptionStatusHandler,
        fulfillment_api_service: Arc<dyn FulfillmentApiServiceHelper>,
        subscription_log_repo: Arc<dyn SubscriptionLogRepositoryHelper>,
    ) -> Self {
        Self {
            base,
            fulfillment_api_service,
            subscription_log_repo,
        }
    }
}

#[async_trait]
impl SubscriptionStatusHandler for PendingActivationStatusHandler {
    async fn process(&self, subscription_id: Uuid) -> Result<(), String> {
        info!("PendingActivationStatusHandler {}", subscription_id);
        
        let subscription = self.base.get_subscription_by_id(subscription_id).await?;
        info!("Result subscription plan_id: {}", subscription.amp_plan_id);
        
        let user_details = self.base.get_user_by_id(subscription.user_id).await?;
        let old_status = subscription.subscription_status.clone();

        if subscription.subscription_status == "PendingActivation" {
            match self
                .fulfillment_api_service
                .activate_subscription(subscription_id, &subscription.amp_plan_id)
                .await
            {
                Ok(()) => {
                    info!("UpdateWebJobSubscriptionStatus");
                    
                    self.base
                        .subscription_repo()
                        .update_status_for_subscription(
                            subscription_id,
                            "Subscribed",
                            true,
                        )
                        .await?;

                    let audit_log = SubscriptionAuditLogData {
                        id: 0,
                        subscription_id: subscription.id,
                        attribute: SubscriptionLogAttributes::Status.to_string(),
                        new_value: "Subscribed".to_string(),
                        old_value: old_status,
                        create_date: chrono::Utc::now(),
                        create_by: user_details.as_ref().map(|u| u.user_id),
                    };
                    self.subscription_log_repo.save(&audit_log).await?;

                    self.subscription_log_repo
                        .log_status_during_provisioning(
                            subscription_id,
                            "Activated",
                            "Subscribed",
                        )
                        .await?;
                }
                Err(ex) => {
                    let error_description = format!("Exception: {ex} :: Inner Exception: None");
                    self.subscription_log_repo
                        .log_status_during_provisioning(
                            subscription_id,
                            &error_description,
                            "ActivationFailed",
                        )
                        .await?;
                    
                    info!("{}", error_description);

                    self.base
                        .subscription_repo()
                        .update_status_for_subscription(
                            subscription_id,
                            "ActivationFailed",
                            false,
                        )
                        .await?;

                    let audit_log = SubscriptionAuditLogData {
                        id: 0,
                        subscription_id: subscription.id,
                        attribute: SubscriptionLogAttributes::Status.to_string(),
                        new_value: "ActivationFailed".to_string(),
                        old_value: subscription.subscription_status,
                        create_date: chrono::Utc::now(),
                        create_by: user_details.as_ref().map(|u| u.user_id),
                    };
                    self.subscription_log_repo.save(&audit_log).await?;
                }
            }
        }

        Ok(())
    }
}


