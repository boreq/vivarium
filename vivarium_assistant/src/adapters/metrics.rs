use crate::{
    domain::{
        outputs::{OutputName, OutputState},
        sensors::{SensorName, WaterLevel},
    },
    errors::Result,
};
use prometheus::{labels, GaugeVec, Opts, Registry};

#[derive(Clone)]
pub struct Metrics {
    registry: prometheus::Registry,
    output_gauge: GaugeVec,
    water_level_gauge: GaugeVec,
}

impl Metrics {
    pub fn new() -> Result<Self> {
        let registry = prometheus::Registry::new();

        let output_gauge = GaugeVec::new(Opts::new("outputs", "state of the outputs"), &["name"])?;
        registry.register(Box::new(output_gauge.clone()))?;

        let water_level_gauge = GaugeVec::new(
            Opts::new("water_levels", "water level reported by the sensors"),
            &["name"],
        )?;
        registry.register(Box::new(water_level_gauge.clone()))?;

        Ok(Self {
            registry,
            output_gauge,
            water_level_gauge,
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

    pub fn report_water_level(&mut self, sensor: &SensorName, level: &WaterLevel) {
        self.water_level_gauge
            .with(&labels! {
                "name" => sensor.name(),
            })
            .set(level.percentage().into());
    }

    pub fn registry(&self) -> &Registry {
        &self.registry
    }
}

unsafe impl Send for Metrics {}

unsafe impl Sync for Metrics {}
