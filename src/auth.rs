use crate::settings::{read_settings, save_settings};
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use chrono::Utc;
use rand::Rng;
use reqwest::Client;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

pub const SCOPES: &str = "user:read channel:read chat:write events:subscribe";
const AUTH_BASE: &str = "https://id.kick.com";

#[derive(Deserialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_in: Option<i64>,
    #[serde(default)]
    pub token_type: Option<String>,
    #[serde(default)]
    pub scope: Option<String>,
}

#[derive(Deserialize)]
struct KickUserResponse {
    data: Option<Vec<KickUser>>,
}

#[derive(Deserialize)]
struct KickUser {
    user_id: Option<u64>,
    name: Option<String>,
    slug: Option<String>,
}

#[derive(Debug)]
pub enum AuthError {
    Http(reqwest::Error),
    Api(String),
    Timeout,
}

impl std::fmt::Display for AuthError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuthError::Http(e) => write!(f, "HTTP error: {}", e),
            AuthError::Api(msg) => write!(f, "API error: {}", msg),
            AuthError::Timeout => write!(f, "Authorization timed out"),
        }
    }
}

impl std::error::Error for AuthError {}

impl From<reqwest::Error> for AuthError {
    fn from(err: reqwest::Error) -> Self {
        AuthError::Http(err)
    }
}

/// Generate a random PKCE code verifier (43-128 chars, URL-safe).
fn generate_code_verifier() -> String {
    let mut rng = rand::thread_rng();
    let bytes: Vec<u8> = (0..32).map(|_| rng.r#gen()).collect();
    URL_SAFE_NO_PAD.encode(&bytes)
}

/// Derive the S256 code challenge from a code verifier.
fn generate_code_challenge(verifier: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(verifier.as_bytes());
    let hash = hasher.finalize();
    URL_SAFE_NO_PAD.encode(hash)
}

/// Generate a random state string for CSRF protection.
fn generate_state() -> String {
    let mut rng = rand::thread_rng();
    let bytes: Vec<u8> = (0..16).map(|_| rng.r#gen()).collect();
    URL_SAFE_NO_PAD.encode(&bytes)
}

/// Start a local HTTP server, open the browser for Kick OAuth, and wait for the callback.
/// Returns the TokenResponse on success.
pub async fn start_oauth_flow(
    client_id: &str,
    client_secret: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let code_verifier = generate_code_verifier();
    let code_challenge = generate_code_challenge(&code_verifier);
    let state = generate_state();

    // Bind to a random available port
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let port = listener.local_addr()?.port();
    let redirect_uri = format!("http://localhost:{}/callback", port);

    // Build the authorization URL
    let auth_url = format!(
        "{}/oauth/authorize?response_type=code&client_id={}&redirect_uri={}&scope={}&state={}&code_challenge={}&code_challenge_method=S256",
        AUTH_BASE,
        urlencoded(client_id),
        urlencoded(&redirect_uri),
        urlencoded(SCOPES),
        urlencoded(&state),
        urlencoded(&code_challenge),
    );

    // Open the browser
    log::info!("Opening browser for Kick authorization: {}", auth_url);
    let _ = open::that(&auth_url);

    // Wait for the callback (with timeout)
    let code = tokio::time::timeout(Duration::from_secs(300), async {
        loop {
            let (mut stream, _) = listener.accept().await?;
            let mut buf = vec![0u8; 4096];
            let n = stream.read(&mut buf).await?;
            let request = String::from_utf8_lossy(&buf[..n]);

            // Parse the GET request for the authorization code
            if let Some(query) = request.split(' ').nth(1).and_then(|p| p.split('?').nth(1)) {
                let params: Vec<(&str, &str)> = query
                    .split('&')
                    .filter_map(|p| p.split_once('='))
                    .collect();

                let returned_state = params.iter().find(|(k, _)| *k == "state").map(|(_, v)| *v);
                let code = params.iter().find(|(k, _)| *k == "code").map(|(_, v)| *v);

                if let (Some(rs), Some(c)) = (returned_state, code) {
                    if rs == state {
                        // Send success response to browser
                        let response = "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\n\r\n<html><body><h2>Authorization successful!</h2><p>You can close this tab and return to OpenDeck.</p></body></html>";
                        let _ = stream.write_all(response.as_bytes()).await;
                        return Ok::<String, Box<dyn std::error::Error + Send + Sync>>(c.to_string());
                    }
                }

                // Check for error
                let error = params.iter().find(|(k, _)| *k == "error").map(|(_, v)| *v);
                if let Some(err) = error {
                    let response = "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\n\r\n<html><body><h2>Authorization failed</h2></body></html>";
                    let _ = stream.write_all(response.as_bytes()).await;
                    return Err(format!("OAuth error: {}", err).into());
                }
            }

            // Not the callback we're looking for — send a simple response and continue
            let response = "HTTP/1.1 404 Not Found\r\n\r\n";
            let _ = stream.write_all(response.as_bytes()).await;
        }
    })
    .await
    .map_err(|_| AuthError::Timeout)??;

    // Exchange the authorization code for tokens
    let token = exchange_code(client_id, client_secret, &code, &redirect_uri, &code_verifier).await?;
    store_token(token).await?;
    Ok(())
}

/// Exchange an authorization code for access/refresh tokens.
async fn exchange_code(
    client_id: &str,
    client_secret: &str,
    code: &str,
    redirect_uri: &str,
    code_verifier: &str,
) -> Result<TokenResponse, AuthError> {
    let client = Client::new();
    let params = [
        ("grant_type", "authorization_code"),
        ("client_id", client_id),
        ("client_secret", client_secret),
        ("code", code),
        ("redirect_uri", redirect_uri),
        ("code_verifier", code_verifier),
    ];

    let response = client
        .post(&format!("{}/oauth/token", AUTH_BASE))
        .form(&params)
        .send()
        .await?;

    if !response.status().is_success() {
        let error_text = response.text().await.unwrap_or_default();
        return Err(AuthError::Api(format!("Token exchange failed: {}", error_text)));
    }

    let token: TokenResponse = response.json().await?;
    Ok(token)
}

/// Refresh an expired access token.
pub async fn refresh_access_token(
    client_id: &str,
    client_secret: &str,
    refresh_token: &str,
) -> Result<TokenResponse, AuthError> {
    let client = Client::new();
    let params = [
        ("grant_type", "refresh_token"),
        ("client_id", client_id),
        ("client_secret", client_secret),
        ("refresh_token", refresh_token),
    ];

    let response = client
        .post(&format!("{}/oauth/token", AUTH_BASE))
        .form(&params)
        .send()
        .await?;

    if !response.status().is_success() {
        let error_text = response.text().await.unwrap_or_default();
        return Err(AuthError::Api(format!("Failed to refresh token: {}", error_text)));
    }

    let token: TokenResponse = response.json().await?;
    Ok(token)
}

/// After obtaining a token, fetch Kick user info and store everything.
async fn store_token(token: TokenResponse) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let client = Client::new();
    let resp = client
        .get("https://api.kick.com/public/v1/users")
        .header("Authorization", format!("Bearer {}", token.access_token))
        .send()
        .await?;

    let user_resp: KickUserResponse = resp.json().await.unwrap_or(KickUserResponse { data: None });
    let user = user_resp.data.and_then(|u| u.into_iter().next());

    let expires_at = Utc::now().timestamp() + token.expires_in.unwrap_or(3600);
    let mut settings = read_settings().await;
    settings.access_token = Some(token.access_token);
    settings.refresh_token = token.refresh_token.or(settings.refresh_token);
    settings.token_expires_at = Some(expires_at);
    if let Some(u) = user {
        settings.user_id = u.user_id.map(|id| id.to_string());
        settings.username = u.name;
        settings.channel_slug = u.slug;
    }
    save_settings(settings).await?;
    Ok(())
}

/// Get a valid (non-expired) access token, refreshing if needed.
/// Returns (access_token, user_id) if authenticated, None otherwise.
pub async fn get_valid_token() -> Option<(String, String)> {
    let settings = read_settings().await;
    if !settings.is_authenticated() {
        return None;
    }

    let access_token = settings.access_token.clone()?;
    let user_id = settings.user_id.clone()?;
    let client_id = settings.client_id.clone();
    let client_secret = settings.client_secret.clone();

    let now = Utc::now().timestamp();
    if let Some(expires_at) = settings.token_expires_at {
        if expires_at - now < 60 {
            if let Some(rt) = settings.refresh_token.clone() {
                match refresh_access_token(&client_id, &client_secret, &rt).await {
                    Ok(new_token) => {
                        let new_expires_at = Utc::now().timestamp() + new_token.expires_in.unwrap_or(3600);
                        let mut updated = settings.clone();
                        updated.access_token = Some(new_token.access_token.clone());
                        if new_token.refresh_token.is_some() {
                            updated.refresh_token = new_token.refresh_token;
                        }
                        updated.token_expires_at = Some(new_expires_at);
                        let _ = save_settings(updated).await;
                        return Some((new_token.access_token, user_id));
                    }
                    Err(_) => return None,
                }
            }
        }
    }

    Some((access_token, user_id))
}

/// Simple percent-encoding for URL query params.
fn urlencoded(s: &str) -> String {
    s.bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                String::from(b as char)
            }
            _ => format!("%{:02X}", b),
        })
        .collect()
}
