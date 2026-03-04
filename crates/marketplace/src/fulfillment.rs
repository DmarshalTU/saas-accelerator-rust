use chrono::Utc;
use serde::Deserialize;
use shared::models::{SubscriptionResult, SubscriptionStatus};
use tracing::{error, info};
use uuid::Uuid;

use crate::client::MarketplaceClient;

/// Fulfillment API client for subscription management
pub struct FulfillmentApiClient {
    client: MarketplaceClient,
    api_version: String,
}

impl FulfillmentApiClient {
    pub fn new(client: MarketplaceClient, api_version: String) -> Self {
        Self { client, api_version }
    }

    /// Get all subscriptions
    ///
    /// # Errors
    /// Returns an error if the API request fails or the response is not successful.
    pub async fn list_subscriptions(&self) -> Result<Vec<SubscriptionResult>, anyhow::Error> {
        info!("Fetching all subscriptions from Marketplace API");

        let token = self.client.get_access_token().await?;
        let url = format!(
            "{}/saas/subscriptions?api-version={}",
            self.client.base_url, self.api_version
        );

        let response = self
            .client
            .http_client
            .get(&url)
            .bearer_auth(&token)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await?;
            error!("Failed to list subscriptions: {} - {}", status, text);
            return Err(anyhow::anyhow!("API error: {status} - {text}"));
        }

        let subscriptions: Vec<SubscriptionResponse> = response.json().await?;
        Ok(subscriptions.into_iter().map(std::convert::Into::into).collect())
    }

    /// Get subscription by ID
    ///
    /// # Errors
    /// Returns an error if the API request fails or the response is not successful.
    pub async fn get_subscription(
        &self,
        subscription_id: Uuid,
    ) -> Result<SubscriptionResult, anyhow::Error> {
        info!("Fetching subscription {} from Marketplace API", subscription_id);

        let token = self.client.get_access_token().await?;
        let url = format!(
            "{}/saas/subscriptions/{}?api-version={}",
            self.client.base_url, subscription_id, self.api_version
        );

        let response = self
            .client
            .http_client
            .get(&url)
            .bearer_auth(&token)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await?;
            error!("Failed to get subscription: {} - {}", status, text);
            return Err(anyhow::anyhow!("API error: {status} - {text}"));
        }

        let subscription: SubscriptionResponse = response.json().await?;
        Ok(subscription.into())
    }

    /// Resolve subscription from marketplace token
    ///
    /// # Errors
    /// Returns an error if the API request fails or the response is not successful.
    pub async fn resolve(
        &self,
        market_place_token: &str,
    ) -> Result<ResolvedSubscriptionResult, anyhow::Error> {
        info!("Resolving subscription from marketplace token");

        let token = self.client.get_access_token().await?;
        let url = format!(
            "{}/saas/subscriptions/resolve?api-version={}",
            self.client.base_url, self.api_version
        );

        let response = self
            .client
            .http_client
            .post(&url)
            .bearer_auth(&token)
            .header("x-ms-marketplace-token", market_place_token)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await?;
            error!("Failed to resolve subscription: {} - {}", status, text);
            return Err(anyhow::anyhow!("API error: {status} - {text}"));
        }

        let resolved: SubscriptionResponse = response.json().await?;
        Ok(ResolvedSubscriptionResult {
            subscription_id: resolved.id,
            plan_id: resolved.plan_id,
            offer_id: resolved.offer_id,
        })
    }

    /// Activate subscription
    ///
    /// # Errors
    /// Returns an error if the API request fails or the response is not successful.
    pub async fn activate_subscription(
        &self,
        subscription_id: Uuid,
        plan_id: &str,
    ) -> Result<(), anyhow::Error> {
        info!("Activating subscription {} with plan {}", subscription_id, plan_id);

        let token = self.client.get_access_token().await?;
        let url = format!(
            "{}/saas/subscriptions/{}/activate?api-version={}",
            self.client.base_url, subscription_id, self.api_version
        );

        let body = serde_json::json!({
            "planId": plan_id
        });

        let response = self
            .client
            .http_client
            .post(&url)
            .bearer_auth(&token)
            .json(&body)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await?;
            error!("Failed to activate subscription: {} - {}", status, text);
            return Err(anyhow::anyhow!("API error: {status} - {text}"));
        }

        Ok(())
    }

    /// Update subscription plan
    ///
    /// # Errors
    /// Returns an error if the API request fails or the response is not successful.
    pub async fn update_subscription_plan(
        &self,
        subscription_id: Uuid,
        plan_id: &str,
    ) -> Result<Uuid, anyhow::Error> {
        info!(
            "Updating subscription {} to plan {}",
            subscription_id, plan_id
        );

        let token = self.client.get_access_token().await?;
        let url = format!(
            "{}/saas/subscriptions/{}?api-version={}",
            self.client.base_url, subscription_id, self.api_version
        );

        let body = serde_json::json!({
            "planId": plan_id
        });

        let response = self
            .client
            .http_client
            .patch(&url)
            .bearer_auth(&token)
            .json(&body)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await?;
            error!("Failed to update subscription plan: {} - {}", status, text);
            return Err(anyhow::anyhow!("API error: {status} - {text}"));
        }

        let operation_id = response
            .headers()
            .get("Operation-Location")
            .and_then(|h| h.to_str().ok())
            .and_then(|s| {
                s.split('/')
                    .next_back()
                    .and_then(|id| Uuid::parse_str(id).ok())
            })
            .ok_or_else(|| anyhow::anyhow!("Failed to extract operation ID"))?;

        Ok(operation_id)
    }

    /// Update subscription quantity
    ///
    /// # Errors
    /// Returns an error if the API request fails or the response is not successful.
    pub async fn update_subscription_quantity(
        &self,
        subscription_id: Uuid,
        quantity: i32,
    ) -> Result<Uuid, anyhow::Error> {
        info!(
            "Updating subscription {} quantity to {}",
            subscription_id, quantity
        );

        let token = self.client.get_access_token().await?;
        let url = format!(
            "{}/saas/subscriptions/{}?api-version={}",
            self.client.base_url, subscription_id, self.api_version
        );

        let body = serde_json::json!({
            "quantity": quantity
        });

        let response = self
            .client
            .http_client
            .patch(&url)
            .bearer_auth(&token)
            .json(&body)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await?;
            error!("Failed to update subscription quantity: {} - {}", status, text);
            return Err(anyhow::anyhow!("API error: {status} - {text}"));
        }

        let operation_id = response
            .headers()
            .get("Operation-Location")
            .and_then(|h| h.to_str().ok())
            .and_then(|s| {
                s.split('/')
                    .next_back()
                    .and_then(|id| Uuid::parse_str(id).ok())
            })
            .ok_or_else(|| anyhow::anyhow!("Failed to extract operation ID"))?;

        Ok(operation_id)
    }

    /// Delete subscription
    ///
    /// # Errors
    /// Returns an error if the API request fails or the response is not successful.
    pub async fn delete_subscription(
        &self,
        subscription_id: Uuid,
    ) -> Result<Uuid, anyhow::Error> {
        info!("Deleting subscription {}", subscription_id);

        let token = self.client.get_access_token().await?;
        let url = format!(
            "{}/saas/subscriptions/{}?api-version={}",
            self.client.base_url, subscription_id, self.api_version
        );

        let response = self
            .client
            .http_client
            .delete(&url)
            .bearer_auth(&token)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await?;
            error!("Failed to delete subscription: {} - {}", status, text);
            return Err(anyhow::anyhow!("API error: {status} - {text}"));
        }

        // Extract operation ID from response headers
        let operation_id = response
            .headers()
            .get("Operation-Location")
            .and_then(|h| h.to_str().ok())
            .and_then(|s| {
                s.split('/')
                    .next_back()
                    .and_then(|id| Uuid::parse_str(id).ok())
            })
            .ok_or_else(|| anyhow::anyhow!("Failed to extract operation ID"))?;

        Ok(operation_id)
    }
}

/// Internal subscription response from Marketplace API
#[derive(Debug, Deserialize)]
struct SubscriptionResponse {
    id: Uuid,
    #[serde(rename = "subscriptionName")]
    subscription_name: String,
    #[serde(rename = "offerId")]
    offer_id: String,
    #[serde(rename = "planId")]
    plan_id: String,
    quantity: Option<u32>,
    #[serde(rename = "saasSubscriptionStatus")]
    saas_subscription_status: String,
    beneficiary: BeneficiaryResponse,
    purchaser: PurchaserResponse,
    term: TermResponse,
}

impl From<SubscriptionResponse> for SubscriptionResult {
    fn from(resp: SubscriptionResponse) -> Self {
        use SubscriptionStatus::{PendingFulfillmentStart, Subscribed, Suspended, Unsubscribed};
        let status = match resp.saas_subscription_status.as_str() {
            "PendingFulfillmentStart" => PendingFulfillmentStart,
            "Subscribed" => Subscribed,
            "Suspended" => Suspended,
            "Unsubscribed" => Unsubscribed,
            _ => SubscriptionStatus::NotStarted,
        };

        Self {
            id: resp.id,
            subscription_id: resp.id,
            name: resp.subscription_name,
            offer_id: resp.offer_id,
            plan_id: resp.plan_id,
            quantity: resp.quantity.and_then(|q| q.try_into().ok()),
            status,
            saas_subscription_status: resp.saas_subscription_status,
            beneficiary: resp.beneficiary.into(),
            purchaser: resp.purchaser.into(),
            term: resp.term.into(),
        }
    }
}

#[derive(Debug, Deserialize)]
struct BeneficiaryResponse {
    #[serde(rename = "tenantId")]
    tenant_id: Uuid,
    #[serde(rename = "emailId")]
    email_id: Option<String>,
    #[serde(rename = "objectId")]
    object_id: Option<Uuid>,
    puid: Option<String>,
}

impl From<BeneficiaryResponse> for shared::models::BeneficiaryResult {
    fn from(resp: BeneficiaryResponse) -> Self {
        Self {
            tenant_id: resp.tenant_id,
            email_id: resp.email_id,
            object_id: resp.object_id,
            puid: resp.puid,
        }
    }
}

#[derive(Debug, Deserialize)]
struct PurchaserResponse {
    #[serde(rename = "tenantId")]
    tenant_id: Uuid,
    #[serde(rename = "emailId")]
    email_id: Option<String>,
    #[serde(rename = "objectId")]
    object_id: Option<Uuid>,
    puid: Option<String>,
}

impl From<PurchaserResponse> for shared::models::PurchaserResult {
    fn from(resp: PurchaserResponse) -> Self {
        Self {
            tenant_id: resp.tenant_id,
            email_id: resp.email_id,
            object_id: resp.object_id,
            puid: resp.puid,
        }
    }
}

#[derive(Debug, Deserialize)]
struct TermResponse {
    #[serde(rename = "termUnit")]
    term_unit: String,
    #[serde(rename = "startDate")]
    start_date: chrono::DateTime<Utc>,
    #[serde(rename = "endDate")]
    end_date: Option<chrono::DateTime<Utc>>,
}

impl From<TermResponse> for shared::models::TermResult {
    fn from(resp: TermResponse) -> Self {
        Self {
            term_unit: resp.term_unit,
            start_date: resp.start_date,
            end_date: resp.end_date,
        }
    }
}

/// Resolved subscription result
#[derive(Debug)]
pub struct ResolvedSubscriptionResult {
    pub subscription_id: Uuid,
    pub plan_id: String,
    pub offer_id: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use shared::models::SubscriptionStatus;

    #[test]
    fn subscription_response_deserialize_and_convert_to_result() {
        let json = r#"{
            "id": "550e8400-e29b-41d4-a716-446655440000",
            "subscriptionName": "Test Sub",
            "offerId": "offer-1",
            "planId": "plan-1",
            "quantity": 5,
            "saasSubscriptionStatus": "Subscribed",
            "beneficiary": {
                "tenantId": "550e8400-e29b-41d4-a716-446655440001",
                "emailId": "ben@example.com",
                "objectId": null,
                "puid": null
            },
            "purchaser": {
                "tenantId": "550e8400-e29b-41d4-a716-446655440001",
                "emailId": "buyer@example.com",
                "objectId": null,
                "puid": null
            },
            "term": {
                "termUnit": "P1Y",
                "startDate": "2024-01-01T00:00:00Z",
                "endDate": "2025-01-01T00:00:00Z"
            }
        }"#;
        let resp: SubscriptionResponse = serde_json::from_str(json).expect("deserialize");
        let result = SubscriptionResult::from(resp);
        assert_eq!(result.name, "Test Sub");
        assert_eq!(result.offer_id, "offer-1");
        assert_eq!(result.plan_id, "plan-1");
        assert_eq!(result.quantity, Some(5));
        assert!(matches!(result.status, SubscriptionStatus::Subscribed));
    }
}

