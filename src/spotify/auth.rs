use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use rand::{RngExt, distr::Alphanumeric};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::io;
use std::net::SocketAddr;
use std::time::{Duration, SystemTime};
use thiserror::Error;

mod server_handler;

// --- Configuration Constants ---
pub const CLIENT_ID: &str = "782ae96ea60f4cdf986a766049607005";
pub const REDIRECT_HOST: &str = "127.0.0.1:8898";
pub const AUTH_ENDPOINT: &str = "https://accounts.spotify.com/authorize";
pub const TOKEN_ENDPOINT: &str = "https://accounts.spotify.com/api/token";
pub const SCOPES: &str = "user-read-private,playlist-read-private,playlist-read-collaborative,user-library-read,user-library-modify,user-top-read,user-read-recently-played,user-read-playback-state,playlist-modify-public,playlist-modify-private,user-modify-playback-state,streaming,playlist-modify-public";

// --- Data Structures ---

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Tokens {
    pub access_token: String,
    pub refresh_token: String,
    pub token_expiry_time: SystemTime,
}

#[derive(Deserialize, Debug)]
struct SpotifyTokenResponse {
    access_token: String,
    refresh_token: Option<String>,
    expires_in: u64,
}

#[derive(Debug, Error)]
pub enum OAuthError {
    #[error("Auth code param not found in URI")]
    AuthCodeNotFound,
    #[error("CSRF token param not found in URI")]
    CsrfTokenNotFound,
    #[error("Failed to bind server to {addr} ({e})")]
    AuthCodeListenerBind { addr: SocketAddr, e: io::Error },
    #[error("Request error: {0}")]
    RequestError(#[from] reqwest::Error),
    #[error("No refresh token provided")]
    NoRefreshToken,
    #[error("Mismatched state (CSRF)")]
    InvalidState,
}
pub struct Authenticator {
    client: reqwest::Client,
}

impl Authenticator {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }

    pub async fn authenticate(&self) -> Result<Tokens, OAuthError> {
        let (verifier, challenge) = generate_pkce();
        let state: String = rand_alphanum(22);

        let redirect_url = format!("http://{REDIRECT_HOST}/login");

        let auth_url = format!(
            "{}?client_id={}&response_type=code&redirect_uri={}&code_challenge_method=S256&code_challenge={}&state={}&scope={}",
            AUTH_ENDPOINT,
            CLIENT_ID,
            urlencoding::encode(&redirect_url),
            challenge,
            state,
            urlencoding::encode(SCOPES).replace("%2C", "%20")
        );
        println!("Open this URL in your browser:\n\n{}\n", auth_url);

        // 1. Wait for the redirect callback
        let code = self.wait_for_authcode(state).await?;

        // 2. Exchange code for tokens
        let params = [
            ("client_id", CLIENT_ID),
            ("grant_type", "authorization_code"),
            ("code", &code),
            ("redirect_uri", &redirect_url),
            ("code_verifier", &verifier),
        ];

        let res = self
            .client
            .post(TOKEN_ENDPOINT)
            .form(&params)
            .send()
            .await?;

        let data: SpotifyTokenResponse = res.json().await?;

        Ok(Tokens {
            access_token: data.access_token,
            refresh_token: data.refresh_token.ok_or(OAuthError::NoRefreshToken)?,
            token_expiry_time: SystemTime::now() + Duration::from_secs(data.expires_in),
        })
    }

    pub async fn refresh_token(&self, old: &Tokens) -> Result<Tokens, OAuthError> {
        let params = [
            ("client_id", CLIENT_ID),
            ("grant_type", "refresh_token"),
            ("refresh_token", &old.refresh_token),
        ];

        let res = self
            .client
            .post(TOKEN_ENDPOINT)
            .form(&params)
            .send()
            .await?;
        let data: SpotifyTokenResponse = res.json().await?;

        Ok(Tokens {
            access_token: data.access_token,
            refresh_token: data
                .refresh_token
                .unwrap_or_else(|| old.refresh_token.clone()),
            token_expiry_time: SystemTime::now() + Duration::from_secs(data.expires_in),
        })
    }
}

fn rand_alphanum(len: usize) -> String {
    rand::rng()
        .sample_iter(&Alphanumeric)
        .take(len)
        .map(char::from)
        .collect()
}

fn generate_pkce() -> (String, String) {
    let verifier = rand_alphanum(64);

    let mut hasher = Sha256::new();
    hasher.update(verifier.as_bytes());
    let challenge = URL_SAFE_NO_PAD.encode(hasher.finalize());

    (verifier, challenge)
}
