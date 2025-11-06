use crate::client::MarketplaceClient;
use shared::models::{MeteringUsageRequest, MeteringUsageResult};
use serde::Deserialize;
use tracing::{error, info};
use uuid::Uuid;

/// Metering API client for usage reporting
pub struct MeteringApiClient {
    client: MarketplaceClient,
    api_version: String,
}

impl MeteringApiClient {
    pub fn new(client: MarketplaceClient, api_version: String) -> Self {
        Self { client, api_version }
    }

    /// Emit a usage event for metered billing
    pub async fn emit_usage_event(
        &self,
        request: &MeteringUsageRequest,
    ) -> Result<MeteringUsageResult, anyhow::Error> {
        info!(
            "Emitting usage event for subscription {}: {} {}",
            request.resource_id, request.quantity, request.dimension
        );

        let token = self.client.get_access_token().await?;
        let url = format!(
            "{}/metered/usageEvents?api-version={}",
            self.client.base_url, self.api_version
        );

        let body = serde_json::json!({
            "resourceId": request.resource_id,
            "quantity": request.quantity,
            "dimension": request.dimension,
            "effectiveStartTime": request.effective_start_time.to_rfc3339(),
            "planId": request.plan_id
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
            error!("Failed to emit usage event: {} - {}", status, text);

            // Try to parse error response for usage event ID
            if let Ok(error_json) = serde_json::from_str::<serde_json::Value>(&text) {
                if let Some(usage_event_id) = error_json.get("usageEventId") {
                    if let Some(id_str) = usage_event_id.as_str() {
                        if let Ok(usage_id) = Uuid::parse_str(id_str) {
                            return Ok(MeteringUsageResult {
                                usage_event_id: Some(usage_id),
                                status: "Duplicate".to_string(),
                                message_time: Some(chrono::Utc::now()),
                                resource_id: Some(request.resource_id),
                                quantity: Some(request.quantity),
                                dimension: Some(request.dimension.clone()),
                                effective_start_time: Some(request.effective_start_time),
                                plan_id: Some(request.plan_id.clone()),
                            });
                        }
                    }
                }
            }

            return Err(anyhow::anyhow!("API error: {} - {}", status, text));
        }

        let result: UsageEventResponse = response.json().await?;
        Ok(result.into())
    }

    /// Emit batch usage events
    pub async fn emit_batch_usage_events(
        &self,
        requests: &[MeteringUsageRequest],
    ) -> Result<Vec<MeteringUsageResult>, anyhow::Error> {
        info!("Emitting batch usage events: {} events", requests.len());

        let token = self.client.get_access_token().await?;
        let url = format!(
            "{}/metered/batchUsageEvents?api-version={}",
            self.client.base_url, self.api_version
        );

        let usage_events: Vec<serde_json::Value> = requests
            .iter()
            .map(|req| {
                serde_json::json!({
                    "resourceId": req.resource_id,
                    "quantity": req.quantity,
                    "dimension": req.dimension,
                    "effectiveStartTime": req.effective_start_time.to_rfc3339(),
                    "planId": req.plan_id
                })
            })
            .collect();

        let body = serde_json::json!({
            "request": usage_events
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
            error!("Failed to emit batch usage events: {} - {}", status, text);
            return Err(anyhow::anyhow!("API error: {} - {}", status, text));
        }

        let result: BatchUsageEventResponse = response.json().await?;
        Ok(result
            .results
            .into_iter()
            .map(|r| r.into())
            .collect())
    }
}

#[derive(Debug, Deserialize)]
struct UsageEventResponse {
    #[serde(rename = "usageEventId")]
    usage_event_id: Uuid,
    status: String,
    #[serde(rename = "messageTime")]
    message_time: chrono::DateTime<chrono::Utc>,
    #[serde(rename = "resourceId")]
    resource_id: Uuid,
    quantity: f64,
    dimension: String,
    #[serde(rename = "effectiveStartTime")]
    effective_start_time: chrono::DateTime<chrono::Utc>,
    #[serde(rename = "planId")]
    plan_id: String,
}

impl From<UsageEventResponse> for MeteringUsageResult {
    fn from(resp: UsageEventResponse) -> Self {
        MeteringUsageResult {
            usage_event_id: Some(resp.usage_event_id),
            status: resp.status,
            message_time: Some(resp.message_time),
            resource_id: Some(resp.resource_id),
            quantity: Some(resp.quantity),
            dimension: Some(resp.dimension),
            effective_start_time: Some(resp.effective_start_time),
            plan_id: Some(resp.plan_id),
        }
    }
}

#[derive(Debug, Deserialize)]
struct BatchUsageEventResponse {
    results: Vec<UsageEventResponse>,
}

