use crate::{
    adapters::metrics::{self},
    config,
    errors::Result,
};
use axum::extract::State;
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

        let listener = tokio::net::TcpListener::bind(config.address())
            .await
            .unwrap();
        axum::serve(listener, app).await?;
        Ok(())
    }
}

// no longer compiles if I use a custom error type, dunno I'm tired
async fn handle_metrics<M>(
    State(state): State<AppStateGeneric<M>>,
) -> std::result::Result<String, String>
where
    M: Metrics,
{
    match handle_metrics_custom(state) {
        Err(err) => Err(err.to_string()),
        Ok(ok) => Ok(ok),
    }
}

fn handle_metrics_custom<M>(state: AppStateGeneric<M>) -> Result<String>
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
