use std::{sync::Arc, time::Duration};

use axum::{
    Router,
    extract::{Path, State},
    routing::post,
};
use librespot::{
    core::{Session, SessionConfig, SpotifyId, SpotifyUri},
    discovery::Credentials,
    playback::{
        audio_backend,
        config::{AudioFormat, PlayerConfig, VolumeCtrl},
        mixer::{Mixer as _, MixerConfig, NoOpVolume, softmixer::SoftMixer},
        player::Player,
    },
};
pub type Result<T = ()> = anyhow::Result<T>;

use crate::spotify::{AuthManager, Settings, SpotifyManager};

mod spotify;

async fn control(State(app): State<App>, Path(action): Path<String>) {
    println!("Action: {action}");
}

type App = Arc<AppState>;
struct AppState {
    spotify: SpotifyManager,
}

#[tokio::main]
async fn main() {
    let track_id = SpotifyUri::from_uri("spotify:track:3iyagZIYPdmiXQQI3Ig36j").unwrap();
    let spotify = SpotifyManager::new(Settings::new()).await.unwrap();

    spotify.player.load(track_id, true, 0);
    spotify
        .player
        .set_sink_event_callback(Some(Box::new(|ev| println!("Sink status: {ev:?}"))));
    let mut ch = spotify.player.get_player_event_channel();
    while let Some(ev) = ch.recv().await {
        println!("Player event: {ev:?}");
        if matches!(
            ev,
            librespot::playback::player::PlayerEvent::Stopped { .. }
                | librespot::playback::player::PlayerEvent::Unavailable { .. }
        ) {
            break;
        }
    }
    // return;

    // let app_state = AppState {};
    // let app_state = Arc::new(app_state);
    // let app = Router::new()
    //     .route("/control/{action}", post(control))
    //     .with_state(app_state);

    // let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    // axum::serve(listener, app).await.unwrap();
}
