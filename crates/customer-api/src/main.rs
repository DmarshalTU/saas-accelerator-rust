mod handlers;

use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::{Html, IntoResponse, Json},
    routing::{get, patch, post},
    Router,
};
use data::{
    pool::create_pool,
    repositories::{
        PlanRepository, PostgresPlanRepository, PostgresSubscriptionRepository,
        PostgresUserRepository, SubscriptionRepository, UserRepository,
    },
};
use handlers::AppState;
use marketplace::{client::MarketplaceClient, fulfillment::FulfillmentApiClient};
use shared::models::WebhookPayload;
use std::sync::Arc;
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    dotenv::dotenv().ok();

    info!("Starting Customer API server");

    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
    let pool = create_pool(&database_url).await?;

    let marketplace_base_url = std::env::var("MARKETPLACE_API_BASE_URL")
        .unwrap_or_else(|_| "https://marketplaceapi.microsoft.com/api".to_string());
    let api_version = std::env::var("MARKETPLACE_API_VERSION")
        .unwrap_or_else(|_| "2018-08-31".to_string());

    let marketplace_client = MarketplaceClient::builder(marketplace_base_url)
        .with_client_secret(
            &std::env::var("SaaS_API_TENANT_ID").unwrap_or_default(),
            &std::env::var("SaaS_API_CLIENT_ID").unwrap_or_default(),
            &std::env::var("SaaS_API_CLIENT_SECRET").unwrap_or_default(),
        )
        .build();

    let fulfillment_client = Arc::new(FulfillmentApiClient::new(marketplace_client, api_version));

    let webhook_state = webhook::build_state(pool.clone())
        .await
        .map_err(|e| e.to_string())?;

    let subscription_repo: Arc<dyn SubscriptionRepository> =
        Arc::new(PostgresSubscriptionRepository::new(pool.clone()));
    let user_repo: Arc<dyn UserRepository> =
        Arc::new(PostgresUserRepository::new(pool.clone()));
    let plan_repo: Arc<dyn PlanRepository> =
        Arc::new(PostgresPlanRepository::new(pool));

    let app_state = AppState {
        subscription_repo,
        user_repo,
        plan_repo,
        fulfillment_client,
        webhook_state,
    };

    let app = Router::new()
        .route("/", get(customer_root_page))
        .route("/health", get(health_check))
        .route("/api/landing", get(handlers::get_landing_page))
        .route("/api/subscriptions/:id", get(handlers::get_subscription))
        .route("/api/subscriptions/:id/activate", post(handlers::activate_subscription))
        .route("/api/subscriptions/:id/plan", patch(handlers::change_plan))
        .route("/api/subscriptions/:id/quantity", patch(handlers::change_quantity))
        .route("/api/plans", get(handlers::get_plans))
        .route("/api/users/:email/subscriptions", get(handlers::get_user_subscriptions))
        .route("/api/users/:email", get(handlers::get_user))
        .route(
            "/api/webhook",
            post(webhook_handler).layer(axum::middleware::from_fn_with_state(
                app_state.clone(),
                webhook_auth_middleware,
            )),
        )
        .route("/api/webhook/health", post(webhook::health_check))
        .with_state(app_state)
        .layer(tower_http::trace::TraceLayer::new_for_http())
        .layer(tower_http::cors::CorsLayer::permissive());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3001").await?;
    info!("Customer API server listening on http://0.0.0.0:3001");

    axum::serve(listener, app).await?;

    Ok(())
}

async fn webhook_auth_middleware(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> impl IntoResponse {
    if let Err(status) =
        webhook::validate_webhook_token(request.headers(), &state.webhook_state.jwt_validator).await
    {
        return status.into_response();
    }
    next.run(request).await
}

async fn webhook_handler(
    State(state): State<AppState>,
    Json(payload): Json<WebhookPayload>,
) -> Result<StatusCode, StatusCode> {
    webhook::handle_webhook_ref(&state.webhook_state, payload).await
}

async fn customer_root_page() -> Html<&'static str> {
    Html(
        r#"<!DOCTYPE html>
<html><head><meta charset="utf-8"><title>Customer API</title></head>
<body>
  <h1>SaaS Accelerator – Customer API</h1>
  <p>This is the API backend for the customer portal. Use the endpoints below (e.g. with curl or a frontend).</p>
  <ul>
    <li><a href="/health">/health</a> – readiness</li>
    <li><a href="/api/landing">/api/landing</a> – landing page (JSON)</li>
    <li>POST /api/webhook – marketplace webhook (Bearer token)</li>
    <li>POST /api/webhook/health – webhook readiness</li>
    <li>/api/subscriptions/:id – subscription details</li>
    <li>/api/users/:email/subscriptions – user subscriptions</li>
  </ul>
</body></html>"#,
    )
}

async fn health_check() -> StatusCode {
    StatusCode::OK
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::Request;
    use tower::util::ServiceExt;

    #[tokio::test]
    async fn health_returns_200() {
        let app = Router::new().route("/health", get(health_check));
        let req = Request::builder().uri("/health").body(Body::empty()).unwrap();
        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn unknown_route_returns_404() {
        let app = Router::new().route("/health", get(health_check));
        let req = Request::builder()
            .uri("/api/nonexistent")
            .body(Body::empty())
            .unwrap();
        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
}

