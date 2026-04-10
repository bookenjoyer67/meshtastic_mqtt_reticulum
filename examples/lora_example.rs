//! Example program demonstrating direct LoRa hardware interface
//!
//! This example shows how to use the LoRa interface with different hardware types.

use meshtastic_reticulum_bridge::lora_interface::{LoRaConfig, LoRaHardware, LoRaInterface, LoRaManager};
use anyhow::Result;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    
    println!("=== Direct LoRa Hardware Interface Example ===");
    
    // Create LoRa manager
    let mut lora_manager = LoRaManager::new();
    
    // Example 1: Serial LoRa module (e.g., RAK3172)
    println!("\n1. Creating serial LoRa interface...");
    let serial_config = LoRaConfig {
        frequency: 868_000_000,  // 868 MHz (EU)
        ..Default::default()
    };
    
    let serial_hardware = LoRaHardware::Serial {
        port: "/dev/ttyUSB0".to_string(),
        baud_rate: 9600,
    };
    
    let serial_interface = LoRaInterface::new(serial_config, serial_hardware);
    let serial_handle = lora_manager.add_interface(serial_interface);
    
    // Example 2: SPI LoRa module (e.g., SX1276 on Raspberry Pi)
    println!("\n2. Creating SPI LoRa interface...");
    let spi_config = LoRaConfig {
        frequency: 915_000_000,  // 915 MHz (US)
        spreading_factor: 9,      // SF9 for longer range
        tx_power: 20,            // Max power
        ..Default::default()
    };
    
    let spi_hardware = LoRaHardware::Spi {
        device: "/dev/spidev0.0".to_string(),
        cs_pin: 8,               // GPIO8 (CE0)
        reset_pin: Some(25),     // GPIO25
        dio0_pin: Some(17),      // GPIO17 for interrupts
    };
    
    let spi_interface = LoRaInterface::new(spi_config, spi_hardware);
    let spi_handle = lora_manager.add_interface(spi_interface);
    
    // Example 3: USB LoRa dongle
    println!("\n3. Creating USB LoRa interface...");
    let usb_config = LoRaConfig {
        frequency: 433_000_000,  // 433 MHz
        bandwidth: 250_000,      // 250 kHz bandwidth
        ..Default::default()
    };
    
    let usb_hardware = LoRaHardware::Usb {
        vid: 0x0483,  // STMicroelectronics
        pid: 0x5740,  // Example PID
    };
    
    let usb_interface = LoRaInterface::new(usb_config, usb_hardware);
    let usb_handle = lora_manager.add_interface(usb_interface);
    
    println!("\nTotal interfaces created: {}", lora_manager.get_interfaces().len());
    
    // Initialize interfaces (in a real application, you'd handle errors)
    println!("\nInitializing interfaces...");
    
    // Try to initialize serial interface
    match serial_handle.lock().await.initialize().await {
        Ok(_) => println!("Serial interface initialized"),
        Err(e) => println!("Failed to initialize serial interface: {}", e),
    }
    
    // Try to initialize SPI interface  
    match spi_handle.lock().await.initialize().await {
        Ok(_) => println!("SPI interface initialized"),
        Err(e) => println!("Failed to initialize SPI interface: {}", e),
    }
    
    // Try to initialize USB interface
    match usb_handle.lock().await.initialize().await {
        Ok(_) => println!("USB interface initialized"),
        Err(e) => println!("Failed to initialize USB interface: {}", e),
    }
    
    // Demonstrate configuration changes
    println!("\nDemonstrating configuration changes...");
    
    // Change frequency on SPI interface
    if let Ok(mut spi) = spi_handle.try_lock() {
        if let Err(e) = spi.set_frequency(923_000_000).await {
            println!("Failed to change frequency: {}", e);
        } else {
            println!("Changed SPI interface frequency to 923 MHz");
        }
    }
    
    // Change spreading factor on serial interface
    if let Ok(mut serial) = serial_handle.try_lock() {
        if let Err(e) = serial.set_spreading_factor(11).await {
            println!("Failed to change spreading factor: {}", e);
        } else {
            println!("Changed serial interface spreading factor to SF11");
        }
    }
    
    // Demonstrate sending data
    println!("\nDemonstrating data transmission...");
    
    let test_message = b"Hello from LoRa interface!";
    
    // Try to send via SPI interface
    if let Ok(mut spi) = spi_handle.try_lock() {
        match spi.send(test_message).await {
            Ok(_) => println!("Message sent via SPI interface"),
            Err(e) => println!("Failed to send via SPI: {}", e),
        }
    }
    
    // Demonstrate receiving data (non-blocking check)
    println!("\nChecking for received data...");
    
    // Check all interfaces for received data
    for (i, interface) in lora_manager.get_interfaces().iter().enumerate() {
        if let Ok(mut iface) = interface.try_lock() {
            match iface.receive().await {
                Ok(Some(data)) => println!("Interface {} received {} bytes", i, data.len()),
                Ok(None) => println!("Interface {} has no data", i),
                Err(e) => println!("Interface {} receive error: {}", i, e),
            }
        }
    }
    
    println!("\n=== Example completed ===");
    println!("\nNext steps:");
    println!("1. Connect actual LoRa hardware");
    println!("2. Implement hardware-specific drivers");
    println!("3. Integrate with Reticulum protocol");
    println!("4. Add to main bridge application");
    
    Ok(())
}