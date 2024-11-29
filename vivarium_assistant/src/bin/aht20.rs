use rppal::i2c::I2c;
use vivarium_assistant::{adapters::raspberrypi::AHT20, errors::Result};

fn main() -> Result<()> {
    let i2c = I2c::new()?;

    let mut aht20 = AHT20::new(i2c)?;
    let result = aht20.measure()?;
    println!("result: {:?}", result);

    Ok(())
}
