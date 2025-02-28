use crate::{
    adapters::{
        config::DURATION_PARSER,
        metrics::{self},
    },
    config,
    domain::outputs::{self},
    errors::{Error, Result},
};
use anyhow::anyhow;
use axum::{
    extract::{Json, Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use axum::{
    routing::{delete, get, post},
    Router,
};
use prometheus::{Registry, TextEncoder};
use serde::Deserialize;

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
            .route("/outputs/:name/overrides", post(handle_overrides_post))
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
    State(mut deps): State<Deps<M, C>>,
    Path(name): Path<String>,
) -> std::result::Result<(), AppError>
where
    C: Controller,
{
    let name = outputs::OutputName::new(name)?;
    Ok(deps.controller.clear_overrides(name)?)
}

async fn handle_overrides_post<M, C>(
    State(mut deps): State<Deps<M, C>>,
    Path(name): Path<String>,
    Json(payload): Json<SerializedOverride>,
) -> std::result::Result<(), AppError>
where
    C: Controller,
{
    let name = outputs::OutputName::new(name)?;
    let state = parse_state(&payload.state)?;
    let when = chrono::Local::now().naive_local().time();
    let for_seconds = DURATION_PARSER
        .parse(&payload.for_string)?
        .as_secs()
        .try_into()?;
    let activation = outputs::ScheduledActivation::new(when, for_seconds)?;
    Ok(deps.controller.add_override(name, state, activation)?)
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
    fn clear_overrides(&mut self, output_name: outputs::OutputName) -> Result<()>;
    fn add_override(
        &mut self,
        output_name: outputs::OutputName,
        state: outputs::OutputState,
        activation: outputs::ScheduledActivation,
    ) -> Result<()>;
}

struct AppError(Error);

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

#[derive(Deserialize)]
struct SerializedOverride {
    state: String,
    #[serde(rename = "for")]
    for_string: String,
}

fn parse_state(s: &str) -> Result<outputs::OutputState> {
    match s.to_uppercase().as_str() {
        "ON" => Ok(outputs::OutputState::On),
        "OFF" => Ok(outputs::OutputState::Off),
        _ => Err(anyhow!("invalid state")),
    }
}
