#![feature(duration_constructors)]

use anyhow::anyhow;
use env_logger::Env;
use log::{error, info};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::{env, fs};
use tokio::time;
use vivarium_assistant::adapters::{self, config, metrics};
use vivarium_assistant::config::Config;
use vivarium_assistant::domain::outputs::{CurrentTimeProvider, OutputStatus};
use vivarium_assistant::domain::sensors::{MedianCache, WaterLevel};
use vivarium_assistant::domain::{self, GPIO};
use vivarium_assistant::domain::{outputs, sensors};
use vivarium_assistant::errors::Result;
use vivarium_assistant::ports::http::{self, Server};

#[cfg(feature = "raspberry_pi")]
use vivarium_assistant::adapters::raspberrypi;

const UPDATE_SENSORS_EVERY: Duration = Duration::from_secs(10);
const UPDATE_OUTPUTS_EVERY: Duration = Duration::from_millis(100);
const WATER_SENSOR_SMOOTHING_PERIOD: Duration = Duration::from_mins(5); // should presumably be
                                                                        // significantly larger
                                                                        // than
                                                                        // UPDATE_SENSORS_EVERY

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    #[cfg(not(feature = "raspberry_pi"))]
    let gpio = adapters::MockGPIO::new();

    #[cfg(feature = "raspberry_pi")]
    let gpio = raspberrypi::GPIO::new()?;

    #[cfg(not(feature = "raspberry_pi"))]
    let i2c = adapters::MockI2C::new();

    #[cfg(feature = "raspberry_pi")]
    let i2c = raspberrypi::I2C::new()?;

    let aht20 = sensors::AHT20::new(i2c)?;

    let current_time_provider = adapters::CurrentTimeProvider::new();
    let mut metrics = metrics::Metrics::new()?;
    metrics.set_startup_time(&current_time_provider.now());

    let config = load_config()?;

    let controller = SafeController::new(outputs::Controller::new(
        config.outputs(),
        gpio.clone(),
        current_time_provider.clone(),
    )?);
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
        water_level_sensors.push(QueriedWaterLevelSensor {
            name: definition.name().clone(),
            sensor,
            cache: MedianCache::new(WATER_SENSOR_SMOOTHING_PERIOD)?,
        });
    }

    setup_failsafe_hook(controller.clone());

    tokio::spawn({
        let metrics = metrics.clone();
        async move { update_water_sensors_loop(water_level_sensors, metrics).await }
    });

    if let Some(aht_20_name) = config.aht_20() {
        tokio::spawn({
            let metrics = metrics.clone();
            let aht_20_name = aht_20_name.clone();
            async move { update_aht20_loop(&aht_20_name, aht20, metrics).await }
        });
    }

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
    mut sensors: Vec<QueriedWaterLevelSensor<T>>,
    mut metrics: M,
) where
    T: sensors::DistanceSensor,
    M: Metrics,
{
    let zero = sensors::WaterLevel::new(0.0).unwrap();

    loop {
        for sensor in &mut sensors {
            match sensor.sensor.measure() {
                Ok(value) => {
                    info!(
                        "Water level sensor '{name}' reported water level '{level}'",
                        name = sensor.name,
                        level = value
                    );
                    sensor.cache.put(value);
                }
                Err(err) => {
                    error!(
                        "Water level sensor '{name}' returned an error: {err}",
                        name = sensor.name,
                        err = err
                    );
                }
            };

            let level = match sensor.cache.get() {
                Some(value) => value,
                None => &zero,
            };
            metrics.report_water_level(&sensor.name, level);
        }
        time::sleep(UPDATE_SENSORS_EVERY).await;
    }
}

async fn update_aht20_loop<M, I>(
    sensor_name: &sensors::SensorName,
    mut sensor: sensors::AHT20<I>,
    mut metrics: M,
) where
    M: Metrics,
    I: domain::I2C,
{
    let zero_temperature = sensors::Temperature::new(0.0).unwrap();
    let zero_humidity = sensors::Humidity::new(0.0).unwrap();

    loop {
        match sensor.measure() {
            Ok(value) => {
                info!(
                        "AHT20 sensor '{name}' reported temperature '{temperature}' and humidity '{humidity}'",
                        name = sensor_name,
                        temperature = value.temperature(),
                        humidity = value.humidity(),
                    );
                metrics.report_temperature(sensor_name, &value.temperature());
                metrics.report_humidity(sensor_name, &value.humidity());
            }
            Err(err) => {
                error!(
                    "AHT20 sensor '{name}' returned an error: {err}",
                    name = sensor_name,
                    err = err
                );
                metrics.report_temperature(sensor_name, &zero_temperature);
                metrics.report_humidity(sensor_name, &zero_humidity);
            }
        };

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

struct QueriedWaterLevelSensor<T: sensors::DistanceSensor> {
    name: sensors::SensorName,
    sensor: sensors::WaterLevelSensor<T>,
    cache: sensors::MedianCache<WaterLevel>,
}

trait Metrics {
    fn report_output(&mut self, output: &outputs::OutputName, state: &outputs::OutputState);
    fn report_water_level(&mut self, sensor: &sensors::SensorName, level: &sensors::WaterLevel);
    fn report_temperature(
        &mut self,
        sensor: &sensors::SensorName,
        temperature: &sensors::Temperature,
    );
    fn report_humidity(&mut self, sensor: &sensors::SensorName, humidity: &sensors::Humidity);
}

impl Metrics for metrics::Metrics {
    fn report_output(&mut self, output: &outputs::OutputName, state: &outputs::OutputState) {
        metrics::Metrics::report_output(self, output, state);
    }

    fn report_water_level(&mut self, sensor: &sensors::SensorName, level: &sensors::WaterLevel) {
        metrics::Metrics::report_water_level(self, sensor, level);
    }

    fn report_temperature(
        &mut self,
        sensor: &sensors::SensorName,
        temperature: &sensors::Temperature,
    ) {
        metrics::Metrics::report_temperature(self, sensor, temperature);
    }

    fn report_humidity(&mut self, sensor: &sensors::SensorName, humidity: &sensors::Humidity) {
        metrics::Metrics::report_humidity(self, sensor, humidity);
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
