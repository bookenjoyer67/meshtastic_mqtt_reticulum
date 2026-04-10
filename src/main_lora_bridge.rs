//! LoRa-enabled bridge for Meshtastic Reticulum
//!
//! This extends the existing bridge with direct LoRa hardware support.

use meshtastic_reticulum_bridge::lora_interface::{LoRaConfig, LoRaHardware, LoRaInterface, LoRaManager};
use meshtastic_reticulum_bridge::config;
use meshtastic_reticulum_bridge::rate_limit::RateLimiter;
use meshtastic_reticulum_bridge::structured_logging::{self, LogHelper};
use reticulum::iface::tcp_client::TcpClient;
use reticulum::transport::{Transport, TransportConfig};
use reticulum::hash::AddressHash;
use reticulum::destination::DestinationName;
use reticulum::identity::PrivateIdentity;
use reticulum::packet::{
    Packet, PacketDataBuffer, Header, IfacFlag, HeaderType,
    PropagationType, DestinationType, PacketType, PacketContext,
};
use rand_core::OsRng;
use tokio::net::TcpListener;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::sync::mpsc;
use serde_json::{json, Value};
use std::sync::Arc;
use tokio::sync::Mutex;
use anyhow::Result;
use std::path::PathBuf;
use tokio::time::{sleep, Duration};
use chrono;

#[tokio::main]
async fn main() -> Result<()> {
    // Load configuration
    let config = config::load_config();
    
    // Initialize structured logging
    let logger = structured_logging::init_structured_logging(
        config.log_to_console,
        config.log_to_file,
        Some(PathBuf::from(&config.log_file_path)),
    );
    
    let log_helper = LogHelper::new(logger.clone(), "lora_bridge");
    
    // Log startup
    log_helper.info("main", "Starting Meshtastic Reticulum Bridge with LoRa support").await;
    log_helper.info("config", &format!("Loaded configuration: MQTT host={}, Reticulum server={}", 
        config.mqtt_host, config.reticulum_server)).await;
    
    // Validate configuration
    if let Err(errors) = config.validate() {
        for error in errors {
            log_helper.error("config", &error).await;
        }
        return Err(anyhow::anyhow!("Configuration validation failed"));
    }

    // Create LoRa manager
    let mut lora_manager = LoRaManager::new();
    let mut lora_enabled;
    let mut lora_interface_handle: Option<Arc<Mutex<LoRaInterface>>> = None;
    
    // Check if LoRa is configured
    lora_enabled = std::env::var("LORA_ENABLED").unwrap_or("false".to_string()) == "true";
    
    if lora_enabled {
        log_helper.info("lora", "LoRa support is enabled").await;
        
        // Configure LoRa interface from environment or config
        let lora_frequency = std::env::var("LORA_FREQUENCY")
            .unwrap_or("915000000".to_string())
            .parse::<u64>()
            .unwrap_or(915_000_000);
        
        let lora_type = std::env::var("LORA_TYPE").unwrap_or("serial".to_string());
        
        match lora_type.as_str() {
            "serial" => {
                let port = std::env::var("LORA_SERIAL_PORT").unwrap_or("/dev/ttyUSB0".to_string());
                let baud_rate = std::env::var("LORA_SERIAL_BAUD")
                    .unwrap_or("9600".to_string())
                    .parse::<u32>()
                    .unwrap_or(9600);
                
                log_helper.info("lora", &format!("Creating serial LoRa interface on {} at {} baud", port, baud_rate)).await;
                
                let lora_config = LoRaConfig {
                    frequency: lora_frequency,
                    ..Default::default()
                };
                
                let lora_hardware = LoRaHardware::Serial {
                    port,
                    baud_rate,
                };
                
                let lora_interface = LoRaInterface::new(lora_config, lora_hardware);
                let lora_handle = lora_manager.add_interface(lora_interface);
                lora_interface_handle = Some(lora_handle.clone());
                
                // Initialize LoRa interface
                let init_result = {
                    let mut lora_interface = lora_handle.lock().await;
                    lora_interface.initialize().await
                };
                match init_result {
                    Ok(_) => log_helper.info("lora", "LoRa interface initialized successfully").await,
                    Err(e) => {
                        log_helper.error("lora", &format!("Failed to initialize LoRa interface: {}", e)).await;
                        log_helper.warn("lora", "Continuing without LoRa support...").await;
                        lora_enabled = false;
                    }
                }
            }
            "spi" => {
                let device = std::env::var("LORA_SPI_DEVICE").unwrap_or("/dev/spidev0.0".to_string());
                let cs_pin = std::env::var("LORA_SPI_CS_PIN")
                    .unwrap_or("8".to_string())
                    .parse::<u8>()
                    .unwrap_or(8);
                
                log_helper.info("lora", &format!("Creating SPI LoRa interface on {} with CS pin {}", device, cs_pin)).await;
                
                let lora_config = LoRaConfig {
                    frequency: lora_frequency,
                    ..Default::default()
                };
                
                let lora_hardware = LoRaHardware::Spi {
                    device,
                    cs_pin,
                    reset_pin: None,
                    dio0_pin: None,
                };
                
                let lora_interface = LoRaInterface::new(lora_config, lora_hardware);
                let lora_handle = lora_manager.add_interface(lora_interface);
                lora_interface_handle = Some(lora_handle.clone());
                
                // Initialize LoRa interface
                let init_result = {
                    let mut lora_interface = lora_handle.lock().await;
                    lora_interface.initialize().await
                };
                match init_result {
                    Ok(_) => log_helper.info("lora", "LoRa interface initialized successfully").await,
                    Err(e) => {
                        log_helper.error("lora", &format!("Failed to initialize LoRa interface: {}", e)).await;
                        log_helper.warn("lora", "Continuing without LoRa support...").await;
                        lora_enabled = false;
                    }
                }
            }
            _ => {
                log_helper.warn("lora", &format!("Unknown LoRa type: {}, skipping LoRa initialization", lora_type)).await;
                lora_enabled = false;
            }
        }
    } else {
        log_helper.info("lora", "LoRa support is disabled (set LORA_ENABLED=true to enable)").await;
    }
    
    // Create Reticulum transport
    let mut transport = Transport::new(TransportConfig::default());
    
    // Use Reticulum server address from configuration
    transport
        .iface_manager()
        .lock()
        .await
        .spawn(
            TcpClient::new(&config.reticulum_server),
            TcpClient::spawn,
        );
    log_helper.info("transport", "Transport ready").await;

    let identity = PrivateIdentity::new_from_rand(OsRng);
    let dest_name = DestinationName::new("meshtastic_bridge_lora", "app");
    let destination = transport.add_destination(identity, dest_name).await;
    let my_hash = destination.lock().await.desc.address_hash;
    log_helper.info("identity", &format!("My hash: {}", my_hash)).await;

    let announce = destination.lock().await.announce(OsRng, None)
        .map_err(|e| anyhow::anyhow!("Announce error: {:?}", e))?;
    transport.send_packet(announce).await;
    log_helper.info("network", "Announced to network").await;

    let transport = Arc::new(Mutex::new(transport));
    let _destination = Arc::new(Mutex::new(destination));

    // Channel to send discovered peer hashes to the GUI
    let (peer_tx, mut peer_rx) = mpsc::unbounded_channel::<String>();

    // Listen for announces (explicit peer announcements)
    let transport_clone = transport.clone();
    let peer_tx_announce = peer_tx.clone();
    let log_helper_announce = log_helper.clone();
    tokio::spawn(async move {
        let mut announce_stream = transport_clone.lock().await.recv_announces().await;
        while let Ok(announce) = announce_stream.recv().await {
            let remote_hash = announce.destination.lock().await.desc.address_hash;
            if remote_hash != my_hash {
                log_helper_announce.info("discovery", &format!("Announce from peer: {}", remote_hash)).await;
                let _ = peer_tx_announce.send(remote_hash.to_string());
            }
        }
    });

    // TCP server for GUI
    let listener = TcpListener::bind(config.gui_bind_addr()).await?;
    log_helper.info("gui", &format!("Listening for GUI on {}", config.gui_bind_addr())).await;

    let (gui_writer, gui_reader) = {
        let (socket, _) = listener.accept().await?;
        let (reader, writer) = socket.into_split();
        let reader = BufReader::new(reader);
        (writer, reader)
    };
    log_helper.info("gui", "GUI connected").await;

    let (to_gui_tx, mut to_gui_rx) = mpsc::unbounded_channel::<Vec<u8>>();
    let mut gui_writer_task = gui_writer;
    tokio::spawn(async move {
        while let Some(data) = to_gui_rx.recv().await {
            let _ = gui_writer_task.write_all(&data).await;
            let _ = gui_writer_task.write_all(b"\n").await;
        }
    });

    // Forward discovered peers to GUI (deduplicate)
    let to_gui_tx_peer = to_gui_tx.clone();
    let mut seen_peers = std::collections::HashSet::new();
    let log_helper_peer = log_helper.clone();
    tokio::spawn(async move {
        while let Some(peer_hash) = peer_rx.recv().await {
            if seen_peers.insert(peer_hash.clone()) {
                log_helper_peer.info("peers", &format!("New peer discovered: {}", peer_hash)).await;
                let msg = json!({
                    "type": "announce",
                    "main_hash": peer_hash,
                    "file_hash": peer_hash,
                    "last_seen": chrono::Utc::now().to_rfc3339(),
                    "signal_strength": null,
                    "link_quality": null,
                    "interface": null
                });
                let _ = to_gui_tx_peer.send(msg.to_string().into_bytes());
            }
        }
    });

    // Create rate limiter for message sending
    let rate_limiter = Arc::new(RateLimiter::new());
    
    // Start LoRa receive task if enabled
    if lora_enabled {
        if let Some(lora_interface_handle_ref) = lora_interface_handle.as_ref() {
            let lora_interface = lora_interface_handle_ref.clone();
            let log_helper_lora = log_helper.clone();
            let to_gui_tx_lora = to_gui_tx.clone();
        
            tokio::spawn(async move {
                log_helper_lora.info("lora", "Starting LoRa receive task...").await;
                
                loop {
                    if let Ok(mut iface) = lora_interface.try_lock() {
                        match iface.receive().await {
                            Ok(Some(data)) => {
                                log_helper_lora.info("lora", &format!("Received {} bytes via LoRa", data.len())).await;
                                
                                // Try to parse as text and forward to GUI
                                if let Ok(text) = String::from_utf8(data.clone()) {
                                    log_helper_lora.info("lora", &format!("LoRa message: {}", text)).await;
                                    // Send to GUI as a special event
                                    let msg = json!({
                                        "type": "lora_receive",
                                        "data": text
                                    });
                                    let _ = to_gui_tx_lora.send(msg.to_string().into_bytes());
                                } else {
                                    // Binary data – send hex encoded
                                    let hex_data = hex::encode(&data);
                                    log_helper_lora.info("lora", &format!("Received binary data: {}", hex_data)).await;
                                    let msg = json!({
                                        "type": "lora_receive",
                                        "data": hex_data,
                                        "binary": true
                                    });
                                    let _ = to_gui_tx_lora.send(msg.to_string().into_bytes());
                                }
                            }
                            Ok(None) => {
                                // No data available
                            }
                            Err(e) => {
                                log_helper_lora.error("lora", &format!("LoRa receive error: {}", e)).await;
                            }
                        }
                    }
                    
                    sleep(Duration::from_millis(100)).await;
                }
            });
        }
    }

    // Handle commands from GUI (send messages)
    let transport_cmd = transport.clone();
    let rate_limiter_cmd = rate_limiter.clone();
    let log_helper_cmd = log_helper.clone();
    let lora_interface_cmd = lora_interface_handle.clone();
    let to_gui_tx_cmd = to_gui_tx.clone();
    let mut lines = gui_reader.lines();
    while let Ok(Some(line)) = lines.next_line().await {
        if let Ok(json_val) = serde_json::from_str::<Value>(&line) {
            // Check for LoRa configuration commands
            if let Some(cmd_type) = json_val.get("type").and_then(|t| t.as_str()) {
                if cmd_type == "lora_config" {
                    // Handle LoRa configuration from GUI
                    if lora_enabled {
                        if let Some(lora_iface) = lora_interface_cmd.as_ref() {
                            let lora_iface = lora_iface.clone();
                            let log_helper_config = log_helper_cmd.clone();
                            let to_gui_tx_config = to_gui_tx_cmd.clone();
                            
                            tokio::spawn(async move {
                                let mut iface = lora_iface.lock().await;
                                if let Some(frequency) = json_val.get("frequency").and_then(|f| f.as_u64()) {
                                    if let Err(e) = iface.set_frequency(frequency).await {
                                        log_helper_config.error("lora_config", &format!("Failed to set frequency: {}", e)).await;
                                    } else {
                                        log_helper_config.info("lora_config", &format!("Frequency set to {} Hz", frequency)).await;
                                    }
                                }
                                
                                if let Some(sf) = json_val.get("spreading_factor").and_then(|sf| sf.as_u64()) {
                                    if let Err(e) = iface.set_spreading_factor(sf as u8).await {
                                        log_helper_config.error("lora_config", &format!("Failed to set spreading factor: {}", e)).await;
                                    } else {
                                        log_helper_config.info("lora_config", &format!("Spreading factor set to SF{}", sf)).await;
                                    }
                                }
                                
                                if let Some(power) = json_val.get("tx_power").and_then(|p| p.as_i64()) {
                                    if let Err(e) = iface.set_tx_power(power as i8).await {
                                        log_helper_config.error("lora_config", &format!("Failed to set TX power: {}", e)).await;
                                    } else {
                                        log_helper_config.info("lora_config", &format!("TX power set to {} dBm", power)).await;
                                    }
                                }
                                
                                // Send acknowledgment back to GUI
                                let ack = json!({"type": "lora_config_ack", "status": "ok"});
                                let _ = to_gui_tx_config.send(ack.to_string().into_bytes());
                            });
                        continue;
                    } else {
                        let err = json!({"type": "error", "text": "LoRa is not enabled or initialized"});
                        let _ = to_gui_tx_cmd.send(err.to_string().into_bytes());
                        continue;
                    }
                }
            } else {
                // Handle message sending (original functionality)
                if let (Some(dest), Some(text)) = (json_val.get("dest").and_then(|d| d.as_str()),
                                                    json_val.get("text").and_then(|t| t.as_str())) {
                    // Validate destination hash format
                    if dest.len() != 64 || !dest.chars().all(|c| c.is_ascii_hexdigit()) {
                        let err = json!({"type": "error", "text": format!("Invalid destination hash format. Must be 64 hex characters, got: {}", dest)});
                        let _ = to_gui_tx_cmd.send(err.to_string().into_bytes());
                        log_helper_cmd.warn("validation", &format!("Invalid destination hash format: {}", dest)).await;
                        continue;
                    }
                
                        // Apply rate limiting
                        match rate_limiter_cmd.check_rate_limit(dest).await {
                            Ok(_) => {
                                // Rate limit check passed, continue with sending
                            }
                            Err(rate_limit_error) => {
                                let err = json!({"type": "error", "text": format!("Rate limit: {}", rate_limit_error)});
                                let _ = to_gui_tx_cmd.send(err.to_string().into_bytes());
                                log_helper_cmd.rate_limit_hit("messaging", dest, "message_send").await;
                                continue;
                            }
                        }
                        // Validate text input - limit length and check for potentially dangerous content
                        if text.len() > 10000 {
                            let err = json!({"type": "error", "text": "Message too long (max 10000 chars)"});
                            let _ = to_gui_tx_cmd.send(err.to_string().into_bytes());
                            log_helper_cmd.warn("validation", "Message too long").await;
                            continue;
                        }
                
                        // Basic sanitization - remove null bytes and control characters (except newline and tab)
                        let sanitized_text: String = text.chars().filter(|c| *c != '\0' && (!c.is_control() || *c == '\n' || *c == '\t')).collect();
                
                        if sanitized_text.is_empty() {
                            let err = json!({"type": "error", "text": "Message is empty after sanitization"});
                            let _ = to_gui_tx_cmd.send(err.to_string().into_bytes());
                            log_helper_cmd.warn("validation", "Message empty after sanitization").await;
                            continue;
                        }
                
                        let address_hash = match AddressHash::new_from_hex_string(dest) {
                            Ok(h) => h,
                            Err(e) => {
                                let err = json!({"type": "error", "text": format!("Invalid hash: {:?}", e)});
                                let _ = to_gui_tx_cmd.send(err.to_string().into_bytes());
                                log_helper_cmd.error("validation", &format!("Invalid destination hash: {:?}", e)).await;
                                continue;
                            }
                        };
                
                        let mut packet_data = PacketDataBuffer::new();
                        // FIX: Correct error handling – no `?` inside closure
                        if let Err(e) = packet_data.write(sanitized_text.as_bytes()) {
                            let err = json!({"type": "error", "text": format!("Failed to write packet data: {:?}", e)});
                            let _ = to_gui_tx_cmd.send(err.to_string().into_bytes());
                            log_helper_cmd.error("packet", &format!("Packet data write error: {:?}", e)).await;
                            continue;
                        }
                
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
                
                        // Send packet over Reticulum
                        transport_cmd.lock().await.send_packet(packet).await;
                        log_helper_cmd.info("messaging", &format!("Sent packet to {} via Reticulum", dest)).await;
                
                        // Also send over LoRa if enabled
                        if lora_enabled {
                            if let Some(lora_iface) = lora_interface_cmd.as_ref() {
                            let lora_iface = lora_iface.clone();
                            let log_helper_lora_send = log_helper_cmd.clone();
                            let text_to_send = sanitized_text.clone();
                        
                            tokio::spawn(async move {
                            let mut iface = lora_iface.lock().await;
                            match iface.send(text_to_send.as_bytes()).await {
                                Ok(_) => log_helper_lora_send.info("lora", &format!("Sent message to LoRa: {}", text_to_send)).await,
                                Err(e) => log_helper_lora_send.error("lora", &format!("Failed to send over LoRa: {}", e)).await,
                            }
                            });
                            }
                        }
                
                let ack = json!({"type": "ack", "dest": dest, "text": sanitized_text});
                let _ = to_gui_tx_cmd.send(ack.to_string().into_bytes());
                }
        }
    }
    }
    }

    log_helper.info("main", "LoRa Bridge shutting down").await;
    Ok(())
}
