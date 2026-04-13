use std::env;

use librespot::core::{authentication::Credentials, config::SessionConfig, session::Session};

mod auth;

pub const CLIENT_ID: &str = "782ae96ea60f4cdf986a766049607005";
pub const REDIRECT_URI: &str = "http://127.0.0.1:8898/login";
pub const SCOPES: &str = "user-read-private,\
playlist-read-private,\
playlist-read-collaborative,\
user-library-read,\
user-library-modify,\
user-top-read,\
user-read-recently-played,\
user-read-playback-state,\
playlist-modify-public,\
playlist-modify-private,\
user-modify-playback-state,\
streaming,\
playlist-modify-public";

async fn main() {
    let mut session_config = SessionConfig::default();

    let args: Vec<_> = env::args().collect();
    if args.len() == 3 {
        // Only special client IDs have sufficient privileges e.g. Spotify's.
        session_config.client_id = args[2].clone()
    } else if args.len() != 2 {
        eprintln!("Usage: {} ACCESS_TOKEN [CLIENT_ID]", args[0]);
        return;
    }
    let access_token = &args[1];

    // Now create a new session with that token.
    let session = Session::new(session_config.clone(), None);
    let credentials = Credentials::with_access_token(access_token);
    println!("Connecting with token..");
    match session.connect(credentials, false).await {
        Ok(()) => println!("Session username: {:#?}", session.username()),
        Err(e) => {
            println!("Error connecting: {e}");
            return;
        }
    };

    let token = session.token_provider().get_token(SCOPES).await.unwrap();
    println!("Got me a token: {token:#?}");
}
