use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use data::repositories::*;
use marketplace::fulfillment::FulfillmentApiClient;
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

#[derive(Clone)]
pub struct AppState {
    pub subscription_repo: Arc<dyn SubscriptionRepository>,
    pub user_repo: Arc<dyn UserRepository>,
    pub fulfillment_client: FulfillmentApiClient,
}

