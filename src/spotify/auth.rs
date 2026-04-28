use rspotify::{
    AuthCodePkceSpotify, Credentials, Token,
    prelude::{BaseClient, OAuthClient},
};

use crate::Result;

pub struct AuthManager {
    client: AuthCodePkceSpotify,
}

pub const CLIENT_ID: &str = "69ae761c99634e4395fb20ae42104971";
pub const REDIRECT_URL: &str = "http://127.0.0.1:8891/login";
pub const SCOPES: &str = "user-read-private,playlist-read-private,playlist-read-collaborative,user-library-read,user-library-modify,user-top-read,user-read-recently-played,user-read-playback-state,playlist-modify-public,playlist-modify-private,user-modify-playback-state,streaming,playlist-modify-public";

impl AuthManager {
    pub fn new() -> Self {
        Self {
            client: AuthCodePkceSpotify::with_config(
                Credentials::new_pkce(CLIENT_ID),
                rspotify::OAuth {
                    redirect_uri: REDIRECT_URL.into(),
                    scopes: SCOPES.split(",").map(ToOwned::to_owned).collect(),
                    ..Default::default()
                },
                rspotify::Config {
                    token_cached: true,
                    token_refreshing: true,
                    ..Default::default()
                },
            ),
        }
    }
    pub async fn authenticate(&mut self) -> Result<Token> {
        let url = &*self.client.get_authorize_url(None)?;

        match self.client.read_token_cache(true).await {
            Ok(Some(new_token)) => {
                let expired = new_token.is_expired();

                // Load token into client regardless of whether it's expired o
                // not, since it will be refreshed later anyway.
                *self.client.get_token().lock().await.unwrap() = Some(new_token);

                if expired {
                    // Ensure that we actually got a token from the refetch
                    match self.client.refetch_token().await {
                        Ok(Some(refreshed_token)) => {
                            println!("Successfully refreshed expired token from token cache");
                            *self.client.get_token().lock().await.unwrap() = Some(refreshed_token)
                        }
                        // If not, prompt the user for it
                        Ok(None) => {
                            println!("Unable to refresh expired token from token cache");
                            let code = self.client.get_code_from_user(url)?;
                            self.client.request_token(&code).await?;
                        }
                        Err(err) => {
                            println!("Error refetching token: {err}. Falling back to user prompt.");
                            // If the cached token contains invalid data, we want to re-login
                            let code = self.client.get_code_from_user(url)?;
                            self.client.request_token(&code).await?;
                        }
                    }
                }
            }
            // Otherwise following the usual procedure to get the token.
            _ => {
                println!("Open:\n{url}");
                let code = self
                    .client
                    .get_authcode_listener(self.client.get_socket_address(REDIRECT_URL).unwrap())?;
                self.client.request_token(&code).await?;
            }
        }

        self.client.write_token_cache().await?;
        let token = self.client.token.lock().await.unwrap().clone().unwrap();
        Ok(token)
    }
}
