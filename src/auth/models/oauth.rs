use axum::response::{IntoResponse, Redirect, Response};
use serde::{Deserialize, Serialize};

use crate::error::AppError;

#[derive(Debug, Serialize, Deserialize)]
pub struct GoogleUser {
    pub id: String,
    pub email: String,
    pub verified_email: bool,
    pub name: String,
    pub picture: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AuthRequest {
    pub code: String,
    pub state: String,
}

pub struct AuthRedirect;

impl IntoResponse for AuthRedirect {
    fn into_response(self) -> Response {
        Redirect::to("/").into_response()
    }
}

impl From<AppError> for AuthRedirect {
    fn from(_: AppError) -> Self {
        AuthRedirect
    }
}

impl From<async_session::Error> for AuthRedirect {
    fn from(_: async_session::Error) -> Self {
        AuthRedirect
    }
}
