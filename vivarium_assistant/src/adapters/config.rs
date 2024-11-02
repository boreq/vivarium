use crate::{
    config::Config,
    domain::{Output, OutputName, Outputs, PinNumber, ScheduledActivation, ScheduledActivations},
    errors::Result,
};
use chrono::NaiveTime;
use serde::Deserialize;

pub fn load(config: &str) -> Result<Config> {
    let config: ConfigTransport = toml::from_str(config)?;

    let mut outputs_vec = vec![];
    for output in &config.outputs {
        let mut activations_vec = vec![];
        for activation in &output.activations {
            let when = NaiveTime::parse_from_str(&activation.when, "%H:%M")?;
            activations_vec.push(ScheduledActivation::new(when, activation.for_seconds)?);
        }

        outputs_vec.push(Output::new(
            OutputName::new(&output.name)?,
            PinNumber::new(output.pin)?,
            ScheduledActivations::new(&activations_vec)?,
        ));
    }

    Config::new(Outputs::new(&outputs_vec)?)
}

#[derive(Deserialize)]
struct ConfigTransport {
    outputs: Vec<OutputTransport>,
}

#[derive(Deserialize)]
struct OutputTransport {
    name: String,
    pin: u8,
    activations: Vec<ScheduledActivationTransport>,
}

#[derive(Deserialize)]
struct ScheduledActivationTransport {
    when: String,
    #[serde(rename = "for")]
    for_seconds: u32,
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
