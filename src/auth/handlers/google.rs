use crate::{
    auth::models::{
        oauth::{AuthRequest, GoogleUser},
        session::{COOKIE_NAME, CSRF_TOKEN, PKCE_VERIFIER},
        user::User,
    },
    error::AppError,
};
use anyhow;
use async_session::{Session, SessionStore};
use async_sqlx_session::PostgresSessionStore;
use axum::{
    extract::{Query, State},
    http::header::{HeaderMap, SET_COOKIE},
    response::{IntoResponse, Redirect},
};
use axum_extra::{headers, TypedHeader};
use chrono::{Duration as ChronoDuration, Utc};
use oauth2::{
    basic::BasicClient, reqwest::async_http_client, AuthorizationCode, CsrfToken,
    PkceCodeChallenge, PkceCodeVerifier, Scope, TokenResponse,
};
use tracing;

pub async fn google_auth(
    State(client): State<BasicClient>,
    State(session_store): State<PostgresSessionStore>,
) -> Result<impl IntoResponse, AppError> {
    let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

    let (auth_url, csrf_token) = client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("email".to_string()))
        .add_scope(Scope::new("profile".to_string()))
        .set_pkce_challenge(pkce_challenge)
        .url();

    // Create a session to store both csrf_token and pkce_verifier
    let mut session = Session::new();
    session.insert(CSRF_TOKEN, csrf_token.secret())?;
    session.insert(PKCE_VERIFIER, pkce_verifier.secret())?;

    // Store the session
    let cookie_value = session_store
        .store_session(session)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Failed to retrieve cookie value"))?;

    //let cookie = format!("{COOKIE_NAME}={cookie_value}; SameSite=Lax; HttpOnly; Secure; Path=/");
    let cookie = format!("{COOKIE_NAME}={cookie_value}; SameSite=Lax; HttpOnly; Path=/");
    let mut headers = HeaderMap::new();
    headers.insert(SET_COOKIE, cookie.parse()?);

    Ok((headers, Redirect::to(auth_url.as_ref())))
}

async fn validate_csrf_token(
    auth_request: &AuthRequest,
    cookies: &headers::Cookie,
    session_store: &PostgresSessionStore,
) -> Result<Session, AppError> {
    let cookie = cookies
        .get(COOKIE_NAME)
        .ok_or_else(|| anyhow::anyhow!("Session cookie not found"))?
        .to_string();

    let session = session_store
        .load_session(cookie)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Session not found"))?;

    let stored_csrf_token = session
        .get::<String>(CSRF_TOKEN)
        .ok_or_else(|| anyhow::anyhow!("CSRF token not found in session"))?;

    if stored_csrf_token != auth_request.state {
        session_store.destroy_session(session).await?;
        return Err(anyhow::anyhow!("CSRF token mismatch").into());
    }

    Ok(session)
}

pub async fn google_callback(
    Query(query): Query<AuthRequest>,
    State(session_store): State<PostgresSessionStore>,
    State(oauth_client): State<BasicClient>,
    TypedHeader(cookies): TypedHeader<headers::Cookie>,
) -> Result<impl IntoResponse, AppError> {
    // Validate CSRF and get session
    let session = validate_csrf_token(&query, &cookies, &session_store).await?;

    let pkce_verifier = session
        .get::<String>(PKCE_VERIFIER)
        .ok_or_else(|| anyhow::anyhow!("PKCE verifier not found in session"))?;

    // Exchange the code with PKCE verification
    let token = oauth_client
        .exchange_code(AuthorizationCode::new(query.code))
        .set_pkce_verifier(PkceCodeVerifier::new(pkce_verifier))
        .request_async(async_http_client)
        .await
        .map_err(|e| {
            tracing::error!("Token exchange error: {:?}", e);
            anyhow::anyhow!("Token exchange failed: {}", e)
        })?;

    let client = reqwest::Client::new();
    let response = client
        .get("https://www.googleapis.com/oauth2/v2/userinfo")
        .bearer_auth(token.access_token().secret())
        .send()
        .await
        .map_err(|e| {
            tracing::error!("Google request error: {:?}", e);
            anyhow::anyhow!("Google request failed: {}", e)
        })?;

    let user_data = response.json::<GoogleUser>().await.map_err(|e| {
        tracing::error!("JSON parsing error: {:?}", e);
        anyhow::anyhow!("JSON parsing failed: {}", e)
    })?;

    session_store.destroy_session(session).await?;
    let mut session = Session::new();
    let user = User {
        id: None,
        name: Some(user_data.name),
        email: user_data.email,
        email_verified: if user_data.verified_email {
            Some(Utc::now())
        } else {
            None
        },
        image: user_data.picture,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    session.set_expiry(Utc::now() + ChronoDuration::days(30));

    session.insert("user", &user)?;

    let cookie_value = session_store
        .store_session(session)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Failed to retrieve cookie value"))?;

    let cookie = format!("{COOKIE_NAME}={cookie_value}; SameSite=Lax; HttpOnly; Secure; Path=/");
    let mut headers = HeaderMap::new();
    headers.insert(SET_COOKIE, cookie.parse()?);

    Ok((headers, Redirect::to("/")))
}
