use anyhow;
use async_session::SessionStore;
use axum::{
    extract::State,
    response::{IntoResponse, Redirect},
};
use axum_extra::{headers, TypedHeader};

use crate::{auth::models::session::COOKIE_NAME, error::AppError, state::AppState};

pub async fn logout(
    State(app_state): State<AppState>,
    TypedHeader(cookies): TypedHeader<headers::Cookie>,
) -> Result<impl IntoResponse, AppError> {
    let cookie = cookies.get(COOKIE_NAME).expect("Session cookie not found");

    let session = app_state
        .session_store
        .load_session(cookie.to_string())
        .await?
        .ok_or_else(|| anyhow::anyhow!("Session not found"))?;

    app_state.session_store.destroy_session(session).await?;

    Ok(Redirect::to("/"))
}
