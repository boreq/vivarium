use crate::{
    domain::{outputs::OutputDefinitions, sensors::WaterLevelSensors},
    errors::Result,
};
#[derive(Debug, Clone, PartialEq)]
pub struct Config {
    outputs: OutputDefinitions,
    water_level_sensors: WaterLevelSensors,
}

impl Config {
    pub fn new(
        outputs: OutputDefinitions,
        water_level_sensors: WaterLevelSensors,
    ) -> Result<Config> {
        Ok(Self {
            outputs,
            water_level_sensors,
        })
    }

    pub fn outputs(&self) -> &OutputDefinitions {
        &self.outputs
    }

    pub fn water_level_sensors(&self) -> &WaterLevelSensors {
        &self.water_level_sensors
    }
}
