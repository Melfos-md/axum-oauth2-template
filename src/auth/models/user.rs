use crate::error::AppError;
use async_session::SessionStore;
use async_sqlx_session::PostgresSessionStore;
use axum::{
    async_trait,
    extract::{FromRef, FromRequestParts},
    http::request::Parts,
    RequestPartsExt,
};
use axum_extra::TypedHeader;
use axum_extra::{headers, typed_header::TypedHeaderRejectionReason};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::auth::models::oauth::AuthRedirect;

use super::session::COOKIE_NAME;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct User {
    pub id: Option<i32>,
    pub name: Option<String>,
    pub email: String,
    pub email_verified: Option<DateTime<Utc>>,
    pub image: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[async_trait]
impl<S> FromRequestParts<S> for User
where
    PostgresSessionStore: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = AuthRedirect;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let cookies = parts
            .extract::<TypedHeader<headers::Cookie>>()
            .await
            .map_err(|e| match *e.reason() {
                TypedHeaderRejectionReason::Missing => AuthRedirect,
                _ => AppError::from(e).into(),
            })?;

        let session_store = PostgresSessionStore::from_ref(state);

        let session_cookie = cookies.get(COOKIE_NAME).ok_or(AuthRedirect)?;

        let session = session_store
            .load_session(session_cookie.to_string())
            .await?
            .ok_or(anyhow::anyhow!("Session not found"))?;

        if session.is_expired() {
            session_store.destroy_session(session).await?;
            return Err(AuthRedirect);
        }

        let user = session.get::<User>("user").ok_or(AuthRedirect)?;
        Ok(user)
    }
}
