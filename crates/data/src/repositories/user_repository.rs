use async_trait::async_trait;
use crate::models::User;
use crate::pool::DbPool;

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn get_by_id(&self, id: i32) -> Result<Option<User>, sqlx::Error>;
    async fn get_by_email(&self, email: &str) -> Result<Option<User>, sqlx::Error>;
    async fn get_partner_detail_from_email(&self, email: &str) -> Result<Option<User>, sqlx::Error>;
    async fn create(&self, user: &User) -> Result<User, sqlx::Error>;
    async fn save(&self, user: &User) -> Result<i32, sqlx::Error>;
}

pub struct PostgresUserRepository {
    pool: DbPool,
}

impl PostgresUserRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl UserRepository for PostgresUserRepository {
    async fn get_by_id(&self, id: i32) -> Result<Option<User>, sqlx::Error> {
        sqlx::query_as::<_, User>(
            "SELECT user_id, email_address, created_date, full_name FROM users WHERE user_id = $1",
        )
        .bind(id)
        .fetch_optional(&{self.pool.get()})
        .await
    }

    async fn get_by_email(&self, email: &str) -> Result<Option<User>, sqlx::Error> {
        sqlx::query_as::<_, User>(
            "SELECT user_id, email_address, created_date, full_name FROM users WHERE email_address = $1",
        )
        .bind(email)
        .fetch_optional(&{self.pool.get()})
        .await
    }

    async fn get_partner_detail_from_email(&self, email: &str) -> Result<Option<User>, sqlx::Error> {
        self.get_by_email(email).await
    }

    async fn create(&self, user: &User) -> Result<User, sqlx::Error> {
        let result = sqlx::query_as::<_, User>(
            "INSERT INTO users (email_address, created_date, full_name) 
             VALUES ($1, $2, $3)
             RETURNING user_id, email_address, created_date, full_name",
        )
        .bind(&user.email_address)
        .bind(Some(chrono::Utc::now()))
        .bind(&user.full_name)
        .fetch_one(&{self.pool.get()})
        .await?;

        Ok(result)
    }

    async fn save(&self, user: &User) -> Result<i32, sqlx::Error> {
        if user.user_id > 0 {
            sqlx::query(
                "UPDATE users SET email_address = $2, full_name = $3 WHERE user_id = $1",
            )
            .bind(user.user_id)
            .bind(&user.email_address)
            .bind(&user.full_name)
            .execute(&{self.pool.get()})
            .await?;
            Ok(user.user_id)
        } else {
            let result = self.create(user).await?;
            Ok(result.user_id)
        }
    }
}

