use std::collections::VecDeque;

use axum::{Json, extract::State, response::IntoResponse};
use futures::lock::Mutex;
use serde::{Deserialize, Serialize};

use crate::{
    App, Result,
    data::{PlaylistId, TrackId},
    error::RouteResult,
    spotify::{CacheApi, SpotifyManagerArc},
    utils::IdBridge,
};

const CHUNK_SIZE: usize = 8;

pub struct Queue {
    spotify: SpotifyManagerArc,
    current_playlist: Mutex<Option<PlaylistId>>,
    queue: Mutex<VecDeque<TrackId>>,
}

impl Queue {
    pub fn new(spotify: SpotifyManagerArc) -> Self {
        Self {
            spotify,
            current_playlist: Default::default(),
            queue: Default::default(),
        }
    }

    pub async fn set_playlist(&self, id: PlaylistId, track: Option<TrackId>) -> Result {
        let tracks = self.spotify.api.get_playlist_songs(id).await?;
        let tracks = tracks
            .into_iter()
            .filter(|i| i.playable)
            .filter_map(|i| i.id)
            .collect::<VecDeque<_>>();

        *self.queue.lock().await = tracks;
        Ok(())
    }

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
