use serde::{Deserialize, Serialize};

/// SaaS API configuration for Marketplace integration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaaSApiConfig {
    pub ad_authentication_endpoint: String,
    pub client_id: String,
    pub client_secret: String,
    pub tenant_id: String,
    pub fulfillment_api_base_url: String,
    pub fulfillment_api_version: String,
    pub grant_type: String,
    pub resource: String,
    pub mt_client_id: String,
    pub is_admin_portal_multi_tenant: bool,
    pub signed_out_redirect_uri: String,
}

impl Default for SaaSApiConfig {
    fn default() -> Self {
        Self {
            ad_authentication_endpoint: "https://login.microsoftonline.com".to_string(),
            client_id: String::new(),
            client_secret: String::new(),
            tenant_id: String::new(),
            fulfillment_api_base_url: "https://marketplaceapi.microsoft.com/api".to_string(),
            fulfillment_api_version: "2018-08-31".to_string(),
            grant_type: "client_credentials".to_string(),
            resource: "20e940b3-4c77-4b0b-9a53-9e16a1b010a7".to_string(),
            mt_client_id: String::new(),
            is_admin_portal_multi_tenant: false,
            signed_out_redirect_uri: String::new(),
        }
    }
}

impl SaaSApiConfig {
    pub fn from_env() -> Result<Self, config::ConfigError> {
        let cfg = config::Config::builder()
            .add_source(config::Environment::with_prefix("SaaS_API"))
            .build()?;
        cfg.try_deserialize()
    }
    
    pub fn from_env_var() -> Self {
        Self {
            ad_authentication_endpoint: std::env::var("SaaS_API_AD_AUTHENTICATION_ENDPOINT")
                .unwrap_or_else(|_| "https://login.microsoftonline.com".to_string()),
            client_id: std::env::var("SaaS_API_CLIENT_ID").unwrap_or_default(),
            client_secret: std::env::var("SaaS_API_CLIENT_SECRET").unwrap_or_default(),
            tenant_id: std::env::var("SaaS_API_TENANT_ID").unwrap_or_default(),
            fulfillment_api_base_url: std::env::var("SaaS_API_FULFILLMENT_API_BASE_URL")
                .unwrap_or_else(|_| "https://marketplaceapi.microsoft.com/api".to_string()),
            fulfillment_api_version: std::env::var("SaaS_API_FULFILLMENT_API_VERSION")
                .unwrap_or_else(|_| "2018-08-31".to_string()),
            grant_type: std::env::var("SaaS_API_GRANT_TYPE")
                .unwrap_or_else(|_| "client_credentials".to_string()),
            resource: std::env::var("SaaS_API_RESOURCE")
                .unwrap_or_else(|_| "20e940b3-4c77-4b0b-9a53-9e16a1b010a7".to_string()),
            mt_client_id: std::env::var("SaaS_API_MT_CLIENT_ID").unwrap_or_default(),
            is_admin_portal_multi_tenant: std::env::var("SaaS_API_IS_ADMIN_PORTAL_MULTI_TENANT")
                .unwrap_or_else(|_| "false".to_string())
                .parse()
                .unwrap_or(false),
            signed_out_redirect_uri: std::env::var("SaaS_API_SIGNED_OUT_REDIRECT_URI").unwrap_or_default(),
        }
    }
}

