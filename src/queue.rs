use axum::{Json, extract::State, response::IntoResponse};
use librespot::connect::{LoadRequest, LoadRequestOptions};
use rspotify::model::Id;
use serde::{Deserialize, Serialize};

use utoipa::ToSchema;

use crate::{
    App,
    data::{PlaylistId, TrackId},
    error::RouteResult,
};
#[derive(Serialize, Deserialize, ToSchema)]
pub struct AddTrack {
    #[schema(value_type = String)]
    uri: TrackId,
}

#[utoipa::path(post, path = "/queue/add-track")]
pub async fn add_track(
    State(app): State<App>,
    Json(AddTrack { uri }): Json<AddTrack>,
) -> RouteResult {
    app.spotify.api.add_to_queue(uri).await?;
    Ok(().into_response())
}

#[derive(Serialize, Deserialize, ToSchema)]
pub enum Play {
    #[schema(value_type = String)]
    Track(TrackId),
    #[schema(value_type = String)]
    Playlist(PlaylistId),
}
#[utoipa::path(post, path = "/queue/play")]
pub async fn play(State(app): State<App>, Json(play): Json<Play>) -> RouteResult {
    let uri = match play {
        Play::Track(id) => id.uri(),
        Play::Playlist(id) => id.uri(),
    };
    app.spotify.spirc.load(LoadRequest::from_context_uri(
        uri,
        LoadRequestOptions {
            start_playing: true,
            ..Default::default()
        },
    ))?;
    Ok(().into_response())
}
