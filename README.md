# Axum OAuth2 Template

This project is inspired by the [official Axum OAuth example](https://github.com/tokio-rs/axum/blob/main/examples/oauth/src/main.rs). I created this template to facilitate the implementation of OAuth2 authentication with Google in Rust/Axum applications. As I am not an IT professional, any comments and suggestions for improvement are welcome!

## Prerequisites

Before using this template, ensure you have the following:

1. **Rust Installation**

2. **Database**
   - PostgreSQL
   - A database created for the project
   ```sql
   CREATE DATABASE your_database_name;
   ```

3. **Google OAuth2 Client**
   - Create a project in [Google Cloud Console](https://console.cloud.google.com/)
   - Enable Google OAuth2 API
   - Create OAuth2 credentials (Client ID and Client Secret)
   - Configure authorized redirect URIs (example: `http://localhost:3000/api/auth/callback/google`)
   - Configure the Authorized JavaScript origins (example: `http://localhost:3000`)

4. **Environment Variables**
   En développement, créez un fichier `.env` dans la racine du projet :
   ```env
   DATABASE_URL=postgresql://user:password@localhost/your_database_name
   GOOGLE_CLIENT_ID=your_client_id
   GOOGLE_CLIENT_SECRET=your_client_secret
   ```
   
   En production, les variables d'environnement doivent être passées directement lors du lancement de l'application :
   ```bash
   DATABASE_URL=postgresql://user:password@localhost/your_database_name \
   GOOGLE_CLIENT_ID=your_client_id \
   GOOGLE_CLIENT_SECRET=your_client_secret \
   ./your_application
   ```

5. **Fichiers de Configuration**
   L'application utilise des fichiers de configuration TOML selon l'environnement.
   
   Pour le développement, créez un fichier `config.dev.toml` :
   ```toml
   [server]
   host = "127.0.0.1"
   port = 3000

   [ssl]
   enabled = false

   [database]
   url = "postgresql://user:password@localhost/your_database_name" # will be replaced by .env

   [google]
   client_id = "your_client_id" # will be replaced by .env
   client_secret = "your_client_secret" # will be replaced by .env
   redirect_url = "http://your_redirect_url"
   auth_url = "https://accounts.google.com/o/oauth2/v2/auth"
   token_url = "https://oauth2.googleapis.com/token"
   ```
   
   Pour la production, créez un fichier `config.prod.toml` :
   ```toml
   [server]
   host = "0.0.0.0"
   port = 443

   [ssl]
   enabled = true
   cert_path = "/etc/letsencrypt/live/sosplanning.r-mont.fr/fullchain.pem"
   key_path = "/etc/letsencrypt/live/sosplanning.r-mont.fr/privkey.pem"

   [database]
   url = "postgresql://user:password@localhost/your_database_name" # will be replaced by env variable

   [google]
   client_id = "your_client_id" # will be replaced by env variable
   client_secret = "your_client_secret" # will be replaced by env variable
   redirect_url = "http://your_redirect_url"
   auth_url = "https://accounts.google.com/o/oauth2/v2/auth"
   token_url = "https://oauth2.googleapis.com/token"
   ```
   
   Ces fichiers doivent être ajoutés au `.gitignore` pour éviter d'exposer des informations sensibles.

6. **Development Tools**
   ```bash
   # For development with auto-reload
   cargo install cargo-watch
   
   # For database migrations
   cargo install sqlx-cli
   ```

Start the project with:
```bash
cargo run
# or with auto-reload
cargo watch -x run
```

The server will start on `http://localhost:3000` by default.

## Authentication Flow

1. **Initial Authentication Request**
   - The user visits the `/auth/google` endpoint.
   - The server generates a CSRF token and creates a session.
   - The user is redirected to Google's consent page to grant permissions.

2. **Google Callback**
   - After the user grants permission, Google redirects back to the application with an authorization code.
   - The server validates the CSRF token.
   - The server exchanges the authorization code for user information.
   - A user session is created with an expiration.

3. **Protected Routes**
   - Middleware checks for a valid session before allowing access to protected routes.

4. **Session Management**
   - Sessions are stored in a PostgreSQL database.
   - The cookie only contains the session ID.
   - Secure cookie settings are set: HttpOnly, Secure, and SameSite=Lax.


## Security Measures

1. **CSRF Protection**: Each authentication request generates a unique CSRF token stored in the session to prevent Cross-Site Request Forgery attacks. This token is validated when Google calls back our application.

2. **PKCE (Proof Key for Code Exchange)**: The OAuth2 flow is secured using PKCE, which generates a unique code verifier and challenge for each authentication attempt. This prevents authorization code interception attacks.

3. **Secure Cookie Configuration**:
   - HttpOnly
   - Secure
   - SameSite=Lax