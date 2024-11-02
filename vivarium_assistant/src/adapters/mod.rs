use crate::{
    domain::{self, PinNumber},
    errors::Result,
};
use chrono::Local;
pub mod config;
pub mod raspberrypi;

pub struct CurrentTimeProvider {}

impl CurrentTimeProvider {
    pub fn new() -> Self {
        Self {}
    }
}

impl domain::CurrentTimeProvider for CurrentTimeProvider {
    fn now(&self) -> chrono::NaiveTime {
        let now = Local::now();
        now.time()
    }
}

impl Default for CurrentTimeProvider {
    fn default() -> Self {
        Self::new()
    }
}

pub struct MockGPIO {}

impl MockGPIO {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for MockGPIO {
    fn default() -> Self {
        Self::new()
    }
}

impl domain::GPIO<MockOutputPin> for MockGPIO {
    fn output(&self, number: &PinNumber) -> Result<MockOutputPin> {
        Ok(MockOutputPin::new(number.clone()))
    }
}

pub struct MockOutputPin {
    number: PinNumber,
}

impl MockOutputPin {
    fn new(number: PinNumber) -> MockOutputPin {
        Self { number }
    }
}

impl domain::OutputPin for MockOutputPin {
    fn set_low(&mut self) {
        println!("setting {} low", self.number.number())
    }

    fn set_high(&mut self) {
        println!("setting {} high", self.number.number())
    }
}
