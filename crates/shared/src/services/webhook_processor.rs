use async_trait::async_trait;
use crate::models::{WebhookPayload, WebhookAction};
use crate::config::SaaSApiConfig;

/// Webhook processor trait
#[async_trait]
pub trait WebhookProcessor: Send + Sync {
    async fn process_webhook_notification(
        &self,
        payload: &WebhookPayload,
        config: &SaaSApiConfig,
    ) -> Result<(), String>;
}

/// Webhook handler trait - matches `IWebhookHandler` from original
#[async_trait]
pub trait WebhookHandler: Send + Sync {
    async fn change_plan(&self, payload: &WebhookPayload) -> Result<(), String>;
    async fn change_quantity(&self, payload: &WebhookPayload) -> Result<(), String>;
    async fn reinstated(&self, payload: &WebhookPayload) -> Result<(), String>;
    async fn renewed(&self) -> Result<(), String>;
    async fn suspended(&self, payload: &WebhookPayload) -> Result<(), String>;
    async fn unsubscribed(&self, payload: &WebhookPayload) -> Result<(), String>;
    async fn unknown_action(&self, payload: &WebhookPayload) -> Result<(), String>;
}

/// Webhook processor implementation
pub struct WebhookProcessorImpl {
    webhook_handler: Box<dyn WebhookHandler>,
    // web_notification_service: Option<Box<dyn WebNotificationService>>, // TODO: Port WebNotificationService
}

impl WebhookProcessorImpl {
    #[must_use]
    pub fn new(webhook_handler: Box<dyn WebhookHandler>) -> Self {
        Self {
            webhook_handler,
        }
    }
}

#[async_trait]
impl WebhookProcessor for WebhookProcessorImpl {
    async fn process_webhook_notification(
        &self,
        payload: &WebhookPayload,
        _config: &SaaSApiConfig,
    ) -> Result<(), String> {
        // TODO: Call web notification service when ported
        // if let Some(ref service) = self.web_notification_service {
        //     service.push_external_web_notification(payload).await?;
        // }

        match payload.action {
            WebhookAction::Unsubscribe => {
                self.webhook_handler.unsubscribed(payload).await
            }
            WebhookAction::ChangePlan => {
                self.webhook_handler.change_plan(payload).await
            }
            WebhookAction::ChangeQuantity => {
                self.webhook_handler.change_quantity(payload).await
            }
            WebhookAction::Suspend => {
                self.webhook_handler.suspended(payload).await
            }
            WebhookAction::Reinstate => {
                self.webhook_handler.reinstated(payload).await
            }
            WebhookAction::Renew => {
                self.webhook_handler.renewed().await
            }
            _ => {
                self.webhook_handler.unknown_action(payload).await
            }
        }
    }
}

