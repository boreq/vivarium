use vivarium_assistant::{
    adapters::raspberrypi,
    domain::{sensors::HCSR04, PinNumber, GPIO},
    errors::Result,
};

fn main() -> Result<()> {
    let mut gpio = raspberrypi::GPIO::new()?;
    let trig = gpio.output(&PinNumber::new(22)?)?;
    let echo = gpio.input(&PinNumber::new(23)?)?;

    let mut sensor = HCSR04::new(trig, echo)?;
    let distance = sensor.measure()?;

    println!("distance: {:?}", distance);

    Ok(())
}
