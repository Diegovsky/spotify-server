use std::collections::VecDeque;

use axum::{Json, extract::State, response::IntoResponse};
use futures::lock::Mutex;
use serde::{Deserialize, Serialize};

use crate::{
    App,
    data::{PlaylistId, TrackId},
    error::RouteResult,
    utils::IdBridge,
};

#[derive(Default)]
pub struct Queue {
    current_playlist: Mutex<Option<PlaylistId>>,
    queue: Mutex<VecDeque<TrackId>>,
}

impl Queue {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn set_playlist(&self, id: PlaylistId, track: Option<TrackId>) {}

    pub async fn add_track(&self, id: TrackId) {
        let mut queue = self.queue.lock().await;
        queue.push_front(id);
    }
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Add {
    Track(TrackId),
    Playlist(PlaylistId),
}

pub async fn add(State(app): State<App>, Json(add): Json<Add>) -> RouteResult {
    let spot = &app.spotify;
    match add {
        Add::Track(track) => {
            spot.player.load(track.sp_uri(), true, 0);
        }
        Add::Playlist(playlist) => {
            todo!()
        }
    }
    Ok(().into_response())
}
