use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// Subscription status enum
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub enum SubscriptionStatus {
    NotStarted,
    PendingFulfillmentStart,
    Subscribed,
    Suspended,
    Unsubscribed,
}

/// Webhook action types from Microsoft Marketplace
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum WebhookAction {
    Unsubscribe,
    ChangePlan,
    ChangeQuantity,
    Suspend,
    Reinstate,
    Renew,
    Transfer,
}

/// Subscription result model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionResult {
    pub id: Uuid,
    pub subscription_id: Uuid,
    pub name: String,
    pub offer_id: String,
    pub plan_id: String,
    pub quantity: Option<i32>,
    pub status: SubscriptionStatus,
    pub saas_subscription_status: String,
    pub beneficiary: BeneficiaryResult,
    pub purchaser: PurchaserResult,
    pub term: TermResult,
}

/// Beneficiary information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeneficiaryResult {
    pub tenant_id: Uuid,
    pub email_id: Option<String>,
    pub object_id: Option<Uuid>,
    pub puid: Option<String>,
}

/// Purchaser information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PurchaserResult {
    pub tenant_id: Uuid,
    pub email_id: Option<String>,
    pub object_id: Option<Uuid>,
    pub puid: Option<String>,
}

/// Term information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TermResult {
    pub term_unit: String,
    pub start_date: DateTime<Utc>,
    pub end_date: Option<DateTime<Utc>>,
}

/// Metering usage request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeteringUsageRequest {
    pub resource_id: Uuid,
    pub plan_id: String,
    pub dimension: String,
    pub quantity: f64,
    pub effective_start_time: DateTime<Utc>,
}

/// Metering usage result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeteringUsageResult {
    pub usage_event_id: Option<Uuid>,
    pub status: String,
    pub message_time: Option<DateTime<Utc>>,
    pub resource_id: Option<Uuid>,
    pub quantity: Option<f64>,
    pub dimension: Option<String>,
    pub effective_start_time: Option<DateTime<Utc>>,
    pub plan_id: Option<String>,
}

/// Webhook payload from Microsoft Marketplace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookPayload {
    #[serde(rename = "id")]
    pub operation_id: Option<uuid::Uuid>,
    #[serde(rename = "activityId")]
    pub activity_id: uuid::Uuid,
    #[serde(rename = "subscriptionId")]
    pub subscription_id: String,
    #[serde(rename = "publisherId")]
    pub publisher_id: Option<String>,
    #[serde(rename = "offerId")]
    pub offer_id: String,
    #[serde(rename = "planId")]
    pub plan_id: Option<String>,
    #[serde(rename = "quantity")]
    pub quantity: Option<u32>,
    #[serde(rename = "timeStamp")]
    pub time_stamp: DateTime<Utc>,
    #[serde(rename = "action")]
    pub action: WebhookAction,
    #[serde(rename = "status")]
    pub status: String,
    #[serde(rename = "operationRequestSource")]
    pub operation_request_source: Option<String>,
    #[serde(rename = "subscription")]
    pub subscription: Option<SubscriptionWebhookResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionWebhookResult {
    #[serde(rename = "id")]
    pub subscription_id: uuid::Uuid,
    #[serde(rename = "planId")]
    pub plan_id: String,
    #[serde(rename = "quantity")]
    pub quantity: Option<u32>,
    #[serde(rename = "status")]
    pub status: String,
}

/// Term Unit Enum
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TermUnitEnum {
    P1M,
    P1Y,
    P2Y,
    P3Y,
    P4Y,
    P5Y,
}

impl From<&str> for TermUnitEnum {
    fn from(s: &str) -> Self {
        match s.to_uppercase().as_str() {
            "P1M" => Self::P1M,
            "P1Y" => Self::P1Y,
            "P2Y" => Self::P2Y,
            "P3Y" => Self::P3Y,
            "P4Y" => Self::P4Y,
            "P5Y" => Self::P5Y,
            _ => Self::P1M,
        }
    }
}

/// Email Content Model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailContentModel {
    pub from_email: String,
    pub user_name: String,
    pub password: String,
    pub port: i32,
    pub ssl: bool,
    pub subject: String,
    pub smtp_host: String,
    pub body: String,
    pub to_emails: String,
    pub cc_emails: String,
    pub bcc_emails: String,
    pub customer_email: Option<String>,
    pub copy_to_customer: bool,
    pub is_active: bool,
}

impl Default for EmailContentModel {
    fn default() -> Self {
        Self {
            from_email: String::new(),
            user_name: String::new(),
            password: String::new(),
            port: 0,
            ssl: false,
            subject: String::new(),
            smtp_host: String::new(),
            body: String::new(),
            to_emails: String::new(),
            cc_emails: String::new(),
            bcc_emails: String::new(),
            customer_email: None,
            copy_to_customer: false,
            is_active: false,
        }
    }
}

