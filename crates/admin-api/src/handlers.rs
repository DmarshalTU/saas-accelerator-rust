use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use chrono::Utc;
use data::models::{ApplicationLog, EmailTemplate, KnownUser, MeteredDimension, MeteredPlanSchedulerManagement, SchedulerFrequency, Subscription};
use data::repositories::{KnownUserInsert, MeteredPlanSchedulerInsert};
use data::repositories::{
    ApplicationConfigRepository, ApplicationLogRepository, EmailTemplateRepository,
    KnownUsersRepository, MeteredDimensionsRepository, MeteredPlanSchedulerRepository,
    OfferRepository, PlanRepository, SchedulerFrequencyRepository,
    SubscriptionAuditLogRepository, SubscriptionRepository,
};
use marketplace::fulfillment::FulfillmentApiClient;
use marketplace::metering::MeteringApiClient;
use oauth2::basic::BasicClient;
use serde::Deserialize;
use shared::auth::user_auth::AuthConfig;
use shared::models::MeteringUsageRequest;
use std::sync::Arc;
use tracing::{error, info};
use uuid::Uuid;

pub async fn get_subscriptions(
    State(state): State<AppState>,
) -> Result<Json<Vec<Subscription>>, StatusCode> {
    let subscriptions = state
        .subscription_repo
        .get_all()
        .await
        .map_err(|e| {
            error!("Failed to get subscriptions: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(subscriptions))
}

pub async fn get_subscription(
    State(state): State<AppState>,
    Path(subscription_id): Path<Uuid>,
) -> Result<Json<Subscription>, StatusCode> {
    let subscription = state
        .subscription_repo
        .get_by_amp_subscription_id(subscription_id)
        .await
        .map_err(|e| {
            error!("Failed to get subscription: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    subscription
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND)
}

pub async fn activate_subscription(
    State(state): State<AppState>,
    Path(subscription_id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    info!("Activating subscription {}", subscription_id);

    let subscription = state
        .subscription_repo
        .get_by_amp_subscription_id(subscription_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    state
        .fulfillment_client
        .activate_subscription(subscription_id, &subscription.amp_plan_id)
        .await
        .map_err(|e| {
            error!("Failed to activate subscription: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let mut updated = subscription;
    updated.subscription_status = "Subscribed".to_string();
    updated.is_active = Some(true);

    state
        .subscription_repo
        .update(&updated)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::OK)
}

pub async fn change_plan(
    State(state): State<AppState>,
    Path(subscription_id): Path<Uuid>,
    Json(body): Json<ChangePlanRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    info!("Changing plan for subscription {} to {}", subscription_id, body.plan_id);

    let operation_id = state
        .fulfillment_client
        .update_subscription_plan(subscription_id, &body.plan_id)
        .await
        .map_err(|e| {
            error!("Failed to change plan: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(serde_json::json!({
        "operation_id": operation_id,
        "status": "success"
    })))
}

pub async fn change_quantity(
    State(state): State<AppState>,
    Path(subscription_id): Path<Uuid>,
    Json(body): Json<ChangeQuantityRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    info!("Changing quantity for subscription {} to {}", subscription_id, body.quantity);

    let operation_id = state
        .fulfillment_client
        .update_subscription_quantity(subscription_id, body.quantity)
        .await
        .map_err(|e| {
            error!("Failed to change quantity: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(serde_json::json!({
        "operation_id": operation_id,
        "status": "success"
    })))
}

pub async fn emit_usage_event(
    State(state): State<AppState>,
    Path(subscription_id): Path<Uuid>,
    Json(body): Json<EmitUsageRequest>,
) -> Result<Json<shared::models::MeteringUsageResult>, StatusCode> {
    info!("Emitting usage event for subscription {}", subscription_id);

    let subscription = state
        .subscription_repo
        .get_by_amp_subscription_id(subscription_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let usage_request = MeteringUsageRequest {
        resource_id: subscription_id,
        plan_id: subscription.amp_plan_id.clone(),
        dimension: body.dimension,
        quantity: body.quantity,
        effective_start_time: chrono::Utc::now(),
    };

    let result = state
        .metering_client
        .emit_usage_event(&usage_request)
        .await
        .map_err(|e| {
            error!("Failed to emit usage event: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(result))
}

pub async fn delete_subscription(
    State(state): State<AppState>,
    Path(subscription_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    info!("Deleting subscription {}", subscription_id);

    let subscription = state
        .subscription_repo
        .get_by_amp_subscription_id(subscription_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let operation_id = state
        .fulfillment_client
        .delete_subscription(subscription_id)
        .await
        .map_err(|e| {
            error!("Failed to delete subscription: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let mut updated = subscription;
    updated.subscription_status = "Unsubscribed".to_string();
    updated.is_active = Some(false);

    state
        .subscription_repo
        .update(&updated)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(serde_json::json!({
        "operation_id": operation_id,
        "status": "success"
    })))
}

#[derive(serde::Deserialize)]
pub struct ChangePlanRequest {
    pub plan_id: String,
}

#[derive(serde::Deserialize)]
pub struct ChangeQuantityRequest {
    pub quantity: i32,
}

#[derive(serde::Deserialize)]
pub struct EmitUsageRequest {
    pub dimension: String,
    pub quantity: f64,
}

pub async fn get_subscription_audit_logs(
    State(state): State<AppState>,
    Path(subscription_id): Path<Uuid>,
) -> Result<Json<Vec<data::models::SubscriptionAuditLog>>, StatusCode> {
    let subscription = state
        .subscription_repo
        .get_by_amp_subscription_id(subscription_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let audit_logs = state
        .audit_log_repo
        .get_by_subscription_id(subscription.id)
        .await
        .map_err(|e| {
            error!("Failed to get audit logs: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(audit_logs))
}

pub async fn get_plans(
    State(state): State<AppState>,
) -> Result<Json<Vec<data::models::Plan>>, StatusCode> {
    let plans = state
        .plan_repo
        .get_all()
        .await
        .map_err(|e| {
            error!("Failed to get plans: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(plans))
}

pub async fn get_events(
    State(state): State<AppState>,
) -> Result<Json<Vec<data::models::Events>>, StatusCode> {
    let events = state
        .events_repo
        .get_all()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(events))
}

pub async fn get_plan(
    State(state): State<AppState>,
    Path(plan_id): Path<i32>,
) -> Result<Json<data::models::Plan>, StatusCode> {
    let plan = state
        .plan_repo
        .get_by_id(plan_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(plan))
}

#[derive(serde::Serialize)]
pub struct PlanDetailResponse {
    #[serde(flatten)]
    pub plan: data::models::Plan,
    pub plan_events: Vec<data::models::PlanEventsMapping>,
    pub offer_attribute_ids: Vec<i32>,
}

pub async fn get_plan_by_guid(
    State(state): State<AppState>,
    Path(guid): Path<Uuid>,
) -> Result<Json<PlanDetailResponse>, StatusCode> {
    let plan = state
        .plan_repo
        .get_by_internal_reference(guid)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;
    let plan_events = state
        .plan_events_repo
        .get_all_by_plan_id(plan.id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let attribute_mappings = state
        .plan_attribute_mapping_repo
        .get_by_plan_id(plan.id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let offer_attribute_ids = attribute_mappings
        .into_iter()
        .map(|m| m.offer_attribute_id)
        .collect();
    Ok(Json(PlanDetailResponse {
        plan,
        plan_events,
        offer_attribute_ids,
    }))
}

#[derive(serde::Deserialize)]
pub struct SavePlanRequest {
    pub plan_events: Option<Vec<PlanEventDto>>,
    pub offer_attribute_ids: Option<Vec<i32>>,
}

#[derive(serde::Deserialize)]
pub struct PlanEventDto {
    pub id: Option<i32>,
    pub event_id: i32,
    pub success_state_emails: Option<String>,
    pub failure_state_emails: Option<String>,
    pub copy_to_customer: Option<bool>,
}

pub async fn save_plan_by_guid(
    State(state): State<AppState>,
    Path(guid): Path<Uuid>,
    Json(body): Json<SavePlanRequest>,
) -> Result<StatusCode, StatusCode> {
    let plan = state
        .plan_repo
        .get_by_internal_reference(guid)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;
    if let Some(events) = &body.plan_events {
        for e in events {
            let insert = data::repositories::PlanEventsMappingInsert {
                id: e.id.unwrap_or(0),
                plan_id: plan.id,
                event_id: e.event_id,
                success_state_emails: e.success_state_emails.clone(),
                failure_state_emails: e.failure_state_emails.clone(),
                copy_to_customer: e.copy_to_customer,
            };
            state
                .plan_events_repo
                .upsert(&insert)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            }
    }
    if let Some(ids) = &body.offer_attribute_ids {
        state
            .plan_attribute_mapping_repo
            .replace_for_plan(plan.id, ids)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }
    Ok(StatusCode::OK)
}

pub async fn get_offers(
    State(state): State<AppState>,
) -> Result<Json<Vec<data::models::Offer>>, StatusCode> {
    let offers = state
        .offer_repo
        .get_all()
        .await
        .map_err(|e| {
            error!("Failed to get offers: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(offers))
}

#[derive(serde::Serialize)]
pub struct OfferWithAttributes {
    #[serde(flatten)]
    pub offer: data::models::Offer,
    pub attributes: Vec<data::models::OfferAttributes>,
}

pub async fn get_offer_by_guid(
    State(state): State<AppState>,
    Path(guid): Path<Uuid>,
) -> Result<Json<OfferWithAttributes>, StatusCode> {
    let offer = state
        .offer_repo
        .get_by_offer_guid(guid)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;
    let attributes = state
        .offer_attributes_repo
        .get_all_offer_attributes_by_offer_id(guid)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(OfferWithAttributes { offer, attributes }))
}

#[derive(serde::Deserialize)]
pub struct OfferAttributeDto {
    pub id: Option<i32>,
    pub parameter_id: Option<String>,
    pub display_name: Option<String>,
    pub description: Option<String>,
    #[serde(rename = "type")]
    pub type_: Option<String>,
    pub values_list: Option<String>,
}

#[derive(serde::Deserialize)]
pub struct SaveOfferAttributesRequest {
    pub attributes: Vec<OfferAttributeDto>,
}

pub async fn save_offer_attributes(
    State(state): State<AppState>,
    Path(guid): Path<Uuid>,
    Json(body): Json<SaveOfferAttributesRequest>,
) -> Result<StatusCode, StatusCode> {
    let offer = state
        .offer_repo
        .get_by_offer_guid(guid)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;
    for attr in &body.attributes {
        let oa = data::models::OfferAttributes {
            id: attr.id.unwrap_or(0),
            offer_id: offer.id,
            parameter_id: attr.parameter_id.clone(),
            display_name: attr.display_name.clone(),
            description: attr.description.clone(),
            type_: attr.type_.clone(),
            values_list: attr.values_list.clone(),
            create_date: None,
        };
        state
            .offer_attributes_repo
            .add(&oa)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }
    Ok(StatusCode::OK)
}

pub async fn get_application_configs(
    State(state): State<AppState>,
) -> Result<Json<Vec<data::models::ApplicationConfiguration>>, StatusCode> {
    let configs = state
        .config_repo
        .get_all()
        .await
        .map_err(|e| {
            error!("Failed to get application configs: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(configs))
}

pub async fn update_application_config(
    State(state): State<AppState>,
    Path(name): Path<String>,
    Json(body): Json<UpdateConfigRequest>,
) -> Result<StatusCode, StatusCode> {
    state
        .config_repo
        .set_value(&name, &body.value)
        .await
        .map_err(|e| {
            error!("Failed to update application config: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(StatusCode::OK)
}

#[derive(Deserialize)]
pub struct UploadConfigFileRequest {
    pub config_name: String,
    pub value: String, // base64-encoded file content
}

pub async fn upload_config_file(
    State(state): State<AppState>,
    Json(body): Json<UploadConfigFileRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let name = body.config_name.trim();
    if name != "LogoFile" && name != "FaviconFile" {
        return Err(StatusCode::BAD_REQUEST);
    }
    if body.value.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }
    state
        .config_repo
        .set_value(name, &body.value)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(serde_json::json!({ "status": "ok", "config_name": name })))
}
pub async fn get_known_users(
    State(state): State<AppState>,
) -> Result<Json<Vec<KnownUser>>, StatusCode> {
    let users = state
        .known_users_repo
        .get_all()
        .await
        .map_err(|e| {
            error!("Failed to get known users: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    Ok(Json(users))
}

#[derive(Deserialize)]
pub struct KnownUserDto {
    pub user_email: String,
    #[serde(default)]
    pub role_id: i32,
}

pub async fn save_known_users(
    State(state): State<AppState>,
    Json(users): Json<Vec<KnownUserDto>>,
) -> Result<StatusCode, StatusCode> {
    let inserts: Vec<KnownUserInsert> = users
        .into_iter()
        .filter(|u| !u.user_email.trim().is_empty())
        .map(|u| KnownUserInsert {
            user_email: u.user_email,
            role_id: if u.role_id <= 0 { 1 } else { u.role_id },
        })
        .collect();
    state
        .known_users_repo
        .save_all(&inserts)
        .await
        .map_err(|e| {
            error!("Failed to save known users: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    Ok(StatusCode::OK)
}

// ----- Application Logs -----
pub async fn get_application_logs(
    State(state): State<AppState>,
) -> Result<Json<Vec<ApplicationLog>>, StatusCode> {
    let logs = state
        .app_log_repo
        .get_logs()
        .await
        .map_err(|e| {
            error!("Failed to get application logs: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    Ok(Json(logs))
}

// ----- Email Templates -----
pub async fn get_email_templates(
    State(state): State<AppState>,
) -> Result<Json<Vec<EmailTemplate>>, StatusCode> {
    let templates = state
        .email_template_repo
        .get_all()
        .await
        .map_err(|e| {
            error!("Failed to get email templates: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    Ok(Json(templates))
}

pub async fn get_email_template_by_status(
    State(state): State<AppState>,
    Path(status): Path<String>,
) -> Result<Json<Option<EmailTemplate>>, StatusCode> {
    let template = state
        .email_template_repo
        .get_template_for_status(&status)
        .await
        .map_err(|e| {
            error!("Failed to get email template: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    Ok(Json(template))
}

pub async fn save_email_template(
    State(state): State<AppState>,
    Path(status): Path<String>,
    Json(body): Json<EmailTemplateSaveRequest>,
) -> Result<StatusCode, StatusCode> {
    let template = data::models::EmailTemplate {
        id: 0,
        status: Some(status),
        description: body.description,
        insert_date: None,
        template_body: body.template_body,
        subject: body.subject,
        to_recipients: body.to_recipients,
        cc: body.cc,
        bcc: body.bcc,
        is_active: body.is_active.unwrap_or(false),
    };
    state
        .email_template_repo
        .save_email_template_by_status(&template)
        .await
        .map_err(|e| {
            error!("Failed to save email template: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    Ok(StatusCode::OK)
}

#[derive(Deserialize)]
pub struct EmailTemplateSaveRequest {
    pub description: Option<String>,
    pub template_body: Option<String>,
    pub subject: Option<String>,
    pub to_recipients: Option<String>,
    pub cc: Option<String>,
    pub bcc: Option<String>,
    pub is_active: Option<bool>,
}

// ----- Scheduler -----
pub async fn get_scheduler_frequencies(
    State(state): State<AppState>,
) -> Result<Json<Vec<SchedulerFrequency>>, StatusCode> {
    let list = state
        .scheduler_frequency_repo
        .get_all()
        .await
        .map_err(|e| {
            error!("Failed to get scheduler frequencies: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    Ok(Json(list))
}

#[derive(Deserialize)]
pub struct DimensionsBySubscriptionQuery {
    pub subscription_id: Option<i32>,
}

pub async fn get_dimensions_by_subscription(
    State(state): State<AppState>,
    Query(params): Query<DimensionsBySubscriptionQuery>,
) -> Result<Json<Vec<MeteredDimension>>, StatusCode> {
    let sub_id = params.subscription_id.ok_or(StatusCode::BAD_REQUEST)?;
    let subscription = state
        .subscription_repo
        .get_by_id(sub_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;
    let plan = state
        .plan_repo
        .get_by_plan_id(&subscription.amp_plan_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;
    let dimensions = state
        .metered_dimensions_repo
        .get_by_plan_id(plan.id)
        .await
        .map_err(|e| {
            error!("Failed to get dimensions: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    Ok(Json(dimensions))
}

pub async fn get_scheduler_list(
    State(state): State<AppState>,
) -> Result<Json<Vec<MeteredPlanSchedulerManagement>>, StatusCode> {
    let list = state
        .scheduler_repo
        .get_all()
        .await
        .map_err(|e| {
            error!("Failed to get scheduler list: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    Ok(Json(list))
}

#[derive(Deserialize)]
pub struct AddSchedulerRequest {
    pub scheduler_name: String,
    pub subscription_id: i32,
    pub plan_id: i32,
    pub dimension_id: i32,
    pub frequency_id: i32,
    pub quantity: f64,
    pub start_date: String,
}

pub async fn add_scheduler(
    State(state): State<AppState>,
    Json(body): Json<AddSchedulerRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let start_date = chrono::DateTime::parse_from_rfc3339(&body.start_date)
        .map_err(|_| StatusCode::BAD_REQUEST)?
        .with_timezone(&Utc);
    let insert = MeteredPlanSchedulerInsert {
        scheduler_name: body.scheduler_name,
        subscription_id: body.subscription_id,
        plan_id: body.plan_id,
        dimension_id: body.dimension_id,
        frequency_id: body.frequency_id,
        quantity: body.quantity,
        start_date,
    };
    let id = state
        .scheduler_repo
        .insert(&insert)
        .await
        .map_err(|e| {
            error!("Failed to add scheduler: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    Ok(Json(serde_json::json!({ "id": id, "status": "success" })))
}

pub async fn get_scheduler_by_id(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<MeteredPlanSchedulerManagement>, StatusCode> {
    let item = state
        .scheduler_repo
        .get_by_id(id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;
    Ok(Json(item))
}

pub async fn get_scheduler_log(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<Vec<data::models::MeteredAuditLog>>, StatusCode> {
    let item = state
        .scheduler_repo
        .get_by_id(id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;
    let logs = state
        .metered_audit_log_repo
        .get_by_subscription_id(item.subscription_id)
        .await
        .map_err(|e| {
            error!("Failed to get scheduler log: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    Ok(Json(logs))
}

pub async fn delete_scheduler(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<StatusCode, StatusCode> {
    state
        .scheduler_repo
        .delete_by_id(id)
        .await
        .map_err(|e| {
            error!("Failed to delete scheduler: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    Ok(StatusCode::OK)
}

#[derive(serde::Deserialize)]
pub struct UpdateConfigRequest {
    pub value: String,
}

#[derive(Clone)]
pub struct AppState {
    pub subscription_repo: Arc<dyn SubscriptionRepository>,
    pub plan_repo: Arc<dyn PlanRepository>,
    pub offer_repo: Arc<dyn OfferRepository>,
    pub offer_attributes_repo: Arc<dyn data::repositories::OfferAttributesRepository>,
    pub plan_events_repo: Arc<dyn data::repositories::PlanEventsMappingRepository>,
    pub plan_attribute_mapping_repo: Arc<dyn data::repositories::PlanAttributeMappingRepository>,
    pub events_repo: Arc<dyn data::repositories::EventsRepository>,
    pub audit_log_repo: Arc<dyn SubscriptionAuditLogRepository>,
    pub config_repo: Arc<dyn ApplicationConfigRepository>,
    pub app_log_repo: Arc<dyn ApplicationLogRepository>,
    pub email_template_repo: Arc<dyn EmailTemplateRepository>,
    pub known_users_repo: Arc<dyn KnownUsersRepository>,
    pub scheduler_repo: Arc<dyn MeteredPlanSchedulerRepository>,
    pub scheduler_frequency_repo: Arc<dyn SchedulerFrequencyRepository>,
    pub metered_dimensions_repo: Arc<dyn MeteredDimensionsRepository>,
    pub metered_audit_log_repo: Arc<dyn data::repositories::MeteredAuditLogRepository>,
    pub fulfillment_client: Arc<FulfillmentApiClient>,
    pub metering_client: Arc<MeteringApiClient>,
    pub auth_client: Option<BasicClient>,
    pub auth_config: Option<AuthConfig>,
}
