use std::{thread, time::Duration};

use crate::errors::Result;
use anyhow::anyhow;

use super::{InputPin, OutputPin, PinNumber};

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Humidity {
    percentage: f32,
}

impl Humidity {
    pub fn new(percentage: f32) -> Result<Self> {
        if !percentage.is_finite() {
            return Err(anyhow!("WHY CAN'T YOU JUST BE NORMAL?!"));
        }

        if percentage < 0.0 {
            return Err(anyhow!("percentage can't be negative"));
        }

        if percentage > 100.0 {
            return Err(anyhow!("percentage can't be above 100"));
        }

        Ok(Self { percentage })
    }

    pub fn percentage(&self) -> f32 {
        self.percentage
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Temperature {
    celcius: f32,
}

impl Temperature {
    pub fn new(celcius: f32) -> Result<Self> {
        if !celcius.is_finite() {
            return Err(anyhow!("WHY CAN'T YOU JUST BE NORMAL?!"));
        }

        if celcius < 0.0 {
            return Err(anyhow!("impossible value: time to worry 🥶"));
        }

        if celcius > 100.0 {
            return Err(anyhow!("impossible value: time to worry 🥵"));
        }

        Ok(Self { celcius })
    }

    pub fn celcius(&self) -> f32 {
        self.celcius
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Distance {
    meters: f32,
}

impl Distance {
    pub fn new(meters: f32) -> Result<Self> {
        if !meters.is_finite() {
            return Err(anyhow!("WHY CAN'T YOU JUST BE NORMAL?!"));
        }

        if meters < 0.0 {
            return Err(anyhow!("distance can't be negative"));
        }

        if meters > 5.0 {
            return Err(anyhow!("impossible value: too large"));
        }

        Ok(Self { meters })
    }

    pub fn meters(&self) -> f32 {
        self.meters
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct WaterLevel {
    percentage: f32,
}

impl WaterLevel {
    pub fn new(percentage: f32) -> Result<Self> {
        if !percentage.is_finite() {
            return Err(anyhow!("WHY CAN'T YOU JUST BE NORMAL?!"));
        }

        if percentage < 0.0 {
            return Err(anyhow!("percentage can't be negative"));
        }

        Ok(Self { percentage })
    }

    pub fn percentage(&self) -> f32 {
        self.percentage
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SensorName {
    name: String,
}

impl SensorName {
    pub fn new(name: impl Into<String>) -> Result<Self> {
        let name = name.into();
        if name.is_empty() {
            return Err(anyhow!("sensor name can't be empty"));
        }
        Ok(Self { name })
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WaterLevelSensorDefinition {
    name: SensorName,
    echo_pin: PinNumber,
    trig_pin: PinNumber,
    min_distance: Distance,
    max_distance: Distance,
}

impl WaterLevelSensorDefinition {
    pub fn new(
        name: SensorName,
        echo_pin: PinNumber,
        trig_pin: PinNumber,
        min_distance: Distance,
        max_distance: Distance,
    ) -> Result<Self> {
        if echo_pin == trig_pin {
            return Err(anyhow!("pins must be different"));
        }

        if min_distance <= max_distance {
            return Err(anyhow!(
                "min water level distance must be larger than max water level distance"
            ));
        }

        Ok(Self {
            name,
            echo_pin,
            trig_pin,
            min_distance,
            max_distance,
        })
    }

    pub fn name(&self) -> &SensorName {
        &self.name
    }

    pub fn echo_pin(&self) -> PinNumber {
        self.echo_pin
    }

    pub fn trig_pin(&self) -> PinNumber {
        self.trig_pin
    }

    pub fn min_distance(&self) -> Distance {
        self.min_distance
    }

    pub fn max_distance(&self) -> Distance {
        self.max_distance
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WaterLevelSensorDefinitions {
    sensors: Vec<WaterLevelSensorDefinition>,
}

impl WaterLevelSensorDefinitions {
    pub fn new(sensors: &[WaterLevelSensorDefinition]) -> Result<Self> {
        let mut v = vec![];
        for (i, a) in sensors.iter().enumerate() {
            for (j, b) in sensors.iter().enumerate() {
                if i == j {
                    continue;
                }

                if a.name == b.name {
                    return Err(anyhow!("identical sensors names"));
                }

                if a.echo_pin == b.echo_pin
                    || a.echo_pin == b.trig_pin
                    || a.trig_pin == b.echo_pin
                    || a.trig_pin == b.trig_pin
                {
                    return Err(anyhow!("duplicate pin numbers"));
                }
            }
            v.push(a.clone());
        }

        Ok(Self { sensors: v })
    }

    pub fn sensors(&self) -> &[WaterLevelSensorDefinition] {
        &self.sensors
    }
}

pub trait DistanceSensor {
    fn measure(&mut self) -> Result<Distance>;
}

pub struct WaterLevelSensor<S: DistanceSensor> {
    min_distance: Distance,
    max_distance: Distance,
    distance_sensor: S,
}

impl<S: DistanceSensor> WaterLevelSensor<S> {
    pub fn new(min_distance: Distance, max_distance: Distance, distance_sensor: S) -> Result<Self> {
        if min_distance <= max_distance {
            return Err(anyhow!(
                "min water level distance must be larger than max water level distance"
            ));
        }

        Ok(Self {
            min_distance,
            max_distance,
            distance_sensor,
        })
    }

    pub fn measure(&mut self) -> Result<WaterLevel> {
        let distance = self.distance_sensor.measure()?;

        if distance > self.min_distance {
            return WaterLevel::new(0.0);
        }

        let distance_from_bottom = self.min_distance.meters() - distance.meters();
        let range = self.min_distance.meters() - self.max_distance.meters();
        WaterLevel::new(distance_from_bottom / range)
    }
}

pub struct HCSR04<A: OutputPin, B: InputPin> {
    trig: A,
    echo: B,
}

impl<A: OutputPin, B: InputPin> HCSR04<A, B> {
    pub fn new(trig: A, echo: B) -> Result<Self> {
        Ok(Self { trig, echo })
    }

    fn measure_with_interrupt(&mut self) -> Result<Distance> {
        self.echo.set_interrupt()?;

        self.trig.set_high();
        thread::sleep(Duration::new(0, 1000));
        self.trig.set_low();

        let start = self.poll_rising_edge()?;
        let end = self.poll_falling_edge()?;

        if start >= end {
            return Err(anyhow!("start must be smaller than end"));
        }

        let duration = end - start;
        let meters = (duration.as_micros() as f32 / 1000000.0) * 340.0 / 2.0;
        Distance::new(meters)
    }

    fn poll_rising_edge(&mut self) -> Result<Duration> {
        match self.echo.poll_interrupt(Some(self.timeout()))? {
            Some(event) => match event.trigger {
                super::Trigger::RisingEdge => Ok(event.timestamp),
                super::Trigger::FallingEdge => Err(anyhow!(
                    "detected a falling edge when a rising edge was expected"
                )),
            },
            None => Err(anyhow!("no rising edge detected")),
        }
    }

    fn poll_falling_edge(&mut self) -> Result<Duration> {
        match self.echo.poll_interrupt(Some(self.timeout()))? {
            Some(event) => match event.trigger {
                super::Trigger::RisingEdge => Err(anyhow!(
                    "detected a rising edge when a falling edge was expected"
                )),
                super::Trigger::FallingEdge => Ok(event.timestamp),
            },
            None => Err(anyhow!("no falling edge detected")),
        }
    }

    fn timeout(&self) -> Duration {
        Duration::new(0, 100 * 1000000)
    }
}

impl<A: OutputPin, B: InputPin> DistanceSensor for HCSR04<A, B> {
    fn measure(&mut self) -> Result<Distance> {
        let r = self.measure_with_interrupt();
        self.echo.clear_interrupt()?;
        r
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(test)]
    mod water_level_sensor {
        use super::*;

        struct MockDistanceSensor {
            distance: Distance,
        }

        impl MockDistanceSensor {
            fn new(distance: Distance) -> Self {
                Self { distance }
            }
        }

        impl DistanceSensor for MockDistanceSensor {
            fn measure(&mut self) -> Result<Distance> {
                Ok(self.distance)
            }
        }

        #[test]
        fn check_water_level() -> Result<()> {
            struct TestCase<'a> {
                name: &'a str,
                distance: Distance,
                water_level: WaterLevel,
            }

            let test_cases = vec![
                TestCase {
                    name: "min_level",
                    distance: Distance::new(0.2)?,
                    water_level: WaterLevel::new(0.0)?,
                },
                TestCase {
                    name: "max_level",
                    distance: Distance::new(0.1)?,
                    water_level: WaterLevel::new(1.0)?,
                },
                TestCase {
                    name: "middle",
                    distance: Distance::new(0.15)?,
                    water_level: WaterLevel::new(0.5)?,
                },
                TestCase {
                    name: "below_min_level",
                    distance: Distance::new(0.3)?,
                    water_level: WaterLevel::new(0.0)?,
                },
                TestCase {
                    name: "above_max_level",
                    distance: Distance::new(0.05)?,
                    water_level: WaterLevel::new(1.5)?,
                },
            ];

            for test_case in &test_cases {
                print!("test case: {}", test_case.name);

                let distance_sensor = MockDistanceSensor::new(test_case.distance);
                let mut sensor = WaterLevelSensor::new(
                    Distance::new(0.2)?,
                    Distance::new(0.1)?,
                    distance_sensor,
                )?;
                let water_level = sensor.measure()?;

                assert!(in_epsilon(
                    test_case.water_level.percentage,
                    water_level.percentage,
                    0.01
                ))
            }

            Ok(())
        }

        fn in_epsilon(a: f32, b: f32, epsilon: f32) -> bool {
            println!("a: {} b: {} epsilon: {}", a, b, epsilon);

            if a == b {
                return true;
            }

            let actual_epsilon = (a - b).abs() / a.abs();
            actual_epsilon < epsilon
        }
    }
}
