use async_trait::async_trait;
use uuid::Uuid;
use std::sync::Arc;
use tracing::info;
use super::status_handlers::{AbstractSubscriptionStatusHandler, SubscriptionStatusHandler};
use super::email_service::EmailServiceTrait;
use super::email_helper::EmailHelper;
use super::subscription_service::SubscriptionStatusEnumExtension;

/// Notification Status Handler
pub struct NotificationStatusHandler {
    base: AbstractSubscriptionStatusHandler,
    email_helper: Arc<EmailHelper>,
    email_service: Arc<dyn EmailServiceTrait>,
    application_config_repo: Arc<dyn ApplicationConfigRepositoryForNotification>,
}

/// Application configuration repository trait for notification handler
#[async_trait]
pub trait ApplicationConfigRepositoryForNotification: Send + Sync {
    async fn get_by_name(&self, name: &str) -> Result<Option<String>, String>;
}

impl NotificationStatusHandler {
    pub fn new(
        base: AbstractSubscriptionStatusHandler,
        email_helper: Arc<EmailHelper>,
        email_service: Arc<dyn EmailServiceTrait>,
        application_config_repo: Arc<dyn ApplicationConfigRepositoryForNotification>,
    ) -> Self {
        Self {
            base,
            email_helper,
            email_service,
            application_config_repo,
        }
    }
}

#[async_trait]
impl SubscriptionStatusHandler for NotificationStatusHandler {
    async fn process(&self, subscription_id: Uuid) -> Result<(), String> {
        info!("NotificationStatusHandler Process for subscription: {}", subscription_id);
        
        let subscription = self.base.get_subscription_by_id(subscription_id).await?;
        info!("Get PlanById");
        let plan_details = self.base.get_plan_by_id(&subscription.amp_plan_id).await?;
        let plan_guid = plan_details
            .ok_or_else(|| format!("Plan not found for plan_id: {}", subscription.amp_plan_id))?
            .plan_guid;
        info!("Get User");
        let user_details = self.base.get_user_by_id(subscription.user_id).await?;

        let plan_event_name = if subscription.subscription_status == SubscriptionStatusEnumExtension::Unsubscribed.to_string() ||
            subscription.subscription_status == SubscriptionStatusEnumExtension::UnsubscribeFailed.to_string() {
            "Unsubscribe"
        } else {
            "Activate"
        };

        let process_status = if subscription.subscription_status == SubscriptionStatusEnumExtension::ActivationFailed.to_string() ||
            subscription.subscription_status == SubscriptionStatusEnumExtension::UnsubscribeFailed.to_string() {
            "failure"
        } else {
            "success"
        };

        let is_email_enabled_for_unsubscription = self
            .application_config_repo
            .get_by_name("IsEmailEnabledForUnsubscription")
            .await?
            .unwrap_or_else(|| "false".to_string())
            .parse::<bool>()
            .unwrap_or(false);

        let is_email_enabled_for_pending_activation = self
            .application_config_repo
            .get_by_name("IsEmailEnabledForPendingActivation")
            .await?
            .unwrap_or_else(|| "false".to_string())
            .parse::<bool>()
            .unwrap_or(false);

        let is_email_enabled_for_subscription_activation = self
            .application_config_repo
            .get_by_name("IsEmailEnabledForSubscriptionActivation")
            .await?
            .unwrap_or_else(|| "false".to_string())
            .parse::<bool>()
            .unwrap_or(false);

        let trigger_email = (plan_event_name == "Activate"
            && ((is_email_enabled_for_pending_activation
                && subscription.subscription_status
                    == SubscriptionStatusEnumExtension::PendingActivation.to_string())
                || (is_email_enabled_for_subscription_activation
                    && subscription.subscription_status
                        != SubscriptionStatusEnumExtension::PendingActivation.to_string())))
            || (plan_event_name == "Unsubscribe" && is_email_enabled_for_unsubscription);

        if trigger_email {
            let email_content = self
                .email_helper
                .prepare_email_content(
                    subscription_id,
                    plan_guid,
                    process_status,
                    plan_event_name,
                    &subscription.subscription_status,
                )
                .await?;

            self.email_service.send_email(&email_content).await?;

            if email_content.copy_to_customer
                && let Some(ref user) = user_details
                && let Some(ref email_address) = user.email_address
                && !email_address.is_empty()
            {
                let mut customer_email_content = email_content.clone();
                customer_email_content.to_emails = email_address.clone();
                self.email_service.send_email(&customer_email_content).await?;
            }
        }

        Ok(())
    }
}

