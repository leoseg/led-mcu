use std::sync::mpsc::Sender;
use std::time::Duration;
use embedded_svc::mqtt::client::QoS;
use esp_idf_hal::sys::EspError;
use esp_idf_svc::mqtt::client::{EspMqttClient, EspMqttConnection, MqttClientConfiguration, MqttProtocolVersion};
use log::{error, info};
use anyhow::Result;
use crate::led;
use crate::led::LedController;


/// Function to initialize MQTT client
/// host: &str - MQTT Broker host
/// port: u16 - MQTT Broker port
/// return: Result<(EspMqttClient<'static>, EspMqttConnection),EspError> - MQTT Client and Connection
pub fn init_mqtt_client(host : & str, port: u16) -> Result<(EspMqttClient<'static>, EspMqttConnection),EspError>{
    let mqtt_config = MqttClientConfiguration {
        client_id: Some("led-mcu"),
        protocol_version: Some(MqttProtocolVersion::V3_1_1),
        keep_alive_interval: Some(Duration::from_secs(30)),
        ..MqttClientConfiguration::default()
    };
    let (mqtt_client,mqtt_conn) = EspMqttClient::new(
        &format!("mqtt://{host}:{port}"),
        &mqtt_config
    ).expect("Failed to create MQTT client");
    Ok((mqtt_client, mqtt_conn))
}

/// Function to run MQTT client in loop subscribing to a topic and updating led state based on received messages
/// client: &mut EspMqttClient - MQTT Client
/// conn: &mut EspMqttConnection - MQTT Connection
/// topic: &str - MQTT Topic to subscribe
/// led_controller: &mut LedController - Led Controller used for updating led state
pub fn run(client : & mut EspMqttClient, conn : & mut EspMqttConnection, topic: &str, sender: Sender<led>) {
    std::thread::scope(|function_scope|
    {

        let handle = std::thread::Builder::new()
            .stack_size(6000)
            .spawn_scoped(function_scope,move || {
                info!("MQTT Listening for messages");

                while let Ok(event) = conn.next() {
                    info!("[Queue] Event: {}", event.payload());
                    if event.payload().to_string().starts_with("Received"){
                        let payload = event.payload().to_string();
                        let led = extract_and_parse_payload(&payload).expect("Failed to parse payload");
                        sender.send(led).expect("Error sending message to Led Controller");
                    }
                }
                info!("Connection closed");
            })
            .unwrap();
        std::thread::sleep(Duration::from_millis(5000));
        loop {
            if let Err(e) = client.subscribe(topic, QoS::AtMostOnce) {
                error!("Failed to subscribe to topic \"{}\": {:?}", topic, e);
                std::thread::sleep(Duration::from_millis(5000));
                continue;
            }
            info!("Subscribed to topic \"{topic}\"");
            break
        }
        handle.join().expect("Thread panicked");
    })
}

fn extract_and_parse_payload(payload: &str) -> Result<led::Led> {
    // Extract the content inside the payload string
    let payload = payload.replace("\\", "");
    let start = payload.find("data: Ok(\"").ok_or_else(|| anyhow::anyhow!("Invalid payload format"))? + 10;
    let end = payload[start..].find("\")").ok_or_else(|| anyhow::anyhow!("Invalid payload format"))?+start;
    let content = &payload[start..end];
    info!("Content: {content}");
    // Parse the extracted content to JSON
    let json: led::Led = serde_json::from_str(content).unwrap_or_else(|err| {
        error!("Failed to parse JSON: {err}");
        panic!("Failed to parse JSON: {err}");
    });
    Ok(json)
}