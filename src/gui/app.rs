use tokio::sync::mpsc;
use std::collections::HashMap;
use crate::gui::{BridgeCommand, BridgeEvent, spawn_reticulum_bridge_with_gui_config, GuiToMqtt, MqttToGui, Config};
use crate::gui::{ReticulumConfig, InterfaceType};
use crate::gui::peers::Peer;
use crate::gui::nodes::NodeInfo;
use crate::gui::relay::RelayDirection;

/// Reticulum connection status
#[derive(Debug, Clone, PartialEq)]
pub enum ReticulumStatus {
    Disconnected,
    Connecting,
    Connected,
    Error(String),
}

/// Interface status
#[derive(Debug, Clone)]
pub struct InterfaceStatus {
    pub name: String,
    pub interface_type: String,
    pub enabled: bool,
    pub connected: bool,
    pub last_seen: Option<std::time::Instant>,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub error: Option<String>,
}

/// Message filter source options
#[derive(Debug, Clone, PartialEq)]
pub enum MessageFilterSource {
    All,
    Mqtt,
    Reticulum,
    System,
    Custom(String),
}

impl Default for MessageFilterSource {
    fn default() -> Self {
        Self::All
    }
}

/// Reticulum statistics
#[derive(Debug, Clone)]
pub struct ReticulumStats {
    pub packets_sent: u64,
    pub packets_received: u64,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub peers_discovered: u64,
    pub links_established: u64,
    pub announces_sent: u64,
    pub announces_received: u64,
    pub last_update: std::time::Instant,
}

impl Default for ReticulumStats {
    fn default() -> Self {
        Self {
            packets_sent: 0,
            packets_received: 0,
            bytes_sent: 0,
            bytes_received: 0,
            peers_discovered: 0,
            links_established: 0,
            announces_sent: 0,
            announces_received: 0,
            last_update: std::time::Instant::now(),
        }
    }
}

pub struct MeshtasticGuiApp {
    pub messages: Vec<(String, String)>,
    pub input_text: String,
    pub new_channel_name: String,
    pub new_channel_psk: String,
    pub active_channel: String,
    pub channels: Vec<String>,
    pub channel_psks: HashMap<String, String>,
    pub peers: Vec<Peer>,
    pub peer_nicknames: HashMap<String, String>,
    pub selected_peer: Option<Peer>,
    pub selected_file_path: Option<String>,
    pub transfer_progress: Option<(String, f32)>,
    pub nodes: HashMap<String, NodeInfo>,
    pub channel_url_input: String,
    pub relay_enabled: bool,
    pub relay_direction: RelayDirection,
    pub relay_target_channel: String,
    #[allow(dead_code)]
    pub relay_target_peer: Option<String>,
    pub nickname: String,
    pub mqtt_cmd_tx: mpsc::UnboundedSender<GuiToMqtt>,
    pub mqtt_msg_rx: mpsc::UnboundedReceiver<MqttToGui>,
    pub bridge_cmd_tx: mpsc::UnboundedSender<BridgeCommand>,
    pub bridge_msg_rx: mpsc::UnboundedReceiver<BridgeEvent>,
    pub show_qr_window: bool,
    pub current_qr_channel: Option<String>,
    pub qr_texture: Option<egui::TextureHandle>,
    // Configuration fields
    pub mqtt_username: String,
    pub mqtt_password: String,
    pub mqtt_host: String,
    pub mqtt_port: String,
    pub mqtt_use_tls: bool,
    pub reticulum_server: String,
    pub show_config_window: bool,
    // Message search/filtering
    pub message_search_text: String,
    pub message_filter_source: MessageFilterSource,
    pub show_search_panel: bool,
    // Reticulum configuration
    pub reticulum_config: ReticulumConfig,
    pub show_reticulum_config_window: bool,
    pub selected_interface: Option<String>,
    pub new_interface_type: InterfaceType,
    // Reticulum connection management
    pub reticulum_status: ReticulumStatus,
    pub interface_statuses: HashMap<String, InterfaceStatus>,
    pub reticulum_connected: bool,
    pub reticulum_my_hash: Option<String>,
    pub reticulum_peers: HashMap<String, String>, // hash -> nickname
    pub reticulum_stats: ReticulumStats,
    // Theme support
    pub dark_mode: bool,
    pub show_theme_settings: bool,
    // Nickname editing
    pub editing_nickname_for_peer: Option<String>,
    pub editing_nickname_temp: String,
}

impl MeshtasticGuiApp {
    pub fn new(
        mqtt_cmd_tx: mpsc::UnboundedSender<GuiToMqtt>,
        mqtt_msg_rx: mpsc::UnboundedReceiver<MqttToGui>,
        bridge_cmd_tx: mpsc::UnboundedSender<BridgeCommand>,
        bridge_msg_rx: mpsc::UnboundedReceiver<BridgeEvent>,
        channel_url_input: String,
    ) -> Self {
        let mut app = Self {
            messages: Vec::new(),
            input_text: String::new(),
            new_channel_name: String::new(),
            new_channel_psk: String::new(),
            active_channel: "STLIW-MC".to_string(),
            channels: vec!["STLIW-MC".to_string()],
            channel_psks: HashMap::new(),
            peers: Vec::new(),
            peer_nicknames: HashMap::new(),
            selected_peer: None,
            selected_file_path: None,
            transfer_progress: None,
            nodes: HashMap::new(),
            channel_url_input,
            relay_enabled: false,
            relay_direction: RelayDirection::default(),
            relay_target_channel: "STLIW-MC".to_string(),
            relay_target_peer: None,
            nickname: "Anonymous".to_string(),
            mqtt_cmd_tx,
            mqtt_msg_rx,
            bridge_cmd_tx,
            bridge_msg_rx,
            show_qr_window: false,
            current_qr_channel: None,
            qr_texture: None,
            // Configuration fields with defaults
            mqtt_username: "".to_string(),
            mqtt_password: "".to_string(),
            mqtt_host: "mqtt.meshtastic.org".to_string(),
            mqtt_port: "8883".to_string(),  // Default to TLS port
            mqtt_use_tls: true,  // Default to using TLS
            reticulum_server: "RNS.MichMesh.net:7822".to_string(),
            show_config_window: false,
            // Message search/filtering
            message_search_text: String::new(),
            message_filter_source: MessageFilterSource::default(),
            show_search_panel: false,
            // Reticulum configuration
            reticulum_config: ReticulumConfig::default_with_interfaces(),
            show_reticulum_config_window: false,
            selected_interface: None,
            new_interface_type: InterfaceType::default(),
            // Reticulum connection management
            reticulum_status: ReticulumStatus::Disconnected,
            interface_statuses: HashMap::new(),
            reticulum_connected: false,
            reticulum_my_hash: None,
            reticulum_peers: HashMap::new(),
            reticulum_stats: ReticulumStats {
                last_update: std::time::Instant::now(),
                ..Default::default()
            },
            // Theme support
            dark_mode: false,
            show_theme_settings: false,
            // Nickname editing
            editing_nickname_for_peer: None,
            editing_nickname_temp: String::new(),
        };
        app.load_peers();
        app.load_nicknames();
        app.load_nickname();
        app.load_config();
        app
    }

    pub fn save_nickname(&self) {
        let _ = std::fs::write("nickname.txt", &self.nickname);
    }

    fn load_nickname(&mut self) {
        if let Ok(data) = std::fs::read_to_string("nickname.txt") {
            let trimmed = data.trim().to_string();
            if !trimmed.is_empty() {
                self.nickname = trimmed;
            }
        }
    }

    pub fn load_config(&mut self) {
        // Note: GUI configuration is for display only
        // Actual connections use environment variables via Config::from_env()
        if let Ok(data) = std::fs::read_to_string("gui_config.json") {
            if let Ok(config) = serde_json::from_str::<serde_json::Value>(&data) {
                // Only load non-sensitive display settings
                if let Some(host) = config.get("mqtt_host").and_then(|v| v.as_str()) {
                    self.mqtt_host = host.to_string();
                }
                if let Some(port) = config.get("mqtt_port").and_then(|v| v.as_str()) {
                    self.mqtt_port = port.to_string();
                }
                if let Some(use_tls) = config.get("mqtt_use_tls").and_then(|v| v.as_bool()) {
                    self.mqtt_use_tls = use_tls;
                }
                if let Some(server) = config.get("reticulum_server").and_then(|v| v.as_str()) {
                    self.reticulum_server = server.to_string();
                }
                // Load theme settings
                if let Some(dark_mode) = config.get("dark_mode").and_then(|v| v.as_bool()) {
                    self.dark_mode = dark_mode;
                }
                // Note: We don't load usernames/passwords for security
            }
        }
    }

    pub fn save_config(&self) {
        // Only save non-sensitive display settings
        // Sensitive credentials should be set via environment variables
        let config = serde_json::json!({
            // Note: We don't save usernames/passwords for security
            "mqtt_host": self.mqtt_host,
            "mqtt_port": self.mqtt_port,
            "mqtt_use_tls": self.mqtt_use_tls,
            "reticulum_server": self.reticulum_server,
            "dark_mode": self.dark_mode,
        });
        if let Ok(json_string) = serde_json::to_string_pretty(&config) {
            let _ = std::fs::write("gui_config.json", json_string);
        }
    }

    /// Connect to Reticulum using current configuration
    pub async fn connect_reticulum(&mut self) {
        self.reticulum_status = ReticulumStatus::Connecting;
        
        // Update interface statuses
        self.update_interface_statuses();
        
        // Create config from GUI settings
        let config = Config::from_env();
        
        // Clone the reticulum config to pass to the bridge
        let reticulum_config = self.reticulum_config.clone();
        
        // Clone the command sender for the bridge
        let bridge_cmd_tx = self.bridge_cmd_tx.clone();
        
        // Create new channels for bridge communication
        let (new_bridge_cmd_tx, new_bridge_cmd_rx) = mpsc::unbounded_channel();
        let (new_bridge_event_tx, new_bridge_event_rx) = mpsc::unbounded_channel();
        
        // Replace our bridge channels with the new ones
        self.bridge_cmd_tx = new_bridge_cmd_tx;
        self.bridge_msg_rx = new_bridge_event_rx;
        
        // Spawn the reticulum bridge with GUI configuration
        let bridge_result = spawn_reticulum_bridge_with_gui_config(
            config,
            &reticulum_config,
            new_bridge_cmd_rx,
            new_bridge_event_tx,
        );
        
        // Handle bridge spawning
        match bridge_result.await {
            Ok(_) => {
                self.reticulum_status = ReticulumStatus::Connected;
                self.reticulum_connected = true;
                // Note: We'll get the actual hash from bridge events
                self.messages.push(("System".to_string(), "Reticulum bridge started".to_string()));
            }
            Err(e) => {
                self.reticulum_status = ReticulumStatus::Error(format!("Failed to start bridge: {}", e));
                self.messages.push(("System".to_string(), format!("Failed to start reticulum bridge: {}", e)));
                // Restore original bridge command sender
                self.bridge_cmd_tx = bridge_cmd_tx;
            }
        }
    }

    /// Disconnect from Reticulum
    pub async fn disconnect_reticulum(&mut self) {
        self.reticulum_status = ReticulumStatus::Disconnected;
        self.reticulum_connected = false;
        
        // Clear interface statuses
        for status in self.interface_statuses.values_mut() {
            status.connected = false;
            status.error = Some("Disconnected".to_string());
        }
        
        // Note: We can't actually stop the bridge task easily
        // The bridge will continue running but we won't send commands to it
        // In a more complete implementation, we would have a proper shutdown mechanism
        
        self.messages.push(("System".to_string(), "Reticulum disconnected".to_string()));
    }

    /// Update interface statuses from configuration
    fn update_interface_statuses(&mut self) {
        for interface in &self.reticulum_config.interfaces {
            let status = InterfaceStatus {
                name: interface.name.clone(),
                interface_type: interface.interface_type.as_str().to_string(),
                enabled: interface.enabled,
                connected: false, // Will be updated when actually connected
                last_seen: None,
                bytes_sent: 0,
                bytes_received: 0,
                error: None,
            };
            self.interface_statuses.insert(interface.name.clone(), status);
        }
    }

    /// Send a message via Reticulum
    pub async fn send_reticulum_message(&mut self, dest_hash: &str, text: &str) {
        if !self.reticulum_connected {
            self.messages.push(("System".to_string(), "Reticulum not connected".to_string()));
            return;
        }

        let cmd = BridgeCommand::SendMessage {
            dest_hash: dest_hash.to_string(),
            text: text.to_string(),
        };

        if let Err(e) = self.bridge_cmd_tx.send(cmd) {
            self.messages.push(("System".to_string(), format!("Failed to send message: {}", e)));
        } else {
            self.messages.push(("System".to_string(), format!("Message sent to {}", dest_hash)));
            self.reticulum_stats.packets_sent += 1;
            self.reticulum_stats.bytes_sent += text.len() as u64;
            self.reticulum_stats.last_update = std::time::Instant::now();
        }
    }

    /// Send a file via Reticulum
    pub async fn send_reticulum_file(&mut self, dest_hash: &str, file_path: &str) {
        if !self.reticulum_connected {
            self.messages.push(("System".to_string(), "Reticulum not connected".to_string()));
            return;
        }

        let cmd = BridgeCommand::SendFile {
            dest_hash: dest_hash.to_string(),
            file_path: file_path.to_string(),
        };

        if let Err(e) = self.bridge_cmd_tx.send(cmd) {
            self.messages.push(("System".to_string(), format!("Failed to send file: {}", e)));
        } else {
            self.messages.push(("System".to_string(), format!("File sent to {}", dest_hash)));
        }
    }

    /// Refresh reticulum announces
    pub async fn refresh_reticulum(&mut self) {
        if !self.reticulum_connected {
            self.messages.push(("System".to_string(), "Reticulum not connected".to_string()));
            return;
        }

        let cmd = BridgeCommand::Refresh;

        if let Err(e) = self.bridge_cmd_tx.send(cmd) {
            self.messages.push(("System".to_string(), format!("Failed to refresh: {}", e)));
        } else {
            self.messages.push(("System".to_string(), "Refresh sent".to_string()));
            self.reticulum_stats.announces_sent += 1;
            self.reticulum_stats.last_update = std::time::Instant::now();
        }
    }


}