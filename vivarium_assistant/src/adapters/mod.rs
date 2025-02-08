pub mod config;
pub mod metrics;
pub mod raspberrypi;

use crate::{
    domain::{self, outputs, PinNumber},
    errors::Result,
};
use anyhow::anyhow;
use chrono::Utc;
use log::debug;

#[derive(Clone)]
pub struct CurrentTimeProvider {}

impl CurrentTimeProvider {
    pub fn new() -> Self {
        Self {}
    }
}

impl outputs::CurrentTimeProvider for CurrentTimeProvider {
    fn now(&self) -> chrono::DateTime<Utc> {
        Utc::now()
    }
}

impl Default for CurrentTimeProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone)]
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
        Ok(MockOutputPin::new(*number))
    }

    fn input(&self, _number: &PinNumber) -> Result<MockInputPin> {
        Ok(MockInputPin::new())
    }
}

pub struct MockOutputPin {
    number: PinNumber,
    state: domain::OutputPinState,
}

impl MockOutputPin {
    pub fn new(number: PinNumber) -> Self {
        Self {
            state: domain::OutputPinState::High,
            number,
        }
    }
}

impl domain::OutputPin for MockOutputPin {
    fn set_low(&mut self) {
        debug!("setting pin {:?} low", self.number);
        self.state = domain::OutputPinState::Low;
    }

    fn set_high(&mut self) {
        debug!("setting pin {:?} high", self.number);
        self.state = domain::OutputPinState::High;
    }

    fn state(&self) -> domain::OutputPinState {
        self.state
    }
}

pub struct MockInputPin {}

impl MockInputPin {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for MockInputPin {
    fn default() -> Self {
        Self::new()
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

pub struct MockI2C {}

impl MockI2C {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for MockI2C {
    fn default() -> Self {
        Self::new()
    }
}

impl domain::I2C for MockI2C {
    fn set_slave_address(&mut self, _slave_address: u16) -> Result<()> {
        Err(anyhow!("not implemented"))
    }

    fn write_read(&mut self, _write_buffer: &[u8], _read_buffer: &mut [u8]) -> Result<()> {
        Err(anyhow!("not implemented"))
    }

    fn block_write(&mut self, _command: u8, _buffer: &[u8]) -> Result<()> {
        Err(anyhow!("not implemented"))
    }

    fn read(&mut self, _buffer: &mut [u8]) -> Result<usize> {
        Err(anyhow!("not implemented"))
    }

    fn write(&mut self, _buffer: &[u8]) -> Result<usize> {
        Err(anyhow!("not implemented"))
    }
}
