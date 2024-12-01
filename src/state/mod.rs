use async_sqlx_session::PostgresSessionStore;
use axum::extract::FromRef;
use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;
use oauth2::basic::BasicClient;
use tokio_postgres::NoTls;

#[derive(Clone)]
pub struct AppState {
    pub oauth_client: BasicClient,
    pub db_pool: Pool<PostgresConnectionManager<NoTls>>,
    pub session_store: PostgresSessionStore,
}

impl FromRef<AppState> for BasicClient {
    fn from_ref(state: &AppState) -> Self {
        state.oauth_client.clone()
    }
}

impl FromRef<AppState> for Pool<PostgresConnectionManager<NoTls>> {
    fn from_ref(state: &AppState) -> Self {
        state.db_pool.clone()
    }
}

impl FromRef<AppState> for PostgresSessionStore {
    fn from_ref(state: &AppState) -> Self {
        state.session_store.clone()
    }
}
