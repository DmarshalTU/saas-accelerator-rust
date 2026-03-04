use oauth2::{
    AuthUrl, ClientId, ClientSecret, CsrfToken, RedirectUrl, TokenUrl,
};
use serde::{Deserialize, Serialize};

const JWKS_URL: &str = "https://login.microsoftonline.com/common/discovery/v2.0/keys";

/// User information extracted from Azure AD
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct User {
    pub id: String,
    pub email: String,
    pub name: String,
}

/// Azure AD OAuth configuration
#[derive(Clone)]
pub struct AuthConfig {
    pub client_id: String,
    pub client_secret: String,
    pub tenant_id: String,
    pub authority: String,
    pub redirect_uri: String,
    pub signed_out_redirect_uri: String,
    pub is_multi_tenant: bool,
}

impl AuthConfig {
    /// Load auth config from environment variables (optional).
    /// Returns `None` if any of the required vars are missing or empty (e.g. for local/dev without Azure AD).
    #[must_use]
    pub fn from_env_optional() -> Option<Self> {
        let tenant_id = std::env::var("AZURE_AD_TENANT_ID").ok().filter(|s| !s.is_empty())?;
        let client_id = std::env::var("AZURE_AD_CLIENT_ID").ok().filter(|s| !s.is_empty())?;
        let client_secret = std::env::var("AZURE_AD_CLIENT_SECRET").ok().filter(|s| !s.is_empty())?;
        let authority = std::env::var("AZURE_AD_AUTHORITY")
            .unwrap_or_else(|_| "https://login.microsoftonline.com".to_string());
        let redirect_uri = std::env::var("AZURE_AD_REDIRECT_URI")
            .unwrap_or_else(|_| "http://localhost:3000/auth/callback".to_string());
        let signed_out_redirect_uri = std::env::var("AZURE_AD_SIGNED_OUT_REDIRECT_URI")
            .unwrap_or_else(|_| "http://localhost:3000/".to_string());
        let is_multi_tenant = std::env::var("AZURE_AD_MULTI_TENANT")
            .unwrap_or_else(|_| "false".to_string())
            .parse::<bool>()
            .unwrap_or(false);

        Some(Self {
            client_id,
            client_secret,
            tenant_id,
            authority,
            redirect_uri,
            signed_out_redirect_uri,
            is_multi_tenant,
        })
    }

    /// Load auth config from environment variables.
    ///
    /// # Errors
    /// Returns an error string if required env vars are missing.
    pub fn from_env() -> Result<Self, String> {
        let tenant_id = std::env::var("AZURE_AD_TENANT_ID")
            .map_err(|_| "AZURE_AD_TENANT_ID must be set")?;
        let client_id = std::env::var("AZURE_AD_CLIENT_ID")
            .map_err(|_| "AZURE_AD_CLIENT_ID must be set")?;
        let client_secret = std::env::var("AZURE_AD_CLIENT_SECRET")
            .map_err(|_| "AZURE_AD_CLIENT_SECRET must be set")?;
        let authority = std::env::var("AZURE_AD_AUTHORITY")
            .unwrap_or_else(|_| "https://login.microsoftonline.com".to_string());
        let redirect_uri = std::env::var("AZURE_AD_REDIRECT_URI")
            .unwrap_or_else(|_| "http://localhost:3000/auth/callback".to_string());
        let signed_out_redirect_uri = std::env::var("AZURE_AD_SIGNED_OUT_REDIRECT_URI")
            .unwrap_or_else(|_| "http://localhost:3000/".to_string());
        let is_multi_tenant = std::env::var("AZURE_AD_MULTI_TENANT")
            .unwrap_or_else(|_| "false".to_string())
            .parse::<bool>()
            .unwrap_or(false);

        Ok(Self {
            client_id,
            client_secret,
            tenant_id,
            authority,
            redirect_uri,
            signed_out_redirect_uri,
            is_multi_tenant,
        })
    }

    #[must_use]
    pub fn authority_url(&self) -> String {
        if self.is_multi_tenant {
            format!("{}/common/v2.0", self.authority)
        } else {
            format!("{}/{}/v2.0", self.authority, self.tenant_id)
        }
    }
}

/// OAuth callback query parameters
#[derive(Deserialize)]
pub struct AuthCallbackQuery {
    pub code: Option<String>,
    pub state: Option<String>,
    pub error: Option<String>,
    pub error_description: Option<String>,
}

/// `OAuth2` client configuration for Azure AD.
/// We use Authorization Code Flow; Azure AD returns an ID token when using the openid scope.
pub struct OAuthClientConfig {
    pub client_id: ClientId,
    pub client_secret: ClientSecret,
    pub auth_url: AuthUrl,
    pub token_url: TokenUrl,
    pub redirect_url: RedirectUrl,
}

impl OAuthClientConfig {
    /// Build from `AuthConfig`.
    ///
    /// # Errors
    /// Returns an error string if URLs are invalid.
    pub fn from_config(config: &AuthConfig) -> Result<Self, String> {
        let client_id = ClientId::new(config.client_id.clone());
        let client_secret = ClientSecret::new(config.client_secret.clone());
        let auth_url = AuthUrl::new(format!("{}/authorize", config.authority_url()))
            .map_err(|e| format!("Invalid authorization URL: {e}"))?;
        let token_url = TokenUrl::new(format!("{}/token", config.authority_url()))
            .map_err(|e| format!("Invalid token URL: {e}"))?;
        let redirect_url = RedirectUrl::new(config.redirect_uri.clone())
            .map_err(|e| format!("Invalid redirect URI: {e}"))?;

        Ok(Self {
            client_id,
            client_secret,
            auth_url,
            token_url,
            redirect_url,
        })
    }
}

/// Extract user information from ID token
async fn extract_user_from_token(token: &str) -> Result<User, String> {
    use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
    use serde_json::Value;

    let mut validation = Validation::new(Algorithm::RS256);
    validation.validate_exp = true;
    // Don't validate audience to match original behavior (ValidateIssuer = false)

    let header = jsonwebtoken::decode_header(token)
        .map_err(|e| format!("Failed to decode token header: {e}"))?;

    let kid = header
        .kid
        .ok_or_else(|| "Token missing kid".to_string())?;

    let client = reqwest::Client::new();
    let jwks_response = client
        .get(JWKS_URL)
        .send()
        .await
        .map_err(|e| format!("Failed to fetch JWKS: {e}"))?;

    let jwks: Value = jwks_response
        .json()
        .await
        .map_err(|e| format!("Failed to parse JWKS: {e}"))?;

    let keys = jwks["keys"]
        .as_array()
        .ok_or_else(|| "JWKS missing keys array".to_string())?;

    let key = keys
        .iter()
        .find(|k| k["kid"].as_str() == Some(&kid))
        .ok_or_else(|| format!("Key with kid {kid} not found"))?;

    let n = key["n"]
        .as_str()
        .ok_or_else(|| "Key missing n".to_string())?;
    let e = key["e"]
        .as_str()
        .ok_or_else(|| "Key missing e".to_string())?;

    let decoding_key = DecodingKey::from_rsa_components(n, e)
        .map_err(|e| format!("Failed to create decoding key: {e}"))?;

    let token_data = decode::<Value>(token, &decoding_key, &validation)
        .map_err(|e| format!("Failed to decode token: {e}"))?;

    let claims = token_data.claims;

    let email = claims["preferred_username"]
        .as_str()
        .or_else(|| claims["email"].as_str())
        .ok_or_else(|| "Token missing email".to_string())?
        .to_string();

    let name = claims["name"]
        .as_str()
        .or_else(|| {
            claims["http://schemas.xmlsoap.org/ws/2005/05/identity/claims/name"]
                .as_str()
        })
        .unwrap_or(&email)
        .to_string();

    let id = claims["oid"]
        .as_str()
        .or_else(|| claims["sub"].as_str())
        .ok_or_else(|| "Token missing user ID".to_string())?
        .to_string();

    Ok(User { id, email, name })
}

/// Generate CSRF token for OAuth flow
pub fn generate_csrf_token() -> CsrfToken {
    CsrfToken::new_random()
}

/// Extract user information from ID token (public for use in API handlers).
///
/// # Errors
/// Returns an error string if the token is invalid, missing required claims, or JWKS fetch/parse fails.
pub async fn extract_user_from_id_token(token: &str) -> Result<User, String> {
    extract_user_from_token(token).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_csrf_token_produces_non_empty_secret() {
        let token = generate_csrf_token();
        assert!(!token.secret().is_empty());
    }

    #[test]
    fn auth_callback_query_deserialize() {
        let json = r#"{"code":"abc","state":"xyz"}"#;
        let q: AuthCallbackQuery = serde_json::from_str(json).unwrap();
        assert_eq!(q.code.as_deref(), Some("abc"));
        assert_eq!(q.state.as_deref(), Some("xyz"));
    }
}

