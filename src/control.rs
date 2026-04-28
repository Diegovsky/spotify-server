use axum::{Json, extract::State, response::IntoResponse};
use serde::{Deserialize, Serialize};

use crate::{App, bail, error::RouteResult, spotify::PlaybackState};

pub async fn toggle_pause(State(app): State<App>) -> RouteResult {
    app.spotify.spirc.play_pause()?;

    Ok(().into_response())
}

#[derive(Serialize, Deserialize)]
pub struct Volume {
    volume: f64,
}

fn to_norm(volume: u16) -> f64 {
    volume as f64 / u16::MAX as f64
}

fn from_norm(volume: f64) -> u16 {
    (volume * u16::MAX as f64) as u16
}

pub async fn volume(State(app): State<App>, volume_set: Option<Json<Volume>>) -> RouteResult {
    let volume = match volume_set {
        None => to_norm(app.spotify.mixer.volume()),
        Some(Json(Volume { volume })) => {
            let volume = volume / 100f64;
            app.spotify.spirc.set_volume(from_norm(volume))?;
            volume
        }
    };
    let volume = volume * 100f64;
    Ok(Json(Volume { volume }).into_response())
}
