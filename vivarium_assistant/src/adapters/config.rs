use crate::domain::outputs::{
    OutputDefinition, OutputDefinitions, OutputName, ScheduledActivation, ScheduledActivations,
};
use crate::domain::sensors::{Distance, SensorName, WaterLevelSensorDefinitions};
use crate::errors::Error;
use crate::{
    config::Config,
    domain::{sensors::WaterLevelSensorDefinition, PinNumber},
    errors::Result,
};
use chrono::NaiveTime;
use serde::Deserialize;

pub fn load(config: &str) -> Result<Config> {
    let config: SerializedConfig = toml::from_str(config)?;

    let mut output_definitions = vec![];
    for output in &config.outputs {
        output_definitions.push(OutputDefinition::try_from(output)?);
    }

    let mut water_level_sensors = vec![];
    for water_level_sensor in &config.water_level_sensors {
        water_level_sensors.push(WaterLevelSensorDefinition::try_from(water_level_sensor)?);
    }

    Config::new(
        config.address,
        OutputDefinitions::new(&output_definitions)?,
        WaterLevelSensorDefinitions::new(&water_level_sensors)?,
    )
}

#[derive(Deserialize)]
struct SerializedConfig {
    address: String,
    outputs: Vec<SerializedOutput>,
    water_level_sensors: Vec<SerializedWaterLevelSensor>,
}

#[derive(Deserialize)]
struct SerializedOutput {
    name: String,
    pin: u8,
    #[serde(default)]
    activations: Vec<SerializedScheduledActivation>,
}

impl TryFrom<&SerializedOutput> for OutputDefinition {
    type Error = Error;

    fn try_from(value: &SerializedOutput) -> std::result::Result<Self, Self::Error> {
        let mut activations_vec = vec![];
        for activation in &value.activations {
            let when = NaiveTime::parse_from_str(&activation.when, "%H:%M")?;
            activations_vec.push(ScheduledActivation::new(when, activation.for_seconds)?);
        }

        Ok(Self::new(
            OutputName::new(&value.name)?,
            PinNumber::new(value.pin)?,
            ScheduledActivations::new(&activations_vec)?,
        ))
    }
}

#[derive(Deserialize)]
struct SerializedScheduledActivation {
    when: String,
    #[serde(rename = "for")]
    for_seconds: u32,
}

#[derive(Deserialize)]
struct SerializedWaterLevelSensor {
    name: String,
    echo_pin: u8,
    trig_pin: u8,
    max_distance: f32,
    min_distance: f32,
}

impl TryFrom<&SerializedWaterLevelSensor> for WaterLevelSensorDefinition {
    type Error = Error;

    fn try_from(value: &SerializedWaterLevelSensor) -> std::result::Result<Self, Self::Error> {
        Self::new(
            SensorName::new(&value.name)?,
            PinNumber::new(value.echo_pin)?,
            PinNumber::new(value.trig_pin)?,
            Distance::new(value.min_distance)?,
            Distance::new(value.max_distance)?,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixtures;
    use std::fs;

    #[test]
    fn test_load() -> Result<()> {
        let test_file_path = fixtures::test_file_path("./example_config.toml");
        println!("{:?}", test_file_path);
        let config_string = fs::read_to_string(test_file_path)?;
        let config = load(&config_string)?;

        println!("{:?}", config);

        let expected_config = Config::new(
            "localhost:8118",
            OutputDefinitions::new(
                vec![
                    OutputDefinition::new(
                        OutputName::new("Output 1")?,
                        PinNumber::new(27)?,
                        ScheduledActivations::new(
                            vec![ScheduledActivation::new(
                                NaiveTime::from_hms_opt(17, 30, 00).unwrap(),
                                600,
                            )?]
                            .as_ref(),
                        )?,
                    ),
                    OutputDefinition::new(
                        OutputName::new("Output 2")?,
                        PinNumber::new(28)?,
                        ScheduledActivations::new(
                            vec![
                                ScheduledActivation::new(
                                    NaiveTime::from_hms_opt(17, 30, 00).unwrap(),
                                    600,
                                )?,
                                ScheduledActivation::new(
                                    NaiveTime::from_hms_opt(18, 30, 00).unwrap(),
                                    600,
                                )?,
                            ]
                            .as_ref(),
                        )?,
                    ),
                    OutputDefinition::new(
                        OutputName::new("Output 3")?,
                        PinNumber::new(29)?,
                        ScheduledActivations::new(vec![].as_ref())?,
                    ),
                ]
                .as_ref(),
            )?,
            WaterLevelSensorDefinitions::new(
                vec![WaterLevelSensorDefinition::new(
                    SensorName::new("Water level sensor")?,
                    PinNumber::new(18)?,
                    PinNumber::new(17)?,
                    Distance::new(0.2)?,
                    Distance::new(0.05)?,
                )?]
                .as_ref(),
            )?,
        )?;

        assert_eq!(config, expected_config);

        Ok(())
    }
}
