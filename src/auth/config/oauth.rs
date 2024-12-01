use std::env;

use oauth2::{basic::BasicClient, AuthUrl, ClientId, ClientSecret,RedirectUrl, TokenUrl};

use crate::error::AppError;

pub fn oauth_client() -> Result<BasicClient, AppError> {
    let client_id = env::var("GOOGLE_CLIENT_ID")?;
    let client_secret = env::var("GOOGLE_CLIENT_SECRET")?;
    let redirect_url = env::var("REDIRECT_URL")
        .unwrap_or_else(|_| "http://localhost:3000/api/auth/callback/google".to_string());

    Ok(BasicClient::new(
        ClientId::new(client_id),
        Some(ClientSecret::new(client_secret)),
        AuthUrl::new("https://accounts.google.com/o/oauth2/v2/auth".to_string())?,
        Some(TokenUrl::new(
            "https://oauth2.googleapis.com/token".to_string(),
        )?),
    )
    .set_redirect_uri(RedirectUrl::new(redirect_url)?))
}
