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


pub struct PressureTemperatureSensor {
    i2c:

}

impl  PressureTemperatureSensor {
    pub fn new()  -> {

    }

    pub fn begin() {
        if (isConnected() == false)
        return false;

    //Wait 40 ms after power-on before reading temp or humidity. Datasheet pg 8
    delay(40);

    //Check if the calibrated bit is set. If not, init the sensor.
    if (isCalibrated() == false)
    {
        //Send 0xBE0800
        initialize();

        //Immediately trigger a measurement. Send 0xAC3300
        triggerMeasurement();

        delay(75); //Wait for measurement to complete

        uint8_t counter = 0;
        while (isBusy())
        {
            delay(1);
            if (counter++ > 100)
                return (false); //Give up after 100ms
        }

        //This calibration sequence is not completely proven. It's not clear how and when the cal bit clears
        //This seems to work but it's not easily testable
        if (isCalibrated() == false)
        {
            return (false);
        }
    }

    //Check that the cal bit has been set
    if (isCalibrated() == false)
        return false;

    //Mark all datums as fresh (not read before)
    sensorQueried.temperature = true;
    sensorQueried.humidity = true;

    return true;
    }


}
