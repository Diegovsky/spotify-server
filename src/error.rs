use axum::{
    Json,
    response::{IntoResponse, Response},
};
use http::StatusCode;
use serde::Serialize;

pub struct Error(anyhow::Error);

pub type RouteResult<T = Response> = std::result::Result<T, Error>;

impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl<T> From<T> for Error
where
    T: Into<anyhow::Error>,
{
    fn from(value: T) -> Self {
        Self(value.into())
    }
}

#[derive(Serialize)]
struct ErrorJson<'a> {
    message: &'a str,
    causes: Vec<String>,
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorJson {
                message: &self.0.to_string(),
                causes: self.0.chain().skip(1).map(ToString::to_string).collect(),
            }),
        )
            .into_response()
    }
}

#[macro_export]
macro_rules! bail {
    ($($tt:tt)*) => {
        return Err($crate::error::Error::from(anyhow::anyhow!($($tt)*)))
    };
}
pub use bail;
