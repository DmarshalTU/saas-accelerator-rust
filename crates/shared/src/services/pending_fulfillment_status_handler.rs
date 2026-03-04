use async_trait::async_trait;
use uuid::Uuid;
use std::sync::Arc;
use tracing::info;
use super::status_handlers::{
    AbstractSubscriptionStatusHandler, SubscriptionLogAttributes, SubscriptionStatusHandler,
};
use super::pending_activation_status_handler::{SubscriptionLogRepositoryHelper, SubscriptionAuditLogData};

/// Pending Fulfillment Status Handler
pub struct PendingFulfillmentStatusHandler {
    base: AbstractSubscriptionStatusHandler,
    subscription_log_repo: Arc<dyn SubscriptionLogRepositoryHelper>,
}

impl PendingFulfillmentStatusHandler {
    pub fn new(
        base: AbstractSubscriptionStatusHandler,
        subscription_log_repo: Arc<dyn SubscriptionLogRepositoryHelper>,
    ) -> Self {
        Self {
            base,
            subscription_log_repo,
        }
    }
}

#[async_trait]
impl SubscriptionStatusHandler for PendingFulfillmentStatusHandler {
    async fn process(&self, subscription_id: Uuid) -> Result<(), String> {
        info!("PendingFulfillmentStatusHandler {}", subscription_id);
        
        let subscription = self.base.get_subscription_by_id(subscription_id).await?;
        info!("Result subscription plan_id: {}", subscription.amp_plan_id);
        
        let user_details = self.base.get_user_by_id(subscription.user_id).await?;

        if subscription.subscription_status == "PendingFulfillmentStart" {
            match self
                .base
                .subscription_repo()
                .update_status_for_subscription(subscription_id, "PendingActivation", true)
                .await
            {
                Ok(()) => {
                    let audit_log = SubscriptionAuditLogData {
                        id: 0,
                        subscription_id: subscription.id,
                        attribute: SubscriptionLogAttributes::Status.to_string(),
                        new_value: "PendingActivation".to_string(),
                        old_value: "PendingFulfillmentStart".to_string(),
                        create_date: chrono::Utc::now(),
                        create_by: user_details.as_ref().map(|u| u.user_id),
                    };
                    self.subscription_log_repo.save(&audit_log).await?;
                }
                Err(ex) => {
                    let error_description = format!("Exception: {ex} :: Inner Exception: None");
                    info!("{}", error_description);

                    self.base
                        .subscription_repo()
                        .update_status_for_subscription(subscription_id, "PendingActivation", true)
                        .await?;

                    let audit_log = SubscriptionAuditLogData {
                        id: 0,
                        subscription_id: subscription.id,
                        attribute: SubscriptionLogAttributes::Status.to_string(),
                        new_value: "PendingActivation".to_string(),
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

