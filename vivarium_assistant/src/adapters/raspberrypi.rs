#![cfg(feature = "raspberry_pi")]

use crate::{
    domain::{self, PinNumber},
    errors::{Error, Result},
};
use anyhow::anyhow;
use rppal::{
    gpio::{self},
    i2c::I2c,
};
use std::time::Duration;

#[derive(Clone)]
pub struct GPIO {
    gpio: gpio::Gpio,
}

impl GPIO {
    pub fn new() -> Result<Self> {
        Ok(Self {
            gpio: gpio::Gpio::new()?,
        })
    }
}

impl domain::GPIO<OutputPin, InputPin> for GPIO {
    fn output(&self, number: &PinNumber) -> Result<OutputPin> {
        let pin = self.gpio.get(number.into())?;
        let output_pin = pin.into_output();
        Ok(OutputPin::new(output_pin))
    }

    fn input(&self, number: &PinNumber) -> Result<InputPin> {
        let pin = self.gpio.get(number.into())?;
        let input_pin = pin.into_input();
        Ok(InputPin::new(input_pin))
    }
}

pub struct OutputPin {
    pin: gpio::OutputPin,
}

impl OutputPin {
    fn new(pin: gpio::OutputPin) -> Self {
        Self { pin }
    }
}

impl domain::OutputPin for OutputPin {
    fn set_low(&mut self) {
        self.pin.set_low();
    }

    fn set_high(&mut self) {
        self.pin.set_high();
    }

    fn state(&self) -> domain::OutputPinState {
        if self.pin.is_set_high() {
            domain::OutputPinState::High
        } else {
            domain::OutputPinState::Low
        }
    }
}

pub struct InputPin {
    pin: gpio::InputPin,
}

impl InputPin {
    fn new(pin: gpio::InputPin) -> Self {
        Self { pin }
    }
}

impl domain::InputPin for InputPin {
    fn set_interrupt(&mut self) -> Result<()> {
        self.pin.set_interrupt(gpio::Trigger::Both, None)?;
        Ok(())
    }

    fn clear_interrupt(&mut self) -> Result<()> {
        self.pin.clear_interrupt()?;
        Ok(())
    }

    fn poll_interrupt(&mut self, timeout: Option<Duration>) -> Result<Option<domain::Event>> {
        match self.pin.poll_interrupt(false, timeout)? {
            Some(event) => Ok(Some(domain::Event::try_from(event)?)),
            None => Ok(None),
        }
    }
}

impl TryFrom<gpio::Event> for domain::Event {
    type Error = Error;

    fn try_from(value: gpio::Event) -> std::result::Result<Self, Self::Error> {
        Ok(Self {
            timestamp: value.timestamp,
            trigger: domain::Trigger::try_from(value.trigger)?,
        })
    }
}

impl TryFrom<gpio::Trigger> for domain::Trigger {
    type Error = Error;

    fn try_from(value: gpio::Trigger) -> std::result::Result<Self, Self::Error> {
        match value {
            gpio::Trigger::Disabled => Err(anyhow!("invalid value: disabled")),
            gpio::Trigger::RisingEdge => Ok(domain::Trigger::RisingEdge),
            gpio::Trigger::FallingEdge => Ok(domain::Trigger::FallingEdge),
            gpio::Trigger::Both => Err(anyhow!("invalid value: both")),
        }
    }
}

pub struct I2C {
    i2c: I2c,
}

impl I2C {
    pub fn new() -> Result<Self> {
        Ok(I2C { i2c: I2c::new()? })
    }
}

impl domain::I2C for I2C {
    fn set_slave_address(&mut self, slave_address: u16) -> Result<()> {
        Ok(self.i2c.set_slave_address(slave_address)?)
    }

    fn write_read(&mut self, write_buffer: &[u8], read_buffer: &mut [u8]) -> Result<()> {
        Ok(self.i2c.write_read(write_buffer, read_buffer)?)
    }

    fn block_write(&mut self, command: u8, buffer: &[u8]) -> Result<()> {
        Ok(self.i2c.block_write(command, buffer)?)
    }

    fn read(&mut self, buffer: &mut [u8]) -> Result<usize> {
        Ok(self.i2c.read(buffer)?)
    }

    fn write(&mut self, buffer: &[u8]) -> Result<usize> {
        Ok(self.i2c.write(buffer)?)
    }
}
