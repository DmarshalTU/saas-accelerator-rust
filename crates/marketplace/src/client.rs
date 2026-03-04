use azure_core::auth::TokenCredential;
use azure_core::new_http_client;
use azure_identity::{ClientSecretCredential, DefaultAzureCredential, TokenCredentialOptions};
use std::sync::Arc;
use tracing::info;
use url::Url;

/// Marketplace API client builder
pub struct MarketplaceClientBuilder {
    base_url: String,
    credential: Option<Arc<dyn TokenCredential>>,
}

impl MarketplaceClientBuilder {
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
            credential: None,
        }
    }

    #[must_use]
    pub fn with_credential(mut self, credential: Arc<dyn TokenCredential>) -> Self {
        self.credential = Some(credential);
        self
    }

    /// # Panics
    /// Panics if the authority host URL is invalid (hardcoded Microsoft login URL).
    #[must_use]
    pub fn with_client_secret(
        mut self,
        tenant_id: &str,
        client_id: &str,
        client_secret: &str,
    ) -> Self {
        let authority_host = Url::parse("https://login.microsoftonline.com")
            .expect("Invalid authority host");
        let http_client = new_http_client();
        let credential = Arc::new(
            ClientSecretCredential::new(
                http_client,
                authority_host,
                tenant_id.to_string(),
                client_id.to_string(),
                client_secret.to_string(),
            )
        );
        self.credential = Some(credential);
        self
    }

    /// # Panics
    /// Panics if no credential is set and `DefaultAzureCredential::create` fails, or if the HTTP client fails to build.
    pub fn build(self) -> MarketplaceClient {
        let credential = self.credential.unwrap_or_else(|| {
            Arc::new(
                DefaultAzureCredential::create(TokenCredentialOptions::default())
                    .expect("Failed to create DefaultAzureCredential")
            )
        });

        info!("Building Marketplace client with base URL: {}", self.base_url);

        MarketplaceClient {
            base_url: self.base_url,
            credential,
            http_client: reqwest::Client::builder()
                .build()
                .expect("Failed to create HTTP client"),
        }
    }
}

/// Unified Marketplace API client
#[derive(Clone)]
pub struct MarketplaceClient {
    pub(crate) base_url: String,
    pub(crate) credential: Arc<dyn TokenCredential>,
    pub(crate) http_client: reqwest::Client,
}

impl MarketplaceClient {
    pub fn builder(base_url: impl Into<String>) -> MarketplaceClientBuilder {
        MarketplaceClientBuilder::new(base_url)
    }

    /// Get an access token for the marketplace API
    ///
    /// # Errors
    /// Returns an error if token acquisition fails.
    pub async fn get_access_token(&self) -> Result<String, azure_core::Error> {
        let token = self
            .credential
            .get_token(&["https://marketplaceapi.microsoft.com/.default"])
            .await?;

        Ok(token.token.secret().to_string())
    }
}

