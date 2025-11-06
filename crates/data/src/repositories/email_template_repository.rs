use async_trait::async_trait;
use crate::models::EmailTemplate;
use crate::pool::DbPool;
use uuid::Uuid;

#[async_trait]
pub trait EmailTemplateRepository: Send + Sync {
    async fn get_template_for_status(&self, status: &str) -> Result<Option<EmailTemplate>, sqlx::Error>;
    async fn get_email_body_for_subscription(
        &self,
        subscription_id: Uuid,
        process_status: &str,
    ) -> Result<String, sqlx::Error>;
    async fn get_all(&self) -> Result<Vec<EmailTemplate>, sqlx::Error>;
    async fn save_email_template_by_status(
        &self,
        template: &EmailTemplate,
    ) -> Result<String, sqlx::Error>;
}

pub struct PostgresEmailTemplateRepository {
    pool: DbPool,
}

impl PostgresEmailTemplateRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl EmailTemplateRepository for PostgresEmailTemplateRepository {
    async fn get_template_for_status(&self, status: &str) -> Result<Option<EmailTemplate>, sqlx::Error> {
        sqlx::query_as::<_, EmailTemplate>(
            "SELECT id, status, description, insert_date, template_body, subject, to_recipients, cc, bcc, is_active
             FROM email_template WHERE status = $1",
        )
        .bind(status)
        .fetch_optional(&self.pool)
        .await
    }

    async fn get_email_body_for_subscription(
        &self,
        subscription_id: Uuid,
        _process_status: &str,
    ) -> Result<String, sqlx::Error> {
        let result: Option<(Option<String>,)> = sqlx::query_as(
            "SELECT et.template_body FROM email_template et
             INNER JOIN subscriptions s ON et.status = s.subscription_status
             WHERE s.amp_subscription_id = $1 AND et.is_active = true
             LIMIT 1",
        )
        .bind(subscription_id)
        .fetch_optional(&self.pool)
        .await?;
        
        Ok(result.and_then(|r| r.0).unwrap_or_else(|| String::new()))
    }

    async fn get_all(&self) -> Result<Vec<EmailTemplate>, sqlx::Error> {
        sqlx::query_as::<_, EmailTemplate>(
            "SELECT id, status, description, insert_date, template_body, subject, to_recipients, cc, bcc, is_active
             FROM email_template ORDER BY status",
        )
        .fetch_all(&self.pool)
        .await
    }

    async fn save_email_template_by_status(
        &self,
        template: &EmailTemplate,
    ) -> Result<String, sqlx::Error> {
        let existing = sqlx::query_as::<_, EmailTemplate>(
            "SELECT id, status, description, insert_date, template_body, subject, to_recipients, cc, bcc, is_active
             FROM email_template WHERE status = $1",
        )
        .bind(&template.status)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(existing_template) = existing {
            sqlx::query(
                "UPDATE email_template SET is_active = $2, subject = $3, description = $4, 
                 template_body = $5, to_recipients = $6, bcc = $7, cc = $8
                 WHERE id = $1",
            )
            .bind(existing_template.id)
            .bind(template.is_active)
            .bind(&template.subject)
            .bind(&template.description)
            .bind(&template.template_body)
            .bind(&template.to_recipients)
            .bind(&template.bcc)
            .bind(&template.cc)
            .execute(&self.pool)
            .await?;
            Ok("Updated".to_string())
        } else {
            sqlx::query(
                "INSERT INTO email_template (status, description, template_body, subject, to_recipients, cc, bcc, is_active, insert_date)
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
            )
            .bind(&template.status)
            .bind(&template.description)
            .bind(&template.template_body)
            .bind(&template.subject)
            .bind(&template.to_recipients)
            .bind(&template.cc)
            .bind(&template.bcc)
            .bind(template.is_active)
            .bind(chrono::Utc::now())
            .execute(&self.pool)
            .await?;
            Ok("Created".to_string())
        }
    }
}

