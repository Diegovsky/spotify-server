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
use tokio::sync::{Mutex, mpsc::UnboundedReceiver};

use crate::{PlaybackMessage, PlayerMessage, Result};

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

pub type SpotifyManagerArc = Arc<SpotifyManager>;
pub struct PlayerState {
    pub playing: bool,
    pub paused: bool,
}

pub struct SpotifyManager {
    pub auth: AuthManager,
    pub settings: Settings,
    pub session: Session,
    pub api: AuthCodeSpotify,
    pub player: Arc<Player>,
    pub player_state: Mutex<PlayerState>,
    channel: Sender<PlayerMessage>,
}

impl SpotifyManager {
    pub async fn new(channel: Sender<PlayerMessage>, settings: Settings) -> Result<Arc<Self>> {
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
            player_state: Mutex::new(PlayerState {
                playing: false,
                paused: false,
            }),
            api: AuthCodeSpotify::from_token(token),
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
                    Stopped | Unavailable | TrackEnded => {
                        player.playing = false;
                        player.paused = false;
                    }
                    Started | Loading { .. } | TrackChanged => {
                        player.playing = true;
                        player.paused = false;
                    }
                    Paused => {
                        player.playing = true;
                        player.paused = true;
                    }
                }
            }
            tx.send(PlayerMessage::PlaybackMessage(msg)).await.unwrap();
        }
    }
}
