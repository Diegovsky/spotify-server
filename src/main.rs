use std::sync::Arc;

use axum::{
    Router,
    routing::{get, post},
};
use futures::channel::mpsc::{Receiver, channel};
pub type Result<T = ()> = anyhow::Result<T>;

use crate::spotify::{Settings, SpotifyManager, SpotifyManagerArc};

mod control;
mod error;
mod list;
mod spotify;

#[derive(Clone, Debug)]
enum PlaybackMessage {
    Stopped,
    Started,
    Paused,
    TrackChanged,
    Loading,
    TrackEnded,
    Unavailable,
}

#[derive(Clone, Debug)]
enum PlayerMessage {
    PlaybackMessage(PlaybackMessage),
}

type App = Arc<AppState>;

struct AppState {
    spotify: SpotifyManagerArc,
    pending_messages: Receiver<PlayerMessage>,
}

#[tokio::main]
async fn main() {
    let (tx, rx) = channel::<PlayerMessage>(12);
    let spotify = SpotifyManager::new(tx.clone(), Settings::new())
        .await
        .unwrap();

    let app_state = AppState {
        spotify,
        pending_messages: rx,
    };
    println!("Username: {}", app_state.spotify.session.username());
    let app_state = Arc::new(app_state);
    let app = Router::new()
        .route("/control/play", post(control::play))
        .route("/control/toggle-pause", post(control::toggle_pause))
        .route("/list/playlists/", get(list::list))
        .route("/list/playlists/{id}", get(list::list))
        .with_state(app_state);

    let ip = "0.0.0.0:8080";
    let listener = tokio::net::TcpListener::bind(ip).await.unwrap();

    println!("Listening on {ip}");
    axum::serve(listener, app).await.unwrap();
}
