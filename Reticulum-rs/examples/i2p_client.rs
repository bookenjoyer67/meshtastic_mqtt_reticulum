//! Example I2P client for Reticulum-rs
//!
//! This example demonstrates how to use the I2P interface with Reticulum-rs.
//! It connects to an I2P SAM bridge and can tunnel Reticulum traffic through the I2P network.
//!
//! Usage:
//!   I2P_SAM_ADDRESS=127.0.0.1 I2P_SAM_PORT=7656 cargo run --example i2p_client --features i2p
//!
//! Note: This requires a running I2P router with SAM bridge enabled.

use reticulum::iface::{InterfaceManager, i2p::I2PInterface};
use tokio;

#[tokio::main]
async fn main() {
    // Initialize logging
    env_logger::init();

    // Get configuration from environment variables
    let sam_address = std::env::var("I2P_SAM_ADDRESS").unwrap_or_else(|_| "127.0.0.1".to_string());
    let sam_port = std::env::var("I2P_SAM_PORT")
        .unwrap_or_else(|_| "7656".to_string())
        .parse()
        .unwrap_or(7656);
    
    let destination = std::env::var("I2P_DESTINATION").ok();
    let session_name = std::env::var("I2P_SESSION_NAME").unwrap_or_else(|_| "reticulum-i2p".to_string());
    let session_type = std::env::var("I2P_SESSION_TYPE").unwrap_or_else(|_| "STREAM".to_string());
    let use_local_dest = std::env::var("I2P_USE_LOCAL_DEST")
        .unwrap_or_else(|_| "true".to_string())
        .parse()
        .unwrap_or(true);
    let max_connection_attempts = std::env::var("I2P_MAX_CONNECTION_ATTEMPTS")
        .unwrap_or_else(|_| "10".to_string())
        .parse()
        .unwrap_or(10);

    println!("I2P Client Configuration:");
    println!("  SAM Address: {}:{}", sam_address, sam_port);
    println!("  Session Name: {}", session_name);
    println!("  Session Type: {}", session_type);
    println!("  Use Local Destination: {}", use_local_dest);
    println!("  Max Connection Attempts: {}", max_connection_attempts);
    
    if let Some(dest) = &destination {
        println!("  Destination: {}", dest);
    } else {
        println!("  Destination: (none - will accept incoming connections)");
    }

    // Create interface manager
    let mut iface_manager = InterfaceManager::new(10);

    // Create I2P interface
    let i2p_iface = I2PInterface::new(sam_address, sam_port)
        .with_session_name(session_name)
        .with_session_type(session_type)
        .with_use_local_dest(use_local_dest)
        .with_max_connection_attempts(max_connection_attempts);

    // Add destination if specified
    let i2p_iface = if let Some(dest) = destination {
        i2p_iface.with_destination(dest)
    } else {
        i2p_iface
    };

    // Spawn I2P interface
    let i2p_address = iface_manager.spawn(i2p_iface, |context| async move {
        I2PInterface::spawn(context).await;
    });

    println!("I2P interface spawned with address: {}", i2p_address);

    // Keep the main task running
    println!("I2P client running. Press Ctrl+C to exit.");
    
    // Wait for Ctrl+C
    tokio::signal::ctrl_c().await.unwrap();
    println!("Shutting down...");
}