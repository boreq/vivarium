use std::{fmt::Display, thread, time::Duration};

use crate::errors::Result;
use anyhow::anyhow;
use chrono::{TimeDelta, Utc};

use super::{InputPin, OutputPin, PinNumber, I2C};

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

        if percentage > 1.0 {
            return Err(anyhow!("percentage can't be above 100"));
        }

        Ok(Self { percentage })
    }

    pub fn percentage(&self) -> f32 {
        self.percentage
    }
}

impl Display for Humidity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:.0}%", self.percentage * 100.0)
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

impl Display for Temperature {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:.1}C", self.celcius)
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

#[derive(Debug, Clone, Copy, PartialEq)]
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

impl PartialOrd for WaterLevel {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for WaterLevel {}

impl Ord for WaterLevel {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // the constructor verifies that percentage is_finite so I understand this can't fail
        self.percentage.partial_cmp(&other.percentage()).unwrap()
    }
}

impl Display for WaterLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:.0}%", self.percentage * 100.0)
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

impl Display for SensorName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
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

pub struct MedianCache<T> {
    period: TimeDelta,
    values: Vec<ValueWithTime<T>>,
}

impl<T> MedianCache<T> {
    pub fn new(period: Duration) -> Result<Self> {
        Ok(Self {
            period: chrono::TimeDelta::from_std(period)?,
            values: vec![],
        })
    }
}

impl<T> MedianCache<T>
where
    T: Ord,
{
    pub fn put(&mut self, value: T) {
        self.values.push(ValueWithTime {
            value,
            time: chrono::Utc::now(),
        });
        self.values.sort_by(|a, b| a.value.cmp(&b.value));
    }

    pub fn get(&mut self) -> Option<&T> {
        let now = chrono::Utc::now();
        self.values.retain(|v| now - v.time < self.period);
        self.values.get(self.values.len() / 2).map(|v| &v.value)
    }
}

struct ValueWithTime<T> {
    value: T,
    time: chrono::DateTime<Utc>,
}

const ATH20_ADDRESS: u16 = 0x38;

// Partially based on the Adafruit's library. Unfortunately reading that code it's sometimes
// difficult to guess the author's intentions. In a couple of places it uses commands not present
// in the datasheet (as far as I can tell, it's possible I can't convert between dec and hex), it
// caches the data in a bit of a weird way for some reason etc. It is therefore possible that the
// mistakes from that implementation were carried over here or I made new mistakes since my
// resulting implementation seems to be a bit different. This is further complicated by the
// datasheet effectively making statements such as "wait for X time and then the measurement MAY be
// completed if not wait some more for an unknown amount of time lol".
pub struct AHT20<T>
where
    T: I2C,
{
    i2c: WrappedI2C<T>,
}

impl<T> AHT20<T>
where
    T: I2C,
{
    pub fn new(i2c: T) -> Result<Self> {
        Ok(Self {
            i2c: WrappedI2C::new(ATH20_ADDRESS, i2c),
        })
    }

    pub fn measure(&mut self) -> Result<AHT20Measurement> {
        match self.confirm_connected() {
            Ok(_) => {
                println!("connected");
            }
            Err(_) => {
                println!("not connected");
            }
        }

        thread::sleep(Duration::from_millis(40));

        if !self.get_status()?.is_calibrated {
            // this wasn't implemented as the sensor always seems to report that it's calibrated?
            return Err(anyhow!(
                "the sensor claims that it's not calibrated, whatever that means"
            ));
        }

        self.trigger_measurement()?;
        thread::sleep(Duration::from_millis(80));

        for _ in 0..100 {
            thread::sleep(Duration::from_millis(1));
            if !self.get_status()?.is_busy {
                return self.read_data();
            }
        }

        Err(anyhow!("the sensor keeps claiming that it's busy"))
    }

    // todo: this seems useless, just try reading the data?
    fn confirm_connected(&mut self) -> Result<()> {
        if self.i2c.write(&[]).is_ok() {
            return Ok(());
        }

        // wait and then retry if we fail
        // the arduino library uses those timings
        thread::sleep(Duration::from_millis(20));
        self.i2c.write(&[])?;
        Ok(())
    }

    fn get_status(&mut self) -> Result<AHT20Status> {
        let mut buf: [u8; 1] = [0];
        self.i2c.write_read(&[0x71], &mut buf)?;
        AHT20Status::new(&buf)
    }

    fn trigger_measurement(&mut self) -> Result<()> {
        self.i2c.block_write(0xAC, &[0x33, 0x00])?;
        Ok(())
    }

    fn read_data(&mut self) -> Result<AHT20Measurement> {
        let mut buf: [u8; 6] = [0; 6];
        self.i2c.read(&mut buf)?;

        let mut humidity: u32 = 0;
        humidity |= (buf[1] as u32) << (8 + 4);
        humidity |= (buf[2] as u32) << 4;
        humidity |= ((buf[3] & 0b11110000) as u32) >> 4;

        let mut temperature: u32 = 0;
        temperature |= ((buf[3] & 0b00001111) as u32) << (8 + 8);
        temperature |= (buf[4] as u32) << 8;
        temperature |= buf[5] as u32;

        let temperature = (temperature as f32 / 1048576.0) * 200.0 - 50.0;
        let humidity = humidity as f32 / 1048576.0;

        let temperature = Temperature::new(temperature)?;
        let humidity = Humidity::new(humidity)?;

        Ok(AHT20Measurement {
            temperature,
            humidity,
        })
    }
}

struct WrappedI2C<T>
where
    T: I2C,
{
    slave_address: u16,
    i2c: T,
}

impl<T> WrappedI2C<T>
where
    T: I2C,
{
    fn new(slave_address: u16, i2c: T) -> Self {
        Self { slave_address, i2c }
    }

    pub fn write_read(&mut self, write_buffer: &[u8], read_buffer: &mut [u8]) -> Result<()> {
        self.i2c.set_slave_address(self.slave_address)?;
        Ok(self.i2c.write_read(write_buffer, read_buffer)?)
    }

    pub fn block_write(&mut self, command: u8, buffer: &[u8]) -> Result<()> {
        self.i2c.set_slave_address(self.slave_address)?;
        Ok(self.i2c.block_write(command, buffer)?)
    }

    pub fn read(&mut self, buffer: &mut [u8]) -> Result<usize> {
        self.i2c.set_slave_address(self.slave_address)?;
        Ok(self.i2c.read(buffer)?)
    }

    pub fn write(&mut self, buffer: &[u8]) -> Result<usize> {
        self.i2c.set_slave_address(self.slave_address)?;
        Ok(self.i2c.write(buffer)?)
    }
}

#[derive(Debug)]
struct AHT20Status {
    is_calibrated: bool,
    is_busy: bool,
}

impl AHT20Status {
    pub fn new(response: &[u8; 1]) -> Result<Self> {
        Ok(Self {
            is_calibrated: response[0] & (1 << 3) != 0,
            is_busy: response[0] & (1 << 7) != 0,
        })
    }
}

#[derive(Debug)]
pub struct AHT20Measurement {
    temperature: Temperature,
    humidity: Humidity,
}

impl AHT20Measurement {
    pub fn temperature(&self) -> Temperature {
        self.temperature
    }

    pub fn humidity(&self) -> Humidity {
        self.humidity
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(test)]
    mod median_cache {
        use super::*;

        #[test]
        fn get_median_value() -> Result<()> {
            struct TestCase<'a> {
                name: &'a str,
                values: Vec<i32>,
                result: i32,
            }

            let test_cases = vec![
                TestCase {
                    name: "asc",
                    values: vec![1, 2, 3],
                    result: 2,
                },
                TestCase {
                    name: "desc",
                    values: vec![3, 2, 1],
                    result: 2,
                },
                TestCase {
                    name: "not_sorted",
                    values: vec![1, 3, 2],
                    result: 2,
                },
            ];

            for test_case in &test_cases {
                print!("test case: {}", test_case.name);

                let mut cache = MedianCache::new(Duration::from_secs(5))?;
                for value in &test_case.values {
                    cache.put(*value);
                }

                assert_eq!(Some(&test_case.result), cache.get());
            }

            Ok(())
        }
    }

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
