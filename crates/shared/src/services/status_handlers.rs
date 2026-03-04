use async_trait::async_trait;
use std::fmt;
use std::sync::Arc;
use uuid::Uuid;

/// Status handler trait for processing subscription status changes
#[async_trait]
pub trait SubscriptionStatusHandler: Send + Sync {
    async fn process(&self, subscription_id: Uuid) -> Result<(), String>;
}

/// Re-export types from services for use in status handlers
pub use super::subscription_service::{SubscriptionData, SubscriptionStatusEnumExtension};
pub use super::plan_service::PlanData;
pub use super::user_service::UserData;

/// Subscription Log Attributes enum
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SubscriptionLogAttributes {
    Plan = 1,
    Status = 2,
    Quantity = 3,
    Deployment = 4,
}

impl fmt::Display for SubscriptionLogAttributes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Plan => "Plan",
            Self::Status => "Status",
            Self::Quantity => "Quantity",
            Self::Deployment => "Deployment",
        })
    }
}

/// Abstract base class for subscription status handlers - provides helper methods
#[allow(clippy::struct_field_names)]
pub struct AbstractSubscriptionStatusHandler {
    subscription_repo: Arc<dyn SubscriptionRepositoryHelper>,
    plan_repo: Arc<dyn PlanRepositoryHelper>,
    user_repo: Arc<dyn UserRepositoryHelper>,
}

/// Helper traits for abstract status handler
#[async_trait]
pub trait SubscriptionRepositoryHelper: Send + Sync {
    async fn get_by_amp_subscription_id(&self, subscription_id: Uuid) -> Result<Option<SubscriptionData>, String>;
    async fn update_status_for_subscription(&self, subscription_id: Uuid, status: &str, is_active: bool) -> Result<(), String>;
}

#[async_trait]
pub trait PlanRepositoryHelper: Send + Sync {
    async fn get_by_plan_id(&self, plan_id: &str) -> Result<Option<PlanData>, String>;
}

#[async_trait]
pub trait UserRepositoryHelper: Send + Sync {
    async fn get_by_id(&self, user_id: i32) -> Result<Option<UserData>, String>;
}


impl AbstractSubscriptionStatusHandler {
    pub fn new(
        subscription_repo: Arc<dyn SubscriptionRepositoryHelper>,
        plan_repo: Arc<dyn PlanRepositoryHelper>,
        user_repo: Arc<dyn UserRepositoryHelper>,
    ) -> Self {
        Self {
            subscription_repo,
            plan_repo,
            user_repo,
        }
    }

    /// # Errors
    /// Returns an error string if the subscription is not found.
    pub async fn get_subscription_by_id(&self, subscription_id: Uuid) -> Result<SubscriptionData, String> {
        self.subscription_repo
            .get_by_amp_subscription_id(subscription_id)
            .await?
            .ok_or_else(|| format!("Subscription not found: {subscription_id}"))
    }

    /// # Errors
    /// Propagates repository errors.
    pub async fn get_plan_by_id(&self, plan_id: &str) -> Result<Option<PlanData>, String> {
        self.plan_repo.get_by_plan_id(plan_id).await
    }

    /// # Errors
    /// Propagates repository errors.
    pub async fn get_user_by_id(&self, user_id: Option<i32>) -> Result<Option<UserData>, String> {
        if let Some(id) = user_id {
            self.user_repo.get_by_id(id).await
        } else {
            Ok(None)
        }
    }
}

impl AbstractSubscriptionStatusHandler {
    #[must_use]
    pub fn subscription_repo(&self) -> &Arc<dyn SubscriptionRepositoryHelper> {
        &self.subscription_repo
    }
}
