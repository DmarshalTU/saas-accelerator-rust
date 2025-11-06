use sqlx::{PgPool, postgres::PgPoolOptions};
use std::time::Duration;
use tracing::info;

pub type DbPool = PgPool;

/// Create a database connection pool
pub async fn create_pool(database_url: &str) -> Result<DbPool, sqlx::Error> {
    info!("Creating database connection pool");

    let pool = PgPoolOptions::new()
        .max_connections(10)
        .acquire_timeout(Duration::from_secs(30))
        .connect(database_url)
        .await?;

    info!("Database connection pool created successfully");
    Ok(pool)
}

