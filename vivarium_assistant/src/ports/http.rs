use crate::errors::Result;
use axum::{routing::get, Router};
use std::sync::Arc;

pub struct Server {}

impl Server {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn run(&self) -> Result<()> {
        let state = Arc::new(State::new());

        let app = Router::new()
            .route(
                "/",
                get({
                    let state = state.clone();
                    move || handle_index(state)
                }),
            )
            .route(
                "/metrics",
                get({
                    let state = state.clone();
                    move || handle_metrics(state)
                }),
            );

        let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
        axum::serve(listener, app).await?;
        Ok(())
    }
}

pub struct State {}

impl State {
    pub fn new() -> Self {
        Self {}
    }
}

async fn handle_index(state: Arc<State>) -> &'static str {
    "Hello, World!"
}

async fn handle_metrics(state: Arc<State>) -> &'static str {
    "metrics"
}
