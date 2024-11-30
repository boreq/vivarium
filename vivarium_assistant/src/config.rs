use crate::{
    domain::{outputs::OutputDefinitions, sensors::WaterLevelSensorDefinitions},
    errors::Result,
};
#[derive(Debug, Clone, PartialEq)]
pub struct Config {
    outputs: OutputDefinitions,
    water_level_sensors: WaterLevelSensorDefinitions,
}

impl Config {
    pub fn new(
        outputs: OutputDefinitions,
        water_level_sensors: WaterLevelSensorDefinitions,
    ) -> Result<Config> {
        Ok(Self {
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
}
