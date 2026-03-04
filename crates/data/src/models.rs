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

/// Plan Attribute Mapping entity (`plan_id` -> `offer_attribute_id` link)
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PlanAttributeMapping {
    pub plan_attribute_id: i32,
    pub plan_id: i32,
    pub offer_attribute_id: i32,
    pub create_date: Option<DateTime<Utc>>,
}

/// Offer Attributes entity
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct OfferAttributes {
    pub id: i32,
    pub offer_id: i32,
    pub parameter_id: Option<String>,
    pub display_name: Option<String>,
    pub description: Option<String>,
    #[serde(rename = "type")]
    #[sqlx(rename = "type")]
    pub type_: Option<String>,
    pub values_list: Option<String>,
    pub create_date: Option<DateTime<Utc>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn user_serialize_deserialize_roundtrip() {
        let user = User {
            user_id: 1,
            email_address: Some("test@example.com".to_string()),
            created_date: None,
            full_name: Some("Test User".to_string()),
        };
        let j = serde_json::to_string(&user).unwrap();
        let u2: User = serde_json::from_str(&j).unwrap();
        assert_eq!(user.user_id, u2.user_id);
        assert_eq!(user.email_address, u2.email_address);
        assert_eq!(user.full_name, u2.full_name);
    }

    #[test]
    fn subscription_serialize_deserialize_roundtrip() {
        let sub = Subscription {
            id: 1,
            amp_subscription_id: Uuid::nil(),
            subscription_status: "Subscribed".to_string(),
            amp_plan_id: "plan1".to_string(),
            amp_offer_id: "offer1".to_string(),
            is_active: Some(true),
            create_by: Some(1),
            create_date: None,
            modify_date: None,
            user_id: Some(1),
            name: Some("My Subscription".to_string()),
            amp_quantity: 10,
            purchaser_email: None,
            purchaser_tenant_id: None,
            term: Some("P1Y".to_string()),
            start_date: None,
            end_date: None,
        };
        let j = serde_json::to_string(&sub).unwrap();
        let s2: Subscription = serde_json::from_str(&j).unwrap();
        assert_eq!(sub.id, s2.id);
        assert_eq!(sub.subscription_status, s2.subscription_status);
        assert_eq!(sub.amp_plan_id, s2.amp_plan_id);
    }

    #[test]
    fn plan_serialize_deserialize_roundtrip() {
        let plan = Plan {
            id: 1,
            plan_id: "plan_guid_1".to_string(),
            description: Some("A plan".to_string()),
            display_name: Some("Plan One".to_string()),
            is_metering_supported: Some(true),
            is_per_user: Some(false),
            plan_guid: Uuid::nil(),
            offer_id: Uuid::nil(),
        };
        let j = serde_json::to_string(&plan).unwrap();
        let p2: Plan = serde_json::from_str(&j).unwrap();
        assert_eq!(plan.plan_id, p2.plan_id);
        assert_eq!(plan.is_metering_supported, p2.is_metering_supported);
    }
}

