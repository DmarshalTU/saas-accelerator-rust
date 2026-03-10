use sqlx::{PgPool, postgres::PgPoolOptions};
use std::sync::{Arc, RwLock};
use std::time::Duration;
use tracing::info;

// ── SharedPool ──────────────────────────────────────────────────────────────
/// Thread-safe, optionally auto-refreshing `PostgreSQL` connection pool.
///
/// Repositories hold a `SharedPool`; each query calls `get()` which returns a
/// cheap `PgPool` clone (just an `Arc` refcount bump). The inner pool can be
/// swapped atomically by a background refresh task (for AAD token renewal).
#[derive(Clone)]
pub struct SharedPool(Arc<RwLock<PgPool>>);

impl SharedPool {
    /// Wrap an existing `PgPool`.
    pub fn new(pool: PgPool) -> Self {
        Self(Arc::new(RwLock::new(pool)))
    }

    /// Return the current pool (cheap `Arc` clone).
    ///
    /// Blocks only for the microseconds it takes to swap the inner pool
    /// during a token refresh (happens every 45 min in production).
    ///
    /// # Panics
    /// Panics if the internal `RwLock` is poisoned (only possible if another
    /// thread panicked while holding the write lock, which should never happen).
    pub fn get(&self) -> PgPool {
        self.0
            .read()
            .expect("SharedPool RwLock poisoned")
            .clone()
    }

    /// Replace the inner pool with a freshly-connected one.
    /// Called by the background AAD token refresh task.
    ///
    /// # Panics
    /// Panics if the internal `RwLock` is poisoned.
    pub fn replace(&self, new_pool: PgPool) {
        *self.0.write().expect("SharedPool RwLock poisoned") = new_pool;
    }
}

/// Database pool type used throughout the codebase.
pub type DbPool = SharedPool;

// ── Pool constructors ────────────────────────────────────────────────────────

/// Create a pool from a postgres URL (password-based, used locally and for
/// the migration container).
///
/// # Errors
/// Returns `sqlx::Error` if the connection URL is invalid or the server is
/// unreachable.
pub async fn create_pool(database_url: &str) -> Result<DbPool, sqlx::Error> {
    info!("Creating database connection pool");
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .acquire_timeout(Duration::from_secs(30))
        .connect(database_url)
        .await?;
    info!("Database connection pool created");
    Ok(SharedPool::new(pool))
}

/// Create a pool from individual connection parameters and an **AAD token**
/// as the password (passwordless / Managed Identity approach).
///
/// # Errors
/// Returns `sqlx::Error` if the connection cannot be established.
pub async fn create_pool_with_token(
    db_host: &str,
    db_user: &str,
    db_name: &str,
    token: &str,
) -> Result<DbPool, sqlx::Error> {
    use sqlx::postgres::{PgConnectOptions, PgSslMode};
    info!("Creating database connection pool with AAD token");
    let opts = PgConnectOptions::new()
        .host(db_host)
        .port(5432)
        .username(db_user)
        .password(token)
        .database(db_name)
        .ssl_mode(PgSslMode::Require);
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .acquire_timeout(Duration::from_secs(30))
        .connect_with(opts)
        .await?;
    info!("Database connection pool (AAD) created");
    Ok(SharedPool::new(pool))
}
