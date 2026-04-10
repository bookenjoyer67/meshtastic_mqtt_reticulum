use tokio::sync::mpsc;
use chrono::{DateTime, Local};

use crate::reticulum_bridge::{BridgeCommand, BridgeEvent, spawn_reticulum_bridge};
use crate::config::Config;

#[derive(Clone)]
pub struct Peer {
    pub main_hash: String,
    pub file_hash: String,
    pub name: Option<String>,      // custom name (user‑set)
    pub last_seen: Option<DateTime<Local>>,
    pub signal_strength: Option<i32>, // RSSI in dBm (if available)
    pub link_quality: Option<u8>,  // link quality 0-100 (if available)
    pub interface: Option<String>, // interface through which peer was discovered
}

pub async fn spawn_socket_bridge(
    cmd_rx: mpsc::UnboundedReceiver<BridgeCommand>,
    event_tx: mpsc::UnboundedSender<BridgeEvent>,
) -> Result<(), anyhow::Error> {
    // Load configuration
    let config = Config::from_env();
    
    // Start the Reticulum bridge
    spawn_reticulum_bridge(config, cmd_rx, event_tx).await?;
    
    Ok(())
}