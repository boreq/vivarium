use crate::{
    domain::outputs::{OutputName, OutputState},
    errors::Result,
};
use prometheus::{labels, GaugeVec, Opts, Registry};

#[derive(Clone)]
pub struct Metrics {
    registry: prometheus::Registry,
    output_gauge: GaugeVec,
}

impl Metrics {
    pub fn new() -> Result<Self> {
        let output_gauge = GaugeVec::new(Opts::new("outputs", "state of the outputs"), &["name"])?;

        let registry = prometheus::Registry::new();
        registry.register(Box::new(output_gauge.clone()))?;

        Ok(Self {
            registry,
            output_gauge,
        })
    }

    pub fn report_output(&mut self, output: &OutputName, state: &OutputState) {
        self.output_gauge
            .with(&labels! {
                "name" => output.name(),
            })
            .set(match state {
                OutputState::On => 1.0,
                OutputState::Off => 0.0,
            });
    }

    pub fn registry(&self) -> &Registry {
        &self.registry
    }
}

unsafe impl Send for Metrics {}

unsafe impl Sync for Metrics {}
