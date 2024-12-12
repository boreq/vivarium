use std::thread;
use std::time::Duration;
use vivarium_assistant::domain::OutputPin;
use vivarium_assistant::{
    adapters::raspberrypi,
    domain::{PinNumber, GPIO},
    errors::Result,
};

fn main() -> Result<()> {
    let gpio = raspberrypi::GPIO::new()?;

    let mut pin_light = gpio.output(&PinNumber::new(25)?)?;
    let mut pin_misting = gpio.output(&PinNumber::new(24)?)?;
    let mut pin_beacon = gpio.output(&PinNumber::new(22)?)?;

    pin_light.set_high();

    thread::sleep(Duration::from_secs(5));

    pin_beacon.set_high();

    thread::sleep(Duration::from_secs(5));

    pin_misting.set_high();

    thread::sleep(Duration::from_secs(5));

    pin_beacon.set_low();
    pin_misting.set_low();

    Ok(())
}
