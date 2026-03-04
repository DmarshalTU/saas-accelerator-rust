//! Standalone webhook server. For the two-site deploy, webhook runs inside customer-api; this binary is optional.

use data::pool::create_pool;
use tracing::info;
use webhook::{build_state, router};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    dotenv::dotenv().ok();

    info!("Starting Webhook API server (standalone)");

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = create_pool(&database_url).await?;

    let state = build_state(pool).await?;
    let app = router(state).layer(tower_http::trace::TraceLayer::new_for_http());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3002").await?;
    info!("Webhook API server listening on http://0.0.0.0:3002");

    axum::serve(listener, app).await?;

    Ok(())
}
