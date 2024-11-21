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

fn main() {
    if !std::path::Path::new("cfg.toml").exists() {
        panic!("You need to create a `cfg.toml` file with your Wi-Fi credentials!");
    }
    embuild::espidf::sysenv::output();
}
