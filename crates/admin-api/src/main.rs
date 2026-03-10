mod handlers;
mod auth_handlers;

use axum::{
    error_handling::HandleErrorLayer,
    http::StatusCode,
    response::Html,
    routing::{get, post, patch, delete},
    BoxError, Router,
};
use data::pool::create_pool;
use data::repositories::{
    ApplicationConfigRepository, ApplicationLogRepository, EmailTemplateRepository,
    KnownUsersRepository, MeteredAuditLogRepository, MeteredDimensionsRepository,
    MeteredPlanSchedulerRepository, OfferRepository, PlanRepository,
    SchedulerFrequencyRepository, SubscriptionAuditLogRepository, SubscriptionRepository,
};
use handlers::AppState;
use marketplace::{client::MarketplaceClient, fulfillment::FulfillmentApiClient, metering::MeteringApiClient};
use oauth2::basic::BasicClient;
use shared::auth::user_auth::{AuthConfig, OAuthClientConfig};
use std::sync::Arc;
use tower::ServiceBuilder;
use tower_sessions::{MemoryStore, SessionManagerLayer};
use tracing::info;

#[tokio::main]
#[allow(clippy::too_many_lines)]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    // Load environment variables
    dotenv::dotenv().ok();

    info!("Starting Admin API server");

    // ── Secrets + database pool ───────────────────────────────────────────────
    // AZURE_AD_AUTH=true  → passwordless via Managed Identity (production, 100% secure)
    // Otherwise           → password from KEY_VAULT_URL or DATABASE_URL (local dev / migration)
    let (pool, saas_client_secret) =
        if std::env::var("AZURE_AD_AUTH").as_deref() == Ok("true") {
            let db_host = std::env::var("DB_HOST")
                .expect("DB_HOST required when AZURE_AD_AUTH=true");
            let db_name = std::env::var("DB_NAME")
                .expect("DB_NAME required when AZURE_AD_AUTH=true");
            // Username = managed identity display name = WEBSITE_SITE_NAME (the web app name)
            let db_user = std::env::var("DB_USER")
                .or_else(|_| std::env::var("WEBSITE_SITE_NAME"))
                .expect("DB_USER or WEBSITE_SITE_NAME required when AZURE_AD_AUTH=true");
            info!("AAD auth enabled — fetching PostgreSQL token via Managed Identity");
            let token = shared::secrets::fetch_postgres_aad_token().await
                .expect("Failed to fetch AAD token for PostgreSQL");
            let pool_aad = data::pool::create_pool_with_token(&db_host, &db_user, &db_name, &token)
                .await?;
            // Refresh the pool every 45 min — AAD tokens for PostgreSQL expire at 60 min
            {
                let refresh_pool = pool_aad.clone();
                let (host, user, name) = (db_host.clone(), db_user.clone(), db_name.clone());
                tokio::spawn(async move {
                    let interval = std::time::Duration::from_secs(45 * 60);
                    loop {
                        tokio::time::sleep(interval).await;
                        match shared::secrets::fetch_postgres_aad_token().await {
                            Ok(token) => {
                                match data::pool::create_pool_with_token(&host, &user, &name, &token).await {
                                    Ok(new_pool) => {
                                        refresh_pool.replace(new_pool.get());
                                        tracing::info!("PostgreSQL pool refreshed with new AAD token");
                                    }
                                    Err(e) => tracing::error!("Pool refresh failed: {e}"),
                                }
                            }
                            Err(e) => tracing::error!("AAD token refresh failed: {e}"),
                        }
                    }
                });
            }
            let ad_secret = shared::secrets::resolve_ad_secret().await
                .unwrap_or_default();
            (pool_aad, ad_secret)
        } else {
            let secrets = shared::secrets::resolve_secrets().await
                .expect("Failed to resolve secrets — check KEY_VAULT_URL / DATABASE_URL");
            let ad_secret = secrets.saas_api_client_secret.clone();
            (create_pool(&secrets.database_url).await?, ad_secret)
        };

    // Marketplace API client
    let marketplace_base_url = std::env::var("MARKETPLACE_API_BASE_URL")
        .unwrap_or_else(|_| "https://marketplaceapi.microsoft.com/api".to_string());
    let api_version = std::env::var("MARKETPLACE_API_VERSION")
        .unwrap_or_else(|_| "2018-08-31".to_string());

    let marketplace_client = MarketplaceClient::builder(marketplace_base_url)
        .with_client_secret(
            &std::env::var("SaaS_API_TENANT_ID").unwrap_or_default(),
            &std::env::var("SaaS_API_CLIENT_ID").unwrap_or_default(),
            &saas_client_secret,
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
    let offer_attributes_repo: Arc<dyn data::repositories::OfferAttributesRepository> =
        Arc::new(data::repositories::PostgresOfferAttributesRepository::new(pool.clone()));
    let plan_events_repo: Arc<dyn data::repositories::PlanEventsMappingRepository> =
        Arc::new(data::repositories::PostgresPlanEventsMappingRepository::new(pool.clone()));
    let plan_attribute_mapping_repo: Arc<dyn data::repositories::PlanAttributeMappingRepository> =
        Arc::new(data::repositories::PostgresPlanAttributeMappingRepository::new(pool.clone()));
    let events_repo: Arc<dyn data::repositories::EventsRepository> =
        Arc::new(data::repositories::PostgresEventsRepository::new(pool.clone()));
    let audit_log_repo: Arc<dyn SubscriptionAuditLogRepository> =
        Arc::new(data::repositories::PostgresSubscriptionAuditLogRepository::new(pool.clone()));
    let config_repo: Arc<dyn ApplicationConfigRepository> =
        Arc::new(data::repositories::PostgresApplicationConfigRepository::new(pool.clone()));
    let app_log_repo: Arc<dyn ApplicationLogRepository> =
        Arc::new(data::repositories::PostgresApplicationLogRepository::new(pool.clone()));
    let email_template_repo: Arc<dyn EmailTemplateRepository> =
        Arc::new(data::repositories::PostgresEmailTemplateRepository::new(pool.clone()));
    let known_users_repo: Arc<dyn KnownUsersRepository> =
        Arc::new(data::repositories::PostgresKnownUsersRepository::new(pool.clone()));
    let scheduler_repo: Arc<dyn MeteredPlanSchedulerRepository> =
        Arc::new(data::repositories::PostgresMeteredPlanSchedulerRepository::new(pool.clone()));
    let scheduler_frequency_repo: Arc<dyn SchedulerFrequencyRepository> =
        Arc::new(data::repositories::PostgresSchedulerFrequencyRepository::new(pool.clone()));
    let metered_dimensions_repo: Arc<dyn MeteredDimensionsRepository> =
        Arc::new(data::repositories::PostgresMeteredDimensionsRepository::new(pool.clone()));
    let metered_audit_log_repo: Arc<dyn MeteredAuditLogRepository> =
        Arc::new(data::repositories::PostgresMeteredAuditLogRepository::new(pool.clone()));

    // Setup authentication (optional for local/Docker without Azure AD)
    let (auth_client, auth_config) = if let Some(cfg) = AuthConfig::from_env_optional() {
            let oauth_config = OAuthClientConfig::from_config(&cfg)
                .map_err(|e| format!("Invalid OAuth config: {e}"))?;
            let client = BasicClient::new(
                oauth_config.client_id,
                Some(oauth_config.client_secret),
                oauth_config.auth_url,
                Some(oauth_config.token_url),
            )
            .set_redirect_uri(oauth_config.redirect_url);
        (Some(client), Some(cfg))
    } else {
        info!("Azure AD auth not configured (AZURE_AD_* unset); auth routes will return 503");
        (None, None)
    };

    let app_state = AppState {
        subscription_repo,
        plan_repo,
        offer_repo,
        offer_attributes_repo,
        plan_events_repo,
        plan_attribute_mapping_repo,
        events_repo,
        audit_log_repo,
        config_repo,
        app_log_repo,
        email_template_repo,
        known_users_repo,
        scheduler_repo,
        scheduler_frequency_repo,
        metered_dimensions_repo,
        metered_audit_log_repo,
        fulfillment_client,
        metering_client,
        auth_client,
        auth_config,
    };

    // Setup session management. HandleErrorLayer converts session errors to HTTP 500 (axum 0.7 requires Infallible).
    let session_store = MemoryStore::default();
    // Use secure cookies in production (HTTPS). In Azure App Service HTTPS_ONLY is enforced at the
    // platform level so the request arrives over HTTP to the container but was HTTPS to the client.
    let is_production = std::env::var("WEBSITE_SITE_NAME").is_ok();
    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(is_production)
        .with_same_site(tower_sessions::cookie::SameSite::Lax);
    let session_layer = ServiceBuilder::new()
        .layer(HandleErrorLayer::new(|e: BoxError| async move {
            tracing::error!("Session layer error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        }))
        .layer(session_layer);

    // Public routes: no auth required (health, auth flow, /api/me)
    let public_routes = Router::new()
        .route("/", get(admin_root_page))
        .route("/health", get(health_check))
        .route("/auth/login", get(auth_handlers::login_handler))
        .route("/auth/callback", get(auth_handlers::callback_handler))
        .route("/auth/logout", get(auth_handlers::logout_handler))
        .route("/api/me", get(auth_handlers::me_handler));

    // Protected routes: require session + (when Azure AD configured) known-user in admin role
    let protected_routes = Router::new()
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
        .route("/api/events", get(handlers::get_events))
        .route("/api/plans/by-guid/:guid", get(handlers::get_plan_by_guid).put(handlers::save_plan_by_guid))
        .route("/api/offers", get(handlers::get_offers))
        .route("/api/offers/by-guid/:guid", get(handlers::get_offer_by_guid))
        .route("/api/offers/by-guid/:guid/attributes", axum::routing::put(handlers::save_offer_attributes))
        .route("/api/offers/by-guid/:guid/attributes/:attr_id", axum::routing::delete(handlers::delete_offer_attribute))
        .route("/api/config", get(handlers::get_application_configs))
        .route("/api/config/upload", post(handlers::upload_config_file))
        .route("/api/config/:name", axum::routing::put(handlers::update_application_config))
        .route("/api/known-users", get(handlers::get_known_users).post(handlers::save_known_users))
        .route("/api/application-logs", get(handlers::get_application_logs))
        .route("/api/email-templates", get(handlers::get_email_templates))
        .route("/api/email-templates/:status", get(handlers::get_email_template_by_status).put(handlers::save_email_template))
        .route("/api/scheduler", get(handlers::get_scheduler_list).post(handlers::add_scheduler))
        .route("/api/scheduler/dimensions", get(handlers::get_dimensions_by_subscription))
        .route("/api/scheduler/frequencies", get(handlers::get_scheduler_frequencies))
        .route("/api/scheduler/:id/log", get(handlers::get_scheduler_log))
        .route("/api/scheduler/:id", get(handlers::get_scheduler_by_id).delete(handlers::delete_scheduler))
        .route_layer(axum::middleware::from_fn_with_state(
            app_state.clone(),
            auth_handlers::require_admin_auth_middleware,
        ));

    let app = Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .with_state(app_state)
        .layer(session_layer)
        .layer(tower_http::trace::TraceLayer::new_for_http())
        .layer(build_cors_layer());

    // Start server
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    info!("Admin API server listening on http://0.0.0.0:3000");

    axum::serve(listener, app).await?;

    Ok(())
}

/// CORS: allow only the configured origin(s).
/// In production the frontend is served from the same nginx (same origin) so CORS
/// is not strictly required, but we set an explicit allowlist for defence in depth.
/// `CORS_ALLOWED_ORIGINS` can be a comma-separated list of origins.
/// Defaults to localhost (local dev only).
fn build_cors_layer() -> tower_http::cors::CorsLayer {
    use axum::http::{header, Method};
    use tower_http::cors::CorsLayer;

    let origins: Vec<axum::http::HeaderValue> = std::env::var("CORS_ALLOWED_ORIGINS")
        .unwrap_or_else(|_| "http://localhost:5173,http://localhost:3000".to_string())
        .split(',')
        .filter_map(|o| o.trim().parse().ok())
        .collect();

    CorsLayer::new()
        .allow_origin(origins)
        .allow_methods([
            Method::GET, Method::POST, Method::PUT,
            Method::PATCH, Method::DELETE, Method::OPTIONS,
        ])
        .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION, header::ACCEPT])
        .allow_credentials(true)
}

async fn admin_root_page() -> Html<&'static str> {
    Html(
        r#"<!DOCTYPE html>
<html><head><meta charset="utf-8"><title>Admin API</title></head>
<body>
  <h1>SaaS Accelerator – Admin API</h1>
  <p>This is the API backend. Use the endpoints below (e.g. with curl or a frontend).</p>
  <ul>
    <li><a href="/health">/health</a> – readiness</li>
    <li><a href="/api/plans">/api/plans</a> – plans (JSON)</li>
    <li><a href="/api/offers">/api/offers</a> – offers (JSON)</li>
    <li><a href="/api/subscriptions">/api/subscriptions</a> – subscriptions (JSON)</li>
    <li><a href="/auth/login">/auth/login</a> – Azure AD login (if configured)</li>
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

