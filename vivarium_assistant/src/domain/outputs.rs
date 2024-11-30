use super::{InputPin, OutputPin, OutputPinState, PinNumber, GPIO};
use crate::errors::Result;
use anyhow::anyhow;
use chrono::NaiveTime;
use chrono::{TimeDelta, Timelike};

pub trait CurrentTimeProvider {
    fn now(&self) -> NaiveTime;
}

#[derive(Copy, Debug, Clone, PartialEq)]
pub struct ScheduledActivation {
    when: NaiveTime,
    for_seconds: u32,
}

impl ScheduledActivation {
    const SECONDS_IN_AN_IMAGINARY_DAY: u32 = 24 * 60 * 60;

    pub fn new(when: NaiveTime, for_seconds: u32) -> Result<Self> {
        if for_seconds == 0 {
            return Err(anyhow!("activating for 0 seconds is nonsense"));
        }

        if for_seconds > ScheduledActivation::SECONDS_IN_AN_IMAGINARY_DAY {
            return Err(anyhow!(
                format!(
                "since this type effectively represents durations on an imaginary clock face this time the day really is up to {} seconds long and it isn't just the programmer's delusion; the provided for_seconds exceeds that",
                ScheduledActivation::SECONDS_IN_AN_IMAGINARY_DAY
            )));
        }

        Ok(Self { when, for_seconds })
    }

    pub fn overlaps(&self, other: &Self) -> bool {
        if self.has_inside(&other.when) {
            return true;
        }
        if self.has_inside(&other.end()) {
            return true;
        }
        if other.has_inside(&self.when) {
            return true;
        }
        if other.has_inside(&self.end()) {
            return true;
        }

        false
    }

    pub fn has_inside(&self, time: &NaiveTime) -> bool {
        let start = self.when;
        let end = self.end();
        let jumps_over_midnight = end.hour() < start.hour();

        if jumps_over_midnight {
            time >= &start || time <= &end
        } else {
            time >= &start && time <= &end
        }
    }

    fn end(&self) -> NaiveTime {
        self.when + TimeDelta::seconds(self.for_seconds as i64)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledActivations {
    activations: Vec<ScheduledActivation>,
}

impl ScheduledActivations {
    pub fn new(activations: &[ScheduledActivation]) -> Result<Self> {
        if activations.is_empty() {
            return Err(anyhow!("activations can't be empty"));
        }

        let mut v = vec![];
        for (i, a) in activations.iter().enumerate() {
            for (j, b) in activations.iter().enumerate() {
                if i != j && a.overlaps(b) {
                    return Err(anyhow!("activations can't overlap"));
                }
            }
            v.push(*a);
        }

        Ok(ScheduledActivations { activations: v })
    }

    pub fn has_inside(&self, time: NaiveTime) -> bool {
        for activation in &self.activations {
            if activation.has_inside(&time) {
                return true;
            }
        }
        false
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OutputName {
    name: String,
}

impl OutputName {
    pub fn new(name: impl Into<String>) -> Result<Self> {
        let name = name.into();
        if name.is_empty() {
            return Err(anyhow!("output name can't be empty"));
        }
        Ok(Self { name })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OutputDefinition {
    name: OutputName,
    pin: PinNumber,
    activations: ScheduledActivations,
}

impl OutputDefinition {
    pub fn new(name: OutputName, pin: PinNumber, activations: ScheduledActivations) -> Self {
        Self {
            name,
            pin,
            activations,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OutputDefinitions {
    outputs: Vec<OutputDefinition>,
}

impl OutputDefinitions {
    pub fn new(outputs: &[OutputDefinition]) -> Result<Self> {
        let mut v = vec![];
        for (i, a) in outputs.iter().enumerate() {
            for (j, b) in outputs.iter().enumerate() {
                if i == j {
                    continue;
                }

                if a.name == b.name {
                    return Err(anyhow!("identical output names"));
                }

                if a.pin == b.pin {
                    return Err(anyhow!("duplicate pin numbers"));
                }
            }
            v.push(a.clone());
        }

        Ok(Self { outputs: v })
    }

    pub fn outputs(&self) -> &[OutputDefinition] {
        &self.outputs
    }
}

pub struct Controller<OP: OutputPin, C: CurrentTimeProvider> {
    outputs: Vec<OutputWithPin<OP>>,
    current_time_provider: C,
}

impl<OP: OutputPin, C: CurrentTimeProvider> Controller<OP, C> {
    pub fn new<IP: InputPin, B: GPIO<OP, IP>>(
        outputs: &OutputDefinitions,
        gpio: B,
        current_time_provider: C,
    ) -> Result<Controller<OP, C>> {
        let outputs_with_pin: Result<Vec<OutputWithPin<OP>>> = outputs
            .outputs()
            .iter()
            .map(|v| {
                Ok(OutputWithPin {
                    definition: v.clone(),
                    pin: gpio.output(&v.pin)?,
                })
            })
            .collect();

        Ok(Controller {
            outputs: outputs_with_pin?,
            current_time_provider,
        })
    }

    pub fn update_outputs(&mut self) {
        let now = self.current_time_provider.now();

        for output in &mut self.outputs {
            if output.definition.activations.has_inside(now) {
                output.pin.set_high();
            } else {
                output.pin.set_low();
            }
        }
    }

    pub fn fail_safe(&mut self) {
        for output in &mut self.outputs {
            output.pin.set_low();
        }
    }

    pub fn status(&mut self) -> Vec<OutputStatus> {
        let mut result = vec![];
        for output in &mut self.outputs {
            let status = OutputStatus {
                name: output.definition.name.clone(),
                state: output.pin.state().into(),
            };
            result.push(status);
        }
        result
    }
}

pub struct OutputStatus {
    pub name: OutputName,
    pub state: OutputState,
}

#[derive(Debug)]
pub enum OutputState {
    On,
    Off,
}

impl From<OutputPinState> for OutputState {
    fn from(value: OutputPinState) -> Self {
        match value {
            OutputPinState::On => Self::On,
            OutputPinState::Off => Self::Off,
        }
    }
}

struct OutputWithPin<A: OutputPin> {
    definition: OutputDefinition,
    pin: A,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(test)]
    mod scheduled_activation {
        use super::*;

        #[test]
        fn test_has_inside() -> Result<()> {
            struct TestCase<'a> {
                name: &'a str,
                activation: ScheduledActivation,
                time: NaiveTime,
                expected_has_inside: bool,
            }

            let test_cases = vec![
                TestCase {
                    name: "midnight_start",
                    activation: ScheduledActivation::new(new_time(23, 59, 55), 10)?,
                    time: new_time(23, 59, 55),
                    expected_has_inside: true,
                },
                TestCase {
                    name: "midnight_middle_before",
                    activation: ScheduledActivation::new(new_time(23, 59, 55), 10)?,
                    time: new_time(23, 59, 59),
                    expected_has_inside: true,
                },
                TestCase {
                    name: "midnight_middle_after",
                    activation: ScheduledActivation::new(new_time(23, 59, 55), 10)?,
                    time: new_time(00, 00, 00),
                    expected_has_inside: true,
                },
                TestCase {
                    name: "midnight_end",
                    activation: ScheduledActivation::new(new_time(23, 59, 55), 10)?,
                    time: new_time(00, 00, 5),
                    expected_has_inside: true,
                },
                TestCase {
                    name: "midnight_outside",
                    activation: ScheduledActivation::new(new_time(23, 59, 55), 10)?,
                    time: new_time(12, 00, 00),
                    expected_has_inside: false,
                },
                TestCase {
                    name: "normal_start",
                    activation: ScheduledActivation::new(new_time(12, 0, 0), 10)?,
                    time: new_time(12, 0, 0),
                    expected_has_inside: true,
                },
                TestCase {
                    name: "normal_middle",
                    activation: ScheduledActivation::new(new_time(12, 0, 0), 10)?,
                    time: new_time(12, 0, 5),
                    expected_has_inside: true,
                },
                TestCase {
                    name: "normal_end",
                    activation: ScheduledActivation::new(new_time(12, 0, 0), 10)?,
                    time: new_time(12, 0, 10),
                    expected_has_inside: true,
                },
                TestCase {
                    name: "normal_outside",
                    activation: ScheduledActivation::new(new_time(12, 0, 0), 10)?,
                    time: new_time(18, 0, 0),
                    expected_has_inside: false,
                },
            ];

            for test_case in &test_cases {
                print!("test case: {}", test_case.name);
                assert_eq!(
                    test_case.activation.has_inside(&test_case.time),
                    test_case.expected_has_inside
                );
            }

            Ok(())
        }

        #[test]
        fn test_overlaps() -> Result<()> {
            struct TestCase<'a> {
                name: &'a str,
                a: ScheduledActivation,
                b: ScheduledActivation,
                expected_overlaps: bool,
            }

            let test_cases = vec![
                TestCase {
                    name: "identical",
                    a: ScheduledActivation::new(new_time(14, 0, 0), 10)?,
                    b: ScheduledActivation::new(new_time(14, 0, 0), 10)?,
                    expected_overlaps: true,
                },
                TestCase {
                    name: "inside",
                    a: ScheduledActivation::new(new_time(14, 0, 0), 20)?,
                    b: ScheduledActivation::new(new_time(14, 0, 5), 10)?,
                    expected_overlaps: true,
                },
                TestCase {
                    name: "overlap",
                    a: ScheduledActivation::new(new_time(14, 0, 0), 10)?,
                    b: ScheduledActivation::new(new_time(14, 0, 5), 10)?,
                    expected_overlaps: true,
                },
                TestCase {
                    name: "outside",
                    a: ScheduledActivation::new(new_time(14, 0, 0), 10)?,
                    b: ScheduledActivation::new(new_time(18, 0, 0), 10)?,
                    expected_overlaps: false,
                },
                TestCase {
                    name: "midnight_inside",
                    a: ScheduledActivation::new(new_time(23, 59, 50), 20)?,
                    b: ScheduledActivation::new(new_time(23, 59, 55), 10)?,
                    expected_overlaps: true,
                },
                TestCase {
                    name: "midnight_inside_before",
                    a: ScheduledActivation::new(new_time(23, 59, 50), 20)?,
                    b: ScheduledActivation::new(new_time(23, 59, 55), 1)?,
                    expected_overlaps: true,
                },
                TestCase {
                    name: "midnight_inside_after",
                    a: ScheduledActivation::new(new_time(23, 59, 50), 20)?,
                    b: ScheduledActivation::new(new_time(00, 0, 5), 1)?,
                    expected_overlaps: true,
                },
                TestCase {
                    name: "midnight_outside",
                    a: ScheduledActivation::new(new_time(23, 59, 50), 20)?,
                    b: ScheduledActivation::new(new_time(18, 0, 0), 10)?,
                    expected_overlaps: false,
                },
            ];

            for test_case in &test_cases {
                print!("test case: {}", test_case.name);

                assert_eq!(
                    test_case.a.overlaps(&test_case.b),
                    test_case.expected_overlaps
                );

                assert_eq!(
                    test_case.b.overlaps(&test_case.a),
                    test_case.expected_overlaps
                );
            }

            Ok(())
        }
    }

    mod scheduled_activations {
        use super::*;
        use anyhow::Error;
        use core::panic;

        #[test]
        fn test_construct() -> Result<()> {
            struct TestCase<'a> {
                name: &'a str,
                activations: Vec<ScheduledActivation>,
                expected_error: Option<Error>,
            }

            let test_cases = vec![
                TestCase {
                    name: "empty",
                    activations: vec![],
                    expected_error: Some(anyhow!("activations can't be empty")),
                },
                TestCase {
                    name: "overlap",
                    activations: vec![
                        ScheduledActivation::new(new_time(12, 0, 0), 10)?,
                        ScheduledActivation::new(new_time(12, 0, 5), 10)?,
                    ],
                    expected_error: Some(anyhow!("activations can't overlap")),
                },
                TestCase {
                    name: "ok",
                    activations: vec![
                        ScheduledActivation::new(new_time(12, 0, 0), 10)?,
                        ScheduledActivation::new(new_time(18, 0, 0), 10)?,
                    ],
                    expected_error: None,
                },
            ];

            for test_case in &test_cases {
                print!("test case: {}", test_case.name);

                let result = ScheduledActivations::new(&test_case.activations);
                match &test_case.expected_error {
                    Some(expected_err) => match result {
                        Ok(_) => {
                            panic!("no error encountered even though an error was expected")
                        }
                        Err(err) => {
                            assert_eq!(err.to_string(), expected_err.to_string());
                        }
                    },
                    None => {
                        match result {
                            Ok(_) => {
                                // ok
                            }
                            Err(_) => {
                                panic!("error encountered even though no error was expected")
                            }
                        }
                    }
                }
            }

            Ok(())
        }
    }

    pub fn new_time(hour: u32, min: u32, sec: u32) -> NaiveTime {
        NaiveTime::from_hms_opt(hour, min, sec).expect("from_hms_opt")
    }
}
