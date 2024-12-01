`cargo watch -x run`
`cargo watch -c -w src -x run` // watch changes in src and clear the console

Création du docker postgres
`docker network create mon_reseau`
`docker volume create mon_volume_postgres`
`docker run --name some-postgres --network mon_reseau -p 5432:5432 -v mon_volume_postgres:/var/lib/postgresql/data -e POSTGRES_PASSWORD=mon_mot_de_passe -d postgres`

Création du docker adminer : 
`docker run --name mon_adminer --network mon_reseau -p 8080:8080 adminerdocker run --name mon_adminer --network mon_reseau -p 8080:8080 adminer`

puis `docker start some-postgres` et `docker start mon_adminer`
Adresse réseau du postgres : `some-postgres`


`cargo build --release`
`mv ./target/release/sosplanning-final ./opt/sosplanning`
`RUST_ENV=prod DATABASE_URL=postgresql://postgres:mon_mot_de_passe@some-postgres:5432/sosplanning GOOGLE_CLIENT_ID=GOOGLE_CLIENT_ID GOOGLE_CLIENT_SECRET=GOOGLE_CLIENT_SECRET ./sosplanning-final`


https://certbot.eff.org/instructions?ws=other&os=pip



TODO:
- [X] Mettre en place le dynDNS
- [ ] Tester l'application en production en http
    - [ ] Mettre à jour les url sur le client Google
- [ ] Mettre en place le HTTPS
    - [ ] Remettre le Secure dans les cookies

```rust
let client = CoreClient::from_provider_metadata(
    ...
    .set_revocation_url(
        RevocationUrl::new(revocation_endpoint).unwrap_or_else(|err| {
            handle_error(&err, "Invalid revocation endpoint URL");
            unreachable!();
        }),
    );
```

```rust
// Middleware pour logger les requêtes
// à supprimer
async fn log_requests(req: axum::http::Request<Body>, next: Next) -> Response {
    let start_time = Instant::now();
    let method = req.method().clone();
    let path = req.uri().path().to_string();

    let response = next.run(req).await;

    let duration = start_time.elapsed();
    info!(
        method = %method,
        path = %path,
        status = %response.status().as_u16(),
        duration = ?duration,
        "Handled request"
    );

    response
}
...
    let app = Router::new()
        .route("/", get(using_connection_pool_extractor))
        .layer(middleware::from_fn(log_requests))
        .with_state(pool);


// requête base de données
type ConnectionPool = Pool<PostgresConnectionManager<NoTls>>;

async fn using_connection_pool_extractor(
    State(pool): State<ConnectionPool>,
) -> Result<String, (StatusCode, String)> {
    let conn = pool.get().await.map_err(internal_error)?;

    let rows = conn
        .query(
            "SELECT id, url, diagnosis, localization FROM \"QuizImage\" LIMIT 5",
            &[],
        )
        .await
        .map_err(internal_error)?;

    let mut results = Vec::new();
    for row in rows {
        let id: i32 = row.try_get("id").map_err(internal_error)?;
        let image_url: String = row.try_get("url").map_err(internal_error)?;
        let diagnosis: String = row.try_get("diagnosis").map_err(internal_error)?;
        let localization: String = row.try_get("localization").map_err(internal_error)?;

        let formatted = format!(
            "ID: {}, URL: {}, Diagnostic: {}, localization : {}",
            id, image_url, diagnosis, localization
        );
        results.push(formatted);
    }

    Ok(results.join("\n"))
}

/// Utility function for mapping any error into a `500 Internal Server Error`
/// response.
fn internal_error<E>(err: E) -> (StatusCode, String)
where
    E: std::error::Error,
{
    (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
}

```
```sql
-- Add migration script here
CREATE TABLE IF NOT EXISTS "users" (
    id SERIAL PRIMARY KEY,
    name TEXT,
    email TEXT UNIQUE NOT NULL,
    emailVerified TIMESTAMP,
    image TEXT,
    createdAt TIMESTAMP DEFAULT NOW(),
    updatedAt TIMESTAMP DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS "account" (
    userId INTEGER NOT NULL,
    type TEXT NOT NULL,
    provider TEXT NOT NULL,
    providerAccountId TEXT NOT NULL,
    refresh_token TEXT,
    access_token TEXT,
    expires_at INT,
    token_type TEXT,
    scope TEXT,
    id_token TEXT,
    session_state TEXT,
    createdAt TIMESTAMP DEFAULT NOW(),
    updatedAt TIMESTAMP DEFAULT NOW(),
    PRIMARY KEY (provider, providerAccountId),
    FOREIGN KEY (userId) REFERENCES "users" (id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS "oauth_sessions" (
    sessionToken TEXT UNIQUE NOT NULL,
    userId INTEGER NOT NULL,
    expires TIMESTAMP NOT NULL,
    createdAt TIMESTAMP DEFAULT NOW(),
    updatedAt TIMESTAMP DEFAULT NOW(),
    FOREIGN KEY (userId) REFERENCES "users" (id) ON DELETE CASCADE
);

```