use axum::extract::Path;
use axum::response::IntoResponse;
use axum::{Json, extract::State};
use rspotify::model::{FromUri, IntoStatic, PlaylistId};

use crate::App;
use crate::error::RouteResult;

#[axum::debug_handler]
pub async fn list(State(app): State<App>, pid: Option<Path<String>>) -> RouteResult {
    let api = &app.spotify.api;
    match pid {
        None => {
            let playlists = api.get_playlists().await?;
            Ok(Json(playlists).into_response())
        }
        Some(Path(pid)) => {
            let pid = PlaylistId::from_uri(&*pid)?.into_static();
            let playlist = api.get_playlist_songs(pid).await?;
            Ok(Json(playlist).into_response())
        }
    }
}
