use vivarium_assistant::domain::sensors::DistanceSensor;
use vivarium_assistant::{
    adapters::raspberrypi,
    domain::{sensors::HCSR04, PinNumber, GPIO},
    errors::Result,
};

fn main() -> Result<()> {
    let gpio = raspberrypi::GPIO::new()?;
    let trig = gpio.output(&PinNumber::new(17)?)?;
    let echo = gpio.input(&PinNumber::new(18)?)?;

    let mut sensor = HCSR04::new(trig, echo)?;
    let distance = sensor.measure()?;

    println!("distance: {:?}", distance);

    Ok(())
}
