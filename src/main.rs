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
        config::{AudioFormat, VolumeCtrl},
        mixer::{Mixer as _, MixerConfig, NoOpVolume, softmixer::SoftMixer},
        player::Player,
    },
};

use crate::spotify::{
    AuthManager,
    auth::{Authenticator, CLIENT_ID},
};

mod spotify;

async fn control(State(app): State<App>, Path(action): Path<String>) {
    println!("Action: {action}");
}

type App = Arc<AppState>;
struct AppState {}

#[tokio::main]
async fn main() {
    let track_id = SpotifyUri::from_uri("spotify:track:3iyagZIYPdmiXQQI3Ig36j").unwrap();
    let mut auth = AuthManager::new();
    auth.authenticate().await.unwrap();
    let res = auth.refresh().await.unwrap();
    let config = SessionConfig {
        client_id: CLIENT_ID.to_owned(),
        ..Default::default()
    };
    let credentials = Credentials::with_access_token(res.access_token.clone());
    let ses = Session::new(config, None);
    ses.connect(credentials, true).await.unwrap();
    println!("Username: {}", ses.username());

    let plconf = librespot::playback::config::PlayerConfig {
        ..Default::default()
    };

    let mixer = Box::new(
        SoftMixer::open(MixerConfig {
            // This value feels reasonable to me. Feel free to change it
            volume_ctrl: VolumeCtrl::Log(VolumeCtrl::DEFAULT_DB_RANGE / 2.0),
            ..Default::default()
        })
        .expect("Failed to create soft mixer"),
    );
    let player = Player::new(plconf, ses, Box::new(NoOpVolume), || {
        let backend = audio_backend::find(Some("rodio".into())).expect("No audio backend found");
        backend(None, AudioFormat::default())
    });
    player.load(track_id, true, 0);
    player.set_sink_event_callback(Some(Box::new(|ev| println!("Sink status: {ev:?}"))));
    let mut ch = player.get_player_event_channel();
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
    println!("Quitting...");
    return;

    let app_state = AppState {};
    let app_state = Arc::new(app_state);
    let app = Router::new()
        .route("/control/{action}", post(control))
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
