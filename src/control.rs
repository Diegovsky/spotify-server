use axum::{extract::State, response::IntoResponse};

use crate::{App, error::RouteResult};

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
