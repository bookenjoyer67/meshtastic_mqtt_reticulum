//! LXMF (LXMF Messaging Framework) support for Meshtastic-Reticulum bridge
//!
//! This module provides LXMF protocol support, enabling secure messaging
//! compatible with Sideband and other LXMF clients.

use lxmf_rs::{LxmfClient, LxmfMessage, Destination};
use reticulum::identity::PrivateIdentity;
use reticulum::hash::AddressHash;
use tokio::sync::mpsc;
use anyhow::Result;
use log::{info, warn, error};
use serde::{Serialize, Deserialize};
use std::sync::Arc;
use std::path::PathBuf;
use std::fs;

/// LXMF client configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LxmfConfig {
    /// Enable LXMF messaging
    pub enabled: bool,
    /// Path to store LXMF messages and state
    pub storage_path: String,
    /// Enable LXMF propagation nodes
    pub enable_propagation: bool,
    /// Propagation node addresses (if any)
    pub propagation_nodes: Vec<String>,
    /// Maximum message size in bytes
    pub max_message_size: usize,
    /// Message retention time in seconds
    pub message_retention: u64,
}

impl Default for LxmfConfig {
    fn default() -> Self {
        LxmfConfig {
            enabled: true,
            storage_path: "~/.lxmf".to_string(),
            enable_propagation: false,
            propagation_nodes: Vec::new(),
            max_message_size: 1024 * 1024, // 1 MB
            message_retention: 30 * 24 * 60 * 60, // 30 days
        }
    }
}

/// LXMF message types
#[derive(Debug, Clone)]
pub enum LxmfMessageType {
    Text(String),
    Image(Vec<u8>),
    Audio(Vec<u8>),
    File(String, Vec<u8>), // filename, data
    Location(f64, f64, Option<f32>), // lat, lon, alt
    Telemetry(TelemetryData),
    Command(String, Vec<String>), // command, args
}

/// Telemetry data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryData {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub location: Option<(f64, f64, Option<f32>)>, // lat, lon, alt
    pub battery_level: Option<f32>, // 0.0-1.0
    pub temperature: Option<f32>, // Celsius
    pub humidity: Option<f32>, // 0.0-1.0
    pub pressure: Option<f32>, // hPa
    pub signal_strength: Option<i32>, // RSSI in dBm
    pub link_quality: Option<u8>, // 0-100
    pub custom_data: serde_json::Value,
}

/// LXMF client wrapper
pub struct LxmfClientWrapper {
    client: LxmfClient,
    config: LxmfConfig,
    message_tx: mpsc::UnboundedSender<LxmfMessageEvent>,
    command_tx: mpsc::UnboundedSender<LxmfCommand>,
}

/// Events from LXMF client
#[derive(Debug, Clone)]
pub enum LxmfMessageEvent {
    MessageReceived {
        from: String,
        message_type: LxmfMessageType,
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    DeliveryStatus {
        message_id: String,
        delivered: bool,
        error: Option<String>,
    },
    PeerDiscovered {
        identity_hash: String,
        name: Option<String>,
        last_seen: chrono::DateTime<chrono::Utc>,
    },
    Error(String),
}

/// Commands to LXMF client
#[derive(Debug, Clone)]
pub enum LxmfCommand {
    SendMessage {
        to: String,
        message: LxmfMessageType,
    },
    RequestTelemetry {
        from: String,
    },
    SendCommand {
        to: String,
        command: String,
        args: Vec<String>,
    },
    GetMessages {
        limit: Option<usize>,
    },
    DeleteMessage {
        message_id: String,
    },
}

impl LxmfClientWrapper {
    /// Create a new LXMF client
    pub async fn new(config: LxmfConfig) -> Result<Self> {
        // Expand storage path
        let storage_path = shellexpand::full(&config.storage_path)?.to_string();
        let storage_path = PathBuf::from(storage_path);
        
        // Create storage directory
        fs::create_dir_all(&storage_path)?;
        
        // Initialize LXMF client
        let client = LxmfClient::new(storage_path).await?;
        
        // Create channels
        let (message_tx, _) = mpsc::unbounded_channel();
        let (command_tx, _) = mpsc::unbounded_channel();
        
        Ok(Self {
            client,
            config,
            message_tx,
            command_tx,
        })
    }
    
    /// Start the LXMF client
    pub async fn start(&mut self) -> Result<()> {
        info!("Starting LXMF client");
        
        // Configure propagation nodes if enabled
        if self.config.enable_propagation && !self.config.propagation_nodes.is_empty() {
            for node in &self.config.propagation_nodes {
                // TODO: Add propagation node
                info!("Adding propagation node: {}", node);
            }
        }
        
        // Start message processing loop
        self.process_messages().await?;
        
        Ok(())
    }
    
    /// Process incoming messages
    async fn process_messages(&mut self) -> Result<()> {
        // TODO: Implement message processing loop
        // This should listen for incoming LXMF messages and convert them to events
        
        Ok(())
    }
    
    /// Send a message via LXMF
    pub async fn send_message(&self, to: &str, message: LxmfMessageType) -> Result<String> {
        // Convert our message type to LXMF message
        let lxmf_message = self.convert_to_lxmf_message(message).await?;
        
        // Get destination hash
        let dest_hash = AddressHash::from_hex(to)?;
        
        // Send message
        let message_id = self.client.send_message(&dest_hash, lxmf_message).await?;
        
        info!("Sent LXMF message to {}: {}", to, message_id);
        Ok(message_id)
    }
    
    /// Convert our message type to LXMF message
    async fn convert_to_lxmf_message(&self, message: LxmfMessageType) -> Result<LxmfMessage> {
        match message {
            LxmfMessageType::Text(text) => {
                Ok(LxmfMessage::text(text))
            }
            LxmfMessageType::Image(data) => {
                // For images, we need to handle as binary data
                // LXMF supports attachments
                let mut msg = LxmfMessage::text("[Image]");
                msg.add_attachment("image.jpg", data)?;
                Ok(msg)
            }
            LxmfMessageType::Audio(data) => {
                let mut msg = LxmfMessage::text("[Audio]");
                msg.add_attachment("audio.opus", data)?;
                Ok(msg)
            }
            LxmfMessageType::File(filename, data) => {
                let mut msg = LxmfMessage::text(format!("[File: {}]", filename));
                msg.add_attachment(&filename, data)?;
                Ok(msg)
            }
            LxmfMessageType::Location(lat, lon, alt) => {
                let text = if let Some(alt) = alt {
                    format!("Location: {:.6}, {:.6}, {:.1}m", lat, lon, alt)
                } else {
                    format!("Location: {:.6}, {:.6}", lat, lon)
                };
                Ok(LxmfMessage::text(text))
            }
            LxmfMessageType::Telemetry(telemetry) => {
                let text = serde_json::to_string(&telemetry)?;
                Ok(LxmfMessage::text(text))
            }
            LxmfMessageType::Command(cmd, args) => {
                let text = format!("!{} {}", cmd, args.join(" "));
                Ok(LxmfMessage::text(text))
            }
        }
    }
    
    /// Convert LXMF message to our message type
    fn convert_from_lxmf_message(&self, msg: LxmfMessage) -> Result<LxmfMessageType> {
        let text = msg.content().to_string();
        
        // Check for special message types
        if text.starts_with("[Image]") && msg.has_attachments() {
            // Extract image from attachments
            if let Some((_, data)) = msg.attachments().iter().next() {
                return Ok(LxmfMessageType::Image(data.clone()));
            }
        } else if text.starts_with("[Audio]") && msg.has_attachments() {
            if let Some((_, data)) = msg.attachments().iter().next() {
                return Ok(LxmfMessageType::Audio(data.clone()));
            }
        } else if text.starts_with("[File:") && msg.has_attachments() {
            if let Some((filename, data)) = msg.attachments().iter().next() {
                return Ok(LxmfMessageType::File(filename.clone(), data.clone()));
            }
        } else if text.starts_with("Location:") {
            // Parse location
            let parts: Vec<&str> = text.split(": ").collect();
            if parts.len() > 1 {
                let coords: Vec<&str> = parts[1].split(", ").collect();
                if coords.len() >= 2 {
                    let lat = coords[0].parse::<f64>().unwrap_or(0.0);
                    let lon = coords[1].parse::<f64>().unwrap_or(0.0);
                    let alt = if coords.len() > 2 {
                        coords[2].trim_end_matches('m').parse::<f32>().ok()
                    } else {
                        None
                    };
                    return Ok(LxmfMessageType::Location(lat, lon, alt));
                }
            }
        } else if text.starts_with('!') {
            // Parse command
            let parts: Vec<&str> = text.split_whitespace().collect();
            if !parts.is_empty() {
                let cmd = parts[0][1..].to_string(); // Remove '!'
                let args = parts[1..].iter().map(|s| s.to_string()).collect();
                return Ok(LxmfMessageType::Command(cmd, args));
            }
        } else if text.trim_start().starts_with('{') {
            // Try to parse as telemetry JSON
            if let Ok(telemetry) = serde_json::from_str::<TelemetryData>(&text) {
                return Ok(LxmfMessageType::Telemetry(telemetry));
            }
        }
        
        // Default to text message
        Ok(LxmfMessageType::Text(text))
    }
    
    /// Get message sender for sending commands
    pub fn get_command_sender(&self) -> mpsc::UnboundedSender<LxmfCommand> {
        self.command_tx.clone()
    }
    
    /// Get message receiver for receiving events
    pub fn get_message_receiver(&self) -> mpsc::UnboundedReceiver<LxmfMessageEvent> {
        // Create a new receiver (in real implementation, this would share with the client)
        let (_, rx) = mpsc::unbounded_channel();
        rx
    }
}

/// Bridge between LXMF and Meshtastic
pub struct LxmfMeshtasticBridge {
    lxmf_client: LxmfClientWrapper,
    mqtt_tx: mpsc::UnboundedSender<crate::mqtt::GuiToMqtt>,
    reticulum_tx: mpsc::UnboundedSender<crate::reticulum_bridge::BridgeCommand>,
}

impl LxmfMeshtasticBridge {
    /// Create a new bridge
    pub async fn new(
        config: LxmfConfig,
        mqtt_tx: mpsc::UnboundedSender<crate::mqtt::GuiToMqtt>,
        reticulum_tx: mpsc::UnboundedSender<crate::reticulum_bridge::BridgeCommand>,
    ) -> Result<Self> {
        let lxmf_client = LxmfClientWrapper::new(config).await?;
        
        Ok(Self {
            lxmf_client,
            mqtt_tx,
            reticulum_tx,
        })
    }
    
    /// Start the bridge
    pub async fn start(&mut self) -> Result<()> {
        // Start LXMF client
        self.lxmf_client.start().await?;
        
        // Start bridge processing
        self.process_bridge().await?;
        
        Ok(())
    }
    
    /// Process bridge messages between LXMF, Meshtastic, and Reticulum
    async fn process_bridge(&mut self) -> Result<()> {
        // TODO: Implement bridge processing
        // This should:
        // 1. Listen for LXMF messages and forward to Meshtastic/Reticulum
        // 2. Listen for Meshtastic messages and forward to LXMF
        // 3. Listen for Reticulum messages and forward to LXMF
        
        Ok(())
    }
    
    /// Forward LXMF message to Meshtastic
    async fn forward_to_meshtastic(&self, event: LxmfMessageEvent) -> Result<()> {
        match event {
            LxmfMessageEvent::MessageReceived { from, message_type, timestamp } => {
                match message_type {
                    LxmfMessageType::Text(text) => {
                        // Forward as regular message
                        let _ = self.mqtt_tx.send(crate::mqtt::GuiToMqtt::SendMessage {
                            channel: "lxmf".to_string(),
                            text: format!("[LXMF from {}]: {}", from, text),
                        });
                    }
                    LxmfMessageType::Location(lat, lon, alt) => {
                        // Forward as location message
                        let text = if let Some(alt) = alt {
                            format!("Location: {:.6}, {:.6}, {:.1}m", lat, lon, alt)
                        } else {
                            format!("Location: {:.6}, {:.6}", lat, lon)
                        };
                        let _ = self.mqtt_tx.send(crate::mqtt::GuiToMqtt::SendMessage {
                            channel: "lxmf".to_string(),
                            text: format!("[LXMF from {}]: {}", from, text),
                        });
                    }
                    _ => {
                        // Other message types need special handling
                        info!("Received LXMF message type that needs special handling: {:?}", message_type);
                    }
                }
            }
            _ => {}
        }
        
        Ok(())
    }
    
    /// Forward Meshtastic message to LXMF
    pub async fn forward_from_meshtastic(&self, channel: &str, text: &str, sender: Option<&str>) -> Result<()> {
        // Only forward from specific channels if needed
        if channel != "lxmf" && channel != "all" {
            return Ok(());
        }
        
        // Convert to LXMF message
        let message_type = LxmfMessageType::Text(text.to_string());
        
        // TODO: Determine destination based on sender or channel
        // For now, we'll need a mapping of Meshtastic users to LXMF identities
        
        info!("Forwarding Meshtastic message to LXMF: {}", text);
        
        Ok(())
    }
}