use async_trait::async_trait;
use std::sync::Arc;

/// User service trait matching the original C# `UserService`
#[async_trait]
pub trait UserServiceTrait: Send + Sync {
    async fn add_user(&self, partner_detail: &PartnerDetailViewModel) -> Result<i32, String>;
    async fn get_user_id_from_email_address(&self, partner_email: &str) -> Result<i32, String>;
}

/// Partner Detail View Model
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PartnerDetailViewModel {
    pub user_id: i32,
    pub email_address: String,
    pub full_name: Option<String>,
}

/// User repository trait for dependency injection
#[async_trait]
pub trait UserRepositoryForService: Send + Sync {
    async fn save(&self, user: &UserData) -> Result<i32, String>;
    async fn get_partner_detail_from_email(&self, email: &str) -> Result<Option<UserData>, String>;
}

/// User data for internal use
#[derive(Debug, Clone)]
pub struct UserData {
    pub user_id: i32,
    pub email_address: Option<String>,
    pub created_date: Option<chrono::DateTime<chrono::Utc>>,
    pub full_name: Option<String>,
}

/// Concrete implementation of `UserService`
pub struct UserServiceImpl {
    user_repo: Arc<dyn UserRepositoryForService>,
}

impl UserServiceImpl {
    pub fn new(user_repo: Arc<dyn UserRepositoryForService>) -> Self {
        Self { user_repo }
    }
}

#[async_trait]
impl UserServiceTrait for UserServiceImpl {
    async fn add_user(&self, partner_detail: &PartnerDetailViewModel) -> Result<i32, String> {
        if partner_detail.email_address.is_empty() {
            return Ok(0);
        }

        let new_partner_detail = UserData {
            user_id: partner_detail.user_id,
            email_address: Some(partner_detail.email_address.clone()),
            full_name: partner_detail.full_name.clone(),
            created_date: Some(chrono::Utc::now()),
        };

        self.user_repo.save(&new_partner_detail).await
    }

    async fn get_user_id_from_email_address(&self, partner_email: &str) -> Result<i32, String> {
        if partner_email.is_empty() {
            return Ok(0);
        }

        let user = self
            .user_repo
            .get_partner_detail_from_email(partner_email)
            .await?;

        Ok(user.map_or(0, |u| u.user_id))
    }
}

