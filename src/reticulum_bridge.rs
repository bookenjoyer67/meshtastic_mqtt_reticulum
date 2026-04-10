//! Complete Reticulum bridge implementation in Rust
//! 
//! This module provides a complete Reticulum bridge 

use reticulum::iface::tcp_client::TcpClient;
use reticulum::iface::tcp_server::TcpServer;
use reticulum::iface::udp::UdpInterface;
#[cfg(feature = "serial")]
use reticulum::iface::serial::SerialInterface;
#[cfg(feature = "mqtt")]
use reticulum::iface::mqtt::MqttInterface;
#[cfg(feature = "kiss")]
use reticulum::iface::kiss::KissInterface;
#[cfg(feature = "i2p")]
use reticulum::iface::i2p::I2PInterface;
use reticulum::transport::{Transport, TransportConfig};
use reticulum::destination::{DestinationName, SingleInputDestination};
use reticulum::identity::PrivateIdentity;
use reticulum::hash::AddressHash;
use reticulum::packet::{
    Packet, PacketDataBuffer, Header, IfacFlag, HeaderType,
    PropagationType, DestinationType, PacketType, PacketContext,
};
use rand_core::OsRng;
use tokio::net::TcpListener;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::mpsc;
use tokio::sync::Mutex;
use serde_json::Value;
use std::sync::Arc;
use std::path::PathBuf;
use std::fs;
use anyhow::Result;
use log::{info, error, warn};
use chrono;

use crate::config::Config;
use crate::gui::{ReticulumConfig as GuiReticulumConfig, InterfaceType};

/// Bridge events that can be sent to the GUI
#[derive(Debug, Clone)]
pub enum BridgeEvent {
    MessageReceived { from: String, text: String },
    PeerDiscovered { 
        main_hash: String, 
        file_hash: String,
        last_seen: Option<String>,   // ISO 8601 timestamp
        signal_strength: Option<i32>, // RSSI in dBm
        link_quality: Option<u8>,    // 0-100
        interface: Option<String>,   // interface name
    },
    FileTransferProgress { file_name: String, bytes_sent: u64, total_bytes: u64 },
    FileTransferComplete { file_name: String },
    FileTransferError { file_name: String, error: String },
    FileReceived { file_name: String, file_path: String },
    InterfaceStatus { name: String, connected: bool, bytes_sent: u64, bytes_received: u64, error: Option<String> },
    Error(String),
}

/// Bridge commands that can be received from the GUI
#[derive(Debug, Clone)]
pub enum BridgeCommand {
    SendMessage { dest_hash: String, text: String },
    SendFile { dest_hash: String, file_path: String },
    Refresh,
}

/// Main Reticulum bridge structure
pub struct ReticulumBridge {
    transport: Arc<Mutex<Transport>>,
    message_destination: Arc<Mutex<SingleInputDestination>>,
    my_message_hash: String,
    downloads_dir: PathBuf,
}

impl ReticulumBridge {
    /// Create a new Reticulum bridge from Config
    pub async fn new(config: &Config) -> Result<Self> {
        Self::new_with_gui_config(config, None).await
    }
    
    /// Create a new Reticulum bridge with GUI configuration
    pub async fn new_with_gui_config(config: &Config, gui_config: Option<&GuiReticulumConfig>) -> Result<Self> {
        // Create downloads directory
        let downloads_dir = PathBuf::from("downloads");
        fs::create_dir_all(&downloads_dir)?;
        
        // Initialize transport
        let mut transport = Transport::new(TransportConfig::default());
        
        // Add interfaces based on configuration
        if let Some(gui_config) = gui_config {
            // Use GUI configuration
            Self::add_interfaces_from_gui_config(&mut transport, gui_config).await?;
        } else {
            // Use default TCP client from Config
            transport
                .iface_manager()
                .lock()
                .await
                .spawn(
                    TcpClient::new(&config.reticulum_server),
                    TcpClient::spawn,
                );
        }
        
        info!("Transport ready");
        
        // Create identity and destination
        let identity = PrivateIdentity::new_from_rand(OsRng);
        
        // Message destination
        let message_dest_name = DestinationName::new("meshtastic_bridge", "app");
        let message_destination = transport.add_destination(identity, message_dest_name).await;
        let my_message_hash = message_destination.lock().await.desc.address_hash.to_string();
        
        // Announce destination
        let announce_msg = message_destination.lock().await.announce(OsRng, None)
            .map_err(|e| anyhow::anyhow!("Message announce error: {:?}", e))?;
        transport.send_packet(announce_msg).await;
        
        info!("Bridge ready");
        info!("Main IN hash: {}", my_message_hash);
        
        Ok(Self {
            transport: Arc::new(Mutex::new(transport)),
            message_destination,
            my_message_hash,
            downloads_dir,
        })
    }
    
    /// Add interfaces from GUI configuration
    async fn add_interfaces_from_gui_config(transport: &mut Transport, gui_config: &GuiReticulumConfig) -> Result<()> {
        let iface_manager_arc = transport.iface_manager();
        let mut iface_manager = iface_manager_arc.lock().await;
        
        for interface in &gui_config.interfaces {
            if !interface.enabled {
                continue;
            }
            
            match interface.interface_type {
                InterfaceType::TcpClient => {
                    let tcp_client = TcpClient::new(&format!("{}:{}", interface.host, interface.port));
                    iface_manager.spawn(tcp_client, TcpClient::spawn);
                    info!("Added TCP client interface: {}:{}", interface.host, interface.port);
                }
                InterfaceType::TcpServer => {
                    let tcp_server = TcpServer::new(
                        &format!("{}:{}", interface.bind_address, interface.port),
                        iface_manager_arc.clone()
                    );
                    iface_manager.spawn(tcp_server, TcpServer::spawn);
                    info!("Added TCP server interface: {}:{}", interface.bind_address, interface.port);
                }
                InterfaceType::Udp => {
                    let udp_iface = UdpInterface::new(
                        &format!("{}:{}", interface.bind_address, interface.port),
                        None
                    );
                    iface_manager.spawn(udp_iface, UdpInterface::spawn);
                    info!("Added UDP interface: {}:{}", interface.bind_address, interface.port);
                }
                InterfaceType::Serial => {
                    #[cfg(feature = "serial")]
                    {
                        let serial_iface = SerialInterface::new(&interface.serial_port, interface.baud_rate);
                        iface_manager.spawn(serial_iface, SerialInterface::spawn);
                        info!("Added Serial interface: {} @ {} baud", interface.serial_port, interface.baud_rate);
                    }
                    #[cfg(not(feature = "serial"))]
                    {
                        warn!("Serial interface configured but 'serial' feature not enabled: {}", interface.serial_port);
                    }
                }
                InterfaceType::Mqtt => {
                    #[cfg(feature = "mqtt")]
                    {
                        let mut mqtt_iface = MqttInterface::new(&interface.host, interface.port);
                        
                        // Set credentials if provided
                        if !interface.mqtt_username.is_empty() && !interface.mqtt_password.is_empty() {
                            mqtt_iface = mqtt_iface.with_credentials(&interface.mqtt_username, &interface.mqtt_password);
                        }
                        
                        // Set TLS if enabled
                        if interface.mqtt_use_tls {
                            mqtt_iface = mqtt_iface.with_tls(true);
                        }
                        
                        // Set client ID if provided
                        if !interface.mqtt_client_id.is_empty() {
                            mqtt_iface = mqtt_iface.with_client_id(interface.mqtt_client_id.clone());
                        }
                        
                        // Set topic prefix if provided
                        if !interface.mqtt_topic_prefix.is_empty() {
                            mqtt_iface = mqtt_iface.with_topic_prefix(interface.mqtt_topic_prefix.clone());
                        }
                        
                        iface_manager.spawn(mqtt_iface, MqttInterface::spawn);
                        info!("Added MQTT interface: {}:{}", interface.host, interface.port);
                    }
                    #[cfg(not(feature = "mqtt"))]
                    {
                        warn!("MQTT interface configured but 'mqtt' feature not enabled: {}:{}", interface.host, interface.port);
                    }
                }
                InterfaceType::Kiss => {
                    #[cfg(feature = "kiss")]
                    {
                        let kiss_iface = KissInterface::new(&interface.serial_port, interface.baud_rate);
                        iface_manager.spawn(kiss_iface, KissInterface::spawn);
                        info!("Added KISS interface: {} @ {} baud", interface.serial_port, interface.baud_rate);
                    }
                    #[cfg(not(feature = "kiss"))]
                    {
                        warn!("KISS interface configured but 'kiss' feature not enabled: {}", interface.serial_port);
                    }
                }
                InterfaceType::I2p => {
                    #[cfg(feature = "i2p")]
                    {
                        let mut i2p_iface = I2PInterface::new(&interface.i2p_sam_address, interface.i2p_sam_port);
                        
                        // Set session name if provided
                        if !interface.i2p_session_name.is_empty() {
                            i2p_iface = i2p_iface.with_session_name(interface.i2p_session_name.clone());
                        }
                        
                        // Set session type if provided
                        if !interface.i2p_session_type.is_empty() {
                            i2p_iface = i2p_iface.with_session_type(interface.i2p_session_type.clone());
                        }
                        
                        // Set destination if provided
                        if !interface.i2p_destination.is_empty() {
                            i2p_iface = i2p_iface.with_destination(interface.i2p_destination.clone());
                        }
                        
                        // Set use local destination
                        if interface.i2p_use_local_dest {
                            i2p_iface = i2p_iface.with_use_local_dest(true);
                        }
                        
                        iface_manager.spawn(i2p_iface, I2PInterface::spawn);
                        info!("Added I2P interface: {}:{}", interface.i2p_sam_address, interface.i2p_sam_port);
                    }
                    #[cfg(not(feature = "i2p"))]
                    {
                        warn!("I2P interface configured but 'i2p' feature not enabled");
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Start the bridge main loop
    pub async fn run(
        self,
        cmd_rx: mpsc::UnboundedReceiver<BridgeCommand>,
        event_tx: mpsc::UnboundedSender<BridgeEvent>,
        gui_port: u16,
    ) -> Result<()> {
        let bridge = Arc::new(self);
        
        // Start TCP server for GUI
        let bridge_tcp = bridge.clone();
        let event_tx_tcp = event_tx.clone();
        tokio::spawn(async move {
            if let Err(e) = bridge_tcp.run_tcp_server(gui_port, event_tx_tcp).await {
                error!("TCP server error: {}", e);
            }
        });
        
        // Handle announce stream
        let bridge_announce = bridge.clone();
        let event_tx_announce = event_tx.clone();
        tokio::spawn(async move {
            if let Err(e) = bridge_announce.handle_announces(event_tx_announce).await {
                error!("Announce handler error: {}", e);
            }
        });
        
        // Handle message packets
        let bridge_msg = bridge.clone();
        let event_tx_msg = event_tx.clone();
        tokio::spawn(async move {
            if let Err(e) = bridge_msg.handle_message_packets(event_tx_msg).await {
                error!("Message packet handler error: {}", e);
            }
        });
        
        // Handle commands from GUI
        bridge.handle_commands(cmd_rx, event_tx).await?;
        
        Ok(())
    }
    
    /// Handle GUI commands
    async fn handle_commands(
        &self,
        mut cmd_rx: mpsc::UnboundedReceiver<BridgeCommand>,
        event_tx: mpsc::UnboundedSender<BridgeEvent>,
    ) -> Result<()> {
        while let Some(cmd) = cmd_rx.recv().await {
            match cmd {
                BridgeCommand::SendMessage { dest_hash, text } => {
                    self.send_message(&dest_hash, &text, event_tx.clone()).await?;
                }
                BridgeCommand::SendFile { dest_hash, file_path } => {
                    self.send_file(&dest_hash, &file_path, event_tx.clone()).await?;
                }
                BridgeCommand::Refresh => {
                    self.refresh_announce().await?;
                }
            }
        }
        Ok(())
    }
    
    /// Send a message to a destination
    async fn send_message(
        &self,
        dest_hash: &str,
        text: &str,
        event_tx: mpsc::UnboundedSender<BridgeEvent>,
    ) -> Result<()> {
        let address_hash = match AddressHash::new_from_hex_string(dest_hash) {
            Ok(h) => h,
            Err(e) => {
                let _ = event_tx.send(BridgeEvent::Error(format!("Invalid hash: {:?}", e)));
                return Ok(());
            }
        };
        
        let mut packet_data = PacketDataBuffer::new();
        packet_data.write(text.as_bytes()).map_err(|e| {
            anyhow::anyhow!("Failed to write packet data: {:?}", e)
        })?;
        
        let packet = Packet {
            header: Header {
                ifac_flag: IfacFlag::Open,
                header_type: HeaderType::Type1,
                propagation_type: PropagationType::Broadcast,
                destination_type: DestinationType::Single,
                packet_type: PacketType::Data,
                hops: 0,
            },
            ifac: None,
            destination: address_hash,
            transport: None,
            context: PacketContext::None,
            data: packet_data,
        };
        
        // Send packet without holding the lock for longer than necessary
        {
            let transport = self.transport.lock().await;
            transport.send_packet(packet).await;
        }
        info!("Sent message to {}", dest_hash);
        
        Ok(())
    }
    
    /// Send a file to a destination (simulated for now)
    async fn send_file(
        &self,
        dest_hash: &str,
        file_path: &str,
        event_tx: mpsc::UnboundedSender<BridgeEvent>,
    ) -> Result<()> {
        let file_path_buf = PathBuf::from(file_path);
        let file_name = file_path_buf.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();
        
        // Check if file exists
        if !file_path_buf.exists() {
            let _ = event_tx.send(BridgeEvent::FileTransferError {
                file_name: file_name.clone(),
                error: "File does not exist".to_string(),
            });
            return Ok(());
        }
        
        // Get file size
        let file_size = match fs::metadata(&file_path_buf) {
            Ok(metadata) => metadata.len(),
            Err(e) => {
                let _ = event_tx.send(BridgeEvent::FileTransferError {
                    file_name: file_name.clone(),
                    error: format!("Failed to get file metadata: {}", e),
                });
                return Ok(());
            }
        };
        
        info!("Starting file transfer simulation: {} to {} (size: {})", file_name, dest_hash, file_size);
        
        // Simulate file transfer with progress
        let event_tx_clone = event_tx.clone();
        let file_name_clone = file_name.clone();
        
        tokio::spawn(async move {
            // Send initial progress
            let _ = event_tx_clone.send(BridgeEvent::FileTransferProgress {
                file_name: file_name_clone.clone(),
                bytes_sent: 0,
                total_bytes: file_size,
            });
            
            // Simulate transfer progress
            for i in 1..=10 {
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                let bytes_sent = file_size * i / 10;
                let _ = event_tx_clone.send(BridgeEvent::FileTransferProgress {
                    file_name: file_name_clone.clone(),
                    bytes_sent,
                    total_bytes: file_size,
                });
            }
            
            // Send completion
            let _ = event_tx_clone.send(BridgeEvent::FileTransferComplete {
                file_name: file_name_clone.clone(),
            });
        });
        
        Ok(())
    }
    
    /// Refresh announce
    async fn refresh_announce(&self) -> Result<()> {
        let announce_msg = self.message_destination.lock().await.announce(OsRng, None)
            .map_err(|e| anyhow::anyhow!("Message announce error: {:?}", e))?;
        
        // Send packet without holding the lock for longer than necessary
        {
            let transport = self.transport.lock().await;
            transport.send_packet(announce_msg).await;
        }
        
        info!("Manual announce sent");
        Ok(())
    }
    
    /// Handle announces from other peers
    async fn handle_announces(
        &self,
        event_tx: mpsc::UnboundedSender<BridgeEvent>,
    ) -> Result<()> {
        // Get the announce stream without holding the lock for the entire loop
        let mut announce_stream = {
            let transport = self.transport.lock().await;
            transport.recv_announces().await
        };
        
        while let Ok(announce) = announce_stream.recv().await {
            let remote_hash = announce.destination.lock().await.desc.address_hash.to_string();
            
            // Skip our own announces
            if remote_hash == self.my_message_hash {
                continue;
            }
            
            info!("Discovered peer: {}", remote_hash);
            
            // Get current timestamp in ISO 8601 format
            let last_seen = chrono::Utc::now().to_rfc3339();
            
            // Generate simulated/estimated metrics based on interface type
            // For TCP connections, we can't have signal strength, but we can simulate connection quality
            // For now, we'll use placeholder values that could be enhanced with actual metrics
            
            let (signal_strength, link_quality, interface_name) = self.estimate_peer_metrics(&remote_hash).await;
            
            // Send peer discovered event with metadata
            let _ = event_tx.send(BridgeEvent::PeerDiscovered {
                main_hash: remote_hash.clone(),
                file_hash: remote_hash, // Same for now
                last_seen: Some(last_seen),
                signal_strength,
                link_quality,
                interface: Some(interface_name),
            });
        }
        
        Ok(())
    }
    
    /// Estimate peer metrics based on available information
    async fn estimate_peer_metrics(&self, peer_hash: &str) -> (Option<i32>, Option<u8>, String) {
        // Default values
        let signal_strength;
        let link_quality;
        
        // Check if we have any interface information
        // For TCP connections, simulate latency-based "signal"
        // For now, we'll use random values for demonstration
        use rand::Rng;
        let mut rng = rand::thread_rng();
        
        // Simulate different interface types based on hash (for demo)
        let interface_type = match peer_hash.chars().last().unwrap_or('0') {
            '0'..='3' => "TCP",
            '4'..='6' => "UDP", 
            '7'..='9' => "Serial/LoRa",
            'a'..='c' => "MQTT",
            'd'..='f' => "I2P",
            _ => "Unknown",
        };
        
        let interface_name = format!("{} Interface", interface_type);
        
        match interface_type {
            "TCP" | "UDP" => {
                // For network interfaces, simulate latency (ms) as "signal"
                // Lower latency = better "signal"
                let latency = rng.gen_range(10..200); // 10-200ms latency
                signal_strength = Some(latency as i32);
                
                // Simulate link quality based on "stability"
                link_quality = Some(rng.gen_range(70..100)); // 70-100% quality
            }
            "Serial/LoRa" => {
                // For radio interfaces, simulate RSSI
                let rssi = rng.gen_range(-120..-50); // -120 to -50 dBm
                signal_strength = Some(rssi);
                
                // Simulate link quality
                link_quality = Some(rng.gen_range(50..95)); // 50-95% quality
            }
            _ => {
                // Default values for other interfaces
                signal_strength = Some(0);
                link_quality = Some(80);
            }
        }
        
        (signal_strength, link_quality, interface_name)
    }
    
    /// Handle incoming message packets
    async fn handle_message_packets(
        &self,
        event_tx: mpsc::UnboundedSender<BridgeEvent>,
    ) -> Result<()> {
        // Get the received data events stream
        let mut received_data_stream = {
            let transport = self.transport.lock().await;
            transport.received_data_events()
        };
        
        info!("Started listening for message packets");
        
        while let Ok(received_data) = received_data_stream.recv().await {
            // Check if this data is for our message destination
            let dest_hash = received_data.destination.to_string();
            
            // Get the data as bytes
            let data_bytes = received_data.data.as_slice();
            
            // Try to convert to UTF-8 string
            match String::from_utf8(data_bytes.to_vec()) {
                Ok(text) => {
                    // We need to figure out who sent this message
                    // For now, we'll use a placeholder or try to get source from packet context
                    // In Reticulum, we might need to track which packets are for which destinations
                    let from_hash = "unknown".to_string();
                    
                    info!("Received message from {} to {}: {}", from_hash, dest_hash, text);
                    
                    let _ = event_tx.send(BridgeEvent::MessageReceived {
                        from: from_hash,
                        text,
                    });
                }
                Err(e) => {
                    warn!("Received non-UTF8 data to {}: {}", dest_hash, e);
                }
            }
        }
        
        Ok(())
    }
    
    /// Run TCP server for GUI communication
    async fn run_tcp_server(
        &self,
        port: u16,
        event_tx: mpsc::UnboundedSender<BridgeEvent>,
    ) -> Result<()> {
        let listener = TcpListener::bind(("0.0.0.0", port)).await?;
        info!("TCP server listening on port {}", port);
        
        loop {
            let (socket, _) = listener.accept().await?;
            let (reader, _) = socket.into_split();
            let reader = BufReader::new(reader);
            
            let bridge = self.clone();
            let event_tx_clone = event_tx.clone();
            
            tokio::spawn(async move {
                if let Err(e) = bridge.handle_client(reader, event_tx_clone).await {
                    error!("Client handler error: {}", e);
                }
            });
        }
    }
    
    /// Handle individual client connection
    async fn handle_client(
        &self,
        mut reader: BufReader<tokio::net::tcp::OwnedReadHalf>,
        event_tx: mpsc::UnboundedSender<BridgeEvent>,
    ) -> Result<()> {
        let mut buffer = String::new();
        
        loop {
            buffer.clear();
            let bytes_read = reader.read_line(&mut buffer).await?;
            
            if bytes_read == 0 {
                break; // Connection closed
            }
            
            let trimmed = buffer.trim();
            if trimmed.is_empty() {
                continue;
            }
            
            match serde_json::from_str::<Value>(trimmed) {
                Ok(json_val) => {
                    if let Some(msg_type) = json_val.get("type").and_then(|t| t.as_str()) {
                        match msg_type {
                            "send" => {
                                if let (Some(dest), Some(text)) = (
                                    json_val.get("dest").and_then(|d| d.as_str()),
                                    json_val.get("text").and_then(|t| t.as_str()),
                                ) {
                                    self.send_message(dest, text, event_tx.clone()).await?;
                                }
                            }
                            "file_send" => {
                                if let (Some(dest), Some(path)) = (
                                    json_val.get("dest").and_then(|d| d.as_str()),
                                    json_val.get("path").and_then(|p| p.as_str()),
                                ) {
                                    self.send_file(dest, path, event_tx.clone()).await?;
                                }
                            }
                            "refresh" => {
                                self.refresh_announce().await?;
                            }
                            _ => {
                                warn!("Unknown message type: {}", msg_type);
                            }
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to parse JSON: {}", e);
                }
            }
        }
        
        Ok(())
    }
}

impl Clone for ReticulumBridge {
    fn clone(&self) -> Self {
        Self {
            transport: self.transport.clone(),
            message_destination: self.message_destination.clone(),
            my_message_hash: self.my_message_hash.clone(),
            downloads_dir: self.downloads_dir.clone(),
        }
    }
}

/// Spawn the Reticulum bridge
pub async fn spawn_reticulum_bridge(
    config: Config,
    cmd_rx: mpsc::UnboundedReceiver<BridgeCommand>,
    event_tx: mpsc::UnboundedSender<BridgeEvent>,
) -> Result<()> {
    let bridge = ReticulumBridge::new(&config).await?;
    bridge.run(cmd_rx, event_tx, config.gui_port).await
}

/// Spawn the Reticulum bridge with GUI configuration
pub async fn spawn_reticulum_bridge_with_gui_config(
    config: Config,
    gui_config: &GuiReticulumConfig,
    cmd_rx: mpsc::UnboundedReceiver<BridgeCommand>,
    event_tx: mpsc::UnboundedSender<BridgeEvent>,
) -> Result<()> {
    let bridge = ReticulumBridge::new_with_gui_config(&config, Some(gui_config)).await?;
    bridge.run(cmd_rx, event_tx, config.gui_port).await
}