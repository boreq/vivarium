pub mod outputs;
pub mod sensors;

use crate::errors::Result;
use std::time::Duration;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct PinNumber {
    number: u8,
}

impl PinNumber {
    pub fn new(number: u8) -> Result<Self> {
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

pub trait GPIO<A: OutputPin, B: InputPin> {
    fn output(&self, number: &PinNumber) -> Result<A>;
    fn input(&self, number: &PinNumber) -> Result<B>;
}

pub trait OutputPin {
    fn set_low(&mut self);
    fn set_high(&mut self);
    fn state(&self) -> OutputPinState;
}

pub trait InputPin {
    fn set_interrupt(&mut self) -> Result<()>;
    fn clear_interrupt(&mut self) -> Result<()>;
    fn poll_interrupt(&mut self, timeout: Option<Duration>) -> Result<Option<Event>>;
}

pub struct Event {
    pub timestamp: Duration, // time since system was booted
    pub trigger: Trigger,
}

pub enum Trigger {
    RisingEdge,
    FallingEdge,
}

#[derive(Clone, Copy, PartialEq)]
pub enum OutputPinState {
    Low,
    High,
}
