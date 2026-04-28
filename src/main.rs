use std::sync::Arc;

use axum::{
    Router,
    routing::{get, post},
};
use futures::channel::mpsc::{Receiver, channel};
use serde::{Deserialize, Serialize};
use utoipa_axum::{router::OpenApiRouter, routes};
pub type Result<T = ()> = anyhow::Result<T>;

use crate::spotify::{Settings, SpotifyManager, SpotifyManagerArc};

pub(crate) mod control;
pub(crate) mod data;
pub(crate) mod error;
pub(crate) mod list;
pub(crate) mod queue;
pub(crate) mod spotify;
pub(crate) mod utils;

#[derive(Clone, Debug, Serialize, Deserialize)]
enum PlaybackMessage {
    Stopped,
    Started,
    Paused,
    TrackChanged,
    Loading,
    TrackEnded,
    Unavailable,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
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

    spotify.spirc.activate().unwrap();

    let app_state = AppState {
        // queue: Queue::new(spotify.clone()),
        spotify: spotify.clone(),
        pending_messages: rx,
    };
    println!("Username: {}", app_state.spotify.session.username());
    let app_state = Arc::new(app_state);
    let (oapp, _) = OpenApiRouter::new()
        .routes(routes!(queue::add_track))
        .routes(routes!(queue::play))
        .split_for_parts();
    let app = Router::new()
        .merge(oapp)
        .route("/control/toggle-pause", post(control::toggle_pause))
        .route("/control/volume", get(control::volume))
        .route("/control/volume", post(control::volume))
        .route("/list/playlists", get(list::list))
        .route("/list/playlists/{id}", get(list::list))
        .with_state(app_state);

    let ip = "0.0.0.0:8080";
    let listener = tokio::net::TcpListener::bind(ip).await.unwrap();

    println!("Listening on {ip}");
    let finish = async move {
        tokio::signal::ctrl_c().await.unwrap();
        println!("Shutting down!");
        spotify.spirc.disconnect(true).unwrap();
    };
    axum::serve(listener, app)
        .with_graceful_shutdown(finish)
        .await
        .unwrap();
}
