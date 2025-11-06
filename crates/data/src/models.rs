use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Subscription entity
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Subscription {
    pub id: i32,
    pub amp_subscription_id: Uuid,
    pub subscription_status: String,
    pub amp_plan_id: String,
    pub amp_offer_id: String,
    pub is_active: Option<bool>,
    pub create_by: Option<i32>,
    pub create_date: Option<DateTime<Utc>>,
    pub modify_date: Option<DateTime<Utc>>,
    pub user_id: Option<i32>,
    pub name: Option<String>,
    pub amp_quantity: i32,
    pub purchaser_email: Option<String>,
    pub purchaser_tenant_id: Option<Uuid>,
    pub term: Option<String>,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
}

/// User entity
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub user_id: i32,
    pub email_address: Option<String>,
    pub created_date: Option<DateTime<Utc>>,
    pub full_name: Option<String>,
}

/// Plan entity
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Plan {
    pub id: i32,
    pub plan_id: String,
    pub description: Option<String>,
    pub display_name: Option<String>,
    pub is_metering_supported: Option<bool>,
    pub is_per_user: Option<bool>,
    pub plan_guid: Uuid,
    pub offer_id: Uuid,
}

/// Offer entity
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Offer {
    pub id: i32,
    pub offer_id: String,
    pub offer_name: Option<String>,
    pub offer_guid: Uuid,
    pub create_date: Option<DateTime<Utc>>,
}

/// Metered Dimension entity
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct MeteredDimension {
    pub id: i32,
    pub plan_id: i32,
    pub dimension: String,
    pub description: Option<String>,
    pub created_date: Option<DateTime<Utc>>,
}

/// Metered Audit Log entity
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct MeteredAuditLog {
    pub id: i32,
    pub subscription_id: i32,
    pub request_json: Option<String>,
    pub response_json: Option<String>,
    pub status_code: Option<String>,
    pub created_date: Option<DateTime<Utc>>,
    pub subscription_usage_date: Option<DateTime<Utc>>,
    pub run_by: Option<String>,
}

/// Application Configuration entity
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ApplicationConfiguration {
    pub id: i32,
    pub name: String,
    pub value: Option<String>,
    pub description: Option<String>,
}

/// Known User entity
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct KnownUser {
    pub id: i32,
    pub user_email: String,
    pub role_id: i32,
}

/// Subscription Audit Log entity
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SubscriptionAuditLog {
    pub id: i32,
    pub subscription_id: i32,
    pub attribute: Option<String>,
    pub old_value: Option<String>,
    pub new_value: Option<String>,
    pub create_date: Option<DateTime<Utc>>,
    pub create_by: Option<i32>,
}

/// Metered Plan Scheduler Management entity
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct MeteredPlanSchedulerManagement {
    pub id: i32,
    pub scheduler_name: String,
    pub subscription_id: i32,
    pub plan_id: i32,
    pub dimension_id: i32,
    pub frequency_id: i32,
    pub quantity: f64,
    pub start_date: DateTime<Utc>,
    pub next_run_time: Option<DateTime<Utc>>,
}

/// Scheduler Frequency entity
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SchedulerFrequency {
    pub id: i32,
    pub frequency: String,
}

/// Application Log entity
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ApplicationLog {
    pub id: i32,
    pub action_time: Option<DateTime<Utc>>,
    pub log_detail: Option<String>,
}

/// Email Template entity
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct EmailTemplate {
    pub id: i32,
    pub status: Option<String>,
    pub description: Option<String>,
    pub insert_date: Option<DateTime<Utc>>,
    pub template_body: Option<String>,
    pub subject: Option<String>,
    pub to_recipients: Option<String>,
    pub cc: Option<String>,
    pub bcc: Option<String>,
    pub is_active: bool,
}

/// Events entity
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Events {
    pub id: i32,
    pub events_name: Option<String>,
    pub is_active: Option<bool>,
    pub create_date: Option<DateTime<Utc>>,
}

/// Plan Events Mapping entity
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PlanEventsMapping {
    pub id: i32,
    pub plan_id: i32,
    pub event_id: i32,
    pub success_state_emails: Option<String>,
    pub failure_state_emails: Option<String>,
    pub create_date: Option<DateTime<Utc>>,
    pub copy_to_customer: Option<bool>,
}

/// Offer Attributes entity
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct OfferAttributes {
    pub id: i32,
    pub offer_id: i32,
    pub parameter_id: Option<String>,
    pub display_name: Option<String>,
    pub description: Option<String>,
    pub type_: Option<String>,
    pub values_list: Option<String>,
    pub create_date: Option<DateTime<Utc>>,
}

