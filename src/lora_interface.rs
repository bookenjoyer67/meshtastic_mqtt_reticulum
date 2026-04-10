//! Direct LoRa hardware interface for Meshtastic Reticulum Bridge
//! 
//! This module provides a direct LoRa hardware interface that can work with:
//! 1. SX127x/SX126x chips via SPI (using embedded-hal traits)
//! 2. Serial LoRa modules (RAK811, RAK3172, etc.)
//! 3. USB LoRa dongles (RAKDAP1, etc.)
//!
//! The interface implements the Reticulum protocol over LoRa physical layer.

use std::sync::Arc;
use tokio::sync::Mutex;
use anyhow::{Result, anyhow};
use log::{info, warn};
use tokio::time::{sleep, Duration};

#[cfg(feature = "lora")]
use sx127x_lora::LoRa;
#[cfg(feature = "lora")]
use embedded_hal::spi::SpiDevice;
#[cfg(feature = "lora")]
use linux_embedded_hal::spidev::{SpidevOptions, Spidev};
#[cfg(feature = "lora")]
use linux_embedded_hal::spidev;
#[cfg(feature = "lora")]
use linux_embedded_hal::sysfs_gpio::Direction;
#[cfg(feature = "lora")]
use linux_embedded_hal::sysfs_gpio::Pin;
#[cfg(feature = "lora")]
use linux_embedded_hal::Delay;

#[cfg(feature = "lora")]
use serialport::{self, SerialPort};

/// LoRa interface configuration
#[derive(Debug, Clone)]
pub struct LoRaConfig {
    /// Frequency in Hz (e.g., 915_000_000 for 915 MHz)
    pub frequency: u64,
    /// Bandwidth in Hz
    pub bandwidth: u32,
    /// Spreading factor (7-12)
    pub spreading_factor: u8,
    /// Coding rate (5-8)
    pub coding_rate: u8,
    /// Transmission power in dBm
    pub tx_power: i8,
    /// Use explicit header mode
    pub explicit_header: bool,
    /// Use CRC
    pub crc_enabled: bool,
    /// Preamble length
    pub preamble_length: u16,
    /// Sync word
    pub sync_word: u8,
    /// RSSI averaging window size
    pub rssi_averaging_window: usize,
}

impl Default for LoRaConfig {
    fn default() -> Self {
        LoRaConfig {
            frequency: 915_000_000,  // 915 MHz (US)
            bandwidth: 125_000,      // 125 kHz
            spreading_factor: 7,     // SF7
            coding_rate: 5,          // 4/5
            tx_power: 17,            // 17 dBm
            explicit_header: true,
            crc_enabled: true,
            preamble_length: 8,
            sync_word: 0x12,         // Private sync word
            rssi_averaging_window: 10, // Average last 10 RSSI readings
        }
    }
}

/// LoRa hardware interface types
pub enum LoRaHardware {
    /// SPI interface for SX127x/SX126x chips
    Spi {
        /// SPI device path (e.g., "/dev/spidev0.0")
        device: String,
        /// Chip select GPIO pin
        cs_pin: u8,
        /// Reset GPIO pin
        reset_pin: Option<u8>,
        /// DIO0 GPIO pin for interrupts
        dio0_pin: Option<u8>,
    },
    /// Serial interface for modules with AT commands
    Serial {
        /// Serial port path (e.g., "/dev/ttyUSB0")
        port: String,
        /// Baud rate
        baud_rate: u32,
    },
    /// USB interface for dongles
    Usb {
        /// USB vendor ID
        vid: u16,
        /// USB product ID
        pid: u16,
    },
}

/// LoRa interface state
pub struct LoRaInterface {
    config: LoRaConfig,
    hardware: LoRaHardware,
    is_initialized: bool,
    #[cfg(feature = "lora")]
    lora_device: Option<Box<dyn LoRaDevice>>,
    rssi_history: Vec<i32>, // Store recent RSSI readings
    packet_count: u64,      // Total packets received
    packet_loss_count: u64, // Packets with CRC errors
    last_rssi: Option<i32>, // Last RSSI reading
    #[allow(dead_code)]
    last_snr: Option<f32>,  // Last SNR reading
}

/// Trait for LoRa device operations
#[cfg(feature = "lora")]
trait LoRaDevice: Send + Sync {
    fn send(&mut self, data: &[u8]) -> Result<()>;
    fn receive(&mut self) -> Result<Option<(Vec<u8>, i32, f32)>>; // Returns (data, rssi, snr)
    fn get_rssi(&mut self) -> Result<i32>;
    fn set_frequency(&mut self, frequency: u64) -> Result<()>;
    fn set_spreading_factor(&mut self, sf: u8) -> Result<()>;
    fn set_tx_power(&mut self, power: i8) -> Result<()>;
    fn get_packet_stats(&self) -> (u64, u64); // (total, lost)
}

#[cfg(feature = "lora")]
struct SpiLoRaDevice {
    lora: LoRa<Spidev, Pin, Pin, Delay>,
    packet_count: u64,
    packet_loss_count: u64,
}

#[cfg(feature = "lora")]
impl LoRaDevice for SpiLoRaDevice {
    fn send(&mut self, data: &[u8]) -> Result<()> {
        self.lora.transmit(data).map_err(|e| anyhow!("Transmit error: {:?}", e))
    }
    
    fn receive(&mut self) -> Result<Option<(Vec<u8>, i32, f32)>> {
        match self.lora.receive() {
            Ok(Some(packet)) => {
                self.packet_count += 1;
                // Get RSSI and SNR from the LoRa module
                let rssi = self.lora.get_packet_rssi().unwrap_or(-120);
                let snr = self.lora.get_packet_snr().unwrap_or(0.0);
                Ok(Some((packet, rssi, snr)))
            }
            Ok(None) => Ok(None),
            Err(e) => {
                self.packet_loss_count += 1;
                Err(anyhow!("Receive error: {:?}", e))
            }
        }
    }
    
    fn get_rssi(&mut self) -> Result<i32> {
        self.lora.get_rssi().map_err(|e| anyhow!("RSSI read error: {:?}", e))
    }
    
    fn set_frequency(&mut self, frequency: u64) -> Result<()> {
        self.lora.set_frequency(frequency).map_err(|e| anyhow!("Frequency set error: {:?}", e))
    }
    
    fn set_spreading_factor(&mut self, sf: u8) -> Result<()> {
        self.lora.set_spreading_factor(sf).map_err(|e| anyhow!("SF set error: {:?}", e))
    }
    
    fn set_tx_power(&mut self, power: i8) -> Result<()> {
        self.lora.set_tx_power(power).map_err(|e| anyhow!("TX power set error: {:?}", e))
    }
    
    fn get_packet_stats(&self) -> (u64, u64) {
        (self.packet_count, self.packet_loss_count)
    }
}

#[cfg(feature = "lora")]
struct SerialLoRaDevice {
    port: Box<dyn SerialPort>,
    packet_count: u64,
    packet_loss_count: u64,
}

#[cfg(feature = "lora")]
impl LoRaDevice for SerialLoRaDevice {
    fn send(&mut self, data: &[u8]) -> Result<()> {
        // Send AT command to transmit data
        let command = format!("AT+SEND={}\r\n", hex::encode(data));
        self.port.write_all(command.as_bytes())?;
        Ok(())
    }
    
    fn receive(&mut self) -> Result<Option<(Vec<u8>, i32, f32)>> {
        let mut buffer = [0u8; 256];
        match self.port.read(&mut buffer) {
            Ok(bytes_read) if bytes_read > 0 => {
                self.packet_count += 1;
                let data = buffer[..bytes_read].to_vec();
                // Parse RSSI from response if available
                let rssi = -85; // Default RSSI for serial modules
                let snr = 0.0; // SNR not typically available from serial modules
                Ok(Some((data, rssi, snr)))
            }
            Ok(_) => Ok(None),
            Err(e) => {
                self.packet_loss_count += 1;
                Err(anyhow!("Serial read error: {}", e))
            }
        }
    }
    
    fn get_rssi(&mut self) -> Result<i32> {
        // Send AT command to get RSSI
        self.port.write_all(b"AT+RSSI?\r\n")?;
        let mut buffer = [0u8; 32];
        let bytes_read = self.port.read(&mut buffer)?;
        let response = String::from_utf8_lossy(&buffer[..bytes_read]);
        
        // Parse RSSI from response
        if let Some(rssi_str) = response.trim().strip_prefix("+RSSI:") {
            rssi_str.trim().parse::<i32>().map_err(|e| anyhow!("RSSI parse error: {}", e))
        } else {
            Ok(-120) // Default if parsing fails
        }
    }
    
    fn set_frequency(&mut self, frequency: u64) -> Result<()> {
        let command = format!("AT+FREQ={}\r\n", frequency);
        self.port.write_all(command.as_bytes())?;
        Ok(())
    }
    
    fn set_spreading_factor(&mut self, sf: u8) -> Result<()> {
        let command = format!("AT+SF={}\r\n", sf);
        self.port.write_all(command.as_bytes())?;
        Ok(())
    }
    
    fn set_tx_power(&mut self, power: i8) -> Result<()> {
        let command = format!("AT+POWER={}\r\n", power);
        self.port.write_all(command.as_bytes())?;
        Ok(())
    }
    
    fn get_packet_stats(&self) -> (u64, u64) {
        (self.packet_count, self.packet_loss_count)
    }
}

impl LoRaInterface {
    /// Create a new LoRa interface
    pub fn new(config: LoRaConfig, hardware: LoRaHardware) -> Self {
        LoRaInterface {
            config,
            hardware,
            is_initialized: false,
            #[cfg(feature = "lora")]
            lora_device: None,
            rssi_history: Vec::new(),
            packet_count: 0,
            packet_loss_count: 0,
            last_rssi: None,
            last_snr: None,
        }
    }
    
    /// Initialize the LoRa hardware
    pub async fn initialize(&mut self) -> Result<()> {
        info!("Initializing LoRa interface...");
        
        #[cfg(feature = "lora")]
        {
            match &self.hardware {
                LoRaHardware::Spi { device, cs_pin, reset_pin, dio0_pin } => {
                    info!("Initializing SPI LoRa on device: {}, CS pin: {}", device, cs_pin);
                    
                    // Create SPI device
                    let mut spi = Spidev::open(device)?;
                    let options = SpidevOptions::new()
                        .bits_per_word(8)
                        .max_speed_hz(500_000)
                        .mode(SpiModeFlags::SPI_MODE_0)
                        .build();
                    spi.configure(&options)?;
                    
                    // Create GPIO pins
                    let cs = Pin::new(*cs_pin as u64);
                    cs.export()?;
                    sleep(Duration::from_millis(100)).await;
                    cs.set_direction(Direction::Out)?;
                    cs.set_value(1)?;
                    
                    let reset = if let Some(pin) = reset_pin {
                        let p = Pin::new(pin as u64);
                        p.export()?;
                        sleep(Duration::from_millis(100)).await;
                        p.set_direction(Direction::Out)?;
                        p.set_value(1)?;
                        Some(p)
                    } else {
                        None
                    };
                    
                    let dio0 = if let Some(pin) = dio0_pin {
                        let p = Pin::new(pin as u64);
                        p.export()?;
                        sleep(Duration::from_millis(100)).await;
                        p.set_direction(Direction::In)?;
                        Some(p)
                    } else {
                        None
                    };
                    
                    // Create LoRa device
                    let mut lora = LoRa::new(
                        spi,
                        cs,
                        dio0.unwrap_or_else(|| Pin::new(999)), // Dummy pin if not provided
                        reset.unwrap_or_else(|| Pin::new(999)), // Dummy pin if not provided
                        Delay,
                    )?;
                    
                    // Configure LoRa
                    lora.set_frequency(self.config.frequency)?;
                    lora.set_spreading_factor(self.config.spreading_factor)?;
                    lora.set_bandwidth(self.config.bandwidth)?;
                    lora.set_coding_rate(self.config.coding_rate)?;
                    lora.set_tx_power(self.config.tx_power, 1)?; // PA boost
                    lora.set_preamble_length(self.config.preamble_length)?;
                    lora.set_sync_word(self.config.sync_word)?;
                    
                    if self.config.explicit_header {
                        lora.set_implicit_header_mode()?;
                    } else {
                        lora.set_explicit_header_mode()?;
                    }
                    
                    if self.config.crc_enabled {
                        lora.enable_crc()?;
                    } else {
                        lora.disable_crc()?;
                    }
                    
                    let device = SpiLoRaDevice {
                        lora,
                        packet_count: 0,
                        packet_loss_count: 0,
                    };
                    
                    self.lora_device = Some(Box::new(device));
                }
                LoRaHardware::Serial { port, baud_rate } => {
                    info!("Initializing Serial LoRa on port: {}, baud: {}", port, baud_rate);
                    
                    let serial_port = serialport::new(port, *baud_rate)
                        .timeout(Duration::from_millis(100))
                        .open()?;
                    
                    let device = SerialLoRaDevice {
                        port: serial_port,
                        packet_count: 0,
                        packet_loss_count: 0,
                    };
                    
                    self.lora_device = Some(Box::new(device));
                }
                LoRaHardware::Usb { vid, pid } => {
                    info!("Initializing USB LoRa with VID: {:04x}, PID: {:04x}", vid, pid);
                    // TODO: Implement USB LoRa device
                    return Err(anyhow!("USB LoRa not yet implemented"));
                }
            }
        }
        
        #[cfg(not(feature = "lora"))]
        {
            warn!("LoRa feature not enabled, using simulation mode");
            match &self.hardware {
                LoRaHardware::Spi { device, cs_pin, .. } => {
                    info!("Simulating SPI LoRa on device: {}, CS pin: {}", device, cs_pin);
                }
                LoRaHardware::Serial { port, baud_rate } => {
                    info!("Simulating Serial LoRa on port: {}, baud: {}", port, baud_rate);
                }
                LoRaHardware::Usb { vid, pid } => {
                    info!("Simulating USB LoRa with VID: {:04x}, PID: {:04x}", vid, pid);
                }
            }
        }
        
        self.is_initialized = true;
        info!("LoRa interface initialized successfully");
        Ok(())
    }
    
    /// Send data over LoRa
    pub async fn send(&mut self, data: &[u8]) -> Result<()> {
        if !self.is_initialized {
            return Err(anyhow!("LoRa interface not initialized"));
        }
        
        info!("Sending {} bytes over LoRa", data.len());
        
        #[cfg(feature = "lora")]
        {
            if let Some(device) = &mut self.lora_device {
                device.send(data)?;
            } else {
                return Err(anyhow!("LoRa device not available"));
            }
        }
        
        #[cfg(not(feature = "lora"))]
        {
            // Simulate sending for now
            sleep(Duration::from_millis(10)).await;
            info!("Simulated: Data sent over LoRa");
        }
        
        Ok(())
    }
    
    /// Receive data from LoRa (non-blocking)
    pub async fn receive(&mut self) -> Result<Option<Vec<u8>>> {
        if !self.is_initialized {
            return Err(anyhow!("LoRa interface not initialized"));
        }
        
        #[cfg(feature = "lora")]
        {
            if let Some(device) = &mut self.lora_device {
                match device.receive() {
                    Ok(Some((data, rssi, snr))) => {
                        // Update RSSI history
                        self.update_rssi(rssi);
                        self.last_rssi = Some(rssi);
                        self.last_snr = Some(snr);
                        
                        // Update packet statistics
                        let (total, lost) = device.get_packet_stats();
                        self.packet_count = total;
                        self.packet_loss_count = lost;
                        
                        info!("Received {} bytes via LoRa, RSSI: {} dBm, SNR: {} dB", 
                              data.len(), rssi, snr);
                        return Ok(Some(data));
                    }
                    Ok(None) => return Ok(None),
                    Err(e) => {
                        error!("LoRa receive error: {}", e);
                        return Err(e);
                    }
                }
            }
        }
        
        // Simulation mode or no device
        Ok(None)
    }
    
    /// Get current RSSI value
    pub async fn get_rssi(&mut self) -> Result<i32> {
        if !self.is_initialized {
            return Err(anyhow!("LoRa interface not initialized"));
        }
        
        #[cfg(feature = "lora")]
        {
            if let Some(device) = &mut self.lora_device {
                let rssi = device.get_rssi()?;
                self.update_rssi(rssi);
                self.last_rssi = Some(rssi);
                return Ok(rssi);
            }
        }
        
        // Return simulated RSSI or last known value
        Ok(self.last_rssi.unwrap_or(-85))
    }
    
    /// Get averaged RSSI value
    pub fn get_average_rssi(&self) -> Option<i32> {
        if self.rssi_history.is_empty() {
            return None;
        }
        
        let sum: i32 = self.rssi_history.iter().sum();
        Some(sum / self.rssi_history.len() as i32)
    }
    
    /// Get packet statistics
    pub fn get_packet_stats(&self) -> (u64, u64, f32) {
        let total = self.packet_count;
        let lost = self.packet_loss_count;
        let loss_rate = if total > 0 {
            (lost as f32 / total as f32) * 100.0
        } else {
            0.0
        };
        (total, lost, loss_rate)
    }
    
    /// Get link quality (0-100) based on RSSI and packet loss
    pub fn get_link_quality(&self) -> u8 {
        let avg_rssi = self.get_average_rssi().unwrap_or(-120);
        let (_, _, loss_rate) = self.get_packet_stats();
        
        // Calculate quality based on RSSI (70% weight) and packet loss (30% weight)
        let rssi_quality = if avg_rssi >= -70 {
            100
        } else if avg_rssi >= -85 {
            80
        } else if avg_rssi >= -100 {
            60
        } else if avg_rssi >= -115 {
            40
        } else {
            20
        };
        
        let loss_quality = if loss_rate < 1.0 {
            100
        } else if loss_rate < 5.0 {
            80
        } else if loss_rate < 10.0 {
            60
        } else if loss_rate < 20.0 {
            40
        } else {
            20
        };
        
        ((rssi_quality as f32 * 0.7 + loss_quality as f32 * 0.3) as u8).min(100)
    }
    
    /// Update RSSI history
    fn update_rssi(&mut self, rssi: i32) {
        self.rssi_history.push(rssi);
        if self.rssi_history.len() > self.config.rssi_averaging_window {
            self.rssi_history.remove(0);
        }
    }
    
    /// Set frequency
    pub async fn set_frequency(&mut self, frequency: u64) -> Result<()> {
        self.config.frequency = frequency;
        if self.is_initialized {
            #[cfg(feature = "lora")]
            {
                if let Some(device) = &mut self.lora_device {
                    device.set_frequency(frequency)?;
                }
            }
            info!("Frequency set to {} Hz", frequency);
        }
        Ok(())
    }
    
    /// Set spreading factor
    pub async fn set_spreading_factor(&mut self, sf: u8) -> Result<()> {
        if !(7..=12).contains(&sf) {
            return Err(anyhow!("Spreading factor must be between 7 and 12"));
        }
        self.config.spreading_factor = sf;
        if self.is_initialized {
            #[cfg(feature = "lora")]
            {
                if let Some(device) = &mut self.lora_device {
                    device.set_spreading_factor(sf)?;
                }
            }
            info!("Spreading factor set to SF{}", sf);
        }
        Ok(())
    }
    
    /// Set transmission power
    pub async fn set_tx_power(&mut self, power: i8) -> Result<()> {
        self.config.tx_power = power;
        if self.is_initialized {
            #[cfg(feature = "lora")]
            {
                if let Some(device) = &mut self.lora_device {
                    device.set_tx_power(power)?;
                }
            }
            info!("TX power set to {} dBm", power);
        }
        Ok(())
    }
}

/// LoRa interface manager for handling multiple interfaces
pub struct LoRaManager {
    interfaces: Vec<Arc<Mutex<LoRaInterface>>>,
}

impl LoRaManager {
    pub fn new() -> Self {
        LoRaManager {
            interfaces: Vec::new(),
        }
    }
    
    /// Add a LoRa interface
    pub fn add_interface(&mut self, interface: LoRaInterface) -> Arc<Mutex<LoRaInterface>> {
        let interface = Arc::new(Mutex::new(interface));
        self.interfaces.push(interface.clone());
        interface
    }
    
    /// Get all interfaces
    pub fn get_interfaces(&self) -> &[Arc<Mutex<LoRaInterface>>] {
        &self.interfaces
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_lora_config_default() {
        let config = LoRaConfig::default();
        assert_eq!(config.frequency, 915_000_000);
        assert_eq!(config.bandwidth, 125_000);
        assert_eq!(config.spreading_factor, 7);
        assert_eq!(config.coding_rate, 5);
    }
    
    #[tokio::test]
    async fn test_lora_interface_creation() {
        let config = LoRaConfig::default();
        let hardware = LoRaHardware::Serial {
            port: "/dev/ttyUSB0".to_string(),
            baud_rate: 9600,
        };
        
        let interface = LoRaInterface::new(config, hardware);
        assert!(!interface.is_initialized);
    }
}