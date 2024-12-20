use embedded_svc::wifi::{AuthMethod, ClientConfiguration, Configuration};
use esp_idf_svc::wifi::{BlockingWifi, EspWifi};
use anyhow::Result;

/// Function to setup wifi connection
/// wifi: BlockingWifi<EspWifi> - Wifi instance
/// ssid: &str - Wifi SSID
/// password: &str - Wifi Password
pub fn setup_wifi(wifi : &mut BlockingWifi<EspWifi>, ssid: &str, password: &str) -> Result<()> {
    wifi.set_configuration(&Configuration::Client(ClientConfiguration {
        ssid: ssid.parse().unwrap(),
        bssid: None,
        auth_method: AuthMethod::None,
        password: password.parse().unwrap(),
        channel: None,
        scan_method: Default::default(),
        pmf_cfg: Default::default(),
    })).expect("Failed to set wifi configuration");

    // Start Wifi
    wifi.start().expect("Failed to start wifi");

    // Connect Wifi
    wifi.connect().expect(&format!("Failed to connect wifi with configuration ssid: {} and password: {}", ssid, password));

    wifi.wait_netif_up().expect("Failed to wait for network interface up");

    while !wifi.is_connected().unwrap() {
        // Get and print connection configuration
        let config = wifi.get_configuration().unwrap();
        println!("Waiting for station {:?}", config);
    }
    println!("Wifi Connected");
    Ok(())
}