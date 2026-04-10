use chrono::{DateTime, Local};
use crate::network_metrics::NetworkMetrics;

#[derive(Clone)]
pub struct Peer {
    pub main_hash: String,
    pub file_hash: String,
    pub name: Option<String>,      // custom name (user‑set)
    pub last_seen: Option<DateTime<Local>>,
    pub signal_strength: Option<i32>, // RSSI in dBm (if available)
    pub link_quality: Option<u8>,  // link quality 0-100 (if available)
    pub interface: Option<String>, // interface through which peer was discovered
    pub network_metrics: Option<NetworkMetrics>, // Actual network metrics
    pub radio_metrics: Option<RadioMetrics>, // Radio-specific metrics
    pub last_metrics_update: Option<DateTime<Local>>, // When metrics were last updated
}

/// Radio-specific metrics for LoRa and other radio interfaces
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RadioMetrics {
    pub rssi_dbm: i32,            // Received Signal Strength Indicator
    pub snr_db: f32,              // Signal-to-Noise Ratio
    pub packet_count: u64,        // Total packets received
    pub packet_loss_count: u64,   // Packets with errors
    pub link_quality: u8,         // Calculated link quality 0-100
    pub timestamp: DateTime<Local>, // When metrics were collected
}

impl Default for RadioMetrics {
    fn default() -> Self {
        RadioMetrics {
            rssi_dbm: -120,
            snr_db: 0.0,
            packet_count: 0,
            packet_loss_count: 0,
            link_quality: 0,
            timestamp: Local::now(),
        }
    }
}