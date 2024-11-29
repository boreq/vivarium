use crate::{
    domain::{self, outputs, PinNumber},
    errors::Result,
};
use anyhow::anyhow;
use chrono::Local;
pub mod config;
pub mod raspberrypi;

pub struct CurrentTimeProvider {}

impl CurrentTimeProvider {
    pub fn new() -> Self {
        Self {}
    }
}

impl outputs::CurrentTimeProvider for CurrentTimeProvider {
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

impl domain::GPIO<MockOutputPin, MockInputPin> for MockGPIO {
    fn output(&self, number: &PinNumber) -> Result<MockOutputPin> {
        Ok(MockOutputPin::new(number.clone()))
    }

    fn input(&self, number: &PinNumber) -> Result<MockInputPin> {
        Ok(MockInputPin::new(number.clone()))
    }
}

pub struct MockOutputPin {
    number: PinNumber,
}

impl MockOutputPin {
    fn new(number: PinNumber) -> Self {
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

pub struct MockInputPin {
    number: PinNumber,
}

impl MockInputPin {
    fn new(number: PinNumber) -> Self {
        Self { number }
    }
}

impl domain::InputPin for MockInputPin {
    fn set_interrupt(&mut self) -> Result<()> {
        Err(anyhow!("not implemented"))
    }

    fn clear_interrupt(&mut self) -> Result<()> {
        Err(anyhow!("not implemented"))
    }

    fn poll_interrupt(
        &mut self,
        _timeout: Option<std::time::Duration>,
    ) -> Result<Option<domain::Event>> {
        Err(anyhow!("not implemented"))
    }
}
