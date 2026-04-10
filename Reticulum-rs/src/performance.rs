//! Performance optimization utilities for Reticulum-rs
//!
//! This module provides performance monitoring, optimization hints,
//! and benchmarking utilities.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Performance metrics collector
pub struct PerformanceMetrics {
    /// Counters for different operations
    counters: HashMap<String, AtomicU64>,
    
    /// Timers for operation durations
    timers: HashMap<String, Vec<Duration>>,
    
    /// Maximum number of samples to keep per timer
    max_samples: usize,
}

impl PerformanceMetrics {
    /// Create a new performance metrics collector
    pub fn new(max_samples: usize) -> Self {
        Self {
            counters: HashMap::new(),
            timers: HashMap::new(),
            max_samples,
        }
    }
    
    /// Increment a counter
    pub fn increment(&self, name: &str) {
        let counter = self.counters
            .entry(name.to_string())
            .or_insert_with(|| AtomicU64::new(0));
        counter.fetch_add(1, Ordering::Relaxed);
    }
    
    /// Record a timer measurement
    pub fn record_timer(&mut self, name: &str, duration: Duration) {
        let samples = self.timers
            .entry(name.to_string())
            .or_insert_with(Vec::new);
        
        samples.push(duration);
        
        // Keep only the most recent samples
        if samples.len() > self.max_samples {
            samples.remove(0);
        }
    }
    
    /// Get counter value
    pub fn get_counter(&self, name: &str) -> Option<u64> {
        self.counters.get(name).map(|c| c.load(Ordering::Relaxed))
    }
    
    /// Get timer statistics
    pub fn get_timer_stats(&self, name: &str) -> Option<TimerStats> {
        self.timers.get(name).map(|samples| {
            if samples.is_empty() {
                return TimerStats::default();
            }
            
            let mut min = samples[0];
            let mut max = samples[0];
            let mut total = Duration::from_secs(0);
            
            for &duration in samples {
                if duration < min {
                    min = duration;
                }
                if duration > max {
                    max = duration;
                }
                total += duration;
            }
            
            let avg = total / samples.len() as u32;
            
            TimerStats {
                count: samples.len(),
                min,
                max,
                avg,
                total,
            }
        })
    }
    
    /// Reset all metrics
    pub fn reset(&mut self) {
        self.counters.clear();
        self.timers.clear();
    }
    
    /// Get all counter names
    pub fn counter_names(&self) -> Vec<String> {
        self.counters.keys().cloned().collect()
    }
    
    /// Get all timer names
    pub fn timer_names(&self) -> Vec<String> {
        self.timers.keys().cloned().collect()
    }
}

/// Timer statistics
#[derive(Debug, Clone, Copy)]
pub struct TimerStats {
    /// Number of samples
    pub count: usize,
    
    /// Minimum duration
    pub min: Duration,
    
    /// Maximum duration
    pub max: Duration,
    
    /// Average duration
    pub avg: Duration,
    
    /// Total duration
    pub total: Duration,
}

impl Default for TimerStats {
    fn default() -> Self {
        Self {
            count: 0,
            min: Duration::from_secs(0),
            max: Duration::from_secs(0),
            avg: Duration::from_secs(0),
            total: Duration::from_secs(0),
        }
    }
}

/// Performance timer for measuring operation durations
pub struct PerfTimer {
    name: String,
    start: Instant,
    metrics: Option<Arc<std::sync::Mutex<PerformanceMetrics>>>,
}

impl PerfTimer {
    /// Start a new performance timer
    pub fn start(name: &str) -> Self {
        Self {
            name: name.to_string(),
            start: Instant::now(),
            metrics: None,
        }
    }
    
    /// Start a timer with metrics collection
    pub fn start_with_metrics(name: &str, metrics: Arc<std::sync::Mutex<PerformanceMetrics>>) -> Self {
        Self {
            name: name.to_string(),
            start: Instant::now(),
            metrics: Some(metrics),
        }
    }
    
    /// Stop the timer and return the duration
    pub fn stop(self) -> Duration {
        let duration = self.start.elapsed();
        
        if let Some(metrics) = self.metrics {
            if let Ok(mut metrics) = metrics.lock() {
                metrics.record_timer(&self.name, duration);
            }
        }
        
        duration
    }
}

/// Performance optimization hints
pub struct OptimizationHints {
    /// Enable/disable specific optimizations
    pub enabled: HashMap<String, bool>,
    
    /// Optimization parameters
    pub parameters: HashMap<String, String>,
}

impl OptimizationHints {
    /// Create new optimization hints
    pub fn new() -> Self {
        Self {
            enabled: HashMap::new(),
            parameters: HashMap::new(),
        }
    }
    
    /// Enable an optimization
    pub fn enable(&mut self, name: &str) {
        self.enabled.insert(name.to_string(), true);
    }
    
    /// Disable an optimization
    pub fn disable(&mut self, name: &str) {
        self.enabled.insert(name.to_string(), false);
    }
    
    /// Check if an optimization is enabled
    pub fn is_enabled(&self, name: &str) -> bool {
        self.enabled.get(name).copied().unwrap_or(false)
    }
    
    /// Set an optimization parameter
    pub fn set_parameter(&mut self, name: &str, value: &str) {
        self.parameters.insert(name.to_string(), value.to_string());
    }
    
    /// Get an optimization parameter
    pub fn get_parameter(&self, name: &str) -> Option<&str> {
        self.parameters.get(name).map(|s| s.as_str())
    }
}

/// Common performance optimizations for Reticulum
pub mod optimizations {
    /// Enable packet batching for better throughput
    pub const PACKET_BATCHING: &str = "packet_batching";
    
    /// Enable connection pooling for TCP interfaces
    pub const CONNECTION_POOLING: &str = "connection_pooling";
    
    /// Enable buffer reuse to reduce allocations
    pub const BUFFER_REUSE: &str = "buffer_reuse";
    
    /// Enable compression for large packets
    pub const COMPRESSION: &str = "compression";
    
    /// Enable asynchronous crypto operations
    pub const ASYNC_CRYPTO: &str = "async_crypto";
    
    /// Enable zero-copy packet processing
    pub const ZERO_COPY: &str = "zero_copy";
}

/// Memory pool for buffer reuse
pub struct BufferPool {
    pool: std::sync::Mutex<Vec<Vec<u8>>>,
    buffer_size: usize,
    max_pool_size: usize,
}

impl BufferPool {
    /// Create a new buffer pool
    pub fn new(buffer_size: usize, max_pool_size: usize) -> Self {
        Self {
            pool: std::sync::Mutex::new(Vec::new()),
            buffer_size,
            max_pool_size,
        }
    }
    
    /// Acquire a buffer from the pool
    pub fn acquire(&self) -> Vec<u8> {
        let mut pool = self.pool.lock().unwrap();
        
        if let Some(mut buffer) = pool.pop() {
            buffer.clear();
            buffer
        } else {
            vec![0u8; self.buffer_size]
        }
    }
    
    /// Release a buffer back to the pool
    pub fn release(&self, mut buffer: Vec<u8>) {
        let mut pool = self.pool.lock().unwrap();
        
        if pool.len() < self.max_pool_size {
            buffer.clear();
            pool.push(buffer);
        }
        // If pool is full, buffer will be dropped
    }
    
    /// Get current pool size
    pub fn size(&self) -> usize {
        self.pool.lock().unwrap().len()
    }
    
    /// Clear the pool
    pub fn clear(&self) {
        self.pool.lock().unwrap().clear();
    }
}

/// Performance-aware configuration
pub struct PerformanceConfig {
    /// Thread pool size for async operations
    pub thread_pool_size: usize,
    
    /// Maximum concurrent operations
    pub max_concurrent_ops: usize,
    
    /// Buffer sizes for different operations
    pub buffer_sizes: BufferSizes,
    
    /// Cache sizes for various caches
    pub cache_sizes: CacheSizes,
    
    /// Timeout configurations
    pub timeouts: TimeoutConfig,
}

/// Buffer size configuration
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

/// Cache size configuration
pub struct CacheSizes {
    /// Packet cache size
    pub packet_cache: usize,
    
    /// Route cache size
    pub route_cache: usize,
    
    /// Connection cache size
    pub connection_cache: usize,
    
    /// DNS cache size
    pub dns_cache: usize,
}

/// Timeout configuration
pub struct TimeoutConfig {
    /// Connection timeout
    pub connection_timeout: Duration,
    
    /// Read timeout
    pub read_timeout: Duration,
    
    /// Write timeout
    pub write_timeout: Duration,
    
    /// Keep-alive timeout
    pub keep_alive_timeout: Duration,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            thread_pool_size: num_cpus::get(),
            max_concurrent_ops: 1000,
            buffer_sizes: BufferSizes {
                packet_buffer: 4096,
                network_buffer: 8192,
                crypto_buffer: 1024,
                file_buffer: 65536,
            },
            cache_sizes: CacheSizes {
                packet_cache: 1024,
                route_cache: 512,
                connection_cache: 256,
                dns_cache: 128,
            },
            timeouts: TimeoutConfig {
                connection_timeout: Duration::from_secs(30),
                read_timeout: Duration::from_secs(60),
                write_timeout: Duration::from_secs(60),
                keep_alive_timeout: Duration::from_secs(300),
            },
        }
    }
}

/// Performance monitor for real-time performance tracking
pub struct PerformanceMonitor {
    metrics: Arc<std::sync::Mutex<PerformanceMetrics>>,
    start_time: Instant,
}

impl PerformanceMonitor {
    /// Create a new performance monitor
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(std::sync::Mutex::new(PerformanceMetrics::new(1000))),
            start_time: Instant::now(),
        }
    }
    
    /// Get metrics reference
    pub fn metrics(&self) -> Arc<std::sync::Mutex<PerformanceMetrics>> {
        self.metrics.clone()
    }
    
    /// Get uptime
    pub fn uptime(&self) -> Duration {
        self.start_time.elapsed()
    }
    
    /// Generate performance report
    pub fn generate_report(&self) -> PerformanceReport {
        let metrics = self.metrics.lock().unwrap();
        let uptime = self.uptime();
        
        let mut report = PerformanceReport {
            uptime,
            counters: HashMap::new(),
            timers: HashMap::new(),
        };
        
        // Collect counter values
        for name in metrics.counter_names() {
            if let Some(value) = metrics.get_counter(&name) {
                report.counters.insert(name, value);
            }
        }
        
        // Collect timer statistics
        for name in metrics.timer_names() {
            if let Some(stats) = metrics.get_timer_stats(&name) {
                report.timers.insert(name, stats);
            }
        }
        
        report
    }
}

/// Performance report
pub struct PerformanceReport {
    /// System uptime
    pub uptime: Duration,
    
    /// Counter values
    pub counters: HashMap<String, u64>,
    
    /// Timer statistics
    pub timers: HashMap<String, TimerStats>,
}

impl PerformanceReport {
    /// Format report as string
    pub fn format(&self) -> String {
        let mut output = String::new();
        
        output.push_str(&format!("Performance Report\n"));
        output.push_str(&format!("Uptime: {:?}\n\n", self.uptime));
        
        output.push_str("Counters:\n");
        for (name, value) in &self.counters {
            output.push_str(&format!("  {}: {}\n", name, value));
        }
        
        output.push_str("\nTimers:\n");
        for (name, stats) in &self.timers {
            output.push_str(&format!("  {}:\n", name));
            output.push_str(&format!("    Count: {}\n", stats.count));
            output.push_str(&format!("    Min: {:?}\n", stats.min));
            output.push_str(&format!("    Max: {:?}\n", stats.max));
            output.push_str(&format!("    Avg: {:?}\n", stats.avg));
            output.push_str(&format!("    Total: {:?}\n", stats.total));
        }
        
        output
    }
}