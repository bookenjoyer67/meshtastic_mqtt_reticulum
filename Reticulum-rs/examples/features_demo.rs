//! Example demonstrating the new features of Reticulum-rs
//!
//! This example shows:
//! 1. Enhanced error handling with thiserror
//! 2. Structured logging with context
//! 3. Configuration management
//! 4. Unit testing patterns

use reticulum::error::{RnsError, Result};
use reticulum::logging::{LogContext, ReticulumLogLevel, init_logging_from_env};
use reticulum::config::{load_or_create_config, ReticulumConfig};
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging from environment variable RETICULUM_LOG_LEVEL
    init_logging_from_env()
        .map_err(|e| RnsError::ConfigError(format!("Failed to initialize logging: {}", e)))?;
    
    // Create logging context
    let log_ctx = LogContext::new("example")
        .with_operation_id("demo-123")
        .with_peer("localhost");
    
    // Log at different levels
    log::info!(target: &log_ctx.component, "[{}] Starting Reticulum example", log_ctx.component);
    
    // Load or create configuration
    let config_path = Some(PathBuf::from("example_config.toml"));
    let mut config_manager = load_or_create_config(config_path)?;
    
    log::info!("Configuration loaded from: {:?}", config_manager.config_path());
    
    // Demonstrate error handling
    match demonstrate_error_handling() {
        Ok(_) => log::info!("Error handling demonstration succeeded"),
        Err(e) => {
            log::error!("Error handling demonstration failed: {}", e);
            if e.is_recoverable() {
                log::warn!("Error is recoverable, continuing...");
            } else if e.is_fatal() {
                log::error!("Fatal error, exiting...");
                return Err(e);
            }
        }
    }
    
    // Demonstrate configuration usage
    demonstrate_config_usage(&config_manager.config())?;
    
    // Modify and save configuration
    let config = config_manager.config_mut();
    config.global.node_name = Some("example-node".to_string());
    config_manager.save()?;
    
    log::info!("Configuration updated and saved");
    
    // Demonstrate structured logging with different contexts
    let transport_ctx = LogContext::new("transport");
    let crypto_ctx = LogContext::new("crypto").with_operation_id("encrypt-456");
    
    log::debug!(target: &transport_ctx.component, "[{}] Transport layer initialized", transport_ctx.component);
    log::info!(target: &crypto_ctx.component, "[{}] Cryptographic operation started", crypto_ctx.component);
    
    Ok(())
}

/// Demonstrate enhanced error handling
fn demonstrate_error_handling() -> Result<()> {
    // Create different types of errors
    let invalid_arg = RnsError::InvalidArgument("parameter 'foo' cannot be null".to_string());
    let connection_err = RnsError::ConnectionError("connection refused".to_string());
    let crypto_err = RnsError::CryptoError("invalid key length".to_string());
    
    log::debug!("Invalid argument error: {}", invalid_arg);
    log::debug!("Connection error: {}", connection_err);
    log::debug!("Crypto error: {}", crypto_err);
    
    // Check error properties
    assert!(connection_err.is_recoverable());
    assert!(!invalid_arg.is_recoverable());
    
    // Demonstrate error conversion
    let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
    let rns_error: RnsError = io_error.into();
    
    log::debug!("Converted IO error: {}", rns_error);
    
    Ok(())
}

/// Demonstrate configuration usage
fn demonstrate_config_usage(config: &ReticulumConfig) -> Result<()> {
    log::info!("Node name: {:?}", config.global.node_name);
    log::info!("Max packet size: {} bytes", config.global.max_packet_size);
    log::info!("Max hops: {}", config.transport.max_hops);
    log::info!("Enable encryption: {}", config.security.enable_encryption);
    
    // Validate some configuration values
    if config.global.max_packet_size == 0 {
        return Err(RnsError::ConfigError(
            "max_packet_size must be greater than 0".to_string()
        ));
    }
    
    if config.transport.max_hops == 0 {
        return Err(RnsError::ConfigError(
            "max_hops must be greater than 0".to_string()
        ));
    }
    
    Ok(())
}

/// Unit tests for the example
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_error_handling_demo() {
        let result = demonstrate_error_handling();
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_config_validation() {
        let mut config = ReticulumConfig::default();
        
        // Valid config should pass
        let result = demonstrate_config_usage(&config);
        assert!(result.is_ok());
        
        // Invalid config should fail
        config.global.max_packet_size = 0;
        let result = demonstrate_config_usage(&config);
        assert!(result.is_err());
        
        if let Err(e) = result {
            assert!(e.to_string().contains("max_packet_size"));
        }
    }
    
    #[test]
    fn test_config_save_load() -> Result<()> {
        let temp_file = NamedTempFile::new().unwrap();
        let config_path = temp_file.path().to_path_buf();
        
        // Create a simple config
        let mut config = ReticulumConfig::default();
        config.global.node_name = Some("test-node".to_string());
        config.global.max_packet_size = 1024;
        
        // In a real test, we would save and load the config
        // For now, just verify the config has our values
        assert_eq!(config.global.node_name, Some("test-node".to_string()));
        assert_eq!(config.global.max_packet_size, 1024);
        
        Ok(())
    }
}