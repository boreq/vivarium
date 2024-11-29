use std::{thread, time::Duration};

use crate::errors::Result;
use anyhow::anyhow;
use chrono::NaiveTime;

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
}

#[derive(Clone)]
pub struct WaterLevelSensor {
    name: SensorName,
    echo_pin: PinNumber,
    trig_pin: PinNumber,
    min_distance: Distance,
    max_distance: Distance,
}

impl WaterLevelSensor {
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
}

pub struct WaterLevelSensors {
    sensors: Vec<WaterLevelSensor>,
}

impl WaterLevelSensors {
    pub fn new(sensors: &[WaterLevelSensor]) -> Result<Self> {
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
}

pub struct HCSR04<A: OutputPin, B: InputPin> {
    trig: A,
    echo: B,
}

impl<A: OutputPin, B: InputPin> HCSR04<A, B> {
    pub fn new(trig: A, echo: B) -> Result<Self> {
        Ok(Self { trig, echo })
    }

    pub fn measure(&mut self) -> Result<Distance> {
        let r = self.measure_with_interrupt();
        self.echo.clear_interrupt()?;
        r
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

        println!("start: {:?} end: {:?}", start, end);

        let duration = end - start;
        let meters = (duration.as_micros() as f32 / 1000000.0) * 340.0 / 2.0;
        Ok(Distance::new(meters)?)
    }

    fn poll_rising_edge(&mut self) -> Result<Duration> {
        match self.echo.poll_interrupt(Some(self.timeout()))? {
            Some(event) => match event.trigger {
                super::Trigger::RisingEdge => {
                    return Ok(event.timestamp);
                }
                super::Trigger::FallingEdge => {
                    return Err(anyhow!(
                        "detected a falling edge when a rising edge was expected"
                    ));
                }
            },
            None => {
                return Err(anyhow!("no rising edge detected"));
            }
        }
    }

    fn poll_falling_edge(&mut self) -> Result<Duration> {
        match self.echo.poll_interrupt(Some(self.timeout()))? {
            Some(event) => match event.trigger {
                super::Trigger::RisingEdge => {
                    return Err(anyhow!(
                        "detected a rising edge when a falling edge was expected"
                    ));
                }
                super::Trigger::FallingEdge => {
                    return Ok(event.timestamp);
                }
            },
            None => {
                return Err(anyhow!("no falling edge detected"));
            }
        }
    }

    fn timeout(&self) -> Duration {
        Duration::new(0, 100 * 1000000)
    }
}
