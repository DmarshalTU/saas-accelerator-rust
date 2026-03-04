pub mod user_auth;

use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Claims {
    aud: String,
    exp: i64,
    iat: i64,
    iss: String,
    tid: Option<String>,
    azp: Option<String>,
    appid: Option<String>,
    #[serde(flatten)]
    extra: HashMap<String, serde_json::Value>,
}

#[derive(Clone)]
struct OidcConfiguration {
    issuer: String,
    jwks_uri: String,
    signing_keys: Vec<DecodingKey>,
}

struct OidcConfigCache {
    config: Option<OidcConfiguration>,
    last_fetched: Option<chrono::DateTime<chrono::Utc>>,
}

pub struct JwtValidator {
    tenant_id: String,
    resource_id: String,
    client_id: String,
    cache: Arc<RwLock<OidcConfigCache>>,
    http_client: reqwest::Client,
}

impl JwtValidator {
    #[must_use]
    pub fn new(tenant_id: String, resource_id: String, client_id: String) -> Self {
        Self {
            tenant_id,
            resource_id,
            client_id,
            cache: Arc::new(RwLock::new(OidcConfigCache {
                config: None,
                last_fetched: None,
            })),
            http_client: reqwest::Client::new(),
        }
    }

    async fn fetch_oidc_configuration(&self) -> Result<OidcConfiguration, String> {
        let oidc_url = format!(
            "https://login.microsoftonline.com/{}/.well-known/openid-configuration",
            self.tenant_id
        );

        info!("Fetching OIDC configuration from {}", oidc_url);

        let response = self
            .http_client
            .get(&oidc_url)
            .send()
            .await
            .map_err(|e| format!("Failed to fetch OIDC configuration: {e}"))?;

        let oidc_config: serde_json::Value = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse OIDC configuration: {e}"))?;

        let jwks_uri = oidc_config["jwks_uri"]
            .as_str()
            .ok_or_else(|| "Missing jwks_uri in OIDC configuration".to_string())?
            .to_string();

        let issuer = oidc_config["issuer"]
            .as_str()
            .ok_or_else(|| "Missing issuer in OIDC configuration".to_string())?
            .to_string();

        let jwks_response = self
            .http_client
            .get(&jwks_uri)
            .send()
            .await
            .map_err(|e| format!("Failed to fetch JWKS: {e}"))?;

        let jwks: serde_json::Value = jwks_response
            .json()
            .await
            .map_err(|e| format!("Failed to parse JWKS: {e}"))?;

        let mut signing_keys = Vec::new();

        if let Some(keys) = jwks["keys"].as_array() {
            for key in keys {
                if let Some(kty) = key["kty"].as_str()
                    && kty == "RSA"
                    && let Some(n) = key["n"].as_str()
                    && let Some(e) = key["e"].as_str()
                    && let Ok(decoding_key) = DecodingKey::from_rsa_components(n, e)
                {
                    signing_keys.push(decoding_key);
                }
            }
        }

        if signing_keys.is_empty() {
            return Err("No valid signing keys found in JWKS".to_string());
        }

        info!(
            "Fetched {} signing keys from JWKS (issuer: {}, jwks_uri: {})",
            signing_keys.len(),
            issuer,
            jwks_uri
        );

        Ok(OidcConfiguration {
            issuer,
            jwks_uri,
            signing_keys,
        })
    }

    async fn get_oidc_configuration(&self) -> Result<OidcConfiguration, String> {
        let mut cache = self.cache.write().await;

        let should_refetch = cache.last_fetched.is_none()
            || cache
                .last_fetched
                .unwrap()
                .signed_duration_since(chrono::Utc::now())
                .num_seconds()
                .abs() > 3600;

        if should_refetch || cache.config.is_none() {
            let config = self.fetch_oidc_configuration().await?;
            cache.config = Some(config.clone());
            cache.last_fetched = Some(chrono::Utc::now());
            Ok(config)
        } else if let Some(ref config) = cache.config {
            Ok(config.clone())
        } else {
            unreachable!("config is Some when !should_refetch && config.is_some()")
        }
    }

    /// Validates a JWT token against the OIDC configuration.
    ///
    /// # Errors
    /// Returns an error string if OIDC config fetch fails, no signing key matches, or tenant/app ID mismatch.
    pub async fn validate_token(&self, token: &str) -> Result<TokenValidationResult, String> {
        let oidc_config = self.get_oidc_configuration().await?;

        let mut validation = Validation::new(Algorithm::RS256);
        validation.validate_aud = true;
        validation.set_audience(&[&self.client_id]);
        validation.validate_exp = true;
        validation.set_issuer(&[&oidc_config.issuer]);
        validation.leeway = 0;

        let mut last_error = None;

        for key in &oidc_config.signing_keys {
            match decode::<Claims>(token, key, &validation) {
                Ok(token_data) => {
                    let claims = token_data.claims;

                    let tenant_id = claims.tid.clone().or_else(|| {
                        claims
                            .extra
                            .get("http://schemas.microsoft.com/identity/claims/tenantid")
                            .and_then(|v| v.as_str())
                            .map(std::string::ToString::to_string)
                    });

                    if tenant_id.as_deref() != Some(self.tenant_id.as_str()) {
                        return Err(format!(
                            "Tenant ID mismatch. Expected: {}, Got: {:?}",
                            self.tenant_id, tenant_id
                        ));
                    }

                    let app_id = claims.azp.clone().or_else(|| claims.appid.clone());

                    if app_id.as_deref() != Some(self.resource_id.as_str()) {
                        return Err(format!(
                            "Application ID mismatch. Expected: {}, Got: {:?}",
                            self.resource_id, app_id
                        ));
                    }

                    let mut claims_map = HashMap::new();
                    claims_map.insert("tenant_id".to_string(), tenant_id.clone().unwrap_or_else(String::new));
                    if let Some(app_id_val) = &app_id {
                        claims_map.insert("app_id".to_string(), app_id_val.clone());
                    }

                    info!("JWT token validated successfully");

                    return Ok(TokenValidationResult {
                        is_valid: true,
                        tenant_id,
                        app_id,
                        claims: claims_map,
                    });
                }
                Err(e) => {
                    last_error = Some(e);
                }
            }
        }

        Err(format!(
            "Token validation failed with all signing keys (issuer: {}, jwks_uri: {}): {:?}",
            oidc_config.issuer,
            oidc_config.jwks_uri,
            last_error
        ))
    }
}

#[derive(Debug, Clone)]
pub struct TokenValidationResult {
    pub is_valid: bool,
    pub tenant_id: Option<String>,
    pub app_id: Option<String>,
    pub claims: HashMap<String, String>,
}
