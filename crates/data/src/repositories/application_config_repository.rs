use async_trait::async_trait;
use crate::models::ApplicationConfiguration;
use crate::pool::DbPool;

#[async_trait]
pub trait ApplicationConfigRepository: Send + Sync {
    async fn get_by_name(&self, name: &str) -> Result<Option<String>, sqlx::Error>;
    async fn get_all(&self) -> Result<Vec<ApplicationConfiguration>, sqlx::Error>;
    async fn set_value(&self, name: &str, value: &str) -> Result<(), sqlx::Error>;
}

pub struct PostgresApplicationConfigRepository {
    pool: DbPool,
}

impl PostgresApplicationConfigRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ApplicationConfigRepository for PostgresApplicationConfigRepository {
    async fn get_by_name(&self, name: &str) -> Result<Option<String>, sqlx::Error> {
        let result = sqlx::query_scalar::<_, Option<String>>(
            "SELECT value FROM application_configuration WHERE name = $1"
        )
        .bind(name)
        .fetch_optional(&self.pool)
        .await?;

        Ok(result.flatten())
    }

    async fn get_all(&self) -> Result<Vec<ApplicationConfiguration>, sqlx::Error> {
        sqlx::query_as::<_, ApplicationConfiguration>(
            "SELECT id, name, value, description FROM application_configuration"
        )
        .fetch_all(&self.pool)
        .await
    }

    async fn set_value(&self, name: &str, value: &str) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO application_configuration (name, value) VALUES ($1, $2) 
             ON CONFLICT (name) DO UPDATE SET value = $2"
        )
        .bind(name)
        .bind(value)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}

