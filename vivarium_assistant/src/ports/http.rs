use crate::{config, errors::Result};
use axum::{routing::get, Router};
use std::sync::Arc;

pub struct Server {}

impl Default for Server {
    fn default() -> Self {
        Self::new()
    }
}

impl Server {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn run(&self, config: &config::Config) -> Result<()> {
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

        let listener = tokio::net::TcpListener::bind(config.address())
            .await
            .unwrap();
        axum::serve(listener, app).await?;
        Ok(())
    }
}

pub struct State {}

impl Default for State {
    fn default() -> Self {
        Self::new()
    }
}

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
