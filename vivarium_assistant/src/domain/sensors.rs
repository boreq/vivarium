use anyhow::{anyhow, Error};

use crate::errors::Error;

pub struct Pressure {
    pascals: f32,
}

impl Pressure {
    pub fn new(pascals: f32) -> Result<Self> {
        if !pascals.is_normal() {
            return Err(anyhow!("WHY CAN'T YOU JUST BE NORMAL?!"));
        }

        if pascals == 0.0 {
            return Err(anyhow!("fuck we have a serious problem because the atmosphere is gone"));
        }

        if pascals < 0 {
            return Err(anyhow!("fuck we have a serious problem because the atmophere is pulling on us"));
        }

        Ok(Self{pascals})
    }

    pub fn pascals(&self) -> f32 {
        self.pascals
    }
}

pub struct Humidity {
    percentage: f32,
}

impl Humidity {
    pub fn new(percentage: f32) -> Result<Self> {
        if !percentage.is_normal() {
            return Err(anyhow!("WHY CAN'T YOU JUST BE NORMAL?!"));
        }

        if percentage < 0 {
            return Err(anyhow!("humidity can't be negative"));
        }

        if percentage > 100.0 {
            return Err(anyhow!("humidity can't be above 100"));
        }

        Ok(Self{percentage})
    }

    pub fn percentage(&self) -> f32 {
        self.percentage
    }
}

pub struct Temperature {
    celcius: f32,
}

impl Temperature {
    pub fn new(celcius: f32) -> Result<Self> {
        if !celcius.is_normal() {
            return Err(anyhow!("WHY CAN'T YOU JUST BE NORMAL?!"));
        }

        if celcius < 0 {
            return Err(anyhow!("time to worry 🥶"));
        }

        if celcius > 100.0 {
            return Err(anyhow!("time to worry 🥵"));
        }

        Ok(Self{celcius})
    }

    pub fn celcius(&self) -> f32 {
        self.celcius
    }
}