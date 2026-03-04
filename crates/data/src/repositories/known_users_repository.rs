use async_trait::async_trait;
use crate::models::KnownUser;
use crate::pool::DbPool;

#[derive(Debug, Clone)]
pub struct KnownUserInsert {
    pub user_email: String,
    pub role_id: i32,
}

/// Role ID for admin/publisher portal access (matches original .NET `KnownUserAttribute`).
pub const ROLE_ID_ADMIN: i32 = 1;

#[async_trait]
pub trait KnownUsersRepository: Send + Sync {
    async fn get_all(&self) -> Result<Vec<KnownUser>, sqlx::Error>;
    /// Returns the known user for the given email and role, if any (used for admin access check).
    async fn get_by_email_and_role(&self, email: &str, role_id: i32) -> Result<Option<KnownUser>, sqlx::Error>;
    async fn save_all(&self, users: &[KnownUserInsert]) -> Result<(), sqlx::Error>;
}

pub struct PostgresKnownUsersRepository {
    pool: DbPool,
}

impl PostgresKnownUsersRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl KnownUsersRepository for PostgresKnownUsersRepository {
    async fn get_all(&self) -> Result<Vec<KnownUser>, sqlx::Error> {
        sqlx::query_as::<_, KnownUser>(
            "SELECT id, user_email, role_id FROM known_users ORDER BY user_email",
        )
        .fetch_all(&self.pool)
        .await
    }

    async fn get_by_email_and_role(&self, email: &str, role_id: i32) -> Result<Option<KnownUser>, sqlx::Error> {
        sqlx::query_as::<_, KnownUser>(
            "SELECT id, user_email, role_id FROM known_users WHERE user_email = $1 AND role_id = $2",
        )
        .bind(email)
        .bind(role_id)
        .fetch_optional(&self.pool)
        .await
    }

    async fn save_all(&self, users: &[KnownUserInsert]) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM known_users")
            .execute(&self.pool)
            .await?;
        for u in users {
            sqlx::query(
                "INSERT INTO known_users (user_email, role_id) VALUES ($1, $2)",
            )
            .bind(&u.user_email)
            .bind(u.role_id)
            .execute(&self.pool)
            .await?;
        }
        Ok(())
    }
}
