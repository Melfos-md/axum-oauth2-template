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
use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;
use state::AppState;

use std::{env, time::Duration};
use tokio_postgres::NoTls;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod auth;
mod error;
mod state;

#[tokio::main]
async fn main() {
    let mut addr = "0.0.0.0:80";
    if std::env::var("PRODUCTION").unwrap_or("false".to_string()) == "false" {
        dotenvy::dotenv().ok();
        addr = "127.0.0.1:80";
    }
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("{}=trace", env!("CARGO_CRATE_NAME")).into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let manager = PostgresConnectionManager::new_from_stringlike(
        "host=localhost user=postgres password=example port=5432 dbname=dermoscopy-quiz",
        NoTls,
    )
    .unwrap();
    let db_pool: Pool<PostgresConnectionManager<NoTls>> = Pool::builder()
        .connection_timeout(Duration::from_secs(5))
        .build(manager)
        .await
        .unwrap();

    let oauth_client = oauth_client().unwrap();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
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

    // run it
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .unwrap();
    tracing::debug!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
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
