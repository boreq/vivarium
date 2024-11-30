use anyhow::anyhow;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::{env, fs, thread};
use vivarium_assistant::adapters::{self, config, metrics, raspberrypi};
use vivarium_assistant::config::Config;
use vivarium_assistant::domain::outputs::{
    CurrentTimeProvider, OutputName, OutputState, OutputStatus,
};
use vivarium_assistant::domain::{outputs, OutputPin};
use vivarium_assistant::errors::Result;
use vivarium_assistant::ports::http::Server;

const UPDATE_SENSORS_EVERY: Duration = Duration::from_secs(10);
const UPDATE_OUTPUTS_EVERY: Duration = Duration::from_secs(1);

#[tokio::main]
async fn main() -> Result<()> {
    #[cfg(feature = "not_raspberry_pi")]
    let gpio = adapters::MockGPIO::new();

    #[cfg(not(feature = "not_raspberry_pi"))]
    let gpio = raspberrypi::GPIO::new()?;

    let config = load_config()?;
    let current_time_provider = adapters::CurrentTimeProvider::new();
    let executor = Arc::new(Mutex::new(outputs::Controller::new(
        config.outputs(),
        gpio,
        current_time_provider,
    )?));

    let default_panic = std::panic::take_hook();
    let closure_executor = executor.clone();
    std::panic::set_hook(Box::new(move |info| {
        if let Ok(mut v) = closure_executor.lock() {
            v.fail_safe();
        }
        default_panic(info);
    }));

    let server = Server::new();
    let metrics = Arc::new(Mutex::new(metrics::Metrics::new()));

    tokio::spawn(async { update_water_sensor_loop().await });
    tokio::spawn(async move { server_loop(&server, &config).await });
    update_outputs_loop(executor, metrics).await;
    Ok(())
}

fn load_config() -> Result<Config> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        return Err(anyhow!("usage: program path_to_config_file.toml"));
    }

    let config_string = fs::read_to_string(args.get(1).unwrap())?;
    config::load(&config_string)
}

async fn server_loop(server: &Server, config: &Config) {
    loop {
        match server.run(config).await {
            Ok(_) => {
                print!("for some reason the server exited without returning any errors?")
            }
            Err(err) => {
                println!("the server exited with an error: {err}")
            }
        }
    }
}

async fn update_water_sensor_loop() {
    loop {
        println!("sensors");
        thread::sleep(UPDATE_SENSORS_EVERY);
    }
}

async fn update_outputs_loop(
    controller: Arc<Mutex<dyn Controller>>,
    metrics: Arc<Mutex<dyn Metrics>>,
) {
    loop {
        let mut controller = controller.lock().unwrap();
        controller.update_outputs();
        let status = controller.status();

        let mut metrics = metrics.lock().unwrap();
        for entry in status {
            metrics.report_output(&entry.name, &entry.state);
        }

        thread::sleep(UPDATE_OUTPUTS_EVERY);
    }
}

trait Controller {
    fn update_outputs(&mut self);
    fn status(&mut self) -> Vec<OutputStatus>;
}

impl<OP: OutputPin, CTP: CurrentTimeProvider> Controller for outputs::Controller<OP, CTP> {
    fn update_outputs(&mut self) {
        self.update_outputs();
    }

    fn status(&mut self) -> Vec<OutputStatus> {
        self.status()
    }
}

trait Metrics {
    fn report_output(&mut self, output: &OutputName, state: &OutputState);
}

impl Metrics for metrics::Metrics {
    fn report_output(&mut self, output: &OutputName, state: &OutputState) {
        self.report_output(output, state);
    }
}
