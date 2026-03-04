use axum::{
    extract::{Query, State},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Json, Redirect, Response},
};
use axum::http::Request;
use data::repositories::ROLE_ID_ADMIN;
use oauth2::Scope;
use serde::Deserialize;
use shared::auth::user_auth::{
    extract_user_from_id_token, AuthCallbackQuery, User, generate_csrf_token,
};
use tower_sessions::Session;
use tracing::{error, info};

/// Azure AD token response may include `id_token` when openid scope is used.
#[derive(Deserialize)]
struct TokenResponse {
    id_token: Option<String>,
}

/// Login handler - redirects to Azure AD
pub async fn login_handler(
    State(state): State<crate::handlers::AppState>,
    session: Session,
) -> Result<Redirect, StatusCode> {
    let auth_client = state
        .auth_client
        .as_ref()
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
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
    State(state): State<crate::handlers::AppState>,
    Query(params): Query<AuthCallbackQuery>,
    session: Session,
) -> Result<Response, StatusCode> {
    let _auth_client = state
        .auth_client
        .as_ref()
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
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
    session.remove::<String>("oauth_csrf").await.ok();

    if let (Some(state), Some(csrf)) = (params.state.as_ref(), stored_csrf.as_ref())
        && csrf.as_str() != state
    {
        error!("CSRF token mismatch");
        return Err(StatusCode::BAD_REQUEST);
    }

    let code = params.code.ok_or(StatusCode::BAD_REQUEST)?;
    let auth_config = state.auth_config.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;

    // Exchange code for tokens via custom request so we can read id_token from the response body.
    let token_url = format!("{}/token", auth_config.authority_url());
    let client = reqwest::Client::new();
    let token_res = client
        .post(&token_url)
        .form(&[
            ("grant_type", "authorization_code"),
            ("client_id", auth_config.client_id.as_str()),
            ("client_secret", auth_config.client_secret.as_str()),
            ("code", code.as_str()),
            ("redirect_uri", auth_config.redirect_uri.as_str()),
        ])
        .send()
        .await
        .map_err(|e| {
            error!("Token exchange request failed: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    if !token_res.status().is_success() {
        let status = token_res.status();
        let body = token_res.text().await.unwrap_or_default();
        error!("Token exchange failed {}: {}", status, body);
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let token_body: TokenResponse = token_res.json().await.map_err(|e| {
        error!("Failed to parse token response: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let id_token = token_body.id_token.ok_or_else(|| {
        error!("Token response missing id_token");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let user = extract_user_from_id_token(&id_token).await.map_err(|e| {
        error!("Failed to extract user from id_token: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    session.insert("user_id", user.id.clone()).await.map_err(|e| {
        error!("Failed to store user_id in session: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    session.insert("user_email", user.email.clone()).await.map_err(|e| {
        error!("Failed to store user_email in session: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    session.insert("user_name", user.name.clone()).await.map_err(|e| {
        error!("Failed to store user_name in session: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    info!("User logged in: {}", user.email);
    let redirect_url = auth_config.signed_out_redirect_uri.as_str();
    Ok(Redirect::to(redirect_url).into_response())
}

/// Logout handler
pub async fn logout_handler(
    State(state): State<crate::handlers::AppState>,
    session: Session,
) -> Redirect {
    session.remove::<String>("user_id").await.ok();
    session.remove::<String>("user_email").await.ok();
    session.remove::<String>("user_name").await.ok();

    if let Err(e) = session.delete().await {
        error!("Failed to delete session: {}", e);
    }

    info!("User logged out");
    let redirect = state
        .auth_config
        .as_ref()
        .map_or("/", |c| c.signed_out_redirect_uri.as_str());
    Redirect::to(redirect)
}

/// Get current user from session
pub async fn get_current_user(session: Session) -> Option<User> {
    let user_id: Option<String> = session.get("user_id").await.ok()??;
    let email: Option<String> = session.get("user_email").await.ok()??;
    let name: Option<String> = session.get("user_name").await.ok()??;

    Some(User {
        id: user_id?,
        email: email?,
        name: name?,
    })
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

/// Current user endpoint - uses `require_auth` and `get_current_user`
pub async fn me_handler(session: Session) -> impl IntoResponse {
    match require_auth(session).await {
        Ok(user) => (StatusCode::OK, Json(user)).into_response(),
        Err(sc) => sc.into_response(),
    }
}

/// Middleware: when Azure AD is configured, require authenticated session and known-user (admin role).
/// When Azure AD is not configured (local/Docker dev), allow all requests. Returns 401 if not signed in, 403 if not in known users.
pub async fn require_admin_auth_middleware(
    State(state): State<crate::handlers::AppState>,
    session: Session,
    request: Request<axum::body::Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    if state.auth_config.is_none() {
        return Ok(next.run(request).await);
    }

    let user = get_current_user(session).await.ok_or_else(|| {
        error!("Admin route accessed without session");
        StatusCode::UNAUTHORIZED
    })?;

    let is_known = state
        .known_users_repo
        .get_by_email_and_role(user.email.as_str(), ROLE_ID_ADMIN)
        .await
        .map_err(|e| {
            error!("Known users lookup failed: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    if is_known.is_none() {
        error!("Admin access denied: {} not in known users", user.email);
        return Err(StatusCode::FORBIDDEN);
    }

    Ok(next.run(request).await)
}

