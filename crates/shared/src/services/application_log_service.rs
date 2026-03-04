use async_trait::async_trait;
use std::sync::Arc;

/// Application log service trait matching the original C# `ApplicationLogService`
#[async_trait]
pub trait ApplicationLogServiceTrait: Send + Sync {
    async fn add_application_log(&self, log_message: &str) -> Result<(), String>;
    async fn get_all_logs(&self) -> Result<Vec<ApplicationLogData>, String>;
}

/// Application log data
#[derive(Debug, Clone)]
pub struct ApplicationLogData {
    pub id: i32,
    pub action_time: Option<chrono::DateTime<chrono::Utc>>,
    pub log_detail: Option<String>,
}

/// Application log repository trait for dependency injection
#[async_trait]
pub trait ApplicationLogRepositoryTrait: Send + Sync {
    async fn add_log(&self, log_detail: &ApplicationLogData) -> Result<i32, String>;
    async fn get_logs(&self) -> Result<Vec<ApplicationLogData>, String>;
}

/// Concrete implementation of `ApplicationLogService`
pub struct ApplicationLogServiceImpl {
    application_log_repo: Arc<dyn ApplicationLogRepositoryTrait>,
}

impl ApplicationLogServiceImpl {
    pub fn new(application_log_repo: Arc<dyn ApplicationLogRepositoryTrait>) -> Self {
        Self {
            application_log_repo,
        }
    }
}

#[async_trait]
impl ApplicationLogServiceTrait for ApplicationLogServiceImpl {
    async fn add_application_log(&self, log_message: &str) -> Result<(), String> {
        let encoded_message = log_message.replace('<', "&lt;").replace('>', "&gt;").replace('&', "&amp;");
        let new_log = ApplicationLogData {
            id: 0,
            action_time: Some(chrono::Utc::now()),
            log_detail: Some(encoded_message),
        };
        self.application_log_repo.add_log(&new_log).await?;
        Ok(())
    }

    async fn get_all_logs(&self) -> Result<Vec<ApplicationLogData>, String> {
        self.application_log_repo.get_logs().await
    }
}

