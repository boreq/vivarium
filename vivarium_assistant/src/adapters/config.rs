use crate::domain::outputs::ScheduledActivation;
use crate::domain::sensors::{Distance, SensorName};
use crate::errors::Error;
use crate::{
    config::Config,
    domain::{
        outputs::{Output, OutputName, Outputs, ScheduledActivations},
        sensors::WaterLevelSensor,
        PinNumber,
    },
    errors::Result,
};
use chrono::NaiveTime;
use serde::Deserialize;

pub fn load(config: &str) -> Result<Config> {
    let config: ConfigTransport = toml::from_str(config)?;

    let mut outputs_vec = vec![];
    for output in &config.outputs {
        outputs_vec.push(Output::try_from(output)?);
    }

    let mut water_level_sensors = vec![];
    for water_level_sensor in &config.water_level_sensors {
        water_level_sensors.push(WaterLevelSensor::try_from(water_level_sensor)?);
    }

    Config::new(Outputs::new(&outputs_vec)?)
}

#[derive(Deserialize)]
struct ConfigTransport {
    outputs: Vec<OutputTransport>,
    water_level_sensors: Vec<WaterLevelSensorTransport>,
}

#[derive(Deserialize)]
struct OutputTransport {
    name: String,
    pin: u8,
    activations: Vec<ScheduledActivationTransport>,
}

impl TryFrom<&OutputTransport> for Output {
    type Error = Error;

    fn try_from(value: &OutputTransport) -> std::result::Result<Self, Self::Error> {
        let mut activations_vec = vec![];
        for activation in &value.activations {
            let when = NaiveTime::parse_from_str(&activation.when, "%H:%M")?;
            activations_vec.push(ScheduledActivation::new(when, activation.for_seconds)?);
        }

        Ok(Output::new(
            OutputName::new(&value.name)?,
            PinNumber::new(value.pin)?,
            ScheduledActivations::new(&activations_vec)?,
        ))
    }
}

#[derive(Deserialize)]
struct ScheduledActivationTransport {
    when: String,
    #[serde(rename = "for")]
    for_seconds: u32,
}

#[derive(Deserialize)]
struct WaterLevelSensorTransport {
    name: String,
    echo_pin: u8,
    trig_pin: u8,
    max_distance: f32,
    min_distance: f32,
}

impl TryFrom<&WaterLevelSensorTransport> for WaterLevelSensor {
    type Error = Error;

    fn try_from(value: &WaterLevelSensorTransport) -> std::result::Result<Self, Self::Error> {
        Ok(WaterLevelSensor::new(
            SensorName::new(&value.name)?,
            PinNumber::new(value.echo_pin)?,
            PinNumber::new(value.trig_pin)?,
            Distance::new(value.min_distance)?,
            Distance::new(value.max_distance)?,
        )?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{domain::outputs::ScheduledActivation, fixtures};
    use std::fs;

    #[test]
    fn test_load() -> Result<()> {
        let test_file_path = fixtures::test_file_path("./example_config.toml");
        println!("{:?}", test_file_path);
        let config_string = fs::read_to_string(test_file_path)?;
        let config = load(&config_string)?;

        println!("{:?}", config);

        let expected_config = Config::new(Outputs::new(
            vec![
                Output::new(
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
                Output::new(
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
            ]
            .as_ref(),
        )?)?;

        assert_eq!(config, expected_config);

        Ok(())
    }
}
