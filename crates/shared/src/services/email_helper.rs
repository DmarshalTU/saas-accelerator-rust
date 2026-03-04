use async_trait::async_trait;
use crate::models::EmailContentModel;
use uuid::Uuid;
use std::sync::Arc;

/// Application configuration repository trait for email helper
#[async_trait]
pub trait ApplicationConfigRepositoryForEmailHelper: Send + Sync {
    async fn get_by_name(&self, name: &str) -> Result<Option<String>, String>;
}

/// Email template repository trait for email helper
#[async_trait]
pub trait EmailTemplateRepositoryForEmailHelper: Send + Sync {
    async fn get_email_body_for_subscription(
        &self,
        subscription_id: Uuid,
        process_status: &str,
    ) -> Result<String, String>;
    async fn get_template_for_status(&self, status: &str) -> Result<Option<EmailTemplateData>, String>;
}

/// Events repository trait for email helper
#[async_trait]
pub trait EventsRepositoryForEmailHelper: Send + Sync {
    async fn get_by_name(&self, name: &str) -> Result<Option<EventsData>, String>;
}

/// Plan events mapping repository trait for email helper
#[async_trait]
pub trait PlanEventsMappingRepositoryForEmailHelper: Send + Sync {
    async fn get_plan_event(
        &self,
        plan_id: Uuid,
        event_id: i32,
    ) -> Result<Option<PlanEventsMappingData>, String>;
}

/// Email template data
#[derive(Debug, Clone)]
pub struct EmailTemplateData {
    pub to_recipients: Option<String>,
    pub cc: Option<String>,
    pub bcc: Option<String>,
    pub subject: Option<String>,
    pub template_body: Option<String>,
}

/// Events data
#[derive(Debug, Clone)]
pub struct EventsData {
    pub id: i32,
}

/// Plan events mapping data
#[derive(Debug, Clone)]
pub struct PlanEventsMappingData {
    pub success_state_emails: Option<String>,
    pub copy_to_customer: Option<bool>,
}

/// Email Helper
#[allow(clippy::struct_field_names)]
pub struct EmailHelper {
    application_config_repo: Arc<dyn ApplicationConfigRepositoryForEmailHelper>,
    email_template_repo: Arc<dyn EmailTemplateRepositoryForEmailHelper>,
    events_repo: Arc<dyn EventsRepositoryForEmailHelper>,
    plan_events_mapping_repo: Arc<dyn PlanEventsMappingRepositoryForEmailHelper>,
}

impl EmailHelper {
    pub fn new(
        application_config_repo: Arc<dyn ApplicationConfigRepositoryForEmailHelper>,
        email_template_repo: Arc<dyn EmailTemplateRepositoryForEmailHelper>,
        events_repo: Arc<dyn EventsRepositoryForEmailHelper>,
        plan_events_mapping_repo: Arc<dyn PlanEventsMappingRepositoryForEmailHelper>,
    ) -> Self {
        Self {
            application_config_repo,
            email_template_repo,
            events_repo,
            plan_events_mapping_repo,
        }
    }

    /// # Errors
    /// Returns an error string if template/event lookup or finalize fails.
    pub async fn prepare_email_content(
        &self,
        subscription_id: Uuid,
        plan_guid: Uuid,
        process_status: &str,
        plan_event_name: &str,
        subscription_status: &str,
    ) -> Result<EmailContentModel, String> {
        let body = self
            .email_template_repo
            .get_email_body_for_subscription(subscription_id, process_status)
            .await?;

        let subscription_event = self
            .events_repo
            .get_by_name(plan_event_name)
            .await?
            .ok_or_else(|| format!("Event not found: {plan_event_name}"))?;

        let email_template_data = if process_status == "failure" {
            self.email_template_repo
                .get_template_for_status("Failed")
                .await?
        } else {
            self.email_template_repo
                .get_template_for_status(subscription_status)
                .await?
        };

        let mut subject = String::new();
        let mut copy_to_customer = false;
        let mut to_recipients = String::new();
        let mut cc_recipients = String::new();
        let mut bcc_recipients = String::new();

        if let Some(ref template) = email_template_data {
            if let Some(ref to) = template.to_recipients {
                to_recipients = to.clone();
            }
            if let Some(ref cc) = template.cc {
                cc_recipients = cc.clone();
            }
            if let Some(ref bcc) = template.bcc {
                bcc_recipients = bcc.clone();
            }
            if let Some(ref subj) = template.subject {
                subject = subj.clone();
            }
        }

        let event_data = self
            .plan_events_mapping_repo
            .get_plan_event(plan_guid, subscription_event.id)
            .await?;

        if let Some(ref event) = event_data {
            if let Some(ref success_emails) = event.success_state_emails
                && !success_emails.is_empty()
            {
                to_recipients = success_emails.clone();
            }
            copy_to_customer = event.copy_to_customer.unwrap_or(false);
        }

        self.finalize_content_email(
            subject,
            body,
            cc_recipients,
            bcc_recipients,
            to_recipients,
            copy_to_customer,
        )
        .await
    }

    /// # Errors
    /// Returns an error string if template lookup or finalize fails.
    pub async fn prepare_metered_email_content(
        &self,
        scheduler_task_name: &str,
        subscription_name: &str,
        subscription_status: &str,
        response_json: &str,
    ) -> Result<EmailContentModel, String> {
        let email_template_data = self
            .email_template_repo
            .get_template_for_status(subscription_status)
            .await?
            .ok_or_else(|| format!("Email template not found for status: {subscription_status}"))?;

        let to_recipients = self
            .application_config_repo
            .get_by_name("SchedulerEmailTo")
            .await?
            .ok_or_else(|| "SchedulerEmailTo configuration not found".to_string())?;

        let body = email_template_data
            .template_body
            .unwrap_or_else(String::new)
            .replace("****SubscriptionName****", subscription_name)
            .replace("****SchedulerTaskName****", scheduler_task_name)
            .replace("****ResponseJson****", response_json);

        let subject = email_template_data.subject.unwrap_or_else(String::new);

        self.finalize_content_email(subject, body, String::new(), String::new(), to_recipients, false)
            .await
    }

    async fn finalize_content_email(
        &self,
        subject: String,
        body: String,
        cc_emails: String,
        bcc_emails: String,
        to_emails: String,
        copy_to_customer: bool,
    ) -> Result<EmailContentModel, String> {
        let from_email = self
            .application_config_repo
            .get_by_name("SMTPFromEmail")
            .await?
            .unwrap_or_else(String::new);
        let password = self
            .application_config_repo
            .get_by_name("SMTPPassword")
            .await?
            .unwrap_or_else(String::new);
        let ssl_str = self
            .application_config_repo
            .get_by_name("SMTPSslEnabled")
            .await?
            .unwrap_or_else(|| "false".to_string());
        let ssl = ssl_str.parse::<bool>().unwrap_or(false);
        let user_name = self
            .application_config_repo
            .get_by_name("SMTPUserName")
            .await?
            .unwrap_or_else(String::new);
        let port_str = self
            .application_config_repo
            .get_by_name("SMTPPort")
            .await?
            .unwrap_or_else(|| "0".to_string());
        let port = port_str.parse::<i32>().unwrap_or(0);
        let smtp_host = self
            .application_config_repo
            .get_by_name("SMTPHost")
            .await?
            .unwrap_or_else(String::new);

        Ok(EmailContentModel {
            from_email,
            user_name,
            password,
            port,
            ssl,
            subject,
            smtp_host,
            body,
            to_emails,
            cc_emails,
            bcc_emails,
            customer_email: None,
            copy_to_customer,
            is_active: false,
        })
    }
}

