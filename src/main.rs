use async_sqlx_session::PostgresSessionStore;
use auth::{
    config::oauth::oauth_client,
    handlers::{
        google::{google_auth, google_callback},
        session::logout,
    },
    models::user::User,
};
use axum::{extract::State, middleware, response::IntoResponse, routing::get, Router};
use axum_server::tls_rustls::RustlsConfig;
use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;
use config::{Config, File};
use settings::{AppConfig, Environment};
use state::AppState;

use rustls::crypto::ring;
use std::{env, net::SocketAddr, path::PathBuf, time::Duration};
use tokio_postgres::NoTls;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod auth;
mod error;
mod settings;
mod state;

#[tokio::main]
async fn main() {
    let provider = ring::default_provider();
    provider
        .install_default()
        .expect("Failed to install rustls crypto provider");

    let environment = match std::env::var("RUST_ENV")
        .unwrap_or("dev".to_string())
        .as_str()
    {
        "prod" => Environment::Production,
        _ => Environment::Development,
    };

    let config = Config::builder()
        .add_source(File::with_name(&format!("config.{}.toml", environment)))
        .add_source(config::Environment::default())
        .build()
        .expect("Failed to build config");

    let app_config: AppConfig = config
        .try_deserialize()
        .expect("Failed to deserialize config");
    let addr = SocketAddr::new(
        app_config
            .server
            .host
            .parse()
            .expect("Failed to parse host"),
        app_config.server.port,
    );

    if environment == Environment::Development {
        dotenvy::dotenv().ok();
    }

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("{}=trace", env!("CARGO_CRATE_NAME")).into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Attempting to connect to database");
    let database_url = app_config.database.url.clone();
    let manager = PostgresConnectionManager::new_from_stringlike(&database_url, NoTls)
        .expect("Failed to create database manager");
    let db_pool: Pool<PostgresConnectionManager<NoTls>> = Pool::builder()
        .connection_timeout(Duration::from_secs(5))
        .build(manager)
        .await
        .expect("Failed to create database pool");

    let oauth_client = oauth_client(&app_config).expect("Failed to create oauth client");

    let session_store = PostgresSessionStore::new(&database_url)
        .await
        .expect("Failed to create session store");
    session_store
        .migrate()
        .await
        .expect("Failed to migrate session store");

    let app_state = AppState {
        oauth_client,
        db_pool,
        session_store,
    };

    let public_routes = Router::new()
        .route("/", get(index))
        .route("/auth/google", get(google_auth))
        .route("/api/auth/callback/google", get(google_callback));

    let protected_routes = Router::new()
        .route("/protected", get(protected))
        .route("/logout", get(logout))
        .layer(middleware::from_fn_with_state(
            app_state.clone(),
            auth::middleware::require_auth,
        ));

    let app = Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .with_state(app_state);

    if app_config.ssl.enabled {
        let cert_path = PathBuf::from(app_config.ssl.cert_path.expect("Cert path is not set"));
        let key_path = PathBuf::from(app_config.ssl.key_path.expect("Key path is not set"));

        let cert_config = RustlsConfig::from_pem_file(cert_path, key_path)
            .await
            .expect("Failed to load TLS certificate files");
        tracing::debug!("listening on https://{}", addr);
        axum_server::bind_rustls(addr, cert_config)
            .serve(app.into_make_service())
            .await
            .expect("Failed to start server");
    } else {
        tracing::debug!("listening on http://{}", addr);
        axum_server::bind(addr)
            .serve(app.into_make_service())
            .await
            .expect("Failed to start server");
    }
}

async fn index(State(_app_state): State<AppState>, user: Option<User>) -> impl IntoResponse {
    match user {
        Some(u) => format!(
            "Hey {}! You're logged in!\nYou may now access `/protected`.\nLog out with `/logout`.",
            u.name.unwrap_or("Unknown".to_string())
        ),
        None => "You're not logged in.\nVisit `/auth/google` to do so.".to_string(),
    }
}

async fn protected(user: User) -> impl IntoResponse {
    format!("Welcome to the protected area :)\nHere's your info:\n{user:?}")
}
