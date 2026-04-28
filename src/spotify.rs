use std::sync::Arc;

pub use auth::AuthManager;
use futures::{SinkExt, channel::mpsc::Sender};
use librespot::{
    core::{Session, SessionConfig},
    discovery::Credentials,
    playback::{
        audio_backend,
        config::{AudioFormat, PlayerConfig},
        mixer::NoOpVolume,
        player::{Player, PlayerEvent},
    },
};
use rspotify::AuthCodeSpotify;
use serde::{Deserialize, Serialize};
use tokio::sync::{Mutex, mpsc::UnboundedReceiver};

use crate::{PlaybackMessage, PlayerMessage, Result};
pub use cache::CacheApi;
use cache::Cacher;

mod auth;
mod cache;

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

pub type SpotifyManagerArc = Arc<SpotifyManager>;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlabackState {
    Playing {
        paused: bool,
    },
    #[default]
    Stopped,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Repeat {
    #[default]
    No,
    Single,
    Playlist,
}

#[derive(Clone, Serialize, Deserialize, Default)]
pub struct PlayerState {
    pub playback: PlabackState,
    pub shuffled: bool,
    pub sorting: Repeat,
}

pub struct SpotifyManager {
    pub auth: AuthManager,
    pub settings: Settings,
    pub session: Session,
    pub api: CacheApi,
    pub player: Arc<Player>,
    pub player_state: Mutex<PlayerState>,
    channel: Sender<PlayerMessage>,
}

impl SpotifyManager {
    pub async fn new(channel: Sender<PlayerMessage>, settings: Settings) -> Result<Arc<Self>> {
        let cacher = Cacher::new().await?;

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
        let tx = player.get_player_event_channel();
        let this = Arc::new(Self {
            api: CacheApi::new(AuthCodeSpotify::from_token(token), cacher),
            player_state: Mutex::new(PlayerState::default()),
            session,
            player,
            settings,
            auth,
            channel,
        });
        tokio::spawn(this.clone().player_message_handler(tx));

        Ok(this)
    }

    async fn player_message_handler(
        self: Arc<Self>,
        mut spotify_ch: UnboundedReceiver<PlayerEvent>,
    ) {
        let mut tx = self.channel.clone();
        while let Some(event) = spotify_ch.recv().await {
            eprintln!("event: {event:#?}");

            let msg = match event {
                PlayerEvent::Stopped { .. } => PlaybackMessage::Stopped,
                PlayerEvent::Loading { .. } => PlaybackMessage::Loading,
                PlayerEvent::Playing { .. } => PlaybackMessage::Started,
                PlayerEvent::Paused { .. } => PlaybackMessage::Paused,
                PlayerEvent::EndOfTrack { .. } => PlaybackMessage::TrackEnded,
                PlayerEvent::Unavailable { .. } => PlaybackMessage::Unavailable,
                PlayerEvent::TrackChanged { .. } => PlaybackMessage::TrackChanged,

                _ => {
                    continue;
                }
            };
            {
                use PlaybackMessage::*;
                let mut player = self.player_state.lock().await;
                match msg {
                    Stopped | Unavailable | TrackEnded => player.playback = PlabackState::Stopped,
                    Started | Loading { .. } | TrackChanged => {
                        player.playback = PlabackState::Playing { paused: false }
                    }

                    Paused => player.playback = PlabackState::Playing { paused: true },
                }
            }
            tx.send(PlayerMessage::PlaybackMessage(msg)).await.unwrap();
        }
    }
}
