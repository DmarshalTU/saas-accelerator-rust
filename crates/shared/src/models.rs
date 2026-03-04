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
    #[serde(other)]
    Unknown,
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
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn term_unit_enum_from_str() {
        assert_eq!(TermUnitEnum::from("P1M"), TermUnitEnum::P1M);
        assert_eq!(TermUnitEnum::from("p1m"), TermUnitEnum::P1M);
        assert_eq!(TermUnitEnum::from("P1Y"), TermUnitEnum::P1Y);
        assert_eq!(TermUnitEnum::from("P5Y"), TermUnitEnum::P5Y);
        assert_eq!(TermUnitEnum::from("unknown"), TermUnitEnum::P1M);
    }

    #[test]
    fn webhook_action_deserialize() {
        let u: WebhookAction = serde_json::from_str(r#""Unsubscribe""#).unwrap();
        assert!(matches!(u, WebhookAction::Unsubscribe));
        let c: WebhookAction = serde_json::from_str(r#""ChangePlan""#).unwrap();
        assert!(matches!(c, WebhookAction::ChangePlan));
        let unknown: WebhookAction = serde_json::from_str(r#""FutureAction""#).unwrap();
        assert!(matches!(unknown, WebhookAction::Unknown));
    }

    #[test]
    fn subscription_status_roundtrip() {
        let s = SubscriptionStatus::Subscribed;
        let j = serde_json::to_string(&s).unwrap();
        assert_eq!(j, r#""Subscribed""#);
        let s2: SubscriptionStatus = serde_json::from_str(&j).unwrap();
        assert_eq!(s, s2);
    }

    #[test]
    fn webhook_payload_deserialize_minimal() {
        let json = r#"{
            "activityId": "550e8400-e29b-41d4-a716-446655440000",
            "subscriptionId": "550e8400-e29b-41d4-a716-446655440001",
            "offerId": "offer-1",
            "timeStamp": "2024-01-15T12:00:00Z",
            "action": "ChangePlan",
            "status": "Success"
        }"#;
        let p: WebhookPayload = serde_json::from_str(json).unwrap();
        assert_eq!(p.subscription_id, "550e8400-e29b-41d4-a716-446655440001");
        assert_eq!(p.offer_id, "offer-1");
        assert!(matches!(p.action, WebhookAction::ChangePlan));
        assert_eq!(p.status, "Success");
    }

    #[test]
    fn metering_usage_request_roundtrip() {
        let req = MeteringUsageRequest {
            resource_id: Uuid::nil(),
            plan_id: "plan1".to_string(),
            dimension: "dim1".to_string(),
            quantity: 42.5,
            effective_start_time: DateTime::parse_from_rfc3339("2024-01-15T12:00:00Z")
                .unwrap()
                .with_timezone(&Utc),
        };
        let j = serde_json::to_string(&req).unwrap();
        let r2: MeteringUsageRequest = serde_json::from_str(&j).unwrap();
        assert_eq!(req.plan_id, r2.plan_id);
        assert_eq!(req.dimension, r2.dimension);
        assert!((req.quantity - r2.quantity).abs() < 1e-9);
    }

    #[test]
    fn metering_usage_result_roundtrip() {
        let res = MeteringUsageResult {
            usage_event_id: Some(Uuid::nil()),
            status: "Accepted".to_string(),
            message_time: None,
            resource_id: Some(Uuid::nil()),
            quantity: Some(10.0),
            dimension: Some("dim".to_string()),
            effective_start_time: None,
            plan_id: Some("plan1".to_string()),
        };
        let j = serde_json::to_string(&res).unwrap();
        let r2: MeteringUsageResult = serde_json::from_str(&j).unwrap();
        assert_eq!(res.status, r2.status);
        assert_eq!(res.quantity, r2.quantity);
    }
}
