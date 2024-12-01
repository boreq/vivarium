use anyhow::anyhow;
use tokio::time;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::{env, fs, thread};
use vivarium_assistant::adapters::{self, config, metrics, raspberrypi};
use vivarium_assistant::config::Config;
use vivarium_assistant::domain::outputs::{CurrentTimeProvider, OutputStatus};
use vivarium_assistant::domain::GPIO;
use vivarium_assistant::domain::{outputs, sensors, OutputPin};
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
    let controller = Arc::new(Mutex::new(outputs::Controller::new(
        config.outputs(),
        gpio.clone(),
        current_time_provider,
    )?));

    let default_panic = std::panic::take_hook();
    let closure_controller = controller.clone();
    std::panic::set_hook(Box::new(move |info| {
        if let Ok(mut v) = closure_controller.lock() {
            v.fail_safe();
        }
        default_panic(info);
    }));

    let metrics = metrics::Metrics::new()?;
    let server = Server::new();

    let mut water_level_sensors = vec![];
    for definition in config.water_level_sensors().sensors() {
        let trig = gpio.output(&definition.trig_pin())?;
        let echo = gpio.input(&definition.echo_pin())?;
        let sensor = sensors::HCSR04::new(trig, echo)?;
        let sensor = sensors::WaterLevelSensor::new(
            definition.min_distance(),
            definition.max_distance(),
            sensor,
        )?;
        water_level_sensors.push(WaterLevelSensorWithName {
            name: definition.name().clone(),
            sensor,
        });
    }

    tokio::spawn({
        let metrics = metrics.clone();
        async move { update_water_sensors_loop(water_level_sensors, metrics).await }
    });
    tokio::spawn({
        let metrics = metrics.clone();
        async move { server_loop(&server, &config, metrics).await }
    });
    update_outputs_loop(controller, metrics.clone()).await;
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

async fn server_loop(server: &Server, config: &Config, metrics: metrics::Metrics) {
    loop {
        match server.run(config, metrics.clone()).await {
            Ok(_) => {
                println!("for some reason the server exited without returning any errors?")
            }
            Err(err) => {
                println!("the server exited with an error: {err}")
            }
        }
    }
}

async fn update_water_sensors_loop<T: sensors::DistanceSensor>(
    mut sensors: Vec<WaterLevelSensorWithName<T>>,
    mut metrics: metrics::Metrics,
) {
    let zero = sensors::WaterLevel::new(0.0).unwrap();

    loop {
        for sensor in &mut sensors {
            let level = match sensor.sensor.measure() {
                Ok(value) => value,
                Err(_) => zero,
            };
            metrics.report_water_level(&sensor.name, &level);
        }
        time::sleep(UPDATE_SENSORS_EVERY).await;
    }
}

async fn update_outputs_loop(
    controller: Arc<Mutex<dyn Controller>>,
    mut metrics: metrics::Metrics,
) {
    loop {
        update_outputs(&controller, &mut metrics);
        time::sleep(UPDATE_OUTPUTS_EVERY).await;
    }
}

fn update_outputs(
    controller: &Arc<Mutex<dyn Controller>>,
    metrics: &mut metrics::Metrics,
) {
        let mut controller = controller.lock().unwrap();
        controller.update_outputs();
        let status = controller.status();
        drop(controller);

        for entry in status {
            metrics.report_output(&entry.name, &entry.state);
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

struct WaterLevelSensorWithName<T: sensors::DistanceSensor> {
    name: sensors::SensorName,
    sensor: sensors::WaterLevelSensor<T>,
}
