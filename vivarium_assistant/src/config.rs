use crate::{
    domain::{outputs::Outputs, sensors::WaterLevelSensors},
    errors::Result,
};
#[derive(Debug, Clone, PartialEq)]
pub struct Config {
    outputs: Outputs,
    water_level_sensors: WaterLevelSensors,
}

impl Config {
    pub fn new(outputs: Outputs, water_level_sensors: WaterLevelSensors) -> Result<Config> {
        Ok(Self {
            outputs,
            water_level_sensors,
        })
    }

    pub fn outputs(&self) -> &Outputs {
        &self.outputs
    }

    pub fn water_level_sensors(&self) -> &WaterLevelSensors {
        &self.water_level_sensors
    }
}
