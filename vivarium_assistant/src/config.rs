use crate::{
    domain::{outputs::OutputDefinitions, sensors::WaterLevelSensorDefinitions},
    errors::Result,
};
#[derive(Debug, Clone, PartialEq)]
pub struct Config {
    outputs: OutputDefinitions,
    water_level_sensors: WaterLevelSensorDefinitions,
    address: String,
}

impl Config {
    pub fn new(
        address: impl Into<String>,
        outputs: OutputDefinitions,
        water_level_sensors: WaterLevelSensorDefinitions,
    ) -> Result<Config> {
        Ok(Self {
            address: address.into(),
            outputs,
            water_level_sensors,
        })
    }

    pub fn outputs(&self) -> &OutputDefinitions {
        &self.outputs
    }

    pub fn water_level_sensors(&self) -> &WaterLevelSensorDefinitions {
        &self.water_level_sensors
    }

    pub fn address(&self) -> &str {
        &self.address
    }
}
