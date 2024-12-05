use crate::{
    adapters::metrics::{self},
    config,
    domain::outputs::OutputName,
    errors::{Error, Result},
};
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::delete,
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

    pub async fn run<M, C>(&self, config: &config::Config, deps: Deps<M, C>) -> Result<()>
    where
        M: Metrics + Sync + Send + Clone + 'static,
        C: Controller + Sync + Send + Clone + 'static,
    {
        let app = Router::new()
            .route("/metrics", get(handle_metrics))
            .route("/outputs/:name/overrides", delete(handle_overrides_delete))
            .with_state(deps);

        let listener = tokio::net::TcpListener::bind(config.address()).await?;
        axum::serve(listener, app).await?;
        Ok(())
    }
}

async fn handle_metrics<M, C>(
    State(deps): State<Deps<M, C>>,
) -> std::result::Result<String, AppError>
where
    M: Metrics,
{
    let registry = deps.metrics.registry();
    let metrics = registry.gather();
    let encoder = TextEncoder::new();
    Ok(encoder.encode_to_string(&metrics)?)
}

async fn handle_overrides_delete<M, C>(
    State(deps): State<Deps<M, C>>,
) -> std::result::Result<String, AppError>
where
    M: Metrics,
{
    let registry = deps.metrics.registry();
    let metrics = registry.gather();
    let encoder = TextEncoder::new();
    Ok(encoder.encode_to_string(&metrics)?)
}

#[derive(Clone)]
pub struct Deps<M, C> {
    metrics: M,
    controller: C,
}

impl<M, C> Deps<M, C> {
    pub fn new(metrics: M, controller: C) -> Self {
        Self {
            metrics,
            controller,
        }
    }
}

pub trait Metrics {
    fn registry(&self) -> &Registry;
}

impl Metrics for metrics::Metrics {
    fn registry(&self) -> &Registry {
        metrics::Metrics::registry(self)
    }
}

pub trait Controller {
    fn clear_overrides(&mut self, output_name: OutputName) -> Result<()>;
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
