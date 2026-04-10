//! Network metrics measurement for TCP connections
//!
//! This module provides utilities for measuring network performance metrics
//! such as latency (round-trip time), packet loss, jitter, and bandwidth.

use std::time::{Duration, Instant};
use tokio::net::TcpStream;
use tokio::time::timeout;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use anyhow::{Result, anyhow};
use log::{warn, debug};
use serde::{Serialize, Deserialize};

/// Network metrics for a connection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkMetrics {
    /// Round-trip time in milliseconds
    pub latency_ms: f32,
    /// Packet loss percentage (0-100)
    pub packet_loss_percent: f32,
    /// Jitter (standard deviation of latency) in milliseconds
    pub jitter_ms: f32,
    /// Available bandwidth in kilobits per second (estimated)
    pub bandwidth_kbps: f32,
    /// Connection quality score (0-100)
    pub quality_score: u8,
    /// Timestamp of last measurement
    pub last_measurement: chrono::DateTime<chrono::Utc>,
}

impl Default for NetworkMetrics {
    fn default() -> Self {
        NetworkMetrics {
            latency_ms: 0.0,
            packet_loss_percent: 0.0,
            jitter_ms: 0.0,
            bandwidth_kbps: 0.0,
            quality_score: 0,
            last_measurement: chrono::Utc::now(),
        }
    }
}

/// Network metrics collector
pub struct NetworkMetricsCollector {
    target_host: String,
    target_port: u16,
    measurement_interval: Duration,
    max_samples: usize,
    latency_samples: Vec<f32>,
    packet_loss_samples: Vec<bool>, // true = packet received, false = packet lost
}

impl NetworkMetricsCollector {
    /// Create a new network metrics collector
    pub fn new(target_host: String, target_port: u16) -> Self {
        NetworkMetricsCollector {
            target_host,
            target_port,
            measurement_interval: Duration::from_secs(5), // Measure every 5 seconds
            max_samples: 20, // Keep last 20 samples for averaging
            latency_samples: Vec::new(),
            packet_loss_samples: Vec::new(),
        }
    }
    
    /// Set measurement interval
    pub fn with_interval(mut self, interval: Duration) -> Self {
        self.measurement_interval = interval;
        self
    }
    
    /// Set maximum number of samples to keep
    pub fn with_max_samples(mut self, max_samples: usize) -> Self {
        self.max_samples = max_samples;
        self
    }
    
    /// Perform a single latency measurement (ping)
    pub async fn measure_latency(&mut self) -> Result<f32> {
        let start = Instant::now();
        
        // Try to connect to the target
        let connect_result = timeout(
            Duration::from_secs(2),
            TcpStream::connect((self.target_host.as_str(), self.target_port))
        ).await;
        
        let latency = start.elapsed().as_secs_f32() * 1000.0; // Convert to ms
        
        match connect_result {
            Ok(Ok(_stream)) => {
                // Connection successful
                self.latency_samples.push(latency);
                self.packet_loss_samples.push(true);
                
                // Trim samples if needed
                if self.latency_samples.len() > self.max_samples {
                    self.latency_samples.remove(0);
                }
                if self.packet_loss_samples.len() > self.max_samples {
                    self.packet_loss_samples.remove(0);
                }
                
                debug!("Latency measurement: {} ms to {}:{}", latency, self.target_host, self.target_port);
                Ok(latency)
            }
            Ok(Err(e)) => {
                // Connection failed
                self.packet_loss_samples.push(false);
                if self.packet_loss_samples.len() > self.max_samples {
                    self.packet_loss_samples.remove(0);
                }
                
                warn!("Connection failed to {}:{}: {}", self.target_host, self.target_port, e);
                Err(anyhow!("Connection failed: {}", e))
            }
            Err(_) => {
                // Timeout
                self.packet_loss_samples.push(false);
                if self.packet_loss_samples.len() > self.max_samples {
                    self.packet_loss_samples.remove(0);
                }
                
                warn!("Connection timeout to {}:{}", self.target_host, self.target_port);
                Err(anyhow!("Connection timeout"))
            }
        }
    }
    
    /// Perform a bandwidth test (simple estimation)
    pub async fn estimate_bandwidth(&self) -> Result<f32> {
        // Simple bandwidth estimation by measuring time to send/receive small amount of data
        let test_data_size = 1024; // 1KB test data
        let test_data = vec![0u8; test_data_size];
        
        let start = Instant::now();
        
        match timeout(
            Duration::from_secs(3),
            self.perform_bandwidth_test(&test_data)
        ).await {
            Ok(Ok(_)) => {
                let elapsed = start.elapsed().as_secs_f32();
                // Calculate bandwidth in kbps (kilobits per second)
                let bandwidth_kbps = (test_data_size as f32 * 8.0 / 1000.0) / elapsed;
                Ok(bandwidth_kbps)
            }
            Ok(Err(e)) => {
                warn!("Bandwidth test failed: {}", e);
                Ok(0.0)
            }
            Err(_) => {
                warn!("Bandwidth test timeout");
                Ok(0.0)
            }
        }
    }
    
    /// Perform actual bandwidth test
    async fn perform_bandwidth_test(&self, test_data: &[u8]) -> Result<()> {
        let mut stream = TcpStream::connect((self.target_host.as_str(), self.target_port)).await?;
        
        // Send test data
        stream.writable().await?;
        stream.try_write(test_data)?;
        
        // Read response (echo)
        let mut buffer = vec![0u8; test_data.len()];
        stream.readable().await?;
        let _bytes_read = stream.try_read(&mut buffer)?;
        
        Ok(())
    }
    
    /// Calculate current network metrics
    pub async fn calculate_metrics(&mut self) -> NetworkMetrics {
        // Measure latency
        let latency_result = self.measure_latency().await;
        let current_latency = match latency_result {
            Ok(latency) => latency,
            Err(_) => {
                // Use last known latency or high value
                self.latency_samples.last().copied().unwrap_or(1000.0)
            }
        };
        
        // Calculate average latency
        let avg_latency = if !self.latency_samples.is_empty() {
            self.latency_samples.iter().sum::<f32>() / self.latency_samples.len() as f32
        } else {
            current_latency
        };
        
        // Calculate jitter (standard deviation of latency)
        let jitter = if self.latency_samples.len() >= 2 {
            let mean = avg_latency;
            let variance = self.latency_samples.iter()
                .map(|&x| (x - mean).powi(2))
                .sum::<f32>() / self.latency_samples.len() as f32;
            variance.sqrt()
        } else {
            0.0
        };
        
        // Calculate packet loss
        let packet_loss_percent = if !self.packet_loss_samples.is_empty() {
            let lost_packets = self.packet_loss_samples.iter().filter(|&&received| !received).count();
            (lost_packets as f32 / self.packet_loss_samples.len() as f32) * 100.0
        } else {
            0.0
        };
        
        // Estimate bandwidth
        let bandwidth = match self.estimate_bandwidth().await {
            Ok(bw) => bw,
            Err(_) => 0.0,
        };
        
        // Calculate quality score (0-100)
        let quality_score = self.calculate_quality_score(avg_latency, packet_loss_percent, jitter);
        
        NetworkMetrics {
            latency_ms: avg_latency,
            packet_loss_percent,
            jitter_ms: jitter,
            bandwidth_kbps: bandwidth,
            quality_score,
            last_measurement: chrono::Utc::now(),
        }
    }
    
    /// Calculate quality score based on metrics
    fn calculate_quality_score(&self, latency: f32, packet_loss: f32, jitter: f32) -> u8 {
        // Weighted scoring:
        // - Latency: 40% weight (lower is better)
        // - Packet loss: 40% weight (lower is better)
        // - Jitter: 20% weight (lower is better)
        
        let latency_score = if latency < 50.0 {
            100.0
        } else if latency < 100.0 {
            80.0
        } else if latency < 200.0 {
            60.0
        } else if latency < 500.0 {
            40.0
        } else {
            20.0
        };
        
        let packet_loss_score = if packet_loss < 1.0 {
            100.0
        } else if packet_loss < 5.0 {
            80.0
        } else if packet_loss < 10.0 {
            60.0
        } else if packet_loss < 20.0 {
            40.0
        } else {
            20.0
        };
        
        let jitter_score = if jitter < 10.0 {
            100.0
        } else if jitter < 30.0 {
            80.0
        } else if jitter < 50.0 {
            60.0
        } else if jitter < 100.0 {
            40.0
        } else {
            20.0
        };
        
        let total_score = (latency_score * 0.4 + packet_loss_score * 0.4 + jitter_score * 0.2) as u8;
        total_score.min(100)
    }
    
    /// Start continuous metrics collection
    pub async fn start_collection(&mut self) -> tokio::sync::mpsc::Receiver<NetworkMetrics> {
        let (tx, rx) = tokio::sync::mpsc::channel(10);
        
        let mut collector = self.clone();
        
        tokio::spawn(async move {
            loop {
                let metrics = collector.calculate_metrics().await;
                
                // Send metrics if there's capacity
                if tx.capacity() > 0 {
                    let _ = tx.send(metrics).await;
                }
                
                // Wait for next measurement interval
                tokio::time::sleep(collector.measurement_interval).await;
            }
        });
        
        rx
    }
}

impl Clone for NetworkMetricsCollector {
    fn clone(&self) -> Self {
        NetworkMetricsCollector {
            target_host: self.target_host.clone(),
            target_port: self.target_port,
            measurement_interval: self.measurement_interval,
            max_samples: self.max_samples,
            latency_samples: self.latency_samples.clone(),
            packet_loss_samples: self.packet_loss_samples.clone(),
        }
    }
}

/// TCP connection monitor for tracking connection health
pub struct TcpConnectionMonitor {
    stream: TcpStream,
    metrics_collector: NetworkMetricsCollector,
    last_activity: Instant,
    bytes_sent: u64,
    bytes_received: u64,
}

impl TcpConnectionMonitor {
    /// Create a new TCP connection monitor
    pub async fn new(host: String, port: u16) -> Result<Self> {
        let stream = TcpStream::connect((host.as_str(), port)).await?;
        let metrics_collector = NetworkMetricsCollector::new(host, port);
        
        Ok(TcpConnectionMonitor {
            stream,
            metrics_collector,
            last_activity: Instant::now(),
            bytes_sent: 0,
            bytes_received: 0,
        })
    }
    
    /// Get current connection metrics
    pub async fn get_metrics(&mut self) -> NetworkMetrics {
        self.metrics_collector.calculate_metrics().await
    }
    
    /// Send data and update statistics
    pub async fn send(&mut self, data: &[u8]) -> Result<usize> {
        let bytes_written = self.stream.write(data).await?;
        self.bytes_sent += bytes_written as u64;
        self.last_activity = Instant::now();
        Ok(bytes_written)
    }
    
    /// Receive data and update statistics
    pub async fn receive(&mut self, buffer: &mut [u8]) -> Result<usize> {
        let bytes_read = self.stream.read(buffer).await?;
        self.bytes_received += bytes_read as u64;
        self.last_activity = Instant::now();
        Ok(bytes_read)
    }
    
    /// Get connection statistics
    pub fn get_stats(&self) -> ConnectionStats {
        ConnectionStats {
            bytes_sent: self.bytes_sent,
            bytes_received: self.bytes_received,
            last_activity: self.last_activity,
            idle_time: self.last_activity.elapsed(),
        }
    }
    
    /// Check if connection is still alive
    pub async fn is_alive(&mut self) -> bool {
        // Try to read 0 bytes to check if connection is still open
        let mut buf = [0u8; 1];
        match self.stream.try_read(&mut buf) {
            Ok(0) => true, // Connection is open but no data
            Ok(_) => true, // Connection is open and has data
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => true, // Would block means connection is open
            Err(_) => false, // Any other error means connection is closed
        }
    }
}

/// Connection statistics
#[derive(Debug, Clone)]
pub struct ConnectionStats {
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub last_activity: Instant,
    pub idle_time: Duration,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_network_metrics_default() {
        let metrics = NetworkMetrics::default();
        assert_eq!(metrics.latency_ms, 0.0);
        assert_eq!(metrics.packet_loss_percent, 0.0);
        assert_eq!(metrics.quality_score, 0);
    }
    
    #[tokio::test]
    async fn test_metrics_collector_creation() {
        let collector = NetworkMetricsCollector::new("example.com".to_string(), 80);
        assert_eq!(collector.target_host, "example.com");
        assert_eq!(collector.target_port, 80);
        assert_eq!(collector.max_samples, 20);
    }
    
    #[tokio::test]
    async fn test_quality_score_calculation() {
        let collector = NetworkMetricsCollector::new("example.com".to_string(), 80);
        
        // Test excellent connection
        let score = collector.calculate_quality_score(30.0, 0.5, 5.0);
        assert!(score >= 80);
        
        // Test poor connection
        let score = collector.calculate_quality_score(600.0, 25.0, 150.0);
        assert!(score <= 40);
    }
}