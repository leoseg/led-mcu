mod wifi;
mod mqtt_client;
mod led;

use esp_idf_hal::gpio::{AnyOutputPin};
use anyhow::Result;
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::nvs::EspDefaultNvsPartition;
use esp_idf_svc::wifi::{BlockingWifi, EspWifi};

#[toml_cfg::toml_config]
pub struct Config {
    #[default("")]
    wifi_name: &'static str,
    #[default("")]
    wifi_password: &'static str,
    #[default(1884)]
    mqtt_port: u16,
    #[default("")]
    mqtt_host: &'static str,
    #[default("led")]
    mqtt_topic: &'static str,
}

fn main() -> Result<()>{
    esp_idf_svc::sys::link_patches();
    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();
    let peripherals = Peripherals::take().unwrap();
    let sysloop = EspSystemEventLoop::take()?;
    let nvs = EspDefaultNvsPartition::take()?;
    let app_config = CONFIG;
    let mut wifi = BlockingWifi::wrap(
        EspWifi::new(peripherals.modem, sysloop.clone(), Some(nvs))?,
        sysloop,
    )?;
    wifi::setup_wifi(&mut wifi, app_config.wifi_name, app_config.wifi_password)?;
    log::info!("Wifi connected!");
    let mut led_controller = led::LedController::new(AnyOutputPin::from(peripherals.pins.gpio3), peripherals.rmt.channel0);
    log::info!("Led Controller initialized!");
    let (mut mqtt_client,mut mqtt_conn) = mqtt_client::init_mqtt_client(app_config.mqtt_host, app_config.mqtt_port)?;
    mqtt_client::run(&mut mqtt_client, &mut mqtt_conn, app_config.mqtt_topic, &mut led_controller);
    Ok(())
}
