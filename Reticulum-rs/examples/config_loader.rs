//! Example demonstrating configuration loading and interface creation
//! from a configuration file.

use reticulum::config::{load_or_create_config, InterfaceConfig};
use reticulum::iface::InterfaceManager;
use reticulum::transport::{Transport, TransportConfig};
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    env_logger::init();
    
    println!("=== Reticulum Configuration Example ===");
    
    // Load configuration from file
    let config_path = Some(std::path::PathBuf::from("examples/config_example.toml"));
    let mut config_manager = load_or_create_config(config_path)?;
    
    println!("Configuration loaded from: {:?}", config_manager.config_path());
    
    let config = config_manager.config();
    
    // Print configuration summary
    println!("\n=== Configuration Summary ===");
    println!("Node name: {:?}", config.global.node_name);
    println!("Max packet size: {} bytes", config.global.max_packet_size);
    println!("Number of interfaces: {}", config.interfaces.len());
    println!("Number of destinations: {}", config.destinations.len());
    
    // Create transport with configuration
    let transport_config = TransportConfig::default();
    let transport = Transport::new(transport_config);
    
    // Create interfaces from configuration
    println!("\n=== Creating Interfaces ===");
    
    for interface_config in &config.interfaces {
        match interface_config {
            InterfaceConfig::TcpClient { name, host, port, enabled, .. } => {
                if *enabled {
                    println!("Creating TCP client interface '{}' to {}:{}", name, host, port);
                    // In a real implementation, you would create the interface here
                    // let iface = TcpClient::new(&format!("{}:{}", host, port));
                    // transport.iface_manager().lock().await.spawn(iface, TcpClient::spawn);
                } else {
                    println!("TCP client interface '{}' is disabled", name);
                }
            }
            InterfaceConfig::TcpServer { name, bind_address, port, enabled, .. } => {
                if *enabled {
                    println!("Creating TCP server interface '{}' on {}:{}", name, bind_address, port);
                    // In a real implementation, you would create the interface here
                } else {
                    println!("TCP server interface '{}' is disabled", name);
                }
            }
            InterfaceConfig::Udp { name, bind_address, port, enabled, .. } => {
                if *enabled {
                    println!("Creating UDP interface '{}' on {}:{}", name, bind_address, port);
                    // In a real implementation, you would create the interface here
                } else {
                    println!("UDP interface '{}' is disabled", name);
                }
            }
            InterfaceConfig::Serial { name, port, baud_rate, enabled, .. } => {
                if *enabled {
                    println!("Creating Serial interface '{}' on {} at {} baud", name, port, baud_rate);
                    // In a real implementation, you would create the interface here
                } else {
                    println!("Serial interface '{}' is disabled", name);
                }
            }
            InterfaceConfig::Mqtt { name, host, port, client_id, username, topic_prefix, use_tls, enabled, .. } => {
                if *enabled {
                    println!("Creating MQTT interface '{}' to {}:{}", name, host, port);
                    println!("  Client ID: {:?}", client_id);
                    println!("  Username: {:?}", username);
                    println!("  Topic prefix: {:?}", topic_prefix);
                    println!("  Use TLS: {}", use_tls);
                    // In a real implementation, you would create the interface here
                } else {
                    println!("MQTT interface '{}' is disabled", name);
                }
            }
            InterfaceConfig::Kiss { name, port, baud_rate, enabled, .. } => {
                if *enabled {
                    println!("Creating KISS interface '{}' on {} at {} baud", name, port, baud_rate);
                    // In a real implementation, you would create the interface here
                } else {
                    println!("KISS interface '{}' is disabled", name);
                }
            }
            InterfaceConfig::I2p { name, sam_address, sam_port, destination, session_name, session_type, use_local_dest, enabled, .. } => {
                if *enabled {
                    println!("Creating I2P interface '{}' to SAM bridge {}:{}", name, sam_address, sam_port);
                    println!("  Destination: {:?}", destination);
                    println!("  Session name: {:?}", session_name);
                    println!("  Session type: {:?}", session_type);
                    println!("  Use local destination: {:?}", use_local_dest);
                    // In a real implementation, you would create the interface here
                } else {
                    println!("I2P interface '{}' is disabled", name);
                }
            }
            InterfaceConfig::Kaonic { name, enabled, .. } => {
                if *enabled {
                    println!("Creating Kaonic interface '{}'", name);
                    // In a real implementation, you would create the interface here
                } else {
                    println!("Kaonic interface '{}' is disabled", name);
                }
            }
            InterfaceConfig::Custom { name, interface_type, enabled, .. } => {
                if *enabled {
                    println!("Creating custom interface '{}' of type '{}'", name, interface_type);
                    // In a real implementation, you would create the interface here
                } else {
                    println!("Custom interface '{}' is disabled", name);
                }
            }
        }
    }
    
    println!("\n=== Configuration Example Complete ===");
    println!("This example demonstrates how to load and parse configuration.");
    println!("In a real application, you would create the actual interfaces");
    println!("and integrate them with the transport layer.");
    
    Ok(())
}