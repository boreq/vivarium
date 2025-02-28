use std::time::Duration;

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
use anyhow::anyhow;
use chrono::NaiveTime;
use lazy_static::lazy_static;
use serde::Deserialize;

lazy_static! {
    pub static ref DURATION_PARSER: duration_parser::Parser = make_parser().unwrap();
}

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

    let aht_20 = match config.aht_20 {
        Some(name) => Some(SensorName::new(name)?),
        None => None,
    };

    Config::new(
        config.address,
        OutputDefinitions::new(&output_definitions)?,
        WaterLevelSensorDefinitions::new(&water_level_sensors)?,
        aht_20,
    )
}

#[derive(Deserialize)]
struct SerializedConfig {
    address: String,
    outputs: Vec<SerializedOutput>,
    water_level_sensors: Vec<SerializedWaterLevelSensor>,
    aht_20: Option<String>,
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
            let err = Err(anyhow!(
                "start_every and times should either be both set or both shouldn't be set"
            ));
            let when = NaiveTime::parse_from_str(&activation.when, "%H:%M:%S")?;
            let duration = DURATION_PARSER.parse(&activation.for_string)?;
            let new_activation = ScheduledActivation::new(when, duration.as_secs() as u32)?;

            match &activation.start_every {
                Some(start_every) => match &activation.times {
                    Some(times) => {
                        let start_every = DURATION_PARSER.parse(start_every)?;
                        activations_vec.append(
                            &mut new_activation.repeat(start_every.as_secs() as u32, *times)?,
                        );
                    }
                    None => {
                        return err;
                    }
                },
                None => match &activation.times {
                    Some(_times) => return err,
                    None => {
                        activations_vec.push(new_activation);
                    }
                },
            }
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
    for_string: String,

    start_every: Option<String>,
    times: Option<u32>,
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

fn make_parser() -> Result<duration_parser::Parser> {
    Ok(duration_parser::Parser::new(
        duration_parser::Config::new(duration_parser::Units::new(&[
            duration_parser::Unit::new(
                duration_parser::UnitMagnitude::new(Duration::from_secs(1))?,
                &[
                    duration_parser::UnitName::new("second".to_string())?,
                    duration_parser::UnitName::new("seconds".to_string())?,
                ],
            )?,
            duration_parser::Unit::new(
                duration_parser::UnitMagnitude::new(Duration::from_secs(60))?,
                &[
                    duration_parser::UnitName::new("minute".to_string())?,
                    duration_parser::UnitName::new("minutes".to_string())?,
                ],
            )?,
            duration_parser::Unit::new(
                duration_parser::UnitMagnitude::new(Duration::from_secs(60 * 60))?,
                &[
                    duration_parser::UnitName::new("hour".to_string())?,
                    duration_parser::UnitName::new("hours".to_string())?,
                ],
            )?,
        ])?)?
        .with_policy_for_spaces_between_value_and_unit(duration_parser::SpacePolicy::RequireOne)
        .with_policy_for_spaces_between_components(duration_parser::SpacePolicy::RequireOne),
    ))
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
                            vec![
                                ScheduledActivation::new(
                                    NaiveTime::from_hms_opt(17, 30, 00).unwrap(),
                                    600,
                                )?,
                                ScheduledActivation::new(
                                    NaiveTime::from_hms_opt(18, 00, 00).unwrap(),
                                    600,
                                )?,
                            ]
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
                                    30,
                                )?,
                                ScheduledActivation::new(
                                    NaiveTime::from_hms_opt(18, 30, 00).unwrap(),
                                    60 * 60 + 10 * 60 + 20,
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
            Some(SensorName::new("AHT20 sensor")?),
        )?;

        assert_eq!(config, expected_config);

        Ok(())
    }
}
