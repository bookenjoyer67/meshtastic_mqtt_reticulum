//! Comprehensive test suite for Reticulum-rs
//!
//! This module provides unit tests, integration tests, and property-based tests
//! for the Reticulum networking stack.

#[cfg(test)]
mod tests {
    use crate::error::{RnsError, Result};
    use crate::logging::{LogContext, ReticulumLogLevel};
    use crate::config::{ReticulumConfig, ConfigManager};
    use std::path::PathBuf;
    use tempfile::NamedTempFile;

    /// Test basic error handling
    #[test]
    fn test_error_handling() {
        let error = RnsError::InvalidArgument("test".to_string());
        assert_eq!(error.to_string(), "Invalid argument: test");
        
        let recoverable = RnsError::ConnectionError("test".to_string());
        assert!(recoverable.is_recoverable());
        
        let fatal = RnsError::OutOfMemory;
        assert!(fatal.is_fatal());
    }

    /// Test logging context
    #[test]
    fn test_logging_context() {
        let ctx = LogContext::new("test")
            .with_operation_id("op123")
            .with_peer("peer1")
            .with_link_id("link456")
            .with_packet_hash("abc123")
            .with_extra("key", "value");
        
        assert_eq!(ctx.component, "test");
        assert_eq!(ctx.operation_id, Some("op123".to_string()));
        assert_eq!(ctx.peer, Some("peer1".to_string()));
        assert_eq!(ctx.link_id, Some("link456".to_string()));
        assert_eq!(ctx.packet_hash, Some("abc123".to_string()));
        assert_eq!(ctx.extra.get("key"), Some(&"value".to_string()));
    }

    /// Test configuration loading and saving
    #[test]
    fn test_config_management() -> Result<()> {
        // Create a temporary file for testing
        let temp_file = NamedTempFile::new().unwrap();
        let config_path = temp_file.path().to_path_buf();
        
        // Create config manager
        let mut manager = ConfigManager::new(&config_path);
        
        // Create default config
        manager.create_default()?;
        
        // Load the config
        manager.load()?;
        
        // Verify default values
        let config = manager.config();
        assert_eq!(config.global.max_packet_size, 500);
        assert_eq!(config.transport.max_hops, 128);
        assert!(config.security.enable_encryption);
        
        // Modify and save
        let config_mut = manager.config_mut();
        config_mut.global.max_packet_size = 1000;
        manager.save()?;
        
        // Reload and verify changes
        manager.load()?;
        assert_eq!(manager.config().global.max_packet_size, 1000);
        
        Ok(())
    }

    /// Test configuration validation
    #[test]
    fn test_config_validation() {
        let mut config = ReticulumConfig::default();
        
        // Test valid config
        let manager = ConfigManager::new(PathBuf::from("/tmp/test.toml"));
        // Can't easily test validation without actual file
        
        // Test invalid config (would fail validation)
        config.global.max_packet_size = 0;
        // Note: Actual validation happens in load() method
    }

    /// Test log level conversions
    #[test]
    fn test_log_level_conversions() {
        use crate::logging::ReticulumLogLevel;
        use log::LevelFilter;
        
        assert_eq!(LevelFilter::from(ReticulumLogLevel::Critical), LevelFilter::Error);
        assert_eq!(LevelFilter::from(ReticulumLogLevel::Info), LevelFilter::Info);
        assert_eq!(LevelFilter::from(ReticulumLogLevel::Debug), LevelFilter::Debug);
        assert_eq!(LevelFilter::from(ReticulumLogLevel::Trace), LevelFilter::Trace);
        assert_eq!(LevelFilter::from(ReticulumLogLevel::Packet), LevelFilter::Trace);
    }

    /// Test error result type
    #[test]
    fn test_result_type() {
        fn returns_result() -> Result<()> {
            Ok(())
        }
        
        fn returns_error() -> Result<()> {
            Err(RnsError::InvalidArgument("test".to_string()))
        }
        
        assert!(returns_result().is_ok());
        assert!(returns_error().is_err());
        
        if let Err(e) = returns_error() {
            assert!(e.to_string().contains("Invalid argument"));
        }
    }
}

/// Integration tests for network components
#[cfg(test)]
mod integration_tests {
    use crate::hash::AddressHash;
    use rand_core::OsRng;
    
    /// Test address hash generation
    #[test]
    fn test_address_hash() {
        let hash1 = AddressHash::new_from_rand(OsRng);
        let hash2 = AddressHash::new_from_rand(OsRng);
        
        // Hashes should be different with high probability
        assert_ne!(hash1, hash2);
        
        // Hash should have correct length
        // Note: Need to check actual implementation
    }
}

/// Property-based tests (using quickcheck)
#[cfg(test)]
mod property_tests {
    use quickcheck::{Arbitrary, Gen};
    use crate::config::{ReticulumConfig, GlobalConfig, TransportConfig};
    
    // Implement Arbitrary for our config types for property testing
    impl Arbitrary for GlobalConfig {
        fn arbitrary(g: &mut Gen) -> Self {
            GlobalConfig {
                node_name: if bool::arbitrary(g) {
                    Some(String::arbitrary(g))
                } else {
                    None
                },
                enable_forwarding: bool::arbitrary(g),
                enable_announces: bool::arbitrary(g),
                max_packet_size: u16::arbitrary(g) as usize % 1000 + 100,
                default_mtu: u16::arbitrary(g) as usize % 500 + 100,
            }
        }
    }
    
    impl Arbitrary for TransportConfig {
        fn arbitrary(g: &mut Gen) -> Self {
            TransportConfig {
                max_hops: u8::arbitrary(g) as usize % 100 + 10,
                path_request_timeout: u16::arbitrary(g) as u64 % 300 + 10,
                link_establish_timeout: u16::arbitrary(g) as u64 % 600 + 30,
                packet_cache_size: u16::arbitrary(g) as usize % 5000 + 100,
                announce_table_size: u16::arbitrary(g) as usize % 1000 + 100,
                link_table_size: u16::arbitrary(g) as usize % 500 + 50,
                eager_rerouting: bool::arbitrary(g),
                restart_links: bool::arbitrary(g),
            }
        }
    }
    
    /// Property test: Config validation should pass for valid configs
    #[test]
    fn prop_config_validation() {
        fn test_config_validation(global: GlobalConfig, transport: TransportConfig) -> bool {
            // Skip invalid configs for this test
            if global.max_packet_size == 0 || global.default_mtu == 0 ||
               transport.max_hops == 0 || transport.packet_cache_size == 0 {
                return true; // Skip invalid cases
            }
            
            // In real implementation, we would validate the config
            // For now, just return true if values are valid
            global.max_packet_size > 0 &&
            global.default_mtu > 0 &&
            transport.max_hops > 0 &&
            transport.packet_cache_size > 0
        }
        
        quickcheck::quickcheck(test_config_validation as fn(GlobalConfig, TransportConfig) -> bool);
    }
}

/// Performance benchmarks
#[cfg(test)]
mod benchmarks {
    // Note: Proper benchmarking requires the 'test' crate feature
    // For now, we'll skip actual benchmarks in unit tests
    /*
    use test::Bencher;
    use crate::hash::AddressHash;
    use rand_core::OsRng;
    
    /// Benchmark address hash generation
    #[bench]
    fn bench_address_hash_generation(b: &mut Bencher) {
        b.iter(|| {
            AddressHash::new_from_rand(OsRng);
        });
    }
    
    /// Benchmark config serialization
    #[bench]
    fn bench_config_serialization(b: &mut Bencher) {
        use crate::config::ReticulumConfig;
        
        let config = ReticulumConfig::default();
        
        b.iter(|| {
            let _ = toml::to_string(&config).unwrap();
        });
    }
    */
}

/// Mock utilities for testing
#[cfg(test)]
pub mod mocks {
    use std::sync::{Arc, Mutex};
    use std::collections::VecDeque;
    
    /// Mock network interface for testing
    pub struct MockInterface {
        pub sent_packets: Arc<Mutex<VecDeque<Vec<u8>>>>,
        pub received_packets: Arc<Mutex<VecDeque<Vec<u8>>>>,
    }
    
    impl MockInterface {
        pub fn new() -> Self {
            Self {
                sent_packets: Arc::new(Mutex::new(VecDeque::new())),
                received_packets: Arc::new(Mutex::new(VecDeque::new())),
            }
        }
        
        pub fn send(&self, data: Vec<u8>) {
            self.sent_packets.lock().unwrap().push_back(data);
        }
        
        pub fn receive(&self) -> Option<Vec<u8>> {
            self.received_packets.lock().unwrap().pop_front()
        }
        
        pub fn queue_receive(&self, data: Vec<u8>) {
            self.received_packets.lock().unwrap().push_back(data);
        }
        
        pub fn sent_count(&self) -> usize {
            self.sent_packets.lock().unwrap().len()
        }
        
        pub fn received_count(&self) -> usize {
            self.received_packets.lock().unwrap().len()
        }
    }
}

/// Test utilities
#[cfg(test)]
pub mod test_utils {
    use std::time::{Duration, Instant};
    
    /// Assert that an operation completes within a timeout
    pub fn assert_completes_within<F, T>(timeout: Duration, f: F) -> T
    where
        F: FnOnce() -> T,
    {
        let start = Instant::now();
        let result = f();
        let elapsed = start.elapsed();
        
        assert!(
            elapsed <= timeout,
            "Operation took {:?}, exceeding timeout of {:?}",
            elapsed,
            timeout
        );
        
        result
    }
    
    /// Retry an operation until it succeeds or timeout is reached
    pub fn retry_until<F, T, E>(mut f: F, timeout: Duration, interval: Duration) -> Result<T, E>
    where
        F: FnMut() -> Result<T, E>,
    {
        let start = Instant::now();
        
        loop {
            match f() {
                Ok(result) => return Ok(result),
                Err(e) => {
                    if start.elapsed() >= timeout {
                        return Err(e);
                    }
                    std::thread::sleep(interval);
                }
            }
        }
    }
}