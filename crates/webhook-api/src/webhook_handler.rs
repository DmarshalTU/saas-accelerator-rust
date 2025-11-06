use data::models::SubscriptionAuditLog;
use data::repositories::*;
use shared::models::WebhookPayload;
use shared::services::{
    SubscriptionServiceTrait, SubscriptionData, SubscriptionStatusHandler,
    ApplicationLogServiceTrait,
};
use std::sync::Arc;
use tracing::{error, info, warn};
use uuid::Uuid;

pub struct WebhookHandler {
    subscription_service: Arc<dyn SubscriptionServiceTrait>,
    subscription_repo: Arc<dyn data::repositories::SubscriptionRepository>,
    audit_log_repo: Arc<dyn SubscriptionAuditLogRepository>,
    config_repo: Arc<dyn ApplicationConfigRepository>,
    application_log_service: Arc<dyn ApplicationLogServiceTrait>,
    notification_status_handler: Option<Arc<dyn SubscriptionStatusHandler>>,
}


impl WebhookHandler {
    pub fn new(
        subscription_service: Arc<dyn SubscriptionServiceTrait>,
        subscription_repo: Arc<dyn data::repositories::SubscriptionRepository>,
        audit_log_repo: Arc<dyn SubscriptionAuditLogRepository>,
        config_repo: Arc<dyn ApplicationConfigRepository>,
        application_log_service: Arc<dyn ApplicationLogServiceTrait>,
        notification_status_handler: Option<Arc<dyn SubscriptionStatusHandler>>,
    ) -> Self {
        Self {
            subscription_service,
            subscription_repo,
            audit_log_repo,
            config_repo,
            application_log_service,
            notification_status_handler,
        }
    }

    pub async fn handle_unsubscribe(&self, payload: &WebhookPayload) -> Result<(), String> {
        info!("Handling Unsubscribe webhook for subscription {}", payload.subscription_id);

        let subscription_id = Uuid::parse_str(&payload.subscription_id)
            .map_err(|e| format!("Invalid subscription ID: {}", e))?;

        let old_subscription = self
            .subscription_service
            .get_subscriptions_by_subscription_id(subscription_id, false)
            .await?;

        if old_subscription.subscribe_id > 0 {
            self.subscription_service
                .update_state_of_subscription(subscription_id, "Unsubscribed", false)
                .await?;

            let old_sub = self
                .subscription_repo
                .get_by_amp_subscription_id(subscription_id)
                .await
                .map_err(|e| format!("Database error: {}", e))?
                .ok_or_else(|| "Subscription not found".to_string())?;

            let audit_log = SubscriptionAuditLog {
                id: 0,
                subscription_id: old_sub.id,
                attribute: Some("Status".to_string()),
                old_value: Some(old_subscription.subscription_status.to_string()),
                new_value: Some("Unsubscribed".to_string()),
                create_date: Some(chrono::Utc::now()),
                create_by: None,
            };

            self.audit_log_repo.save(&audit_log).await
                .map_err(|e| format!("Failed to create audit log: {}", e))?;

            self.application_log_service
                .add_application_log("Offer Successfully UnSubscribed.")
                .await?;

            if let Some(ref handler) = self.notification_status_handler {
                if let Err(e) = handler.process(subscription_id).await {
                    warn!("NotificationStatusHandler failed: {}", e);
                }
            }

            info!("Subscription {} successfully unsubscribed", subscription_id);
        } else {
            warn!("Subscription {} not found in database", subscription_id);
        }

        Ok(())
    }

    pub async fn handle_change_plan(&self, payload: &WebhookPayload) -> Result<(), String> {
        info!("Handling ChangePlan webhook for subscription {}", payload.subscription_id);

        let subscription_id = Uuid::parse_str(&payload.subscription_id)
            .map_err(|e| format!("Invalid subscription ID: {}", e))?;
        let plan_id = payload.plan_id.as_ref()
            .ok_or_else(|| "Plan ID missing in payload".to_string())?;

        let accept_updates = self.config_repo.get_by_name("AcceptSubscriptionUpdates").await
            .unwrap_or(None)
            .unwrap_or_else(|| "false".to_string());

        let old_subscription = self
            .subscription_service
            .get_subscriptions_by_subscription_id(subscription_id, false)
            .await?;

        if old_subscription.subscribe_id > 0 {
            let old_plan_id = old_subscription.plan_id.clone();
            
            if accept_updates != "true" && old_plan_id != *plan_id {
                let old_sub = self
                    .subscription_repo
                    .get_by_amp_subscription_id(subscription_id)
                    .await
                    .map_err(|e| format!("Database error: {}", e))?
                    .ok_or_else(|| "Subscription not found".to_string())?;

                let audit_log = SubscriptionAuditLog {
                    id: 0,
                    subscription_id: old_sub.id,
                    attribute: Some("Plan".to_string()),
                    old_value: Some(old_plan_id.clone()),
                    new_value: Some(old_plan_id),
                    create_date: Some(chrono::Utc::now()),
                    create_by: None,
                };

                self.audit_log_repo.save(&audit_log).await
                    .map_err(|e| format!("Failed to create audit log: {}", e))?;

                return Err("Plan Change rejected due to Config settings or Subscription not in database".to_string());
            }

            self.subscription_service
                .update_subscription_plan(subscription_id, plan_id)
                .await?;

            self.application_log_service
                .add_application_log("Plan Successfully Changed.")
                .await?;

            let old_sub = self
                .subscription_repo
                .get_by_amp_subscription_id(subscription_id)
                .await
                .map_err(|e| format!("Database error: {}", e))?
                .ok_or_else(|| "Subscription not found".to_string())?;

            let audit_log = SubscriptionAuditLog {
                id: 0,
                subscription_id: old_sub.id,
                attribute: Some("Plan".to_string()),
                old_value: Some(old_plan_id),
                new_value: Some(plan_id.clone()),
                create_date: Some(chrono::Utc::now()),
                create_by: None,
            };

            self.audit_log_repo.save(&audit_log).await
                .map_err(|e| format!("Failed to create audit log: {}", e))?;

            info!("Plan successfully changed for subscription {}", subscription_id);
        } else {
            return Err("Subscription not found in database".to_string());
        }

        Ok(())
    }

    pub async fn handle_change_quantity(&self, payload: &WebhookPayload) -> Result<(), String> {
        info!("Handling ChangeQuantity webhook for subscription {}", payload.subscription_id);

        let subscription_id = Uuid::parse_str(&payload.subscription_id)
            .map_err(|e| format!("Invalid subscription ID: {}", e))?;
        let quantity = payload.quantity
            .ok_or_else(|| "Quantity missing in payload".to_string())? as i32;

        let accept_updates = self.config_repo.get_by_name("AcceptSubscriptionUpdates").await
            .unwrap_or(None)
            .unwrap_or_else(|| "false".to_string());

        let old_subscription = self
            .subscription_service
            .get_subscriptions_by_subscription_id(subscription_id, false)
            .await?;

        if old_subscription.subscribe_id > 0 {
            let old_quantity = old_subscription.quantity;
            
            if accept_updates != "true" && old_quantity != quantity {
                let old_sub = self
                    .subscription_repo
                    .get_by_amp_subscription_id(subscription_id)
                    .await
                    .map_err(|e| format!("Database error: {}", e))?
                    .ok_or_else(|| "Subscription not found".to_string())?;

                let audit_log = SubscriptionAuditLog {
                    id: 0,
                    subscription_id: old_sub.id,
                    attribute: Some("Quantity".to_string()),
                    old_value: Some(old_quantity.to_string()),
                    new_value: Some(old_quantity.to_string()),
                    create_date: Some(chrono::Utc::now()),
                    create_by: None,
                };

                self.audit_log_repo.save(&audit_log).await
                    .map_err(|e| format!("Failed to create audit log: {}", e))?;

                return Err("Quantity Change Request reject due to Config settings or Subscription not in database".to_string());
            }

            self.subscription_service
                .update_subscription_quantity(subscription_id, quantity)
                .await?;

            self.application_log_service
                .add_application_log("Quantity Successfully Changed.")
                .await?;

            let old_sub = self
                .subscription_repo
                .get_by_amp_subscription_id(subscription_id)
                .await
                .map_err(|e| format!("Database error: {}", e))?
                .ok_or_else(|| "Subscription not found".to_string())?;

            let audit_log = SubscriptionAuditLog {
                id: 0,
                subscription_id: old_sub.id,
                attribute: Some("Quantity".to_string()),
                old_value: Some(old_quantity.to_string()),
                new_value: Some(quantity.to_string()),
                create_date: Some(chrono::Utc::now()),
                create_by: None,
            };

            self.audit_log_repo.save(&audit_log).await
                .map_err(|e| format!("Failed to create audit log: {}", e))?;

            info!("Quantity successfully changed for subscription {}", subscription_id);
        } else {
            return Err("Subscription not found in database".to_string());
        }

        Ok(())
    }

    pub async fn handle_suspend(&self, payload: &WebhookPayload) -> Result<(), String> {
        info!("Handling Suspend webhook for subscription {}", payload.subscription_id);

        let subscription_id = Uuid::parse_str(&payload.subscription_id)
            .map_err(|e| format!("Invalid subscription ID: {}", e))?;

        let old_subscription = self
            .subscription_service
            .get_subscriptions_by_subscription_id(subscription_id, false)
            .await?;

        if old_subscription.subscribe_id > 0 {
            self.subscription_service
                .update_state_of_subscription(subscription_id, "Suspend", false)
                .await?;

            self.application_log_service
                .add_application_log("Offer Successfully Suspended.")
                .await?;

            let old_sub = self
                .subscription_repo
                .get_by_amp_subscription_id(subscription_id)
                .await
                .map_err(|e| format!("Database error: {}", e))?
                .ok_or_else(|| "Subscription not found".to_string())?;

            let audit_log = SubscriptionAuditLog {
                id: 0,
                subscription_id: old_sub.id,
                attribute: Some("Status".to_string()),
                old_value: Some(old_subscription.subscription_status.to_string()),
                new_value: Some("Suspend".to_string()),
                create_date: Some(chrono::Utc::now()),
                create_by: None,
            };

            self.audit_log_repo.save(&audit_log).await
                .map_err(|e| format!("Failed to create audit log: {}", e))?;

            info!("Subscription {} successfully suspended", subscription_id);
        }

        Ok(())
    }

    pub async fn handle_reinstate(&self, payload: &WebhookPayload) -> Result<(), String> {
        info!("Handling Reinstate webhook for subscription {}", payload.subscription_id);

        let subscription_id = Uuid::parse_str(&payload.subscription_id)
            .map_err(|e| format!("Invalid subscription ID: {}", e))?;

        let accept_updates = self.config_repo.get_by_name("AcceptSubscriptionUpdates").await
            .unwrap_or(None)
            .unwrap_or_else(|| "false".to_string());

        let old_subscription = self
            .subscription_service
            .get_subscriptions_by_subscription_id(subscription_id, false)
            .await?;

        if old_subscription.subscribe_id > 0 {
            let old_sub = self
                .subscription_repo
                .get_by_amp_subscription_id(subscription_id)
                .await
                .map_err(|e| format!("Database error: {}", e))?
                .ok_or_else(|| "Subscription not found".to_string())?;

            if accept_updates == "true" {
                self.subscription_service
                    .update_state_of_subscription(subscription_id, "Subscribed", true)
                    .await?;

                self.application_log_service
                    .add_application_log("Reinstated Successfully.")
                    .await?;

                let audit_log = SubscriptionAuditLog {
                    id: 0,
                    subscription_id: old_sub.id,
                    attribute: Some("Status".to_string()),
                    old_value: Some(old_subscription.subscription_status.to_string()),
                    new_value: Some("Subscribed".to_string()),
                    create_date: Some(chrono::Utc::now()),
                    create_by: None,
                };

                self.audit_log_repo.save(&audit_log).await
                    .map_err(|e| format!("Failed to create audit log: {}", e))?;

                info!("Subscription {} successfully reinstated", subscription_id);
            } else {
                self.application_log_service
                    .add_application_log("Reinstate Change Request Rejected Successfully.")
                    .await?;

                let audit_log = SubscriptionAuditLog {
                    id: 0,
                    subscription_id: old_sub.id,
                    attribute: Some("Status".to_string()),
                    old_value: Some(old_subscription.subscription_status.to_string()),
                    new_value: Some(old_subscription.subscription_status.to_string()),
                    create_date: Some(chrono::Utc::now()),
                    create_by: None,
                };

                self.audit_log_repo.save(&audit_log).await
                    .map_err(|e| format!("Failed to create audit log: {}", e))?;

                warn!("Reinstate rejected due to configuration settings");
            }
        }

        Ok(())
    }

    pub async fn handle_renew(&self, _payload: &WebhookPayload) -> Result<(), String> {
        info!("Handling Renew webhook");
        self.application_log_service
            .add_application_log("Offer Successfully Renewed.")
            .await?;
        Ok(())
    }

    pub async fn handle_transfer(&self, payload: &WebhookPayload) -> Result<(), String> {
        info!("Handling Transfer webhook for subscription {}", payload.subscription_id);
        Ok(())
    }

    pub async fn handle_unknown_action(&self, payload: &WebhookPayload) -> Result<(), String> {
        self.application_log_service
            .add_application_log(&format!("Offer Received an unknown action: {:?}", payload.action))
            .await?;
        Ok(())
    }
}
