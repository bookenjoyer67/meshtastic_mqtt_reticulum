//! Configuration management for Reticulum-rs
//!
//! This module provides configuration loading, validation, and management
//! for Reticulum network settings.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use crate::error::{RnsError, Result};

/// Main configuration structure for Reticulum
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReticulumConfig {
    /// Global settings
    pub global: GlobalConfig,
    
    /// Interface configurations
    pub interfaces: Vec<InterfaceConfig>,
    
    /// Destination configurations
    pub destinations: Vec<DestinationConfig>,
    
    /// Transport settings
    pub transport: TransportConfig,
    
    /// Security settings
    pub security: SecurityConfig,
    
    /// Performance tuning
    pub performance: PerformanceConfig,
    
    /// Logging configuration
    pub logging: LoggingConfig,
}

/// Global configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalConfig {
    /// Node name (optional)
    pub node_name: Option<String>,
    
    /// Enable/disable packet forwarding
    pub enable_forwarding: bool,
    
    /// Enable/disable announce propagation
    pub enable_announces: bool,
    
    /// Maximum packet size in bytes
    pub max_packet_size: usize,
    
    /// Default interface MTU
    pub default_mtu: usize,
}

/// Interface configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum InterfaceConfig {
    /// TCP client interface
    TcpClient {
        name: String,
        host: String,
        port: u16,
        enabled: bool,
        #[serde(default)]
        options: HashMap<String, String>,
    },
    
    /// TCP server interface
    TcpServer {
        name: String,
        bind_address: String,
        port: u16,
        enabled: bool,
        #[serde(default)]
        options: HashMap<String, String>,
    },
    
    /// UDP interface
    Udp {
        name: String,
        bind_address: String,
        port: u16,
        enabled: bool,
        #[serde(default)]
        options: HashMap<String, String>,
    },
    
    /// Kaonic interface
    Kaonic {
        name: String,
        enabled: bool,
        #[serde(default)]
        options: HashMap<String, String>,
    },
    
    /// Serial interface
    Serial {
        name: String,
        port: String,
        baud_rate: u32,
        enabled: bool,
        #[serde(default)]
        options: HashMap<String, String>,
    },
    
    /// MQTT interface
    Mqtt {
        name: String,
        host: String,
        port: u16,
        client_id: Option<String>,
        username: Option<String>,
        password: Option<String>,
        topic_prefix: Option<String>,
        use_tls: bool,
        enabled: bool,
        #[serde(default)]
        options: HashMap<String, String>,
    },
    
    /// KISS interface
    Kiss {
        name: String,
        port: String,
        baud_rate: u32,
        enabled: bool,
        #[serde(default)]
        options: HashMap<String, String>,
    },
    
    /// I2P interface
    I2p {
        name: String,
        sam_address: String,
        sam_port: u16,
        destination: Option<String>,
        session_name: Option<String>,
        session_type: Option<String>,
        use_local_dest: Option<bool>,
        enabled: bool,
        #[serde(default)]
        options: HashMap<String, String>,
    },
    
    /// Custom interface
    Custom {
        name: String,
        interface_type: String,
        enabled: bool,
        #[serde(default)]
        options: HashMap<String, String>,
    },
}

/// Destination configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DestinationConfig {
    /// Destination name
    pub name: String,
    
    /// Application type
    pub app_type: String,
    
    /// Enable/disable this destination
    pub enabled: bool,
    
    /// Optional identity file path
    pub identity_file: Option<PathBuf>,
    
    /// Additional options
    #[serde(default)]
    pub options: HashMap<String, String>,
}

/// Transport configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransportConfig {
    /// Maximum number of hops
    pub max_hops: usize,
    
    /// Path request timeout in seconds
    pub path_request_timeout: u64,
    
    /// Link establishment timeout in seconds
    pub link_establish_timeout: u64,
    
    /// Packet cache size
    pub packet_cache_size: usize,
    
    /// Announce table size
    pub announce_table_size: usize,
    
    /// Link table size
    pub link_table_size: usize,
    
    /// Enable eager rerouting
    pub eager_rerouting: bool,
    
    /// Enable link restart
    pub restart_links: bool,
}

/// Security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// Enable/disable encryption
    pub enable_encryption: bool,
    
    /// Minimum key strength in bits
    pub min_key_strength: u32,
    
    /// Enable/disable signature verification
    pub verify_signatures: bool,
    
    /// Allowed cipher suites
    pub allowed_ciphers: Vec<String>,
    
    /// Certificate validation mode
    pub cert_validation: CertValidationMode,
}

/// Certificate validation mode
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CertValidationMode {
    /// No certificate validation
    None,
    
    /// Basic validation
    Basic,
    
    /// Strict validation
    Strict,
    
    /// Custom validation
    Custom(String),
}

/// Performance configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    /// Thread pool size for async operations
    pub thread_pool_size: usize,
    
    /// Maximum concurrent connections
    pub max_concurrent_connections: usize,
    
    /// Connection pool size
    pub connection_pool_size: usize,
    
    /// Buffer sizes for different operations
    pub buffer_sizes: BufferSizes,
    
    /// Enable/disable compression
    pub enable_compression: bool,
    
    /// Compression level (0-9)
    pub compression_level: u8,
}

/// Buffer size configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BufferSizes {
    /// Packet buffer size
    pub packet_buffer: usize,
    
    /// Network buffer size
    pub network_buffer: usize,
    
    /// Crypto buffer size
    pub crypto_buffer: usize,
    
    /// File transfer buffer size
    pub file_buffer: usize,
}

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Log level
    pub level: LogLevel,
    
    /// Enable/disable file logging
    pub enable_file_logging: bool,
    
    /// Log file path
    pub log_file: Option<PathBuf>,
    
    /// Maximum log file size in MB
    pub max_log_size_mb: u64,
    
    /// Number of log files to keep
    pub max_log_files: usize,
    
    /// Enable/disable console logging
    pub enable_console: bool,
    
    /// Log format
    pub format: LogFormat,
    
    /// Enable/disable structured logging
    pub structured_logging: bool,
}

/// Log level
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogLevel {
    Critical,
    Error,
    Warning,
    Info,
    Debug,
    Trace,
    Packet,
}

/// Log format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogFormat {
    Text,
    Json,
    Csv,
}

impl Default for ReticulumConfig {
    fn default() -> Self {
        Self {
            global: GlobalConfig::default(),
            interfaces: Vec::new(),
            destinations: Vec::new(),
            transport: TransportConfig::default(),
            security: SecurityConfig::default(),
            performance: PerformanceConfig::default(),
            logging: LoggingConfig::default(),
        }
    }
}

impl Default for GlobalConfig {
    fn default() -> Self {
        Self {
            node_name: None,
            enable_forwarding: true,
            enable_announces: true,
            max_packet_size: 500,
            default_mtu: 280,
        }
    }
}

impl Default for TransportConfig {
    fn default() -> Self {
        Self {
            max_hops: 128,
            path_request_timeout: 30,
            link_establish_timeout: 60,
            packet_cache_size: 1024,
            announce_table_size: 512,
            link_table_size: 256,
            eager_rerouting: false,
            restart_links: true,
        }
    }
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            enable_encryption: true,
            min_key_strength: 256,
            verify_signatures: true,
            allowed_ciphers: vec![
                "AES-256-GCM".to_string(),
                "ChaCha20-Poly1305".to_string(),
            ],
            cert_validation: CertValidationMode::Basic,
        }
    }
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            thread_pool_size: 4,
            max_concurrent_connections: 100,
            connection_pool_size: 20,
            buffer_sizes: BufferSizes::default(),
            enable_compression: true,
            compression_level: 6,
        }
    }
}

impl Default for BufferSizes {
    fn default() -> Self {
        Self {
            packet_buffer: 4096,
            network_buffer: 8192,
            crypto_buffer: 1024,
            file_buffer: 65536,
        }
    }
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: LogLevel::Info,
            enable_file_logging: false,
            log_file: None,
            max_log_size_mb: 10,
            max_log_files: 5,
            enable_console: true,
            format: LogFormat::Text,
            structured_logging: true,
        }
    }
}

/// Configuration manager
pub struct ConfigManager {
    config: ReticulumConfig,
    config_path: PathBuf,
    last_modified: Option<std::time::SystemTime>,
}

impl ConfigManager {
    /// Create a new configuration manager
    pub fn new(config_path: impl AsRef<Path>) -> Self {
        Self {
            config: ReticulumConfig::default(),
            config_path: config_path.as_ref().to_path_buf(),
            last_modified: None,
        }
    }
    
    /// Load configuration from file
    pub fn load(&mut self) -> Result<()> {
        let path = &self.config_path;
        
        if !path.exists() {
            return Err(RnsError::ConfigError(format!(
                "Configuration file not found: {}",
                path.display()
            )));
        }
        
        let content = fs::read_to_string(path)
            .map_err(|e| RnsError::ConfigError(format!("Failed to read config file: {}", e)))?;
        
        self.config = toml::from_str(&content)
            .map_err(|e| RnsError::ConfigError(format!("Failed to parse config file: {}", e)))?;
        
        // Get file modification time
        self.last_modified = fs::metadata(path)
            .ok()
            .and_then(|m| m.modified().ok());
        
        self.validate()?;
        
        Ok(())
    }
    
    /// Save configuration to file
    pub fn save(&self) -> Result<()> {
        let content = toml::to_string_pretty(&self.config)
            .map_err(|e| RnsError::ConfigError(format!("Failed to serialize config: {}", e)))?;
        
        fs::write(&self.config_path, content)
            .map_err(|e| RnsError::ConfigError(format!("Failed to write config file: {}", e)))?;
        
        Ok(())
    }
    
    /// Create default configuration file
    pub fn create_default(&mut self) -> Result<()> {
        self.config = ReticulumConfig::default();
        self.save()?;
        Ok(())
    }
    
    /// Check if configuration has changed on disk
    pub fn has_changed(&self) -> Result<bool> {
        if !self.config_path.exists() {
            return Ok(true);
        }
        
        let current_modified = fs::metadata(&self.config_path)
            .ok()
            .and_then(|m| m.modified().ok());
        
        Ok(current_modified != self.last_modified)
    }
    
    /// Reload configuration if changed
    pub fn reload_if_changed(&mut self) -> Result<bool> {
        if self.has_changed()? {
            self.load()?;
            Ok(true)
        } else {
            Ok(false)
        }
    }
    
    /// Validate configuration
    pub fn validate(&self) -> Result<()> {
        // Validate global settings
        if self.config.global.max_packet_size == 0 {
            return Err(RnsError::ConfigError(
                "max_packet_size must be greater than 0".to_string()
            ));
        }
        
        if self.config.global.default_mtu == 0 {
            return Err(RnsError::ConfigError(
                "default_mtu must be greater than 0".to_string()
            ));
        }
        
        // Validate transport settings
        if self.config.transport.max_hops == 0 {
            return Err(RnsError::ConfigError(
                "max_hops must be greater than 0".to_string()
            ));
        }
        
        if self.config.transport.packet_cache_size == 0 {
            return Err(RnsError::ConfigError(
                "packet_cache_size must be greater than 0".to_string()
            ));
        }
        
        // Validate performance settings
        if self.config.performance.thread_pool_size == 0 {
            return Err(RnsError::ConfigError(
                "thread_pool_size must be greater than 0".to_string()
            ));
        }
        
        // Validate security settings
        if self.config.security.min_key_strength < 128 {
            return Err(RnsError::ConfigError(
                "min_key_strength must be at least 128 bits".to_string()
            ));
        }
        
        Ok(())
    }
    
    /// Get configuration reference
    pub fn config(&self) -> &ReticulumConfig {
        &self.config
    }
    
    /// Get mutable configuration reference
    pub fn config_mut(&mut self) -> &mut ReticulumConfig {
        &mut self.config
    }
    
    /// Get configuration path
    pub fn config_path(&self) -> &Path {
        &self.config_path
    }
}

/// Find configuration file in standard locations
pub fn find_config_file() -> Option<PathBuf> {
    let mut possible_paths = Vec::new();
    
    // Current directory
    possible_paths.push(PathBuf::from("reticulum.toml"));
    
    // User config directory
    if let Some(mut p) = dirs::config_dir() {
        p.push("reticulum");
        p.push("config.toml");
        possible_paths.push(p);
    }
    
    // System config directory
    possible_paths.push(PathBuf::from("/etc/reticulum/config.toml"));
    
    // Home directory
    if let Some(mut p) = dirs::home_dir() {
        p.push(".reticulum");
        p.push("config.toml");
        possible_paths.push(p);
    }
    
    for path in possible_paths {
        if path.exists() {
            return Some(path);
        }
    }
    
    None
}

/// Load configuration from file or create default
pub fn load_or_create_config(config_path: Option<PathBuf>) -> Result<ConfigManager> {
    let path = config_path
        .or_else(find_config_file)
        .unwrap_or_else(|| PathBuf::from("reticulum.toml"));
    
    let mut manager = ConfigManager::new(&path);
    
    if !path.exists() {
        log::info!("Creating default configuration at {}", path.display());
        manager.create_default()?;
    } else {
        manager.load()?;
    }
    
    Ok(manager)
}