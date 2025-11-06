use async_trait::async_trait;
use uuid::Uuid;
use std::sync::Arc;
use tracing::info;
use super::status_handlers::*;
use super::pending_activation_status_handler::{FulfillmentApiServiceHelper, SubscriptionLogRepositoryHelper, SubscriptionAuditLogData};

/// Unsubscribe Status Handler
pub struct UnsubscribeStatusHandler {
    base: AbstractSubscriptionStatusHandler,
    fulfillment_api_service: Arc<dyn FulfillmentApiServiceHelper>,
    subscription_log_repo: Arc<dyn SubscriptionLogRepositoryHelper>,
}

impl UnsubscribeStatusHandler {
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
impl SubscriptionStatusHandler for UnsubscribeStatusHandler {
    async fn process(&self, subscription_id: Uuid) -> Result<(), String> {
        info!("UnsubscribeStatusHandler {}", subscription_id);
        
        let subscription = self.base.get_subscription_by_id(subscription_id).await?;
        info!("Result subscription plan_id: {}", subscription.amp_plan_id);

        let user_details = self.base.get_user_by_id(subscription.user_id).await?;
        let status = subscription.subscription_status.clone();
        
        if subscription.subscription_status == "PendingUnsubscribe" {
            match self
                .fulfillment_api_service
                .delete_subscription(subscription_id, &subscription.amp_plan_id)
                .await
            {
                Ok(_) => {
                    self.base
                        .subscription_repo()
                        .update_status_for_subscription(subscription_id, "Unsubscribed", false)
                        .await?;

                    let audit_log = SubscriptionAuditLogData {
                        id: 0,
                        subscription_id: subscription.id,
                        attribute: SubscriptionLogAttributes::Status.to_string(),
                        new_value: "Unsubscribed".to_string(),
                        old_value: status,
                        create_date: chrono::Utc::now(),
                        create_by: user_details.as_ref().map(|u| u.user_id),
                    };
                    self.subscription_log_repo.save(&audit_log).await?;

                    self.subscription_log_repo
                        .log_status_during_provisioning(
                            subscription_id,
                            "Unsubscribe Failed",
                            "UnsubscribeFailed",
                        )
                        .await?;
                }
                Err(ex) => {
                    let error_description = format!("Exception: {} :: Inner Exception: None", ex);
                    self.subscription_log_repo
                        .log_status_during_provisioning(
                            subscription_id,
                            &error_description,
                            "UnsubscribeFailed",
                        )
                        .await?;
                    
                    info!("{}", error_description);

                    self.base
                        .subscription_repo()
                        .update_status_for_subscription(subscription_id, "UnsubscribeFailed", true)
                        .await?;

                    let audit_log = SubscriptionAuditLogData {
                        id: 0,
                        subscription_id: subscription.id,
                        attribute: SubscriptionLogAttributes::Status.to_string(),
                        new_value: "UnsubscribeFailed".to_string(),
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

