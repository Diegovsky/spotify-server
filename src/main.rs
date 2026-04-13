use std::{sync::Arc, time::Duration};

use axum::{
    Router,
    extract::{Path, State},
    routing::post,
};

mod spotify;

async fn control(State(app): State<App>, Path(action): Path<String>) {
    println!("Action: {action}");
}

type App = Arc<AppState>;
struct AppState {}

#[tokio::main]
async fn main() {
    let app_state = AppState {};
    let app_state = Arc::new(app_state);
    let app = Router::new()
        .route("/control/{action}", post(control))
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
