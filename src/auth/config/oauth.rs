use oauth2::{basic::BasicClient, AuthUrl, ClientId, ClientSecret, RedirectUrl, TokenUrl};

use crate::{error::AppError, settings::AppConfig};

pub fn oauth_client(config: &AppConfig) -> Result<BasicClient, AppError> {
    let client_id = config.google_client_id.clone();
    let client_secret = config.google_client_secret.clone();
    let redirect_url = config.google.redirect_url.clone();
    let auth_url = config.google.auth_url.clone();
    let token_url = config.google.token_url.clone();

    Ok(BasicClient::new(
        ClientId::new(client_id),
        Some(ClientSecret::new(client_secret)),
        AuthUrl::new(auth_url)?,
        Some(TokenUrl::new(token_url)?),
    )
    .set_redirect_uri(RedirectUrl::new(redirect_url)?))
}
