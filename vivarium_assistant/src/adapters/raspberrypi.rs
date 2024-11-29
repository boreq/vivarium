use std::{thread, time::Duration};

use crate::{
    domain::{
        self,
        sensors::{Humidity, Temperature},
        PinNumber,
    },
    errors::{Error, Result},
};
use anyhow::anyhow;
use rppal::{
    gpio::{self},
    i2c,
};

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

const ATH20_ADDRESS: u16 = 0x38;

// Partially based on the Adafruit's library. Unfortunately reading that code
// it's really difficult to guess the author's intentions. In a couple of places
// it uses incorrect commands not present in the datasheet (as far as I can
// tell, it's possible I can't convert between dec and hex), it caches the data
// in a bit of a weird way for some reason etc. It is therefore possible that
// the mistakes from that implementation were carried over here or I made a lot
// of new mistakes since my resulting implementation seems to be a bit
// different. This is further complicated by the datasheet effectively making
// statements such as "wait for X time and then the measurement MAY be completed
// if not wait some more for an unknown amount of time lol".
//
// TODO: automatically set slave address, right now this isn't done correctly in all places.
pub struct AHT20 {
    i2c: i2c::I2c,
}

impl AHT20 {
    pub fn new(i2c: i2c::I2c) -> Result<Self> {
        Ok(Self { i2c })
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
        if let Ok(_) = self.send(&[]) {
            return Ok(());
        }

        // wait and then retry if we fail
        // the arduino library uses those timings
        thread::sleep(Duration::from_millis(20));
        if let Err(err) = self.send(&[]) {
            return Err(err);
        }
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
        let humidity = (humidity as f32 / 1048576.0) * 100.0;

        let temperature = Temperature::new(temperature)?;
        let humidity = Humidity::new(humidity)?;

        Ok(AHT20Measurement {
            temperature,
            humidity,
        })
    }

    fn send(&mut self, buffer: &[u8]) -> Result<usize> {
        self.i2c.set_slave_address(ATH20_ADDRESS)?;
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
