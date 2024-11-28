use crate::errors::Result;
use anyhow::{anyhow, Error};

#[derive(Debug)]
pub struct Humidity {
    percentage: f32,
}

impl Humidity {
    pub fn new(percentage: f32) -> Result<Self> {
        if !percentage.is_normal() {
            return Err(anyhow!("WHY CAN'T YOU JUST BE NORMAL?!"));
        }

        if percentage < 0.0 {
            return Err(anyhow!("humidity can't be negative"));
        }

        if percentage > 100.0 {
            return Err(anyhow!("humidity can't be above 100"));
        }

        Ok(Self { percentage })
    }

    pub fn percentage(&self) -> f32 {
        self.percentage
    }
}

#[derive(Debug)]
pub struct Temperature {
    celcius: f32,
}

impl Temperature {
    pub fn new(celcius: f32) -> Result<Self> {
        if !celcius.is_normal() {
            return Err(anyhow!("WHY CAN'T YOU JUST BE NORMAL?!"));
        }

        if celcius < 0.0 {
            return Err(anyhow!("time to worry 🥶"));
        }

        if celcius > 100.0 {
            return Err(anyhow!("time to worry 🥵"));
        }

        Ok(Self { celcius })
    }

    pub fn celcius(&self) -> f32 {
        self.celcius
    }
}
