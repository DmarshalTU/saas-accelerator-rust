//! Webhook state and handlers for the Customer site and standalone webhook-api.

mod adapters;
mod webhook_handler;

use axum::{
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::{IntoResponse, Json},
    routing::post,
    Router,
};
use data::repositories::*;
use shared::models::WebhookPayload;
use shared::services::{
    AbstractSubscriptionStatusHandler, ApplicationLogServiceImpl, ApplicationLogServiceTrait,
    EmailHelper, EmailServiceTrait, NotificationStatusHandler, SmtpEmailService,
    SubscriptionServiceImpl, SubscriptionStatusHandler, SubscriptionServiceTrait,
};
use std::sync::Arc;
use tracing::{error, info};

/// State required for webhook handling. Embed in customer-api's AppState or use for standalone webhook-api.
#[derive(Clone)]
pub struct WebhookState {
    pub webhook_handler: Arc<webhook_handler::WebhookHandler>,
    pub jwt_validator: Arc<shared::auth::JwtValidator>,
    pub webhook_operation_repo: Arc<dyn WebhookOperationRepository>,
}

/// Validates Bearer token for webhook. Use from customer-api middleware with `state.webhook_state.jwt_validator`.
pub async fn validate_webhook_token(
    headers: &HeaderMap,
    validator: &shared::auth::JwtValidator,
) -> Result<(), StatusCode> {
    let auth_header = headers
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or(StatusCode::UNAUTHORIZED)?;

    validator
        .validate_token(token)
        .await
        .map_err(|e| {
            error!("JWT validation failed: {}", e);
            StatusCode::UNAUTHORIZED
        })?;

    Ok(())
}

async fn webhook_auth_middleware(
    State(state): State<WebhookState>,
    request: Request,
    next: Next,
) -> impl IntoResponse {
    let headers = request.headers();
    if let Err(status) = validate_webhook_token(headers, &state.jwt_validator).await {
        return status.into_response();
    }
    next.run(request).await
}

/// Health check for webhook (e.g. readiness). Call with POST when nested at `/api/webhook`.
pub async fn health_check() -> StatusCode {
    StatusCode::OK
}

/// Handle webhook payload (for embedding in customer-api with shared AppState).
pub async fn handle_webhook_ref(
    state: &WebhookState,
    payload: WebhookPayload,
) -> Result<StatusCode, StatusCode> {
    info!(
        "Received webhook: action={:?}, subscription_id={}",
        payload.action, payload.subscription_id
    );

    if let Some(operation_id) = payload.operation_id
        && state
            .webhook_operation_repo
            .is_processed(operation_id)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    {
        info!("Webhook operation_id {} already processed, skipping", operation_id);
        return Ok(StatusCode::OK);
    }

    let result = match payload.action {
        shared::models::WebhookAction::Unsubscribe => {
            state.webhook_handler.handle_unsubscribe(&payload).await
        }
        shared::models::WebhookAction::ChangePlan => {
            state.webhook_handler.handle_change_plan(&payload).await
        }
        shared::models::WebhookAction::ChangeQuantity => {
            state.webhook_handler.handle_change_quantity(&payload).await
        }
        shared::models::WebhookAction::Suspend => {
            state.webhook_handler.handle_suspend(&payload).await
        }
        shared::models::WebhookAction::Reinstate => {
            state.webhook_handler.handle_reinstate(&payload).await
        }
        shared::models::WebhookAction::Renew => state.webhook_handler.handle_renew(&payload).await,
        shared::models::WebhookAction::Transfer => {
            state.webhook_handler.handle_transfer(&payload).await
        }
        shared::models::WebhookAction::Unknown => {
            state.webhook_handler.handle_unknown_action(&payload).await
        }
    };

    match result {
        Ok(_) => {
            if let Some(operation_id) = payload.operation_id
                && let Err(e) = state.webhook_operation_repo.mark_processed(operation_id).await
            {
                error!(
                    "Failed to mark webhook operation_id {} as processed: {}",
                    operation_id, e
                );
            }
            Ok(StatusCode::OK)
        }
        Err(e) => {
            error!("Webhook processing failed: {}", e);
            Err(StatusCode::BAD_REQUEST)
        }
    }
}

async fn handle_webhook(
    State(state): State<WebhookState>,
    Json(payload): Json<WebhookPayload>,
) -> Result<StatusCode, StatusCode> {
    handle_webhook_ref(&state, payload).await
}

/// Builds webhook state from a DB pool. Use in customer-api (embed in AppState) or webhook-api.
pub async fn build_state(pool: data::pool::DbPool) -> Result<WebhookState, anyhow::Error> {
    let subscription_repo: Arc<dyn SubscriptionRepository> =
        Arc::new(data::repositories::PostgresSubscriptionRepository::new(pool.clone()));
    let plan_repo: Arc<dyn PlanRepository> =
        Arc::new(data::repositories::PostgresPlanRepository::new(pool.clone()));
    let user_repo: Arc<dyn UserRepository> =
        Arc::new(data::repositories::PostgresUserRepository::new(pool.clone()));
    let audit_log_repo: Arc<dyn SubscriptionAuditLogRepository> =
        Arc::new(data::repositories::PostgresSubscriptionAuditLogRepository::new(pool.clone()));
    let config_repo: Arc<dyn ApplicationConfigRepository> =
        Arc::new(data::repositories::PostgresApplicationConfigRepository::new(pool.clone()));
    let application_log_repo: Arc<dyn ApplicationLogRepository> =
        Arc::new(data::repositories::PostgresApplicationLogRepository::new(pool.clone()));
    let email_template_repo: Arc<dyn EmailTemplateRepository> =
        Arc::new(data::repositories::PostgresEmailTemplateRepository::new(pool.clone()));
    let events_repo: Arc<dyn EventsRepository> =
        Arc::new(data::repositories::PostgresEventsRepository::new(pool.clone()));
    let plan_events_mapping_repo: Arc<dyn PlanEventsMappingRepository> =
        Arc::new(data::repositories::PostgresPlanEventsMappingRepository::new(pool.clone()));

    let subscription_repo_adapter: Arc<dyn shared::services::subscription_service::SubscriptionRepositoryTrait> =
        Arc::new(adapters::SubscriptionRepositoryAdapter::new(subscription_repo.clone()));
    let plan_repo_adapter: Arc<dyn shared::services::plan_service::PlanRepositoryForService> =
        Arc::new(adapters::PlanRepositoryAdapter::new(plan_repo.clone()));
    let application_log_repo_adapter: Arc<dyn shared::services::application_log_service::ApplicationLogRepositoryTrait> =
        Arc::new(adapters::ApplicationLogRepositoryAdapter::new(application_log_repo.clone()));

    let subscription_service: Arc<dyn SubscriptionServiceTrait> = Arc::new(
        SubscriptionServiceImpl::new(subscription_repo_adapter, plan_repo_adapter, 0),
    );
    let application_log_service: Arc<dyn ApplicationLogServiceTrait> = Arc::new(
        ApplicationLogServiceImpl::new(application_log_repo_adapter),
    );

    let application_log_service_for_email: Arc<dyn shared::services::email_service::ApplicationLogServiceForEmail> =
        Arc::new(adapters::ApplicationLogServiceAdapterForEmail::new(
            application_log_service.clone(),
        ));

    let email_service: Arc<dyn EmailServiceTrait> =
        Arc::new(SmtpEmailService::new(application_log_service_for_email));

    let config_repo_for_email_helper: Arc<dyn shared::services::email_helper::ApplicationConfigRepositoryForEmailHelper> =
        Arc::new(adapters::ApplicationConfigRepositoryAdapter::new(config_repo.clone()));
    let email_template_repo_for_email_helper: Arc<dyn shared::services::email_helper::EmailTemplateRepositoryForEmailHelper> =
        Arc::new(adapters::EmailTemplateRepositoryAdapter::new(email_template_repo.clone()));
    let events_repo_for_email_helper: Arc<dyn shared::services::email_helper::EventsRepositoryForEmailHelper> =
        Arc::new(adapters::EventsRepositoryAdapter::new(events_repo.clone()));
    let plan_events_mapping_repo_for_email_helper: Arc<dyn shared::services::email_helper::PlanEventsMappingRepositoryForEmailHelper> =
        Arc::new(adapters::PlanEventsMappingRepositoryAdapter::new(
            plan_events_mapping_repo.clone(),
        ));

    let email_helper: Arc<EmailHelper> = Arc::new(EmailHelper::new(
        config_repo_for_email_helper,
        email_template_repo_for_email_helper,
        events_repo_for_email_helper,
        plan_events_mapping_repo_for_email_helper,
    ));

    let subscription_repo_for_status_handler: Arc<dyn shared::services::status_handlers::SubscriptionRepositoryHelper> =
        Arc::new(adapters::SubscriptionRepositoryAdapterForStatusHandler::new(
            subscription_repo.clone(),
        ));
    let plan_repo_for_status_handler: Arc<dyn shared::services::status_handlers::PlanRepositoryHelper> =
        Arc::new(adapters::PlanRepositoryAdapterForStatusHandler::new(plan_repo.clone()));
    let user_repo_for_status_handler: Arc<dyn shared::services::status_handlers::UserRepositoryHelper> =
        Arc::new(adapters::UserRepositoryAdapterForStatusHandler::new(user_repo));

    let abstract_status_handler = AbstractSubscriptionStatusHandler::new(
        subscription_repo_for_status_handler,
        plan_repo_for_status_handler,
        user_repo_for_status_handler,
    );

    let config_repo_for_notification: Arc<dyn shared::services::notification_status_handler::ApplicationConfigRepositoryForNotification> =
        Arc::new(adapters::ApplicationConfigRepositoryAdapterForNotification::new(
            config_repo.clone(),
        ));

    let notification_status_handler: Arc<dyn SubscriptionStatusHandler> = Arc::new(
        NotificationStatusHandler::new(
            abstract_status_handler,
            email_helper,
            email_service,
            config_repo_for_notification,
        ),
    );

    let webhook_handler = Arc::new(webhook_handler::WebhookHandler::new(
        subscription_service,
        subscription_repo,
        audit_log_repo,
        config_repo,
        application_log_service,
        Some(notification_status_handler),
    ));

    let tenant_id = std::env::var("SaaS_API_TENANT_ID").unwrap_or_default();
    let resource_id = std::env::var("SaaS_API_RESOURCE").unwrap_or_default();
    let client_id = std::env::var("SaaS_API_CLIENT_ID").unwrap_or_default();

    let jwt_validator = Arc::new(shared::auth::JwtValidator::new(
        tenant_id,
        resource_id,
        client_id,
    ));

    let webhook_operation_repo: Arc<dyn WebhookOperationRepository> =
        Arc::new(data::repositories::PostgresWebhookOperationRepository::new(pool.clone()));

    Ok(WebhookState {
        webhook_handler,
        jwt_validator,
        webhook_operation_repo,
    })
}

/// Builds the full router for standalone webhook-api: `POST /health` and `POST /api/webhook`.
/// Returns `Router<()>` so it can be passed to `axum::serve`.
pub fn router(state: WebhookState) -> Router<()> {
    let state2 = state.clone();
    Router::new()
        .route("/health", post(health_check))
        .route(
            "/api/webhook",
            post(handle_webhook).layer(axum::middleware::from_fn_with_state(
                state2,
                webhook_auth_middleware,
            )),
        )
        .with_state(state)
}
