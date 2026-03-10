use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use data::repositories::{PlanRepository, SubscriptionRepository, UserRepository};
use marketplace::fulfillment::FulfillmentApiClient;
use data::models::Plan;
use serde::Deserialize;
use std::sync::Arc;
use tracing::{error, info};
use uuid::Uuid;

#[derive(Deserialize)]
pub struct LandingPageQuery {
    pub token: Option<String>,
}

pub async fn get_landing_page(
    State(state): State<AppState>,
    Query(params): Query<LandingPageQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    if let Some(token) = params.token {
        info!("Resolving subscription from marketplace token");

        let resolved = state
            .fulfillment_client
            .resolve(&token)
            .await
            .map_err(|e| {
                error!("Failed to resolve subscription: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

        Ok(Json(serde_json::json!({
            "subscription_id": resolved.subscription_id,
            "plan_id": resolved.plan_id,
            "offer_id": resolved.offer_id
        })))
    } else {
        Ok(Json(serde_json::json!({
            "message": "Landing page - provide token parameter to resolve subscription"
        })))
    }
}

pub async fn activate_subscription(
    State(state): State<AppState>,
    Path(subscription_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
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

    Ok(Json(serde_json::json!({
        "status": "success",
        "message": "Subscription activated"
    })))
}

pub async fn get_subscription(
    State(state): State<AppState>,
    Path(subscription_id): Path<Uuid>,
) -> Result<Json<data::models::Subscription>, StatusCode> {
    let subscription = state
        .subscription_repo
        .get_by_amp_subscription_id(subscription_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;
    Ok(Json(subscription))
}

pub async fn get_user_subscriptions(
    State(state): State<AppState>,
    Path(email): Path<String>,
) -> Result<Json<Vec<data::models::Subscription>>, StatusCode> {
    let subscriptions = state
        .subscription_repo
        .get_subscriptions_by_email_address(&email, None, false)
        .await
        .map_err(|e| {
            error!("Failed to get user subscriptions: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    Ok(Json(subscriptions))
}

pub async fn get_plans(
    State(state): State<AppState>,
) -> Result<Json<Vec<Plan>>, StatusCode> {
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

pub async fn get_user(
    State(state): State<AppState>,
    Path(email): Path<String>,
) -> Result<Json<data::models::User>, StatusCode> {
    let user = state
        .user_repo
        .get_by_email(&email)
        .await
        .map_err(|e| {
            error!("Failed to get user: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;
    Ok(Json(user))
}

#[derive(serde::Deserialize)]
pub struct ChangePlanRequest {
    pub plan_id: String,
}

#[derive(serde::Deserialize)]
pub struct ChangeQuantityRequest {
    pub quantity: i32,
}

pub async fn change_plan(
    State(state): State<AppState>,
    Path(subscription_id): Path<Uuid>,
    Json(body): Json<ChangePlanRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    info!("Changing plan for subscription {} to {}", subscription_id, body.plan_id);

    let _subscription = state
        .subscription_repo
        .get_by_amp_subscription_id(subscription_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

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

    let _subscription = state
        .subscription_repo
        .get_by_amp_subscription_id(subscription_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

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

pub async fn cancel_subscription(
    State(state): State<AppState>,
    Path(subscription_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    info!("Customer cancelling subscription {}", subscription_id);

    let subscription = state
        .subscription_repo
        .get_by_amp_subscription_id(subscription_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let _operation_id = state
        .fulfillment_client
        .delete_subscription(subscription_id)
        .await
        .map_err(|e| {
            error!("Failed to cancel subscription: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // Mark as Unsubscribed locally so the UI updates immediately
    let mut updated = subscription;
    updated.subscription_status = "Unsubscribed".to_string();
    updated.is_active = Some(false);
    state
        .subscription_repo
        .save(&updated)
        .await
        .map_err(|e| {
            error!("Failed to update subscription status after cancel: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(serde_json::json!({ "status": "cancelled" })))
}

#[derive(Clone)]
pub struct AppState {
    pub subscription_repo: Arc<dyn SubscriptionRepository>,
    pub user_repo: Arc<dyn UserRepository>,
    pub plan_repo: Arc<dyn PlanRepository>,
    pub fulfillment_client: Arc<FulfillmentApiClient>,
    pub webhook_state: webhook::WebhookState,
}

