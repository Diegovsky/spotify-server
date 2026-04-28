use librespot::core::SpotifyUri;
use rspotify::model::{EpisodeId, Id, PlayableId, TrackId};

pub trait IdBridge<'a>: Sized + Id {
    fn sp_uri(&self) -> SpotifyUri {
        SpotifyUri::from_uri(&Id::uri(self)).unwrap()
    }
}

// macro_rules! impl_idbridge {
//     ($($name:tt)*) => {
//         impl<'a> IdBridge<'a> for $($name)* <'a> {
//             fn from_id(id: &'a str) -> crate::Result<Self> {
//                 let this = Self::from_id(id)?;
//                 Ok(this)
//             }
//         }
//     };
// }

macro_rules! impl_idbridge {
    ($($name:ident),*) => {
        $(impl<'a> IdBridge<'a> for $name<'a> {})*
    };
}

impl_idbridge!(TrackId, EpisodeId, PlayableId);

// pub fn from_id<'a, T: IdBridge<'a>>(text: &'a str) -> crate::Result<SpotifyUri> {
//     let it = T::from_id(text)?;
//     Ok(it.sp_uri())
// }
