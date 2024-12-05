use anyhow::anyhow;
use env_logger::Env;
use log::{error, info};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::{env, fs};
use tokio::time;
use vivarium_assistant::adapters::{self, config, metrics, raspberrypi};
use vivarium_assistant::config::Config;
use vivarium_assistant::domain::outputs::OutputStatus;
use vivarium_assistant::domain::{self, GPIO};
use vivarium_assistant::domain::{outputs, sensors};
use vivarium_assistant::errors::Result;
use vivarium_assistant::ports::http::{self, Server};

const UPDATE_SENSORS_EVERY: Duration = Duration::from_secs(10);
const UPDATE_OUTPUTS_EVERY: Duration = Duration::from_secs(1);

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    #[cfg(feature = "not_raspberry_pi")]
    let gpio = adapters::MockGPIO::new();

    #[cfg(not(feature = "not_raspberry_pi"))]
    let gpio = raspberrypi::GPIO::new()?;

    let config = load_config()?;
    let current_time_provider = adapters::CurrentTimeProvider::new();
    let controller = SafeController::new(outputs::Controller::new(
        config.outputs(),
        gpio.clone(),
        current_time_provider,
    )?);
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

    setup_failsafe_hook(controller.clone());

    tokio::spawn({
        let metrics = metrics.clone();
        async move { update_water_sensors_loop(water_level_sensors, metrics).await }
    });
    tokio::spawn({
        let metrics = metrics.clone();
        let controller = controller.clone();
        async move { server_loop(&server, &config, metrics, controller).await }
    });
    update_outputs_loop(controller, metrics.clone()).await;
    Ok(())
}

fn setup_failsafe_hook<C>(controller: C)
where
    C: Controller + 'static,
{
    std::panic::set_hook(Box::new({
        let default_panic = std::panic::take_hook();
        move |info| {
            controller.fail_safe();
            default_panic(info);
        }
    }));
}

fn load_config() -> Result<Config> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        return Err(anyhow!("usage: program path_to_config_file.toml"));
    }

    let config_string = fs::read_to_string(args.get(1).unwrap())?;
    config::load(&config_string)
}

async fn server_loop<M, C>(server: &Server, config: &Config, metrics: M, controller: C)
where
    M: http::Metrics + Sync + Send + Clone + 'static,
    C: http::Controller + Sync + Send + Clone + 'static,
{
    let deps = http::Deps::new(metrics, controller);

    loop {
        match server.run(config, deps.clone()).await {
            Ok(_) => {
                error!("for some reason the server exited without returning any errors?")
            }
            Err(err) => {
                error!("the server exited with an error: {err}")
            }
        }
    }
}

async fn update_water_sensors_loop<T, M>(
    mut sensors: Vec<WaterLevelSensorWithName<T>>,
    mut metrics: M,
) where
    T: sensors::DistanceSensor,
    M: Metrics,
{
    let zero = sensors::WaterLevel::new(0.0).unwrap();

    loop {
        for sensor in &mut sensors {
            let level = match sensor.sensor.measure() {
                Ok(value) => {
                    info!(
                        "Water level sensor '{name}' reported water level '{level}'",
                        name = sensor.name,
                        level = value
                    );
                    value
                }
                Err(err) => {
                    error!(
                        "Water level sensor '{name}' returned an error: {err}",
                        name = sensor.name,
                        err = err
                    );
                    zero
                }
            };
            metrics.report_water_level(&sensor.name, &level);
        }
        time::sleep(UPDATE_SENSORS_EVERY).await;
    }
}

async fn update_outputs_loop<C, M>(controller: C, mut metrics: M)
where
    C: Controller,
    M: Metrics,
{
    loop {
        controller.update_outputs();
        for entry in controller.status() {
            metrics.report_output(&entry.name, &entry.state);
        }
        time::sleep(UPDATE_OUTPUTS_EVERY).await;
    }
}

struct WaterLevelSensorWithName<T: sensors::DistanceSensor> {
    name: sensors::SensorName,
    sensor: sensors::WaterLevelSensor<T>,
}

trait Metrics {
    fn report_output(&mut self, output: &outputs::OutputName, state: &outputs::OutputState);
    fn report_water_level(&mut self, sensor: &sensors::SensorName, level: &sensors::WaterLevel);
}

impl Metrics for metrics::Metrics {
    fn report_output(&mut self, output: &outputs::OutputName, state: &outputs::OutputState) {
        metrics::Metrics::report_output(self, output, state);
    }

    fn report_water_level(&mut self, sensor: &sensors::SensorName, level: &sensors::WaterLevel) {
        metrics::Metrics::report_water_level(self, sensor, level);
    }
}

trait Controller: Send + Sync {
    fn update_outputs(&self);
    fn status(&self) -> Vec<OutputStatus>;
    fn fail_safe(&self);
}

trait WrappedController: Send {
    fn update_outputs(&mut self);
    fn status(&mut self) -> Vec<OutputStatus>;
    fn clear_overrides(&mut self, output_name: outputs::OutputName) -> Result<()>;
    fn add_override(
        &mut self,
        output_name: outputs::OutputName,
        state: outputs::OutputState,
        activation: outputs::ScheduledActivation,
    ) -> Result<()>;
    fn fail_safe(&mut self);
}

impl<OP, CTP> WrappedController for outputs::Controller<OP, CTP>
where
    OP: domain::OutputPin + Send,
    CTP: outputs::CurrentTimeProvider + Send,
{
    fn update_outputs(&mut self) {
        outputs::Controller::update_outputs(self);
    }

    fn status(&mut self) -> Vec<OutputStatus> {
        outputs::Controller::status(self)
    }

    fn clear_overrides(&mut self, output_name: outputs::OutputName) -> Result<()> {
        outputs::Controller::clear_overrides(self, output_name)
    }

    fn fail_safe(&mut self) {
        outputs::Controller::fail_safe(self)
    }

    fn add_override(
        &mut self,
        output_name: outputs::OutputName,
        state: outputs::OutputState,
        activation: outputs::ScheduledActivation,
    ) -> Result<()> {
        outputs::Controller::add_override(self, output_name, state, activation)
    }
}

struct SafeController<T>
where
    T: WrappedController,
{
    controller: Arc<Mutex<T>>,
}

impl<T> SafeController<T>
where
    T: WrappedController,
{
    fn new(controller: T) -> Self {
        Self {
            controller: Arc::new(Mutex::new(controller)),
        }
    }
}

impl<T> Controller for SafeController<T>
where
    T: WrappedController,
{
    fn update_outputs(&self) {
        let mut controller = self.controller.lock().unwrap();
        (*controller).update_outputs();
    }

    fn status(&self) -> Vec<OutputStatus> {
        let mut controller = self.controller.lock().unwrap();
        (*controller).status()
    }

    fn fail_safe(&self) {
        let mut controller = self.controller.lock().unwrap();
        (*controller).fail_safe()
    }
}

impl<T> http::Controller for SafeController<T>
where
    T: WrappedController,
{
    fn clear_overrides(&mut self, output_name: outputs::OutputName) -> Result<()> {
        let mut controller = self.controller.lock().unwrap();
        (*controller).clear_overrides(output_name)
    }

    fn add_override(
        &mut self,
        output_name: outputs::OutputName,
        state: outputs::OutputState,
        activation: outputs::ScheduledActivation,
    ) -> Result<()> {
        let mut controller = self.controller.lock().unwrap();
        (*controller).add_override(output_name, state, activation)
    }
}

impl<T> Clone for SafeController<T>
where
    T: WrappedController,
{
    fn clone(&self) -> Self {
        Self {
            controller: self.controller.clone(),
        }
    }
}
