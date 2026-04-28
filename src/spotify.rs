use std::sync::Arc;

pub use auth::AuthManager;
use futures::{SinkExt, channel::mpsc::Sender};
use librespot::{
    connect::Spirc,
    core::{Session, SessionConfig},
    discovery::{Credentials, DeviceType},
    playback::{
        audio_backend,
        config::{AudioFormat, PlayerConfig},
        mixer::{self, Mixer, NoOpVolume},
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
pub enum PlaybackState {
    Playing {
        paused: bool,
    },
    #[default]
    Stopped,
}

impl PlaybackState {
    pub fn is_stopped(&self) -> bool {
        *self == Self::Stopped
    }
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
    pub playback: PlaybackState,
    pub shuffled: bool,
    pub sorting: Repeat,
}

pub struct SpotifyManager {
    pub auth: AuthManager,
    pub settings: Settings,
    pub session: Session,
    pub api: CacheApi,
    pub mixer: Arc<dyn Mixer>,
    pub player: Arc<Player>,
    pub player_state: Mutex<PlayerState>,
    pub spirc: Spirc,
    channel: Sender<PlayerMessage>,
}

impl SpotifyManager {
    pub async fn new(channel: Sender<PlayerMessage>, settings: Settings) -> Result<Arc<Self>> {
        let cacher = Cacher::new().await?;

        let mut auth = AuthManager::new();
        let token = auth.authenticate().await?;
        let credentials = Credentials::with_access_token(&token.access_token);
        let session = Session::new(settings.session.clone(), None);
        // session.connect(credentials.clone(), true).await?;

        let mixer = mixer::find(None).unwrap();
        let mixer = mixer(mixer::MixerConfig::default())?;

        let player = Player::new(
            settings.player.clone(),
            session.clone(),
            mixer.get_soft_volume(),
            || {
                let backend = audio_backend::find(None).expect("No audio backend found");
                backend(None, AudioFormat::default())
            },
        );
        let tx = player.get_player_event_channel();

        let (spirc, task) = Spirc::new(
            librespot::connect::ConnectConfig {
                name: "daemon".to_string(),
                device_type: DeviceType::Computer,
                ..Default::default()
            },
            session.clone(),
            credentials.clone(),
            player.clone(),
            mixer.clone(),
        )
        .await
        .unwrap();
        tokio::spawn(task);
        let this = Arc::new(Self {
            mixer,
            spirc,
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
                    Stopped | Unavailable | TrackEnded => player.playback = PlaybackState::Stopped,
                    Started | Loading { .. } | TrackChanged => {
                        player.playback = PlaybackState::Playing { paused: false }
                    }

                    Paused => player.playback = PlaybackState::Playing { paused: true },
                }
            }
            tx.send(PlayerMessage::PlaybackMessage(msg)).await.unwrap();
        }
    }
}
