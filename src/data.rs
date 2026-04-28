use rspotify::model::{self, FullTrack, PlayableItem, SimplifiedPlaylist};
use serde::{Deserialize, Serialize};

pub type TrackId = model::PlayableId<'static>;
pub type PlaylistId = model::PlaylistId<'static>;

pub trait OptFrom<T>: Sized {
    fn opt_from(value: T) -> Option<Self>;
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Track {
    pub id: Option<TrackId>,
    pub name: String,
    pub playable: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PlaylistInfo {
    pub id: PlaylistId,
    pub name: String,
}

impl From<SimplifiedPlaylist> for PlaylistInfo {
    fn from(value: SimplifiedPlaylist) -> Self {
        Self {
            id: value.id,
            name: value.name,
        }
    }
}

impl OptFrom<FullTrack> for Track {
    fn opt_from(value: FullTrack) -> Option<Self> {
        Some(Self {
            id: value.id.map(TrackId::Track),
            name: value.name,
            playable: value.is_playable.unwrap_or(false),
        })
    }
}

impl OptFrom<PlayableItem> for Track {
    fn opt_from(value: PlayableItem) -> Option<Self> {
        match value {
            PlayableItem::Track(full_track) => Self::opt_from(full_track),
            PlayableItem::Episode(ep) => Some(Self {
                id: Some(TrackId::Episode(ep.id)),
                name: ep.name,
                playable: ep.is_playable,
            }),
            PlayableItem::Unknown(_) => None,
        }
    }
}
