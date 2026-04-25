use axum::extract::{Path, Query};
use axum::response::IntoResponse;
use axum::{Json, extract::State};
use futures::{StreamExt, TryStreamExt};
use rspotify::model::PlaylistId;
use rspotify::prelude::{BaseClient, OAuthClient};

use crate::App;
use crate::error::RouteResult;

#[axum::debug_handler]
pub async fn list(State(app): State<App>, pid: Option<Path<String>>) -> RouteResult {
    let api = &app.spotify.api;
    match pid {
        None => {
            let playlists = api.current_user_playlists_manual(None, None).await?;
            Ok(Json(playlists).into_response())
        }
        Some(Path(pid)) => {
            let pid = PlaylistId::from_id(pid)?;
            let playlist = api
                .playlist_items(pid, None, Some(rspotify::model::Market::FromToken))
                .try_collect::<Vec<_>>()
                .await?;
            Ok(Json(playlist).into_response())
        }
    }
}
