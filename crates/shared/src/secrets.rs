//! Secure secret resolution at application startup.
//!
//! ## Production (Azure Web App with Managed Identity)
//! When `KEY_VAULT_URL` is set, secrets are fetched from Azure Key Vault using
//! the app's system-assigned Managed Identity. No passwords ever appear in app settings.
//!
//! KV secrets required:
//!   - `DatabasePassword`      – postgres password
//!   - `ADApplicationSecret`   – Azure AD / Marketplace app registration secret
//!
//! App settings required (non-secret, set in Azure):
//!   - `KEY_VAULT_URL`   – e.g. `https://mycompany-kv.vault.azure.net`
//!   - `DB_HOST`         – e.g. `mycompany-db.postgres.database.azure.com`
//!   - `DB_NAME`         – e.g. `mycompanyAMPSaaSDB`
//!   - `DB_USER`         – defaults to `saasadmin`
//!
//! ## Local development
//! When `KEY_VAULT_URL` is NOT set, falls back to:
//!   - `DATABASE_URL`          – full postgres URL (put in `.env`)
//!   - `SaaS_API_CLIENT_SECRET` – AD app secret (put in `.env`)
//!
//! This design eliminates Key Vault reference race conditions — the app itself
//! resolves secrets at startup rather than relying on the App Service platform.

use std::sync::Arc;
use anyhow::{Context, Result};
use tracing::{info, warn};

/// All runtime secrets needed by the API, resolved at startup.
#[derive(Debug, Clone)]
pub struct RuntimeSecrets {
    /// Full postgres connection URL, ready for sqlx.
    pub database_url: String,
    /// Azure AD / Marketplace app registration client secret.
    pub saas_api_client_secret: String,
}

/// Resolve all secrets at startup (password-based DB auth path).
///
/// # Errors
/// Returns an error if Key Vault is configured but unreachable, or required env vars are missing.
pub async fn resolve_secrets() -> Result<RuntimeSecrets> {
    if let Ok(vault_url) = std::env::var("KEY_VAULT_URL") {
        info!("KEY_VAULT_URL set — fetching secrets from Azure Key Vault: {vault_url}");
        resolve_from_keyvault(&vault_url).await
    } else {
        info!("KEY_VAULT_URL not set — using DATABASE_URL / SaaS_API_CLIENT_SECRET from environment (local dev mode)");
        resolve_from_env()
    }
}

/// Resolve only the AD / Marketplace client secret (used by the AAD auth path
/// where the DB password is replaced by a token but the AD secret is still needed).
///
/// # Errors
/// Returns an error if Key Vault is configured but unreachable.
pub async fn resolve_ad_secret() -> Result<String> {
    if let Ok(vault_url) = std::env::var("KEY_VAULT_URL") {
        use azure_identity::DefaultAzureCredential;
        use azure_security_keyvault::KeyvaultClient;
        let credential = Arc::new(
            DefaultAzureCredential::create(azure_identity::TokenCredentialOptions::default())
                .context("Failed to create DefaultAzureCredential")?,
        );
        let client = KeyvaultClient::new(&vault_url, credential)
            .context("Failed to create Key Vault client")?;
        match client.secret_client().get("ADApplicationSecret").await {
            Ok(s) => Ok(s.value),
            Err(e) => {
                warn!("Could not fetch ADApplicationSecret from KV: {e} — using env var");
                Ok(std::env::var("SaaS_API_CLIENT_SECRET").unwrap_or_default())
            }
        }
    } else {
        Ok(std::env::var("SaaS_API_CLIENT_SECRET").unwrap_or_default())
    }
}

// ── local dev: read directly from env ──────────────────────────────────────────
fn resolve_from_env() -> Result<RuntimeSecrets> {
    let database_url = std::env::var("DATABASE_URL")
        .context("DATABASE_URL must be set when KEY_VAULT_URL is not configured")?;
    let saas_api_client_secret = std::env::var("SaaS_API_CLIENT_SECRET").unwrap_or_default();
    Ok(RuntimeSecrets { database_url, saas_api_client_secret })
}

// ── Azure: fetch from Key Vault via Managed Identity ──────────────────────────
async fn resolve_from_keyvault(vault_url: &str) -> Result<RuntimeSecrets> {
    use azure_identity::DefaultAzureCredential;
    use azure_security_keyvault::KeyvaultClient;

    let credential = Arc::new(
        DefaultAzureCredential::create(azure_identity::TokenCredentialOptions::default())
            .context("Failed to create DefaultAzureCredential — check Managed Identity is enabled")?,
    );
    let client = KeyvaultClient::new(vault_url, credential)
        .context("Failed to create Key Vault client — check KEY_VAULT_URL")?;

    // Fetch DB password from KV
    let db_password = client
        .secret_client()
        .get("DatabasePassword")
        .await
        .context("Failed to fetch 'DatabasePassword' from Key Vault — check Managed Identity has 'get' permission")?
        .value;

    // Build DATABASE_URL from non-secret components (stored in app settings)
    let db_host = std::env::var("DB_HOST")
        .context("DB_HOST must be set as an app setting when KEY_VAULT_URL is configured")?;
    let db_user = std::env::var("DB_USER").unwrap_or_else(|_| "saasadmin".to_string());
    let db_name = std::env::var("DB_NAME")
        .context("DB_NAME must be set as an app setting when KEY_VAULT_URL is configured")?;

    // URL-encode the password so special characters don't break the postgres URL
    let encoded_pass = url_encode_password(&db_password);
    let database_url = format!("postgresql://{db_user}:{encoded_pass}@{db_host}:5432/{db_name}?sslmode=require");

    // Fetch AD secret (non-fatal: fall back to env var so local testing without KV still works)
    let saas_api_client_secret = match client.secret_client().get("ADApplicationSecret").await {
        Ok(s) => {
            info!("ADApplicationSecret fetched from Key Vault");
            s.value
        }
        Err(e) => {
            warn!("Could not fetch ADApplicationSecret from Key Vault: {e} — falling back to SaaS_API_CLIENT_SECRET env var");
            std::env::var("SaaS_API_CLIENT_SECRET").unwrap_or_default()
        }
    };

    info!("Secrets resolved successfully from Key Vault");
    Ok(RuntimeSecrets { database_url, saas_api_client_secret })
}

// ── Passwordless: AAD token for PostgreSQL ────────────────────────────────────

/// Fetch a short-lived Azure AD `OAuth2` access token scoped to Azure Database
/// for `PostgreSQL`. Use it as the `password` in the connection string.
///
/// The token is valid for ~1 hour. Call `spawn_aad_pool_refresh_task` so the
/// pool is automatically replaced before it expires.
///
/// # Errors
/// Returns an error if the Managed Identity / AAD service is not reachable.
pub async fn fetch_postgres_aad_token() -> anyhow::Result<String> {
    use azure_identity::DefaultAzureCredential;
    use azure_core::auth::TokenCredential;

    let credential = DefaultAzureCredential::create(
        azure_identity::TokenCredentialOptions::default(),
    )
    .context("Failed to create DefaultAzureCredential for AAD token")?;

    // Official resource ID for Azure Database for PostgreSQL AAD auth
    let token = credential
        .get_token(&["https://ossrdbms-aad.database.windows.net/.default"])
        .await
        .context("Failed to fetch AAD token for PostgreSQL — is Managed Identity enabled?")?;

    Ok(token.token.secret().to_owned())
}

// Note: spawn_aad_pool_refresh_task lives in the binary crates (admin-api, customer-api)
// to avoid a circular dependency between `shared` and `data`.

/// Percent-encode characters that are not safe in a URL password segment.
fn url_encode_password(s: &str) -> String {
    s.chars()
        .flat_map(|c| {
            // Unreserved chars per RFC 3986 — safe without encoding
            if c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.' | '~') {
                vec![c.to_string()]
            } else {
                vec![format!("%{:02X}", c as u32)]
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn url_encode_safe_chars_unchanged() {
        assert_eq!(url_encode_password("abc123"), "abc123");
        assert_eq!(url_encode_password("Abc-_.~"), "Abc-_.~");
    }

    #[test]
    fn url_encode_special_chars_encoded() {
        assert_eq!(url_encode_password("p@ss!"), "p%40ss%21");
        assert_eq!(url_encode_password("p+w=d"), "p%2Bw%3Dd");
    }
}
