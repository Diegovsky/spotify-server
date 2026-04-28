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
        // let body = serde_json::to_vec(&ErrorJson {
        //     message: &self.0.to_string(),
        // })
        // .unwrap();
        // let mut resp = Body::from(body).into_response();
        // resp.headers_mut().insert(
        //     header::CONTENT_TYPE,
        //     HeaderValue::from_static("application/json"),
        // );
        // resp
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorJson {
                message: &self.0.to_string(),
                causes: self.0.chain().map(ToString::to_string).collect(),
            }),
        )
            .into_response()
    }
}
