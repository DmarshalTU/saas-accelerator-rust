mod handlers;
mod auth_handlers;

use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::{get, post, patch, delete},
    Router,
};
use data::{pool::create_pool, repositories::*};
use handlers::AppState;
use marketplace::{client::MarketplaceClient, fulfillment::FulfillmentApiClient, metering::MeteringApiClient};
use oauth2::basic::BasicClient;
use shared::auth::user_auth::{AuthConfig, OAuthClientConfig};
use std::sync::Arc;
use tower_sessions::{MemoryStore, SessionManagerLayer};
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    // Load environment variables
    dotenv::dotenv().ok();

    info!("Starting Admin API server");

    // Database connection
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
    let pool = create_pool(&database_url).await?;

    // Marketplace API client
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
    let fulfillment_client = Arc::new(FulfillmentApiClient::new(marketplace_client.clone(), api_version.clone()));
    let metering_client = Arc::new(MeteringApiClient::new(marketplace_client, api_version));

    let subscription_repo: Arc<dyn SubscriptionRepository> =
        Arc::new(data::repositories::PostgresSubscriptionRepository::new(pool.clone()));
    let plan_repo: Arc<dyn PlanRepository> =
        Arc::new(data::repositories::PostgresPlanRepository::new(pool.clone()));
    let offer_repo: Arc<dyn OfferRepository> =
        Arc::new(data::repositories::PostgresOfferRepository::new(pool.clone()));
    let audit_log_repo: Arc<dyn SubscriptionAuditLogRepository> =
        Arc::new(data::repositories::PostgresSubscriptionAuditLogRepository::new(pool.clone()));
    let config_repo: Arc<dyn ApplicationConfigRepository> =
        Arc::new(data::repositories::PostgresApplicationConfigRepository::new(pool.clone()));

    let app_state = AppState {
        subscription_repo,
        plan_repo,
        offer_repo,
        audit_log_repo,
        config_repo,
        fulfillment_client,
        metering_client,
    };

    // Setup authentication
    let auth_config = AuthConfig::from_env()
        .map_err(|e| format!("Failed to load auth config: {}", e))?;
    let oauth_config = OAuthClientConfig::from_config(&auth_config)
        .map_err(|e| format!("Failed to create OAuth config: {}", e))?;
    
    let auth_client = BasicClient::new(
        oauth_config.client_id,
        Some(oauth_config.client_secret),
        oauth_config.auth_url,
        Some(oauth_config.token_url),
    )
    .set_redirect_uri(oauth_config.redirect_url);

    // Setup session management
    let session_store = MemoryStore::default();
    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(false) // Set to true in production with HTTPS
        .with_same_site(tower_sessions::cookie::SameSite::Lax);

    let app = Router::new()
        .route("/health", get(health_check))
        .route("/auth/login", get(auth_handlers::login_handler))
        .route("/auth/callback", get(auth_handlers::callback_handler))
        .route("/auth/logout", get(auth_handlers::logout_handler))
        .route("/api/subscriptions", get(handlers::get_subscriptions))
        .route("/api/subscriptions/:id", get(handlers::get_subscription))
        .route("/api/subscriptions/:id/activate", post(handlers::activate_subscription))
        .route("/api/subscriptions/:id/plan", patch(handlers::change_plan))
        .route("/api/subscriptions/:id/quantity", patch(handlers::change_quantity))
        .route("/api/subscriptions/:id/usage", post(handlers::emit_usage_event))
        .route("/api/subscriptions/:id/audit-logs", get(handlers::get_subscription_audit_logs))
        .route("/api/subscriptions/:id", delete(handlers::delete_subscription))
        .route("/api/plans", get(handlers::get_plans))
        .route("/api/plans/:id", get(handlers::get_plan))
        .route("/api/offers", get(handlers::get_offers))
        .route("/api/config", get(handlers::get_application_configs))
        .route("/api/config/:name", axum::routing::put(handlers::update_application_config))
        .with_state(app_state)
        .with_state(auth_client)
        .with_state(auth_config)
        .layer(session_layer)
        .layer(tower_http::trace::TraceLayer::new_for_http())
        .layer(tower_http::cors::CorsLayer::permissive());

    // Start server
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    info!("Admin API server listening on http://0.0.0.0:3000");

    axum::serve(listener, app).await?;

    Ok(())
}

async fn health_check() -> StatusCode {
    StatusCode::OK
}

