use axum::Router;
use axum::extract::{Query, State};
use axum::response::{Html, IntoResponse, Response};
use axum::routing::get;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::Notify;
use tokio::sync::mpsc::Sender;

use crate::spotify::auth::{Authenticator, OAuthError};

type R<T> = Result<T, OAuthError>;

struct OneshotState {
    notify: Notify,
    sender: Sender<R<String>>,
    expected_state: String,
}

#[axum::debug_handler]
async fn code_handler(
    Query(pairs): Query<HashMap<String, String>>,
    State(os): State<Arc<OneshotState>>,
) -> Response {
    let res = async {
        let state = pairs
            .get("state")
            .cloned()
            .ok_or(OAuthError::CsrfTokenNotFound)?;
        if state != os.expected_state {
            return Err(OAuthError::InvalidState);
        }
        let code = pairs
            .get("code")
            .cloned()
            .ok_or(OAuthError::AuthCodeNotFound)?;
        R::Ok(code)
    }
    .await;
    let failed = res.is_err();
    os.sender.send(res).await.unwrap();
    if failed {
        return Html("something went wrong").into_response();
    }
    os.notify.notify_one();
    let html =
        "<html><body><h1>Login Successful!</h1><p>You can close this window.</p></body></html>";
    Html(html).into_response()
}

impl Authenticator {
    pub async fn wait_for_authcode(&self, expected_state: String) -> Result<String, OAuthError> {
        let addr: SocketAddr = super::REDIRECT_HOST.parse().unwrap();
        let listener = tokio::net::TcpListener::bind(addr)
            .await
            .map_err(|e| OAuthError::AuthCodeListenerBind { addr, e })?;

        let (tx, mut rx) = tokio::sync::mpsc::channel(1);
        let os = Arc::new(OneshotState {
            notify: Notify::new(),
            sender: tx,
            expected_state,
        });

        axum::serve::serve(
            listener,
            Router::new().route("/login", get(code_handler).with_state(os.clone())),
        )
        .with_graceful_shutdown(async move { os.notify.notified().await })
        .await
        .unwrap();

        rx.recv().await.ok_or(OAuthError::InvalidState)?
    }
}
