use anyhow::anyhow;
use axum::{extract::State, response::IntoResponse};

use crate::{App, bail, error::RouteResult, spotify::PlabackState};

pub async fn toggle_pause(State(app): State<App>) -> RouteResult {
    let spot = &app.spotify;
    let state = spot.player_state.lock().await;
    let PlabackState::Playing { paused } = state.playback else {
        bail!("Not playing anything");
    };

    if paused {
        app.spotify.player.play();
    } else {
        app.spotify.player.pause();
    }

    Ok(().into_response())
}
