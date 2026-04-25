use axum::{Json, extract::State, response::IntoResponse};
use librespot::core::SpotifyUri;
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
    let track_id: SpotifyUri = SpotifyUri::from_uri(&track)?;
    spot.player.load(track_id, true, 0);
    Ok(().into_response())
}
