use std::sync::Arc;

pub use auth::AuthManager;
use librespot::{
    core::{Session, SessionConfig},
    discovery::Credentials,
    playback::{
        audio_backend,
        config::{AudioFormat, PlayerConfig},
        mixer::NoOpVolume,
        player::Player,
    },
};
use rspotify::AuthCodeSpotify;

use crate::Result;

mod auth;

pub struct Settings {
    session: SessionConfig,
    player: PlayerConfig,
}

impl Settings {
    pub fn new() -> Self {
        Self {
            session: SessionConfig {
                ..Default::default()
            },
            player: PlayerConfig {
                ..Default::default()
            },
        }
    }
}

pub struct SpotifyManager {
    pub auth: AuthManager,
    pub settings: Settings,
    pub session: Session,
    pub api: AuthCodeSpotify,
    pub player: Arc<Player>,
}

impl SpotifyManager {
    pub async fn new(settings: Settings) -> Result<Self> {
        let mut auth = AuthManager::new();
        let token = auth.authenticate().await?;
        let credentials = Credentials::with_access_token(&token.access_token);
        let session = Session::new(settings.session.clone(), None);
        session.connect(credentials, true).await?;

        let player = Player::new(
            settings.player.clone(),
            session.clone(),
            Box::new(NoOpVolume),
            || {
                let backend = audio_backend::find(None).expect("No audio backend found");
                backend(None, AudioFormat::default())
            },
        );

        Ok(Self {
            api: AuthCodeSpotify::from_token(token),
            session,
            player,
            settings,
            auth,
        })
    }
}
