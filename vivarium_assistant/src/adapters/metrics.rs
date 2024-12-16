use crate::{
    domain::{
        outputs::{OutputName, OutputState},
        sensors::{Humidity, SensorName, Temperature, WaterLevel},
    },
    errors::Result,
};
use chrono::Utc;
use prometheus::{labels, Gauge, GaugeVec, Opts, Registry};

#[derive(Clone)]
pub struct Metrics {
    registry: prometheus::Registry,
    output_gauge: GaugeVec,
    water_level_gauge: GaugeVec,
    temperature_gauge: GaugeVec,
    humidity_gauge: GaugeVec,
    startup_time_gauge: Gauge,
}

impl Metrics {
    pub fn new() -> Result<Self> {
        let registry = prometheus::Registry::new_custom(Some("vivarium".into()), None)?;

        let output_gauge = GaugeVec::new(Opts::new("outputs", "state of the outputs"), &["name"])?;
        registry.register(Box::new(output_gauge.clone()))?;

        let water_level_gauge = GaugeVec::new(
            Opts::new("water_levels", "water level reported by the sensors"),
            &["name"],
        )?;
        registry.register(Box::new(water_level_gauge.clone()))?;

        let temperature_gauge = GaugeVec::new(
            Opts::new("temperatures", "temperature reported by the sensors"),
            &["name"],
        )?;
        registry.register(Box::new(temperature_gauge.clone()))?;

        let humidity_gauge = GaugeVec::new(
            Opts::new("humidities", "humidity reported by the sensors"),
            &["name"],
        )?;
        registry.register(Box::new(humidity_gauge.clone()))?;

        let startup_time_gauge = Gauge::new("startup_time", "startup time of the program")?;
        registry.register(Box::new(startup_time_gauge.clone()))?;

        Ok(Self {
            registry,
            output_gauge,
            water_level_gauge,
            temperature_gauge,
            humidity_gauge,
            startup_time_gauge,
        })
    }

    pub fn set_startup_time(&mut self, startup_time: &chrono::DateTime<Utc>) {
        self.startup_time_gauge
            .set(startup_time.to_utc().timestamp() as f64);
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

    pub fn report_temperature(&mut self, sensor: &SensorName, temperature: &Temperature) {
        self.temperature_gauge
            .with(&labels! {
                "name" => sensor.name(),
            })
            .set(temperature.celcius().into());
    }

    pub fn report_humidity(&mut self, sensor: &SensorName, humidity: &Humidity) {
        self.humidity_gauge
            .with(&labels! {
                "name" => sensor.name(),
            })
            .set(humidity.percentage().into());
    }

    pub fn registry(&self) -> &Registry {
        &self.registry
    }
}
