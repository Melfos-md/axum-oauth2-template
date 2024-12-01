use crate::AppState;
use axum::{
    extract::{FromRequestParts, Request, State},
    middleware::Next,
    response::{IntoResponse, Response},
};

use super::models::user::User;

pub async fn require_auth(State(state): State<AppState>, request: Request, next: Next) -> Response {
    let (mut parts, body) = request.into_parts();

    match User::from_request_parts(&mut parts, &state).await {
        Ok(user) => {
            parts.extensions.insert(user);
            let request = Request::from_parts(parts, body);
            next.run(request).await
        }
        Err(redirect) => redirect.into_response(),
    }
}
