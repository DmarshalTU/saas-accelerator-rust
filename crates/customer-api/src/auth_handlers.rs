use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Redirect, Response},
};
use oauth2::{
    basic::BasicClient, reqwest::async_http_client, AuthorizationCode, Scope, TokenResponse,
};
use shared::auth::user_auth::{
    AuthCallbackQuery, AuthConfig, OAuthClientConfig, User, extract_user_from_id_token,
    generate_csrf_token,
};
use tower_sessions::Session;
use tracing::{error, info};

/// Login handler - redirects to Azure AD
pub async fn login_handler(
    State(auth_client): State<BasicClient>,
    session: Session,
) -> Result<Redirect, StatusCode> {
    let csrf_token = generate_csrf_token();
    session
        .insert("oauth_csrf", csrf_token.secret())
        .await
        .map_err(|_| {
            error!("Failed to store CSRF token");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let (auth_url, _csrf_token) = auth_client
        .authorize_url(|| csrf_token.clone())
        .add_scope(Scope::new("openid".to_string()))
        .add_scope(Scope::new("profile".to_string()))
        .add_scope(Scope::new("email".to_string()))
        .url();

    Ok(Redirect::to(auth_url.as_str()))
}

/// Callback handler - processes OAuth callback
pub async fn callback_handler(
    State(auth_client): State<BasicClient>,
    Query(params): Query<AuthCallbackQuery>,
    session: Session,
) -> Result<Response, StatusCode> {
    if let Some(error) = params.error {
        error!(
            "OAuth error: {} - {}",
            error,
            params.error_description.unwrap_or_default()
        );
        return Err(StatusCode::BAD_REQUEST);
    }

    // Verify CSRF token
    let stored_csrf: Option<String> = session.get("oauth_csrf").await.map_err(|e| {
        error!("Failed to get CSRF token from session: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    session.remove("oauth_csrf").await.ok();

    if let Some(ref state) = params.state {
        if stored_csrf.as_ref().map(|s| s.as_str()) != Some(state) {
            error!("CSRF token mismatch");
            return Err(StatusCode::BAD_REQUEST);
        }
    }

    let code = params.code.ok_or(StatusCode::BAD_REQUEST)?;

    let token_result = auth_client
        .exchange_code(AuthorizationCode::new(code))
        .request_async(async_http_client)
        .await
        .map_err(|e| {
            error!("Token exchange failed: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // Azure AD returns ID token in the token response when using openid scope
    let id_token = token_result
        .extra_fields()
        .id_token()
        .map(|token| token.secret().clone())
        .ok_or_else(|| {
            error!("No ID token in response");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let user = extract_user_from_id_token(&id_token)
        .await
        .map_err(|e| {
            error!("Failed to extract user from token: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // Store user in session
    session.insert("user_id", &user.id).await.map_err(|e| {
        error!("Failed to store user in session: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    session.insert("user_email", &user.email).await.map_err(|e| {
        error!("Failed to store email in session: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    session.insert("user_name", &user.name).await.map_err(|e| {
        error!("Failed to store name in session: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    info!("User logged in: {} ({})", user.email, user.name);

    Ok(Redirect::to("/").into_response())
}

/// Logout handler
pub async fn logout_handler(
    State(config): State<AuthConfig>,
    session: Session,
) -> Redirect {
    session.remove("user_id").await.ok();
    session.remove("user_email").await.ok();
    session.remove("user_name").await.ok();

    if let Err(e) = session.delete().await {
        error!("Failed to delete session: {}", e);
    }

    info!("User logged out");
    Redirect::to(&config.signed_out_redirect_uri)
}

/// Get current user from session
pub async fn get_current_user(session: Session) -> Option<User> {
    let user_id: Option<String> = session.get("user_id").await.ok()??;
    let email: Option<String> = session.get("user_email").await.ok()??;
    let name: Option<String> = session.get("user_name").await.ok()??;

    Some(User { id: user_id, email, name })
}

/// Require authentication - returns error if not authenticated
pub async fn require_auth(session: Session) -> Result<User, StatusCode> {
    get_current_user(session)
        .await
        .ok_or_else(|| {
            error!("User not authenticated");
            StatusCode::UNAUTHORIZED
        })
}

