mod handlers;
mod auth_handlers;

use axum::{
    extract::State,
    http::StatusCode,
    routing::{get, post},
    Router,
};
use data::{pool::create_pool, repositories::*};
use handlers::AppState;
use marketplace::{client::MarketplaceClient, fulfillment::FulfillmentApiClient};
use std::sync::Arc;
use tracing::info;

#[derive(Clone)]
struct AppState {
    // Add repositories and services as needed
}

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

    let fulfillment_client = FulfillmentApiClient::new(marketplace_client, api_version);

    let subscription_repo: Arc<dyn SubscriptionRepository> =
        Arc::new(data::repositories::PostgresSubscriptionRepository::new(pool.clone()));
    let user_repo: Arc<dyn UserRepository> =
        Arc::new(data::repositories::PostgresUserRepository::new(pool));

    let app_state = AppState {
        subscription_repo,
        user_repo,
        fulfillment_client,
    };

    let app = Router::new()
        .route("/health", get(health_check))
        .route("/api/landing", get(handlers::get_landing_page))
        .route("/api/subscriptions/:id", get(handlers::get_subscription))
        .route("/api/subscriptions/:id/activate", post(handlers::activate_subscription))
        .route("/api/users/:email/subscriptions", get(handlers::get_user_subscriptions))
        .with_state(app_state)
        .layer(tower_http::trace::TraceLayer::new_for_http())
        .layer(tower_http::cors::CorsLayer::permissive());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3001").await?;
    info!("Customer API server listening on http://0.0.0.0:3001");

    axum::serve(listener, app).await?;

    Ok(())
}

async fn health_check() -> StatusCode {
    StatusCode::OK
}

