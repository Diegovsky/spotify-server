use std::env;
use std::time::SystemTime;

use anyhow::Result;
use librespot::core::{config::SessionConfig, session::Session};

use auth::Authenticator;

use auth::Tokens;

pub mod auth;

pub struct AuthManager {
    authenticator: Authenticator,
    credentials: Option<Tokens>,
}

pub const AUTH_PATH: &str = "./keys.json";

impl AuthManager {
    pub fn new() -> Self {
        Self {
            authenticator: Authenticator::new(),
            credentials: None,
        }
    }

    pub async fn refresh(&mut self) -> Result<&Tokens> {
        let creds = self.credentials.as_mut().unwrap();
        if creds.token_expiry_time < SystemTime::now() {
            let new = self.authenticator.refresh_token(&creds).await?;
            tokio::fs::write(AUTH_PATH, serde_json::to_string(&new).unwrap()).await?;
            *creds = new;
        }
        Ok(creds)
    }

    pub async fn authenticate(&mut self) -> Result<&Tokens> {
        if let Some(ref creds) = self.credentials {
            return Ok(creds);
        }

        let creds: Result<Tokens> = async {
            let contents = tokio::fs::read_to_string(AUTH_PATH).await?;
            let creds: Tokens = serde_json::from_str(&contents).unwrap();
            Result::Ok(creds)
        }
        .await;

        let creds = match creds {
            Ok(creds) => creds,
            Err(_) => {
                let creds = self.authenticator.authenticate().await?;
                tokio::fs::write(AUTH_PATH, serde_json::to_string(&creds).unwrap()).await?;
                creds
            }
        };

        return Ok(self.credentials.insert(creds));
    }
}

// async fn main() {
//     let mut session_config = SessionConfig::default();

//     let args: Vec<_> = env::args().collect();
//     if args.len() == 3 {
//         // Only special client IDs have sufficient privileges e.g. Spotify's.
//         session_config.client_id = args[2].clone()
//     } else if args.len() != 2 {
//         eprintln!("Usage: {} ACCESS_TOKEN [CLIENT_ID]", args[0]);
//         return;
//     }
//     let access_token = &args[1];

//     // Now create a new session with that token.
//     let session = Session::new(session_config.clone(), None);
//     let credentials = Credentials::with_access_token(access_token);
//     println!("Connecting with token..");
//     match session.connect(credentials, false).await {
//         Ok(()) => println!("Session username: {:#?}", session.username()),
//         Err(e) => {
//             println!("Error connecting: {e}");
//             return;
//         }
//     };

//     let token = session.token_provider().get_token(SCOPES).await.unwrap();
//     println!("Got me a token: {token:#?}");
// }
