use async_sqlx_session::PostgresSessionStore;
use auth::{
    config::{oauth::oauth_client, Environment},
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
use state::AppState;

use std::{env, net::SocketAddr, path::PathBuf, time::Duration};
use tokio_postgres::NoTls;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod auth;
mod error;
mod state;

#[tokio::main]
async fn main() {
    rustls::crypto::ring::default_provider().install_default().expect("Failed to install rustls crypto provider");

    let environment = match std::env::var("RUST_ENV")
        .unwrap_or("dev".to_string())
        .as_str()
    {
        "prod" => Environment::Production,
        _ => Environment::Development,
    };
    let config = Config::builder()
        .add_source(File::with_name(&format!("config.{}.toml", environment)))
        .build()
        .unwrap();

    let addr: SocketAddr = config.get::<String>("url").unwrap().parse().unwrap();

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
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let manager = PostgresConnectionManager::new_from_stringlike(&database_url, NoTls).unwrap();
    let db_pool: Pool<PostgresConnectionManager<NoTls>> = Pool::builder()
        .connection_timeout(Duration::from_secs(5))
        .build(manager)
        .await
        .unwrap();

    let oauth_client = oauth_client(&config).unwrap();

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

    if environment == Environment::Production {
        let cert_config = RustlsConfig::from_pem_file(
            PathBuf::from(config.get::<String>("certfile").unwrap()),
            PathBuf::from(config.get::<String>("keyfile").unwrap()),
        )
        .await
        .unwrap();
        tracing::debug!("listening on https://{}", addr);
        axum_server::bind_rustls(addr, cert_config)
            .serve(app.into_make_service())
            .await
            .unwrap();
    } else {
        tracing::debug!("listening on http://{}", addr);
        axum_server::bind(addr)
            .serve(app.into_make_service())
            .await
            .unwrap();
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
