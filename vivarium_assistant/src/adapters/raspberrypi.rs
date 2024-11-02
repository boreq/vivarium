use crate::{
    domain::{self, PinNumber},
    errors::Result,
};
use rppal::gpio;

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

impl domain::GPIO<OutputPin> for GPIO {
    fn output(&self, number: &PinNumber) -> Result<OutputPin> {
        let pin = self.gpio.get(number.into())?;
        let output_pin = pin.into_output();
        Ok(OutputPin::new(output_pin))
    }
}

pub struct OutputPin {
    pin: gpio::OutputPin,
}

impl OutputPin {
    fn new(pin: gpio::OutputPin) -> OutputPin {
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
}
