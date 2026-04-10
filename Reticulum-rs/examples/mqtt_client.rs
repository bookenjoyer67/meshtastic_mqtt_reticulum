use reticulum::iface::mqtt::MqttInterface;
use reticulum::transport::{Transport, TransportConfig};
use reticulum::destination::DestinationName;
use reticulum::identity::PrivateIdentity;
use rand_core::OsRng;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    // Check if mqtt feature is enabled
    #[cfg(not(feature = "mqtt"))]
    {
        eprintln!("MQTT feature not enabled. Build with --features mqtt");
        return Ok(());
    }

    // ---------- Reticulum setup ----------
    let mut transport = Transport::new(TransportConfig::default());
    
    // Get MQTT broker details from environment variables with fallbacks
    let mqtt_host = std::env::var("MQTT_HOST")
        .unwrap_or_else(|_| "localhost".to_string());
    
    let mqtt_port: u16 = std::env::var("MQTT_PORT")
        .unwrap_or_else(|_| "1883".to_string())
        .parse()
        .unwrap_or(1883);
    
    let mqtt_username = std::env::var("MQTT_USERNAME").ok();
    let mqtt_password = std::env::var("MQTT_PASSWORD").ok();
    
    let use_tls: bool = std::env::var("MQTT_USE_TLS")
        .unwrap_or_else(|_| "false".to_string())
        .parse()
        .unwrap_or(false);
    
    // Create MQTT interface
    let mut mqtt_iface = MqttInterface::new(&mqtt_host, mqtt_port);
    
    // Set credentials if provided
    if let (Some(username), Some(password)) = (&mqtt_username, &mqtt_password) {
        mqtt_iface = mqtt_iface.with_credentials(username, password);
    }
    
    // Set TLS if enabled
    if use_tls {
        mqtt_iface = mqtt_iface.with_tls(true);
    }
    
    // Set client ID from environment variable if provided
    if let Ok(client_id) = std::env::var("MQTT_CLIENT_ID") {
        mqtt_iface = mqtt_iface.with_client_id(client_id);
    }
    
    // Set topic prefix from environment variable if provided
    if let Ok(topic_prefix) = std::env::var("MQTT_TOPIC_PREFIX") {
        mqtt_iface = mqtt_iface.with_topic_prefix(topic_prefix);
    }
    
    transport
        .iface_manager()
        .lock()
        .await
        .spawn(
            mqtt_iface,
            MqttInterface::spawn,
        );
    
    log::info!("Transport created with MQTT interface on {}:{}", mqtt_host, mqtt_port);

    let identity = PrivateIdentity::new_from_rand(OsRng);
    let dest_name = DestinationName::new("mqtt_example", "app");
    let destination = transport.add_destination(identity, dest_name).await;
    let my_hash = destination.lock().await.desc.address_hash.clone();
    log::info!("My hash: {}", my_hash);

    let announce = destination.lock().await.announce(OsRng, None)?;
    transport.send_packet(announce).await;
    log::info!("Announced on MQTT interface");

    // Keep running to receive packets
    log::info!("Listening for packets on MQTT interface...");
    log::info!("Press Ctrl+C to exit");
    
    // Simple loop to keep the program running
    loop {
        sleep(Duration::from_secs(1)).await;
    }
}