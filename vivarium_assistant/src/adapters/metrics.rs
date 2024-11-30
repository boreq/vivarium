use crate::domain::outputs::{OutputName, OutputState, OutputStatus};

pub struct Metrics {
    registry: prometheus::Registry,
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
