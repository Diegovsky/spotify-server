use axum::{Json, extract::State, response::IntoResponse};
use librespot::core::SpotifyUri;
use rspotify::model::{Id, TrackId};
use serde::{Deserialize, Serialize};

use crate::{App, error::RouteResult};

#[derive(Serialize, Deserialize)]
pub struct ActionPlay {
    track: String,
}

pub async fn play(
    State(app): State<App>,
    Json(ActionPlay { track }): Json<ActionPlay>,
) -> RouteResult {
    let spot = &app.spotify;
    let track_id: SpotifyUri = SpotifyUri::from_uri(&TrackId::from_id(&track)?.uri()).unwrap();
    spot.player.load(track_id, true, 0);
    Ok(().into_response())
}

pub async fn toggle_pause(State(app): State<App>) -> RouteResult {
    let spot = &app.spotify;
    let state = spot.player_state.lock().await;
    if state.paused {
        spot.player.play();
    } else {
        spot.player.pause();
    }
    Ok(().into_response())
}
