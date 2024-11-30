use crate::domain::outputs::{OutputName, OutputState};

pub struct Metrics {
    registry: prometheus::Registry,
}

impl Default for Metrics {
    fn default() -> Self {
        Self::new()
    }
}

impl Metrics {
    pub fn new() -> Self {
        Self {
            registry: prometheus::Registry::new(),
        }
    }

    pub fn report_output(&mut self, output: &OutputName, state: &OutputState) {
        println!("{:?} {:?})", output, state);
    }
}
