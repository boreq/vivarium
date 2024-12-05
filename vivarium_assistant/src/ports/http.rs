use crate::{
    adapters::metrics::{self},
    config,
    errors::{Error, Result},
};
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use axum::{routing::get, Router};
use prometheus::{Registry, TextEncoder};

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

    pub async fn run(&self, config: &config::Config, metrics: metrics::Metrics) -> Result<()> {
        let app_state = AppStateGeneric { metrics };

        let app = Router::new()
            .route("/metrics", get(handle_metrics::<metrics::Metrics>))
            .with_state(app_state);

        let listener = tokio::net::TcpListener::bind(config.address()).await?;
        axum::serve(listener, app).await?;
        Ok(())
    }
}

async fn handle_metrics<M>(
    State(state): State<AppStateGeneric<M>>,
) -> std::result::Result<String, AppError>
where
    M: Metrics,
{
    let registry = state.metrics.registry();
    let metrics = registry.gather();
    let encoder = TextEncoder::new();
    Ok(encoder.encode_to_string(&metrics)?)
}

#[derive(Clone)]
struct AppStateGeneric<M> {
    metrics: M,
}

pub trait Metrics {
    fn registry(&self) -> &Registry;
}

impl Metrics for metrics::Metrics {
    fn registry(&self) -> &Registry {
        self.registry()
    }
}

struct AppError(anyhow::Error);

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Something went wrong: {}", self.0),
        )
            .into_response()
    }
}

impl<E> From<E> for AppError
where
    E: Into<Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}
