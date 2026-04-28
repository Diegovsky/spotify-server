use crate::{
    Result,
    data::{OptFrom as _, PlaylistId, Track, TrackId},
};
use std::any::Any;

use async_sqlite::{Client, ClientBuilder, rusqlite::OptionalExtension};
use futures::{StreamExt as _, TryStreamExt as _};
use rspotify::{
    AuthCodeSpotify,
    model::{Id, Market},
    prelude::{BaseClient as _, OAuthClient as _},
};
use serde::{Deserialize, Serialize, de::DeserializeOwned};

use crate::data::PlaylistInfo;

#[derive(Serialize, Deserialize)]
struct CachedEntry {
    value: serde_json::Value,
}

pub struct Cacher {
    conn: Client,
}

fn check() {
    fn check2<T: Send + Sync>() {}
    check2::<Cacher>();
}

fn make_key<T>(text: &str) -> String {
    format!("{}-{}", std::any::type_name::<T>(), text)
}

impl Cacher {
    pub async fn new() -> Result<Self> {
        let conn = ClientBuilder::new()
            .journal_mode(async_sqlite::JournalMode::Wal)
            .path("cache.sqlite3")
            .open()
            .await?;

        conn.conn(|c| {
            c.execute(
                "CREATE TABLE IF NOT EXISTS cache (
                    key TEXT PRIMARY KEY,
                    value BLOB
                )",
                (),
            )
        })
        .await?;
        Ok(Self { conn })
    }

    pub async fn put<T: Any + Serialize>(&self, key: &str, value: &T) {
        let key = make_key::<T>(key);
        let value = serde_json::to_string(&value).unwrap();
        self.conn
            .conn(move |c| {
                let mut c = c
                    .prepare("INSERT INTO cache(key, value) VALUES (?, ?)")
                    .unwrap();
                c.execute((key, value))?;
                Ok(())
            })
            .await
            .unwrap();
    }
    pub async fn get<T: Any + DeserializeOwned>(&self, key: &str) -> Option<T> {
        let key = make_key::<T>(key);
        let value = self
            .conn
            .conn(move |i| {
                i.query_row("SELECT value FROM cache WHERE key = ?", (key,), |row| {
                    row.get::<usize, String>(0)
                })
                .optional()
            })
            .await
            .unwrap()?;
        Some(serde_json::from_str(&*value).unwrap())
    }
}

pub struct CacheApi {
    api: AuthCodeSpotify,
    cacher: Cacher,
}

impl CacheApi {
    pub fn new(api: AuthCodeSpotify, cacher: Cacher) -> Self {
        Self { api, cacher }
    }

    pub async fn get_playlists(&self) -> Result<Vec<PlaylistInfo>> {
        match self.cacher.get::<Vec<PlaylistInfo>>("playlists").await {
            Some(e) => return Ok(e),
            None => {
                let playlists = self
                    .api
                    .current_user_playlists()
                    .map_ok(|i| PlaylistInfo::from(i))
                    .try_collect::<Vec<_>>()
                    .await?;
                self.cacher.put("playlists", &playlists).await;
                return Ok(playlists);
            }
        }
    }

    pub async fn get_playlist(&self, id: PlaylistId) -> Result<Option<PlaylistInfo>> {
        let playlists = self.get_playlists().await?;
        Ok(playlists.into_iter().find(|i| i.id == id))
    }

    pub async fn get_playlist_songs(&self, pid: PlaylistId) -> Result<Vec<Track>> {
        let key = pid.id();
        match self.cacher.get::<Vec<Track>>(&key).await {
            Some(e) => return Ok(e),
            None => {
                let playlists = self
                    .api
                    .playlist_items(pid.clone(), None, Some(Market::FromToken))
                    .filter_map(|i| async {
                        i.map(|i| i.item.and_then(Track::opt_from)).transpose()
                    })
                    .try_collect::<Vec<_>>()
                    .await?;
                self.cacher.put(key, &playlists).await;
                return Ok(playlists);
            }
        }
    }

    pub async fn add_to_queue(&self, id: TrackId) -> Result {
        self.api.add_item_to_queue(id, None).await?;
        Ok(())
    }
}
