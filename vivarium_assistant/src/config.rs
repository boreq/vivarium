use crate::{
    domain::{
        outputs::OutputDefinitions,
        sensors::{SensorName, WaterLevelSensorDefinitions},
    },
    errors::Result,
};
#[derive(Debug, Clone, PartialEq)]
pub struct Config {
    outputs: OutputDefinitions,
    water_level_sensors: WaterLevelSensorDefinitions,
    address: String,
    aht_20: Option<SensorName>,
}

impl Config {
    pub fn new(
        address: impl Into<String>,
        outputs: OutputDefinitions,
        water_level_sensors: WaterLevelSensorDefinitions,
        aht_20: Option<SensorName>,
    ) -> Result<Config> {
        Ok(Self {
            address: address.into(),
            outputs,
            water_level_sensors,
            aht_20,
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

    pub fn aht_20(&self) -> &Option<SensorName> {
        &self.aht_20
    }
}
