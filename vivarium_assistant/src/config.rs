use crate::{domain::outputs::Outputs, errors::Result};
#[derive(Debug, Clone, PartialEq)]
pub struct Config {
    outputs: Outputs,
}

impl Config {
    pub fn new(outputs: Outputs) -> Result<Config> {
        Ok(Self { outputs })
    }

    pub fn outputs(&self) -> &Outputs {
        &self.outputs
    }
}
