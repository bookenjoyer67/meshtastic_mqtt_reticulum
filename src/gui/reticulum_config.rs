//! Reticulum interface configuration structures for GUI

use serde::{Deserialize, Serialize};

/// Interface types supported by Reticulum
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum InterfaceType {
    TcpClient,
    TcpServer,
    Udp,
    Serial,
    Mqtt,
    Kiss,
    I2p,
}

impl Default for InterfaceType {
    fn default() -> Self {
        Self::TcpClient
    }
}

impl InterfaceType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::TcpClient => "TcpClient",
            Self::TcpServer => "TcpServer",
            Self::Udp => "Udp",
            Self::Serial => "Serial",
            Self::Mqtt => "Mqtt",
            Self::Kiss => "Kiss",
            Self::I2p => "I2p",
        }
    }
    
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "TcpClient" => Some(Self::TcpClient),
            "TcpServer" => Some(Self::TcpServer),
            "Udp" => Some(Self::Udp),
            "Serial" => Some(Self::Serial),
            "Mqtt" => Some(Self::Mqtt),
            "Kiss" => Some(Self::Kiss),
            "I2p" => Some(Self::I2p),
            _ => None,
        }
    }
}

/// Common interface configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct InterfaceConfig {
    pub interface_type: InterfaceType,
    pub name: String,
    pub enabled: bool,
    
    // TCP/UDP fields
    pub host: String,
    pub port: u16,
    pub bind_address: String,
    
    // Serial/KISS fields
    pub serial_port: String,
    pub baud_rate: u32,
    
    // MQTT fields
    pub mqtt_client_id: String,
    pub mqtt_username: String,
    pub mqtt_password: String,
    pub mqtt_topic_prefix: String,
    pub mqtt_use_tls: bool,
    
    // I2P fields
    pub i2p_sam_address: String,
    pub i2p_sam_port: u16,
    pub i2p_destination: String,
    pub i2p_session_name: String,
    pub i2p_session_type: String,
    pub i2p_use_local_dest: bool,
    pub i2p_max_connection_attempts: u32,
}

impl InterfaceConfig {
    /// Create a new TCP client interface configuration
    pub fn new_tcp_client(name: &str, host: &str, port: u16) -> Self {
        Self {
            interface_type: InterfaceType::TcpClient,
            name: name.to_string(),
            enabled: true,
            host: host.to_string(),
            port,
            bind_address: "0.0.0.0".to_string(),
            ..Default::default()
        }
    }
    
    /// Create a new TCP server interface configuration
    pub fn new_tcp_server(name: &str, bind_address: &str, port: u16) -> Self {
        Self {
            interface_type: InterfaceType::TcpServer,
            name: name.to_string(),
            enabled: true,
            bind_address: bind_address.to_string(),
            port,
            ..Default::default()
        }
    }
    
    /// Create a new UDP interface configuration
    pub fn new_udp(name: &str, bind_address: &str, port: u16) -> Self {
        Self {
            interface_type: InterfaceType::Udp,
            name: name.to_string(),
            enabled: true,
            bind_address: bind_address.to_string(),
            port,
            ..Default::default()
        }
    }
    
    /// Create a new serial interface configuration
    pub fn new_serial(name: &str, port: &str, baud_rate: u32) -> Self {
        Self {
            interface_type: InterfaceType::Serial,
            name: name.to_string(),
            enabled: true,
            serial_port: port.to_string(),
            baud_rate,
            ..Default::default()
        }
    }
    
    /// Create a new MQTT interface configuration
    pub fn new_mqtt(name: &str, host: &str, port: u16) -> Self {
        Self {
            interface_type: InterfaceType::Mqtt,
            name: name.to_string(),
            enabled: true,
            host: host.to_string(),
            port,
            mqtt_client_id: format!("reticulum-{}", name),
            mqtt_topic_prefix: "reticulum".to_string(),
            ..Default::default()
        }
    }
    
    /// Create a new KISS interface configuration
    pub fn new_kiss(name: &str, port: &str, baud_rate: u32) -> Self {
        Self {
            interface_type: InterfaceType::Kiss,
            name: name.to_string(),
            enabled: true,
            serial_port: port.to_string(),
            baud_rate,
            ..Default::default()
        }
    }
    
    /// Create a new I2P interface configuration
    pub fn new_i2p(name: &str) -> Self {
        Self {
            interface_type: InterfaceType::I2p,
            name: name.to_string(),
            enabled: true,
            i2p_sam_address: "127.0.0.1".to_string(),
            i2p_sam_port: 7656,
            i2p_session_name: format!("reticulum-{}", name),
            i2p_session_type: "DATAGRAM".to_string(),
            i2p_use_local_dest: true,
            i2p_max_connection_attempts: 10,
            ..Default::default()
        }
    }
    
    /// Get display name for the interface
    pub fn display_name(&self) -> String {
        format!("{} ({})", self.name, self.interface_type.as_str())
    }
    
    /// Check if this interface requires serial port
    pub fn is_serial_based(&self) -> bool {
        matches!(self.interface_type, InterfaceType::Serial | InterfaceType::Kiss)
    }
    
    /// Check if this interface requires network connection
    pub fn is_network_based(&self) -> bool {
        matches!(self.interface_type, InterfaceType::TcpClient | InterfaceType::TcpServer | InterfaceType::Udp | InterfaceType::Mqtt | InterfaceType::I2p)
    }
}

/// Reticulum global configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ReticulumGlobalConfig {
    pub node_name: String,
    pub enable_forwarding: bool,
    pub enable_announces: bool,
    pub max_packet_size: u32,
    pub default_mtu: u32,
}

/// Reticulum transport configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ReticulumTransportConfig {
    pub max_hops: u32,
    pub path_request_timeout: u32,
    pub link_establish_timeout: u32,
    pub packet_cache_size: u32,
    pub announce_table_size: u32,
    pub link_table_size: u32,
    pub eager_rerouting: bool,
    pub restart_links: bool,
}

/// Reticulum security configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ReticulumSecurityConfig {
    pub enable_encryption: bool,
    pub min_key_strength: u32,
    pub verify_signatures: bool,
    pub allowed_ciphers: Vec<String>,
    pub cert_validation: String,
}

/// Reticulum performance configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ReticulumPerformanceConfig {
    pub thread_pool_size: u32,
    pub max_concurrent_connections: u32,
    pub connection_pool_size: u32,
    pub enable_compression: bool,
    pub compression_level: u32,
}

/// Reticulum logging configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ReticulumLoggingConfig {
    pub level: String,
    pub enable_file_logging: bool,
    pub log_file: String,
    pub max_log_size_mb: u32,
    pub max_log_files: u32,
    pub enable_console: bool,
    pub format: String,
    pub structured_logging: bool,
}

/// Complete Reticulum configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ReticulumConfig {
    pub global: ReticulumGlobalConfig,
    pub interfaces: Vec<InterfaceConfig>,
    pub transport: ReticulumTransportConfig,
    pub security: ReticulumSecurityConfig,
    pub performance: ReticulumPerformanceConfig,
    pub logging: ReticulumLoggingConfig,
}

impl ReticulumConfig {
    /// Create default configuration with common interfaces
    pub fn default_with_interfaces() -> Self {
        let mut config = Self::default();
        
        // Set some reasonable defaults
        config.global.node_name = "meshtastic-bridge".to_string();
        config.global.enable_forwarding = true;
        config.global.enable_announces = true;
        config.global.max_packet_size = 500;
        config.global.default_mtu = 280;
        
        // Add default TCP client interface
        config.interfaces.push(InterfaceConfig::new_tcp_client(
            "default-tcp",
            "RNS.MichMesh.net",
            7822,
        ));
        
        // Add default MQTT interface
        config.interfaces.push(InterfaceConfig::new_mqtt(
            "default-mqtt",
            "mqtt.meshtastic.org",
            8883,
        ));
        
        // Set transport defaults
        config.transport.max_hops = 128;
        config.transport.path_request_timeout = 30;
        config.transport.link_establish_timeout = 60;
        config.transport.packet_cache_size = 1024;
        config.transport.announce_table_size = 512;
        config.transport.link_table_size = 256;
        config.transport.eager_rerouting = false;
        config.transport.restart_links = true;
        
        // Set security defaults
        config.security.enable_encryption = true;
        config.security.min_key_strength = 256;
        config.security.verify_signatures = true;
        config.security.allowed_ciphers = vec!["AES-256-GCM".to_string(), "ChaCha20-Poly1305".to_string()];
        config.security.cert_validation = "Basic".to_string();
        
        // Set performance defaults
        config.performance.thread_pool_size = 4;
        config.performance.max_concurrent_connections = 100;
        config.performance.connection_pool_size = 20;
        config.performance.enable_compression = true;
        config.performance.compression_level = 6;
        
        // Set logging defaults
        config.logging.level = "Info".to_string();
        config.logging.enable_file_logging = false;
        config.logging.log_file = "reticulum.log".to_string();
        config.logging.max_log_size_mb = 10;
        config.logging.max_log_files = 5;
        config.logging.enable_console = true;
        config.logging.format = "Text".to_string();
        config.logging.structured_logging = true;
        
        config
    }
    
    /// Save configuration to file
    pub fn save_to_file(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let toml_string = toml::to_string_pretty(self)?;
        std::fs::write(path, toml_string)?;
        Ok(())
    }
    
    /// Load configuration from file
    pub fn load_from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let config: Self = toml::from_str(&content)?;
        Ok(config)
    }
    
    /// Get enabled interfaces
    pub fn enabled_interfaces(&self) -> Vec<&InterfaceConfig> {
        self.interfaces.iter().filter(|i| i.enabled).collect()
    }
    
    /// Get interface by name
    pub fn get_interface(&self, name: &str) -> Option<&InterfaceConfig> {
        self.interfaces.iter().find(|i| i.name == name)
    }
    
    /// Get mutable interface by name
    pub fn get_interface_mut(&mut self, name: &str) -> Option<&mut InterfaceConfig> {
        self.interfaces.iter_mut().find(|i| i.name == name)
    }
    
    /// Add a new interface
    pub fn add_interface(&mut self, interface: InterfaceConfig) {
        self.interfaces.push(interface);
    }
    
    /// Remove an interface by name
    pub fn remove_interface(&mut self, name: &str) -> bool {
        let original_len = self.interfaces.len();
        self.interfaces.retain(|i| i.name != name);
        self.interfaces.len() < original_len
    }
}