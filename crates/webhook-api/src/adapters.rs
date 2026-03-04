use async_trait::async_trait;
use std::sync::Arc;
use uuid::Uuid;
use shared::services::{
    subscription_service::{SubscriptionRepositoryTrait, SubscriptionData},
    plan_service::{PlanRepositoryForService, PlanData},
    application_log_service::{ApplicationLogRepositoryTrait, ApplicationLogData, ApplicationLogServiceTrait},
    status_handlers::{
        SubscriptionRepositoryHelper, UserRepositoryHelper,
        SubscriptionData as StatusHandlerSubscriptionData, PlanData as StatusHandlerPlanData,
        UserData as StatusHandlerUserData,
    },
};
use data::repositories::*;
use data::models::*;

pub struct SubscriptionRepositoryAdapter {
    repo: Arc<dyn SubscriptionRepository>,
}

impl SubscriptionRepositoryAdapter {
    pub fn new(repo: Arc<dyn SubscriptionRepository>) -> Self {
        Self { repo }
    }
}

#[async_trait]
impl SubscriptionRepositoryTrait for SubscriptionRepositoryAdapter {
    async fn get_by_amp_subscription_id(
        &self,
        amp_subscription_id: Uuid,
    ) -> Result<Option<SubscriptionData>, String> {
        let sub = self.repo
            .get_by_amp_subscription_id(amp_subscription_id)
            .await
            .map_err(|e| e.to_string())?;
        Ok(sub.map(|s| SubscriptionData {
            id: s.id,
            amp_subscription_id: s.amp_subscription_id,
            subscription_status: s.subscription_status,
            amp_plan_id: s.amp_plan_id,
            amp_offer_id: s.amp_offer_id,
            amp_quantity: s.amp_quantity,
            is_active: s.is_active,
            user_id: s.user_id,
            name: s.name,
            purchaser_email: s.purchaser_email,
            purchaser_tenant_id: s.purchaser_tenant_id,
            term: s.term,
            start_date: s.start_date,
            end_date: s.end_date,
            create_date: s.create_date,
            customer_email_address: None,
            customer_name: None,
        }))
    }

    async fn get_by_amp_subscription_id_with_deactivated(
        &self,
        amp_subscription_id: Uuid,
        include_deactivated: bool,
    ) -> Result<Option<SubscriptionData>, String> {
        let sub = self.repo
            .get_by_amp_subscription_id_with_deactivated(amp_subscription_id, include_deactivated)
            .await
            .map_err(|e| e.to_string())?;
        Ok(sub.map(|s| SubscriptionData {
            id: s.id,
            amp_subscription_id: s.amp_subscription_id,
            subscription_status: s.subscription_status,
            amp_plan_id: s.amp_plan_id,
            amp_offer_id: s.amp_offer_id,
            amp_quantity: s.amp_quantity,
            is_active: s.is_active,
            user_id: s.user_id,
            name: s.name,
            purchaser_email: s.purchaser_email,
            purchaser_tenant_id: s.purchaser_tenant_id,
            term: s.term,
            start_date: s.start_date,
            end_date: s.end_date,
            create_date: s.create_date,
            customer_email_address: None,
            customer_name: None,
        }))
    }

    async fn get_subscriptions_by_email_address(
        &self,
        email_address: &str,
        subscription_id: Option<Uuid>,
        include_deactivated: bool,
    ) -> Result<Vec<SubscriptionData>, String> {
        let subs = self.repo
            .get_subscriptions_by_email_address(email_address, subscription_id, include_deactivated)
            .await
            .map_err(|e| e.to_string())?;
        Ok(subs.into_iter().map(|s| SubscriptionData {
            id: s.id,
            amp_subscription_id: s.amp_subscription_id,
            subscription_status: s.subscription_status,
            amp_plan_id: s.amp_plan_id,
            amp_offer_id: s.amp_offer_id,
            amp_quantity: s.amp_quantity,
            is_active: s.is_active,
            user_id: s.user_id,
            name: s.name,
            purchaser_email: s.purchaser_email,
            purchaser_tenant_id: s.purchaser_tenant_id,
            term: s.term,
            start_date: s.start_date,
            end_date: s.end_date,
            create_date: s.create_date,
            customer_email_address: None,
            customer_name: None,
        }).collect())
    }

    async fn save(&self, subscription: &SubscriptionData) -> Result<i32, String> {
        let sub = Subscription {
            id: subscription.id,
            amp_subscription_id: subscription.amp_subscription_id,
            subscription_status: subscription.subscription_status.clone(),
            amp_plan_id: subscription.amp_plan_id.clone(),
            amp_offer_id: subscription.amp_offer_id.clone(),
            amp_quantity: subscription.amp_quantity,
            is_active: subscription.is_active,
            user_id: subscription.user_id,
            name: subscription.name.clone(),
            purchaser_email: subscription.purchaser_email.clone(),
            purchaser_tenant_id: subscription.purchaser_tenant_id,
            term: subscription.term.clone(),
            start_date: subscription.start_date,
            end_date: subscription.end_date,
            create_date: subscription.create_date,
            create_by: None,
            modify_date: None,
        };
        self.repo.save(&sub).await.map_err(|e| e.to_string())
    }

    async fn update_status_for_subscription(
        &self,
        subscription_id: Uuid,
        status: &str,
        is_active: bool,
    ) -> Result<(), String> {
        self.repo
            .update_status_for_subscription(subscription_id, status, is_active)
            .await
            .map_err(|e| e.to_string())
    }

    async fn update_plan_for_subscription(
        &self,
        subscription_id: Uuid,
        plan_id: &str,
    ) -> Result<(), String> {
        self.repo
            .update_plan_for_subscription(subscription_id, plan_id)
            .await
            .map_err(|e| e.to_string())
    }

    async fn update_quantity_for_subscription(
        &self,
        subscription_id: Uuid,
        quantity: i32,
    ) -> Result<(), String> {
        self.repo
            .update_quantity_for_subscription(subscription_id, quantity)
            .await
            .map_err(|e| e.to_string())
    }

    async fn get_all(&self) -> Result<Vec<SubscriptionData>, String> {
        let subs = self.repo.get_all().await.map_err(|e| e.to_string())?;
        Ok(subs.into_iter().map(|s| SubscriptionData {
            id: s.id,
            amp_subscription_id: s.amp_subscription_id,
            subscription_status: s.subscription_status,
            amp_plan_id: s.amp_plan_id,
            amp_offer_id: s.amp_offer_id,
            amp_quantity: s.amp_quantity,
            is_active: s.is_active,
            user_id: s.user_id,
            name: s.name,
            purchaser_email: s.purchaser_email,
            purchaser_tenant_id: s.purchaser_tenant_id,
            term: s.term,
            start_date: s.start_date,
            end_date: s.end_date,
            create_date: s.create_date,
            customer_email_address: None,
            customer_name: None,
        }).collect())
    }
}

pub struct PlanRepositoryAdapter {
    repo: Arc<dyn PlanRepository>,
}

impl PlanRepositoryAdapter {
    pub fn new(repo: Arc<dyn PlanRepository>) -> Self {
        Self { repo }
    }
}

#[async_trait]
impl PlanRepositoryForService for PlanRepositoryAdapter {
    async fn get_plans_by_user(&self) -> Result<Vec<PlanData>, String> {
        let plans = self.repo.get_plans_by_user().await.map_err(|e| e.to_string())?;
        Ok(plans.into_iter().map(|p| PlanData {
            id: p.id,
            plan_id: p.plan_id,
            description: p.description,
            display_name: p.display_name,
            is_metering_supported: p.is_metering_supported,
            is_per_user: p.is_per_user,
            plan_guid: p.plan_guid,
            offer_id: p.offer_id,
        }).collect())
    }

    async fn get_by_internal_reference(&self, plan_guid: Uuid) -> Result<Option<PlanData>, String> {
        let plan = self.repo.get_by_internal_reference(plan_guid).await.map_err(|e| e.to_string())?;
        Ok(plan.map(|p| PlanData {
            id: p.id,
            plan_id: p.plan_id,
            description: p.description,
            display_name: p.display_name,
            is_metering_supported: p.is_metering_supported,
            is_per_user: p.is_per_user,
            plan_guid: p.plan_guid,
            offer_id: p.offer_id,
        }))
    }

    async fn get_all(&self) -> Result<Vec<PlanData>, String> {
        let plans = self.repo.get_all().await.map_err(|e| e.to_string())?;
        Ok(plans.into_iter().map(|p| PlanData {
            id: p.id,
            plan_id: p.plan_id,
            description: p.description,
            display_name: p.display_name,
            is_metering_supported: p.is_metering_supported,
            is_per_user: p.is_per_user,
            plan_guid: p.plan_guid,
            offer_id: p.offer_id,
        }).collect())
    }
}

pub struct ApplicationLogRepositoryAdapter {
    repo: Arc<dyn ApplicationLogRepository>,
}

impl ApplicationLogRepositoryAdapter {
    pub fn new(repo: Arc<dyn ApplicationLogRepository>) -> Self {
        Self { repo }
    }
}

#[async_trait]
impl ApplicationLogRepositoryTrait for ApplicationLogRepositoryAdapter {
    async fn add_log(&self, log_detail: &ApplicationLogData) -> Result<i32, String> {
        let log = ApplicationLog {
            id: log_detail.id,
            action_time: log_detail.action_time,
            log_detail: log_detail.log_detail.clone(),
        };
        self.repo.add_log(&log).await.map_err(|e| e.to_string())
    }

    async fn get_logs(&self) -> Result<Vec<ApplicationLogData>, String> {
        let logs = self.repo.get_logs().await.map_err(|e| e.to_string())?;
        Ok(logs.into_iter().map(|l| ApplicationLogData {
            id: l.id,
            action_time: l.action_time,
            log_detail: l.log_detail,
        }).collect())
    }
}

pub struct SubscriptionRepositoryAdapterForStatusHandler {
    repo: Arc<dyn SubscriptionRepository>,
}

impl SubscriptionRepositoryAdapterForStatusHandler {
    pub fn new(repo: Arc<dyn SubscriptionRepository>) -> Self {
        Self { repo }
    }
}

#[async_trait]
impl SubscriptionRepositoryHelper for SubscriptionRepositoryAdapterForStatusHandler {
    async fn get_by_amp_subscription_id(
        &self,
        amp_subscription_id: Uuid,
    ) -> Result<Option<StatusHandlerSubscriptionData>, String> {
        let sub = self.repo
            .get_by_amp_subscription_id(amp_subscription_id)
            .await
            .map_err(|e| e.to_string())?;
        Ok(sub.map(|s| StatusHandlerSubscriptionData {
            id: s.id,
            amp_subscription_id: s.amp_subscription_id,
            subscription_status: s.subscription_status,
            amp_plan_id: s.amp_plan_id,
            amp_offer_id: s.amp_offer_id,
            amp_quantity: s.amp_quantity,
            is_active: s.is_active,
            user_id: s.user_id,
            name: s.name,
            purchaser_email: s.purchaser_email,
            purchaser_tenant_id: s.purchaser_tenant_id,
            term: s.term,
            start_date: s.start_date,
            end_date: s.end_date,
            create_date: s.create_date,
            customer_email_address: None,
            customer_name: None,
        }))
    }

    async fn update_status_for_subscription(
        &self,
        subscription_id: Uuid,
        status: &str,
        is_active: bool,
    ) -> Result<(), String> {
        self.repo
            .update_status_for_subscription(subscription_id, status, is_active)
            .await
            .map_err(|e| e.to_string())
    }
}

pub struct PlanRepositoryAdapterForStatusHandler {
    repo: Arc<dyn PlanRepository>,
}

impl PlanRepositoryAdapterForStatusHandler {
    pub fn new(repo: Arc<dyn PlanRepository>) -> Self {
        Self { repo }
    }
}

#[async_trait]
impl shared::services::status_handlers::PlanRepositoryHelper for PlanRepositoryAdapterForStatusHandler {
    async fn get_by_plan_id(&self, plan_id: &str) -> Result<Option<StatusHandlerPlanData>, String> {
        let plan = self.repo.get_by_plan_id(plan_id).await.map_err(|e| e.to_string())?;
        Ok(plan.map(|p| StatusHandlerPlanData {
            id: p.id,
            plan_id: p.plan_id,
            description: p.description,
            display_name: p.display_name,
            is_metering_supported: p.is_metering_supported,
            is_per_user: p.is_per_user,
            plan_guid: p.plan_guid,
            offer_id: p.offer_id,
        }))
    }

}

pub struct UserRepositoryAdapterForStatusHandler {
    repo: Arc<dyn UserRepository>,
}

impl UserRepositoryAdapterForStatusHandler {
    pub fn new(repo: Arc<dyn UserRepository>) -> Self {
        Self { repo }
    }
}

#[async_trait]
impl UserRepositoryHelper for UserRepositoryAdapterForStatusHandler {
    async fn get_by_id(&self, user_id: i32) -> Result<Option<StatusHandlerUserData>, String> {
        let user = self.repo.get_by_id(user_id).await.map_err(|e| e.to_string())?;
        Ok(user.map(|u| StatusHandlerUserData {
            user_id: u.user_id,
            email_address: u.email_address,
            created_date: u.created_date,
            full_name: u.full_name,
        }))
    }
}

pub struct ApplicationConfigRepositoryAdapter {
    repo: Arc<dyn ApplicationConfigRepository>,
}

impl ApplicationConfigRepositoryAdapter {
    pub fn new(repo: Arc<dyn ApplicationConfigRepository>) -> Self {
        Self { repo }
    }
}

#[async_trait]
impl shared::services::email_helper::ApplicationConfigRepositoryForEmailHelper for ApplicationConfigRepositoryAdapter {
    async fn get_by_name(&self, name: &str) -> Result<Option<String>, String> {
        self.repo.get_by_name(name).await.map_err(|e| e.to_string())
    }
}

pub struct EmailTemplateRepositoryAdapter {
    repo: Arc<dyn EmailTemplateRepository>,
}

impl EmailTemplateRepositoryAdapter {
    pub fn new(repo: Arc<dyn EmailTemplateRepository>) -> Self {
        Self { repo }
    }
}

#[async_trait]
impl shared::services::email_helper::EmailTemplateRepositoryForEmailHelper for EmailTemplateRepositoryAdapter {
    async fn get_email_body_for_subscription(
        &self,
        subscription_id: uuid::Uuid,
        process_status: &str,
    ) -> Result<String, String> {
        self.repo
            .get_email_body_for_subscription(subscription_id, process_status)
            .await
            .map_err(|e| e.to_string())
    }

    async fn get_template_for_status(
        &self,
        status: &str,
    ) -> Result<Option<shared::services::email_helper::EmailTemplateData>, String> {
        let template = self.repo.get_template_for_status(status).await.map_err(|e| e.to_string())?;
        Ok(template.map(|t| shared::services::email_helper::EmailTemplateData {
            to_recipients: t.to_recipients,
            cc: t.cc,
            bcc: t.bcc,
            subject: t.subject,
            template_body: t.template_body,
        }))
    }
}

pub struct EventsRepositoryAdapter {
    repo: Arc<dyn EventsRepository>,
}

impl EventsRepositoryAdapter {
    pub fn new(repo: Arc<dyn EventsRepository>) -> Self {
        Self { repo }
    }
}

#[async_trait]
impl shared::services::email_helper::EventsRepositoryForEmailHelper for EventsRepositoryAdapter {
    async fn get_by_name(&self, name: &str) -> Result<Option<shared::services::email_helper::EventsData>, String> {
        let event = self.repo.get_by_name(name).await.map_err(|e| e.to_string())?;
        Ok(event.map(|e| shared::services::email_helper::EventsData {
            id: e.id,
        }))
    }
}

pub struct PlanEventsMappingRepositoryAdapter {
    repo: Arc<dyn PlanEventsMappingRepository>,
}

impl PlanEventsMappingRepositoryAdapter {
    pub fn new(repo: Arc<dyn PlanEventsMappingRepository>) -> Self {
        Self { repo }
    }
}

#[async_trait]
impl shared::services::email_helper::PlanEventsMappingRepositoryForEmailHelper for PlanEventsMappingRepositoryAdapter {
    async fn get_plan_event(
        &self,
        plan_id: uuid::Uuid,
        event_id: i32,
    ) -> Result<Option<shared::services::email_helper::PlanEventsMappingData>, String> {
        let mapping = self.repo.get_plan_event(plan_id, event_id).await.map_err(|e| e.to_string())?;
        Ok(mapping.map(|m| shared::services::email_helper::PlanEventsMappingData {
            success_state_emails: m.success_state_emails,
            copy_to_customer: m.copy_to_customer,
        }))
    }
}

pub struct ApplicationConfigRepositoryAdapterForNotification {
    repo: Arc<dyn ApplicationConfigRepository>,
}

impl ApplicationConfigRepositoryAdapterForNotification {
    pub fn new(repo: Arc<dyn ApplicationConfigRepository>) -> Self {
        Self { repo }
    }
}

#[async_trait]
impl shared::services::notification_status_handler::ApplicationConfigRepositoryForNotification for ApplicationConfigRepositoryAdapterForNotification {
    async fn get_by_name(&self, name: &str) -> Result<Option<String>, String> {
        self.repo.get_by_name(name).await.map_err(|e| e.to_string())
    }
}

pub struct ApplicationLogServiceAdapterForEmail {
    service: Arc<dyn ApplicationLogServiceTrait>,
}

impl ApplicationLogServiceAdapterForEmail {
    pub fn new(service: Arc<dyn ApplicationLogServiceTrait>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl shared::services::email_service::ApplicationLogServiceForEmail for ApplicationLogServiceAdapterForEmail {
    async fn add_application_log(&self, log_message: &str) -> Result<(), String> {
        self.service.add_application_log(log_message).await
    }
}
