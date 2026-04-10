//! Configuration module for Meshtastic MQTT Reticulum Bridge
//! 
//! This module provides configuration management through environment variables
//! with sensible defaults for development.

use std::collections::HashMap;

/// Application configuration
#[derive(Debug, Clone)]
pub struct Config {
    /// MQTT broker username
    pub mqtt_username: String,
    /// MQTT broker password
    pub mqtt_password: String,
    /// MQTT broker host
    pub mqtt_host: String,
    /// MQTT broker port
    pub mqtt_port: u16,
    /// Use TLS for MQTT connection
    pub mqtt_use_tls: bool,
    /// Reticulum server address (host:port)
    pub reticulum_server: String,
    /// Initial channels with PSKs (channel:psk,channel2:psk2)
    pub initial_channels: HashMap<String, String>,
    /// GUI TCP bind address
    pub gui_bind_address: String,
    /// GUI TCP port
    pub gui_port: u16,
    /// Enable structured logging to console
    pub log_to_console: bool,
    /// Enable structured logging to file
    pub log_to_file: bool,
    /// Structured log file path
    pub log_file_path: String,
    /// Enable audit logging for security events
    pub enable_audit_logging: bool,
    /// Audit log file path
    pub audit_log_file_path: String,
    /// Webhook configurations
    pub webhook_configs: Vec<crate::webhook::WebhookConfig>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            mqtt_username: "".to_string(),
            mqtt_password: "".to_string(),
            mqtt_host: "".to_string(),
            mqtt_port: 8883, // Default to TLS port
            mqtt_use_tls: true, // Default to using TLS
            reticulum_server: "RNS.MichMesh.net:7822".to_string(),
            initial_channels: HashMap::new(),
            gui_bind_address: "127.0.0.1".to_string(),
            gui_port: 4244,
            log_to_console: true,
            log_to_file: true,
            log_file_path: "meshtastic_bridge.log".to_string(),
            enable_audit_logging: true,
            audit_log_file_path: "meshtastic_audit.log".to_string(),
            webhook_configs: Vec::new(),
        }
    }
}

impl Config {
    /// Create a new configuration from environment variables
    pub fn from_env() -> Self {
        let mut config = Self::default();
        
        // MQTT configuration
        if let Ok(username) = std::env::var("MQTT_USERNAME") {
            config.mqtt_username = username;
        }
        
        if let Ok(password) = std::env::var("MQTT_PASSWORD") {
            config.mqtt_password = password;
        }
        
        if let Ok(host) = std::env::var("MQTT_HOST") {
            config.mqtt_host = host;
        }
        
        if let Ok(port) = std::env::var("MQTT_PORT") {
            if let Ok(port_num) = port.parse::<u16>() {
                config.mqtt_port = port_num;
            }
        }
        
        if let Ok(use_tls) = std::env::var("MQTT_USE_TLS") {
            config.mqtt_use_tls = use_tls.to_lowercase() == "true" || use_tls == "1";
        }
        
        // Reticulum configuration
        if let Ok(server) = std::env::var("RETICULUM_SERVER") {
            config.reticulum_server = server;
        }
        
        // Channels configuration
        if let Ok(channels_env) = std::env::var("MESHTASTIC_CHANNELS") {
            for channel_entry in channels_env.split(',') {
                let parts: Vec<&str> = channel_entry.split(':').collect();
                if parts.len() == 2 {
                    config.initial_channels.insert(
                        parts[0].trim().to_string(),
                        parts[1].trim().to_string(),
                    );
                }
            }
        }
        
        // GUI configuration
        if let Ok(bind_addr) = std::env::var("GUI_BIND_ADDRESS") {
            config.gui_bind_address = bind_addr;
        }
        
        if let Ok(port) = std::env::var("GUI_PORT") {
            if let Ok(port_num) = port.parse::<u16>() {
                config.gui_port = port_num;
            }
        }
        
        // Logging configuration
        if let Ok(log_to_console) = std::env::var("LOG_TO_CONSOLE") {
            config.log_to_console = log_to_console.to_lowercase() == "true" || log_to_console == "1";
        }
        
        if let Ok(log_to_file) = std::env::var("LOG_TO_FILE") {
            config.log_to_file = log_to_file.to_lowercase() == "true" || log_to_file == "1";
        }
        
        if let Ok(log_file_path) = std::env::var("LOG_FILE_PATH") {
            config.log_file_path = log_file_path;
        }
        
        if let Ok(enable_audit_logging) = std::env::var("ENABLE_AUDIT_LOGGING") {
            config.enable_audit_logging = enable_audit_logging.to_lowercase() == "true" || enable_audit_logging == "1";
        }
        
        if let Ok(audit_log_file_path) = std::env::var("AUDIT_LOG_FILE_PATH") {
            config.audit_log_file_path = audit_log_file_path;
        }
        
        // Webhook configuration
        config.webhook_configs = crate::webhook::WebhookManager::from_env();
        
        config
    }
    
    /// Validate configuration for security issues
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        
        // MQTT credentials are optional - bridge can run without MQTT
        // Only validate if user explicitly tries to use MQTT by setting a host
        if !self.mqtt_host.is_empty() && (self.mqtt_username.is_empty() || self.mqtt_password.is_empty()) {
            errors.push("MQTT credentials are empty. Please set MQTT_USERNAME and MQTT_PASSWORD environment variables if you want to use MQTT.".to_string());
        }
        
        // Warn about using non-TLS connection only if MQTT is being used
        if !self.mqtt_host.is_empty() && !self.mqtt_use_tls {
            errors.push("MQTT connection is not using TLS. Consider setting MQTT_USE_TLS=true for secure communication.".to_string());
        }
        
        // Check for weak PSKs in initial channels
        for (channel, psk) in &self.initial_channels {
            if psk.len() < 16 { // Minimum reasonable PSK length in base64
                errors.push(format!("Channel '{}' has a very short PSK ({} chars). Use a longer, cryptographically secure PSK.", channel, psk.len()));
            }
            if psk == "AQ==" || psk.is_empty() {
                errors.push(format!("Channel '{}' is using a weak or empty PSK. This is insecure.", channel));
            }
        }
        
        // Validate reticulum server format
        if !self.reticulum_server.contains(':') {
            errors.push(format!("Reticulum server '{}' should be in format 'host:port'", self.reticulum_server));
        }
        
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
    
    /// Get MQTT connection URL
    pub fn mqtt_url(&self) -> String {
        let protocol = if self.mqtt_use_tls { "mqtts" } else { "mqtt" };
        format!("{}://{}:{}", protocol, self.mqtt_host, self.mqtt_port)
    }
    
    /// Get GUI bind address with port
    pub fn gui_bind_addr(&self) -> String {
        format!("{}:{}", self.gui_bind_address, self.gui_port)
    }
    
    /// Check if configuration has any initial channels
    pub fn has_initial_channels(&self) -> bool {
        !self.initial_channels.is_empty()
    }
}

/// Load configuration from environment variables
pub fn load_config() -> Config {
    Config::from_env()
}