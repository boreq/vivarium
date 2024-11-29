use anyhow::anyhow;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::{env, fs, thread};
use vivarium_assistant::adapters::{self, config, raspberrypi};
use vivarium_assistant::domain::outputs::{CurrentTimeProvider, Executor};
use vivarium_assistant::domain::OutputPin;
use vivarium_assistant::errors::Result;

const UPDATE_SENSORS_EVERY: Duration = Duration::from_secs(1);
const UPDATE_OUTPUTS_EVERY: Duration = Duration::from_secs(1);

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        return Err(anyhow!("usage: program path_to_config_file.toml"));
    }

    let config_string = fs::read_to_string(args.get(1).unwrap())?;
    let config = config::load(&config_string)?;

    #[cfg(feature = "not_raspberry_pi")]
    let gpio = adapters::MockGPIO::new();

    #[cfg(not(feature = "not_raspberry_pi"))]
    let gpio = raspberrypi::GPIO::new()?;

    let current_time_provider = adapters::CurrentTimeProvider::new();
    let executor = Arc::new(Mutex::new(Executor::new(
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

    tokio::spawn(async { update_water_sensor_loop().await });

    update_outputs_loop(executor).await;
    Ok(())
}

async fn update_water_sensor_loop() {
    loop {
        println!("sensors");
        thread::sleep(UPDATE_SENSORS_EVERY);
    }
}

async fn update_outputs_loop<OP: OutputPin, CTP: CurrentTimeProvider>(
    executor: Arc<Mutex<Executor<OP, CTP>>>,
) {
    loop {
        let mut executor = executor.lock().unwrap();
        executor.update_outputs();
        thread::sleep(UPDATE_OUTPUTS_EVERY);
    }
}
