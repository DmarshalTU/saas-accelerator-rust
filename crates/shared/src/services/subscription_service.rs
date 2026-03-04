use async_trait::async_trait;
use crate::models::{PurchaserResult, SubscriptionResult, TermResult};
use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;
use std::sync::Arc;

/// Subscription repository trait for dependency injection
#[async_trait]
pub trait SubscriptionRepositoryTrait: Send + Sync {
    async fn get_by_amp_subscription_id(
        &self,
        amp_subscription_id: Uuid,
    ) -> Result<Option<SubscriptionData>, String>;
    
    async fn get_by_amp_subscription_id_with_deactivated(
        &self,
        amp_subscription_id: Uuid,
        include_deactivated: bool,
    ) -> Result<Option<SubscriptionData>, String>;
    
    async fn get_subscriptions_by_email_address(
        &self,
        email_address: &str,
        subscription_id: Option<Uuid>,
        include_deactivated: bool,
    ) -> Result<Vec<SubscriptionData>, String>;
    
    async fn save(&self, subscription: &SubscriptionData) -> Result<i32, String>;
    
    async fn update_status_for_subscription(
        &self,
        subscription_id: Uuid,
        status: &str,
        is_active: bool,
    ) -> Result<(), String>;
    
    async fn update_plan_for_subscription(
        &self,
        subscription_id: Uuid,
        plan_id: &str,
    ) -> Result<(), String>;
    
    async fn update_quantity_for_subscription(
        &self,
        subscription_id: Uuid,
        quantity: i32,
    ) -> Result<(), String>;
    
    async fn get_all(&self) -> Result<Vec<SubscriptionData>, String>;
}

/// Plan repository trait for dependency injection (re-export from `plan_service`)
pub use super::plan_service::{PlanData, PlanRepositoryForService as PlanRepositoryTrait};

/// Subscription service trait matching the original C# `SubscriptionService`
#[async_trait]
pub trait SubscriptionServiceTrait: Send + Sync {
    async fn add_or_update_partner_subscriptions(
        &self,
        subscription_detail: &SubscriptionResult,
        customer_user_id: Option<i32>,
    ) -> Result<i32, String>;

    async fn update_state_of_subscription(
        &self,
        subscription_id: Uuid,
        status: &str,
        is_activate: bool,
    ) -> Result<(), String>;

    fn is_subscription_deleted(&self, status: &str) -> bool;

    async fn get_partner_subscription(
        &self,
        partner_email_address: &str,
        subscription_id: Option<Uuid>,
        include_unsubscribed: bool,
    ) -> Result<Vec<SubscriptionResultExtension>, String>;

    async fn get_subscriptions_by_subscription_id(
        &self,
        subscription_id: Uuid,
        include_unsubscribed: bool,
    ) -> Result<SubscriptionResultExtension, String>;

    async fn prepare_subscription_response(
        &self,
        subscription: &SubscriptionData,
    ) -> Result<SubscriptionResultExtension, String>;

    async fn update_subscription_plan(
        &self,
        subscription_id: Uuid,
        plan_id: &str,
    ) -> Result<(), String>;

    async fn update_subscription_quantity(
        &self,
        subscription_id: Uuid,
        quantity: i32,
    ) -> Result<(), String>;

    async fn get_all_subscription_plans(&self) -> Result<Vec<PlanDetailResult>, String>;

    async fn get_active_subscriptions_with_metered_plan(
        &self,
    ) -> Result<Vec<SubscriptionData>, String>;
}

/// Subscription data for internal use
#[derive(Debug, Clone)]
pub struct SubscriptionData {
    pub id: i32,
    pub amp_subscription_id: Uuid,
    pub subscription_status: String,
    pub amp_plan_id: String,
    pub amp_offer_id: String,
    pub amp_quantity: i32,
    pub is_active: Option<bool>,
    pub user_id: Option<i32>,
    pub name: Option<String>,
    pub purchaser_email: Option<String>,
    pub purchaser_tenant_id: Option<Uuid>,
    pub term: Option<String>,
    pub start_date: Option<chrono::DateTime<chrono::Utc>>,
    pub end_date: Option<chrono::DateTime<chrono::Utc>>,
    pub create_date: Option<chrono::DateTime<chrono::Utc>>,
    pub customer_email_address: Option<String>,
    pub customer_name: Option<String>,
}

/// Subscription Result Extension - matches C# `SubscriptionResultExtension`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionResultExtension {
    pub id: Uuid,
    pub subscribe_id: i32,
    pub plan_id: String,
    pub offer_id: String,
    pub term: TermResult,
    pub quantity: i32,
    pub name: String,
    pub subscription_status: SubscriptionStatusEnumExtension,
    pub is_active_subscription: bool,
    pub customer_email_address: Option<String>,
    pub customer_name: Option<String>,
    pub is_metering_supported: bool,
    pub purchaser: PurchaserResult,
}

/// Plan Detail Result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanDetailResult {
    pub id: i32,
    pub plan_id: String,
    pub display_name: String,
    pub description: String,
}

/// Subscription Status Enum Extension
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub enum SubscriptionStatusEnumExtension {
    PendingFulfillmentStart,
    Subscribed,
    Unsubscribed,
    UnRecognized,
    PendingActivation,
    PendingUnsubscribe,
    ActivationFailed,
    UnsubscribeFailed,
    Suspend,
}

impl From<&str> for SubscriptionStatusEnumExtension {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "pendingfulfillmentstart" => Self::PendingFulfillmentStart,
            "subscribed" => Self::Subscribed,
            "unsubscribed" => Self::Unsubscribed,
            "pendingactivation" => Self::PendingActivation,
            "pendingunsubscribe" => Self::PendingUnsubscribe,
            "activationfailed" => Self::ActivationFailed,
            "unsubscribefailed" => Self::UnsubscribeFailed,
            "suspend" | "suspended" => Self::Suspend,
            _ => Self::UnRecognized,
        }
    }
}

impl fmt::Display for SubscriptionStatusEnumExtension {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::PendingFulfillmentStart => "PendingFulfillmentStart",
            Self::Subscribed => "Subscribed",
            Self::Unsubscribed => "Unsubscribed",
            Self::PendingActivation => "PendingActivation",
            Self::PendingUnsubscribe => "PendingUnsubscribe",
            Self::ActivationFailed => "ActivationFailed",
            Self::UnsubscribeFailed => "UnsubscribeFailed",
            Self::Suspend => "Suspend",
            Self::UnRecognized => "UnRecognized",
        })
    }
}

impl Default for SubscriptionResultExtension {
    fn default() -> Self {
        Self {
            id: Uuid::nil(),
            subscribe_id: 0,
            plan_id: String::new(),
            offer_id: String::new(),
            term: TermResult {
                term_unit: "P1M".to_string(),
                start_date: chrono::Utc::now(),
                end_date: None,
            },
            quantity: 0,
            name: String::new(),
            subscription_status: SubscriptionStatusEnumExtension::UnRecognized,
            is_active_subscription: false,
            customer_email_address: None,
            customer_name: None,
            is_metering_supported: false,
            purchaser: PurchaserResult {
                tenant_id: Uuid::nil(),
                email_id: None,
                object_id: None,
                puid: None,
            },
        }
    }
}

/// Concrete implementation of `SubscriptionService`
pub struct SubscriptionServiceImpl {
    subscription_repo: Arc<dyn SubscriptionRepositoryTrait>,
    plan_repo: Arc<dyn PlanRepositoryTrait>,
    current_user_id: i32,
}

impl SubscriptionServiceImpl {
    pub fn new(
        subscription_repo: Arc<dyn SubscriptionRepositoryTrait>,
        plan_repo: Arc<dyn PlanRepositoryTrait>,
        current_user_id: i32,
    ) -> Self {
        Self {
            subscription_repo,
            plan_repo,
            current_user_id,
        }
    }
}

#[async_trait]
impl SubscriptionServiceTrait for SubscriptionServiceImpl {
    async fn add_or_update_partner_subscriptions(
        &self,
        subscription_detail: &SubscriptionResult,
        customer_user_id: Option<i32>,
    ) -> Result<i32, String> {
        let is_active = !self.is_subscription_deleted(&subscription_detail.saas_subscription_status);
        
        let subscription = SubscriptionData {
            id: 0,
            amp_subscription_id: subscription_detail.id,
            subscription_status: subscription_detail.saas_subscription_status.clone(),
            amp_plan_id: subscription_detail.plan_id.clone(),
            amp_offer_id: subscription_detail.offer_id.clone(),
            amp_quantity: subscription_detail.quantity.unwrap_or(0),
            is_active: Some(is_active),
            user_id: Some(customer_user_id.unwrap_or(self.current_user_id)),
            name: Some(subscription_detail.name.clone()),
            purchaser_email: subscription_detail.purchaser.email_id.clone(),
            purchaser_tenant_id: Some(subscription_detail.purchaser.tenant_id),
            term: Some(subscription_detail.term.term_unit.clone()),
            start_date: Some(subscription_detail.term.start_date),
            end_date: subscription_detail.term.end_date,
            create_date: Some(chrono::Utc::now()),
            customer_email_address: None,
            customer_name: None,
        };
        
        self.subscription_repo.save(&subscription).await
    }

    async fn update_state_of_subscription(
        &self,
        subscription_id: Uuid,
        status: &str,
        is_activate: bool,
    ) -> Result<(), String> {
        self.subscription_repo
            .update_status_for_subscription(subscription_id, status, is_activate)
            .await
    }

    fn is_subscription_deleted(&self, status: &str) -> bool {
        status.eq_ignore_ascii_case("Unsubscribed")
    }

    async fn get_partner_subscription(
        &self,
        partner_email_address: &str,
        subscription_id: Option<Uuid>,
        include_unsubscribed: bool,
    ) -> Result<Vec<SubscriptionResultExtension>, String> {
        let subscriptions = self
            .subscription_repo
            .get_subscriptions_by_email_address(
                partner_email_address,
                subscription_id,
                include_unsubscribed,
            )
            .await?;

        let mut result = Vec::new();
        for subscription in subscriptions {
            if let Ok(extension) = self.prepare_subscription_response(&subscription).await
                && extension.subscribe_id > 0
            {
                result.push(extension);
            }
        }
        Ok(result)
    }

    async fn get_subscriptions_by_subscription_id(
        &self,
        subscription_id: Uuid,
        include_unsubscribed: bool,
    ) -> Result<SubscriptionResultExtension, String> {
        let subscription = self
            .subscription_repo
            .get_by_amp_subscription_id_with_deactivated(subscription_id, include_unsubscribed)
            .await?;

        if let Some(sub) = subscription {
            self.prepare_subscription_response(&sub).await
        } else {
            Ok(SubscriptionResultExtension::default())
        }
    }

    async fn prepare_subscription_response(
        &self,
        subscription: &SubscriptionData,
    ) -> Result<SubscriptionResultExtension, String> {
        let is_metering_supported = if subscription.amp_plan_id.is_empty() {
            false
        } else {
            self.plan_repo.get_all().await.is_ok_and(|plans| {
                plans
                    .iter()
                    .find(|p| p.plan_id == subscription.amp_plan_id)
                    .and_then(|p| p.is_metering_supported)
                    .unwrap_or(false)
            })
        };

        let term_unit = subscription.term.as_deref().unwrap_or("P1M");

        Ok(SubscriptionResultExtension {
            id: subscription.amp_subscription_id,
            subscribe_id: subscription.id,
            plan_id: subscription.amp_plan_id.clone(),
            offer_id: subscription.amp_offer_id.clone(),
            term: TermResult {
                term_unit: term_unit.to_string(),
                start_date: subscription
                    .start_date
                    .unwrap_or_else(chrono::Utc::now),
                end_date: subscription.end_date,
            },
            quantity: subscription.amp_quantity,
            name: subscription.name.clone().unwrap_or_default(),
            subscription_status: SubscriptionStatusEnumExtension::from(subscription.subscription_status.as_str()),
            is_active_subscription: subscription.is_active.unwrap_or(false),
            customer_email_address: subscription.customer_email_address.clone(),
            customer_name: subscription.customer_name.clone(),
            is_metering_supported,
            purchaser: PurchaserResult {
                tenant_id: subscription.purchaser_tenant_id.unwrap_or(Uuid::nil()),
                email_id: subscription.purchaser_email.clone(),
                object_id: None,
                puid: None,
            },
        })
    }

    async fn update_subscription_plan(
        &self,
        subscription_id: Uuid,
        plan_id: &str,
    ) -> Result<(), String> {
        if subscription_id != Uuid::nil() && !plan_id.is_empty() {
            self.subscription_repo
                .update_plan_for_subscription(subscription_id, plan_id)
                .await
        } else {
            Ok(())
        }
    }

    async fn update_subscription_quantity(
        &self,
        subscription_id: Uuid,
        quantity: i32,
    ) -> Result<(), String> {
        if subscription_id != Uuid::nil() && quantity > 0 {
            self.subscription_repo
                .update_quantity_for_subscription(subscription_id, quantity)
                .await
        } else {
            Ok(())
        }
    }

    async fn get_all_subscription_plans(&self) -> Result<Vec<PlanDetailResult>, String> {
        let plans = self.plan_repo.get_all().await?;
        Ok(plans
            .into_iter()
            .map(|plan| PlanDetailResult {
                id: plan.id,
                plan_id: plan.plan_id,
                display_name: plan.display_name.unwrap_or_default(),
                description: plan.description.unwrap_or_default(),
            })
            .collect())
    }

    async fn get_active_subscriptions_with_metered_plan(
        &self,
    ) -> Result<Vec<SubscriptionData>, String> {
        let all_subscriptions = self.subscription_repo.get_all().await?;
        let all_plans = self.plan_repo.get_all().await?;

        let metered_plan_ids: std::collections::HashSet<_> = all_plans
            .into_iter()
            .filter(|p| p.is_metering_supported == Some(true))
            .map(|p| p.plan_id)
            .collect();

        let metered_subscriptions: Vec<SubscriptionData> = all_subscriptions
            .into_iter()
            .filter(|s| s.subscription_status.eq_ignore_ascii_case("Subscribed"))
            .filter(|s| metered_plan_ids.contains(&s.amp_plan_id))
            .collect();

        Ok(metered_subscriptions)
    }
}
