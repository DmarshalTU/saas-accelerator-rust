use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use data::models::Subscription;
use data::repositories::*;
use marketplace::fulfillment::FulfillmentApiClient;
use marketplace::metering::MeteringApiClient;
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

    match subscription {
        Some(sub) => Ok(Json(sub)),
        None => Err(StatusCode::NOT_FOUND),
    }
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

#[derive(serde::Deserialize)]
pub struct UpdateConfigRequest {
    pub value: String,
}

#[derive(Clone)]
pub struct AppState {
    pub subscription_repo: Arc<dyn SubscriptionRepository>,
    pub plan_repo: Arc<dyn PlanRepository>,
    pub offer_repo: Arc<dyn OfferRepository>,
    pub audit_log_repo: Arc<dyn SubscriptionAuditLogRepository>,
    pub config_repo: Arc<dyn ApplicationConfigRepository>,
    pub fulfillment_client: Arc<FulfillmentApiClient>,
    pub metering_client: Arc<MeteringApiClient>,
}

