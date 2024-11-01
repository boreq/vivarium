use crate::errors::Result;
use anyhow::anyhow;
use chrono::{NaiveTime, TimeDelta, Timelike};

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
    pub fn new(activations: &[ScheduledActivation]) -> Result<ScheduledActivations> {
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
pub struct PinNumber {
    number: u8,
}

impl PinNumber {
    pub fn new(number: u8) -> Result<Self> {
        // todo validate pin numbers
        Ok(Self { number })
    }

    pub fn number(&self) -> u8 {
        self.number
    }
}

impl From<&PinNumber> for u8 {
    fn from(val: &PinNumber) -> Self {
        val.number
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Output {
    name: OutputName,
    pin: PinNumber,
    activations: ScheduledActivations,
}

impl Output {
    pub fn new(name: OutputName, pin: PinNumber, activations: ScheduledActivations) -> Self {
        Self {
            name,
            pin,
            activations,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Outputs {
    outputs: Vec<Output>,
}

impl Outputs {
    pub fn new(outputs: &[Output]) -> Result<Self> {
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

    pub fn outputs(&self) -> &[Output] {
        &self.outputs
    }
}

pub trait GPIO<A: OutputPin> {
    fn output(&self, number: &PinNumber) -> Result<A>;
}

pub trait OutputPin {
    fn set_low(&mut self);
    fn set_high(&mut self);
}

pub trait CurrentTimeProvider {
    fn now(&self) -> NaiveTime;
}

struct OutputWithPin<A: OutputPin> {
    definition: Output,
    pin: A,
}

pub struct Executor<A: OutputPin, C: CurrentTimeProvider> {
    outputs: Vec<OutputWithPin<A>>,
    current_time_provider: C,
}

impl<A: OutputPin, C: CurrentTimeProvider> Executor<A, C> {
    pub fn new<B: GPIO<A>>(
        outputs: &Outputs,
        gpio: B,
        current_time_provider: C,
    ) -> Result<Executor<A, C>> {
        let outputs_with_pin: Result<Vec<OutputWithPin<A>>> = outputs
            .outputs()
            .iter()
            .map(|v| {
                Ok(OutputWithPin {
                    definition: v.clone(),
                    pin: gpio.output(&v.pin)?,
                })
            })
            .collect();

        Ok(Executor {
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
}
pub struct OutputExecutor {}

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
                    time: new_time(00, 00, 05),
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
                    activation: ScheduledActivation::new(new_time(12, 00, 00), 10)?,
                    time: new_time(12, 00, 00),
                    expected_has_inside: true,
                },
                TestCase {
                    name: "normal_middle",
                    activation: ScheduledActivation::new(new_time(12, 00, 00), 10)?,
                    time: new_time(12, 00, 05),
                    expected_has_inside: true,
                },
                TestCase {
                    name: "normal_end",
                    activation: ScheduledActivation::new(new_time(12, 00, 00), 10)?,
                    time: new_time(12, 00, 10),
                    expected_has_inside: true,
                },
                TestCase {
                    name: "normal_outside",
                    activation: ScheduledActivation::new(new_time(12, 00, 00), 10)?,
                    time: new_time(18, 00, 00),
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
                    a: ScheduledActivation::new(new_time(14, 00, 00), 10)?,
                    b: ScheduledActivation::new(new_time(14, 00, 00), 10)?,
                    expected_overlaps: true,
                },
                TestCase {
                    name: "inside",
                    a: ScheduledActivation::new(new_time(14, 00, 00), 20)?,
                    b: ScheduledActivation::new(new_time(14, 00, 05), 10)?,
                    expected_overlaps: true,
                },
                TestCase {
                    name: "overlap",
                    a: ScheduledActivation::new(new_time(14, 00, 00), 10)?,
                    b: ScheduledActivation::new(new_time(14, 00, 05), 10)?,
                    expected_overlaps: true,
                },
                TestCase {
                    name: "outside",
                    a: ScheduledActivation::new(new_time(14, 00, 00), 10)?,
                    b: ScheduledActivation::new(new_time(18, 00, 00), 10)?,
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
                    b: ScheduledActivation::new(new_time(00, 00, 05), 1)?,
                    expected_overlaps: true,
                },
                TestCase {
                    name: "midnight_outside",
                    a: ScheduledActivation::new(new_time(23, 59, 50), 20)?,
                    b: ScheduledActivation::new(new_time(18, 00, 00), 10)?,
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
        use core::panic;

        use anyhow::Error;

        use super::*;

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
                        ScheduledActivation::new(new_time(12, 00, 00), 10)?,
                        ScheduledActivation::new(new_time(12, 00, 05), 10)?,
                    ],
                    expected_error: Some(anyhow!("activations can't overlap")),
                },
                TestCase {
                    name: "ok",
                    activations: vec![
                        ScheduledActivation::new(new_time(12, 00, 00), 10)?,
                        ScheduledActivation::new(new_time(18, 00, 00), 10)?,
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
