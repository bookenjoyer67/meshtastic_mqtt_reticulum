use serde::{Serialize, Deserialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use reqwest::Client;
use tokio::time::{Duration, sleep};
use log::{info, warn};

/// Webhook event types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WebhookEvent {
    MessageReceived {
        source: String,
        channel: Option<String>,
        text: String,
        sender_id: Option<String>,
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    MessageSent {
        destination: String,
        channel: Option<String>,
        text: String,
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    PeerDiscovered {
        peer_id: String,
        peer_hash: String,
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    FileTransferStarted {
        file_name: String,
        file_size: u64,
        peer_id: Option<String>,
        direction: String, // "upload" or "download"
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    FileTransferCompleted {
        file_name: String,
        file_size: u64,
        peer_id: Option<String>,
        direction: String, // "upload" or "download"
        duration_ms: u64,
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    ConnectionEstablished {
        component: String,
        endpoint: String,
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    ConnectionLost {
        component: String,
        endpoint: String,
        error: String,
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    SecurityEvent {
        event_type: String,
        severity: String,
        details: String,
        user_id: Option<String>,
        peer_id: Option<String>,
        timestamp: chrono::DateTime<chrono::Utc>,
    },
}

/// Webhook configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookConfig {
    pub url: String,
    pub secret: Option<String>,
    pub enabled: bool,
    pub events: Vec<String>, // List of event types to send
    pub timeout_seconds: u64,
    pub max_retries: u32,
    pub retry_delay_ms: u64,
}

impl Default for WebhookConfig {
    fn default() -> Self {
        Self {
            url: String::new(),
            secret: None,
            enabled: false,
            events: vec![
                "message_received".to_string(),
                "message_sent".to_string(),
                "peer_discovered".to_string(),
                "file_transfer_completed".to_string(),
                "security_event".to_string(),
            ],
            timeout_seconds: 10,
            max_retries: 3,
            retry_delay_ms: 1000,
        }
    }
}



/// Webhook manager
pub struct WebhookManager {
    client: Client,
    configs: Vec<WebhookConfig>,
    rate_limiter: Arc<Mutex<HashMap<String, std::time::Instant>>>,
    max_requests_per_minute: u32,
}

impl WebhookManager {
    pub fn new(configs: Vec<WebhookConfig>) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .unwrap_or_else(|_| Client::new());
        
        Self {
            client,
            configs,
            rate_limiter: Arc::new(Mutex::new(HashMap::new())),
            max_requests_per_minute: 60, // Default limit
        }
    }
    
    /// Check if we should send this event based on rate limiting
    async fn check_rate_limit(&self, webhook_url: &str) -> bool {
        let mut rate_limiter = self.rate_limiter.lock().await;
        let now = std::time::Instant::now();
        
        // Clean up old entries (older than 1 minute)
        rate_limiter.retain(|_, timestamp| now.duration_since(*timestamp) < Duration::from_secs(60));
        
        // Count requests for this webhook in the last minute
        let count = rate_limiter.keys()
            .filter(|url| url == &webhook_url)
            .count() as u32;
        
        if count >= self.max_requests_per_minute {
            warn!("Rate limit exceeded for webhook: {}", webhook_url);
            return false;
        }
        
        // Add this request
        rate_limiter.insert(webhook_url.to_string(), now);
        true
    }
    
    /// Send an event to all configured webhooks
    pub async fn send_event(&self, event: &WebhookEvent) {
        for config in &self.configs {
            if !config.enabled {
                continue;
            }
            
            // Check if this event type is enabled for this webhook
            let event_type = match event {
                WebhookEvent::MessageReceived { .. } => "message_received",
                WebhookEvent::MessageSent { .. } => "message_sent",
                WebhookEvent::PeerDiscovered { .. } => "peer_discovered",
                WebhookEvent::FileTransferStarted { .. } => "file_transfer_started",
                WebhookEvent::FileTransferCompleted { .. } => "file_transfer_completed",
                WebhookEvent::ConnectionEstablished { .. } => "connection_established",
                WebhookEvent::ConnectionLost { .. } => "connection_lost",
                WebhookEvent::SecurityEvent { .. } => "security_event",
            };
            
            if !config.events.contains(&event_type.to_string()) {
                continue;
            }
            
            // Check rate limiting
            if !self.check_rate_limit(&config.url).await {
                continue;
            }
            
            // Prepare payload
            let payload = self.prepare_payload(event, config);
            
            // Send with retries
            self.send_with_retries(&config.url, payload, config.max_retries, config.retry_delay_ms).await;
        }
    }
    
    /// Prepare the webhook payload
    fn prepare_payload(&self, event: &WebhookEvent, config: &WebhookConfig) -> Value {
        let mut payload = json!({
            "event": match event {
                WebhookEvent::MessageReceived { .. } => "message_received",
                WebhookEvent::MessageSent { .. } => "message_sent",
                WebhookEvent::PeerDiscovered { .. } => "peer_discovered",
                WebhookEvent::FileTransferStarted { .. } => "file_transfer_started",
                WebhookEvent::FileTransferCompleted { .. } => "file_transfer_completed",
                WebhookEvent::ConnectionEstablished { .. } => "connection_established",
                WebhookEvent::ConnectionLost { .. } => "connection_lost",
                WebhookEvent::SecurityEvent { .. } => "security_event",
            },
            "timestamp": chrono::Utc::now().to_rfc3339(),
        });
        
        // Add event-specific data
        match event {
            WebhookEvent::MessageReceived { source, channel, text, sender_id, timestamp } => {
                payload["data"] = json!({
                    "source": source,
                    "channel": channel,
                    "text": text,
                    "sender_id": sender_id,
                    "timestamp": timestamp.to_rfc3339(),
                });
            }
            WebhookEvent::MessageSent { destination, channel, text, timestamp } => {
                payload["data"] = json!({
                    "destination": destination,
                    "channel": channel,
                    "text": text,
                    "timestamp": timestamp.to_rfc3339(),
                });
            }
            WebhookEvent::PeerDiscovered { peer_id, peer_hash, timestamp } => {
                payload["data"] = json!({
                    "peer_id": peer_id,
                    "peer_hash": peer_hash,
                    "timestamp": timestamp.to_rfc3339(),
                });
            }
            WebhookEvent::FileTransferStarted { file_name, file_size, peer_id, direction, timestamp } => {
                payload["data"] = json!({
                    "file_name": file_name,
                    "file_size": file_size,
                    "peer_id": peer_id,
                    "direction": direction,
                    "timestamp": timestamp.to_rfc3339(),
                });
            }
            WebhookEvent::FileTransferCompleted { file_name, file_size, peer_id, direction, duration_ms, timestamp } => {
                payload["data"] = json!({
                    "file_name": file_name,
                    "file_size": file_size,
                    "peer_id": peer_id,
                    "direction": direction,
                    "duration_ms": duration_ms,
                    "timestamp": timestamp.to_rfc3339(),
                });
            }
            WebhookEvent::ConnectionEstablished { component, endpoint, timestamp } => {
                payload["data"] = json!({
                    "component": component,
                    "endpoint": endpoint,
                    "timestamp": timestamp.to_rfc3339(),
                });
            }
            WebhookEvent::ConnectionLost { component, endpoint, error, timestamp } => {
                payload["data"] = json!({
                    "component": component,
                    "endpoint": endpoint,
                    "error": error,
                    "timestamp": timestamp.to_rfc3339(),
                });
            }
            WebhookEvent::SecurityEvent { event_type, severity, details, user_id, peer_id, timestamp } => {
                payload["data"] = json!({
                    "event_type": event_type,
                    "severity": severity,
                    "details": details,
                    "user_id": user_id,
                    "peer_id": peer_id,
                    "timestamp": timestamp.to_rfc3339(),
                });
            }
        }
        
        // Add signature if secret is configured
        if let Some(secret) = &config.secret {
            let _payload_str = payload.to_string();
            // In a real implementation, you would compute HMAC here
            // For simplicity, we'll just add a placeholder
            payload["signature"] = Value::String(format!("hmac-sha256:{}", secret));
        }
        
        payload
    }
    
    /// Send webhook with retry logic
    async fn send_with_retries(&self, url: &str, payload: Value, max_retries: u32, retry_delay_ms: u64) {
        for attempt in 0..=max_retries {
            match self.client.post(url)
                .json(&payload)
                .send()
                .await
            {
                Ok(response) => {
                    if response.status().is_success() {
                        info!("Webhook sent successfully to {}", url);
                        break;
                    } else {
                        warn!("Webhook failed with status {} to {} (attempt {}/{})", 
                            response.status(), url, attempt + 1, max_retries + 1);
                    }
                }
                Err(e) => {
                    warn!("Webhook error to {}: {} (attempt {}/{})", 
                        url, e, attempt + 1, max_retries + 1);
                }
            }
            
            if attempt < max_retries {
                sleep(Duration::from_millis(retry_delay_ms)).await;
            }
        }
    }
    
    /// Load webhook configurations from environment
    pub fn from_env() -> Vec<WebhookConfig> {
        let mut configs = Vec::new();
        
        // Parse webhook configurations from environment variable
        // Format: WEBHOOK_URLS=url1:secret1:events1,url2:secret2:events2
        // Note: URLs with colons (like http://) will break this simple parser
        // In production, use a different delimiter or encoding
        if let Ok(webhook_urls) = std::env::var("WEBHOOK_URLS") {
            for webhook_str in webhook_urls.split(',') {
                // Split on colons, but handle URLs with colons by working from the end
                let parts: Vec<&str> = webhook_str.split(':').collect();
                
                // We need at least 1 part (URL)
                if parts.is_empty() {
                    continue;
                }
                
                // If we have 3 or more parts, the last two are secret and events
                // Everything before that is the URL (may contain colons from http://)
                let mut config = WebhookConfig::default();
                
                if parts.len() >= 3 {
                    // URL is everything except the last 2 parts, joined back with colons
                    let url_parts = &parts[0..parts.len()-2];
                    config.url = url_parts.join(":");
                    
                    // Secret is second to last part
                    if !parts[parts.len()-2].is_empty() {
                        config.secret = Some(parts[parts.len()-2].to_string());
                    }
                    
                    // Events is last part
                    if !parts[parts.len()-1].is_empty() {
                        config.events = parts[parts.len()-1].split('|').map(|s| s.to_string()).collect();
                    }
                } else if parts.len() == 2 {
                    // URL is first part, secret is second part
                    config.url = parts[0].to_string();
                    if !parts[1].is_empty() {
                        config.secret = Some(parts[1].to_string());
                    }
                } else if parts.len() == 1 {
                    // Just URL
                    config.url = parts[0].to_string();
                }
                
                if !config.url.is_empty() {
                    configs.push(config);
                }
            }
        }
        
        configs
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_webhook_config_default() {
        let config = WebhookConfig::default();
        assert_eq!(config.url, "");
        assert_eq!(config.enabled, false);
        assert!(config.events.contains(&"message_received".to_string()));
        assert!(config.events.contains(&"message_sent".to_string()));
        assert_eq!(config.timeout_seconds, 10);
        assert_eq!(config.max_retries, 3);
    }
    
    #[tokio::test]
    async fn test_webhook_manager_creation() {
        let configs = vec![WebhookConfig::default()];
        let manager = WebhookManager::new(configs);
        assert_eq!(manager.configs.len(), 1);
    }
    
    #[tokio::test]
    async fn test_webhook_event_creation() {
        let event = WebhookEvent::MessageReceived {
            source: "test_source".to_string(),
            channel: Some("test_channel".to_string()),
            text: "Test message".to_string(),
            sender_id: Some("test_sender".to_string()),
            timestamp: chrono::Utc::now(),
        };
        
        match event {
            WebhookEvent::MessageReceived { source, text, .. } => {
                assert_eq!(source, "test_source");
                assert_eq!(text, "Test message");
            }
            _ => panic!("Wrong event type"),
        }
    }
    
    #[tokio::test]
    async fn test_webhook_from_env_empty() {
        // Test with no environment variable set
        let configs = WebhookManager::from_env();
        assert_eq!(configs.len(), 0);
    }
    
    #[tokio::test]
    async fn test_webhook_from_env_single() {
        // Temporarily set environment variable
        std::env::set_var("WEBHOOK_URLS", "http://example.com/webhook:secret123:message_received|message_sent");
        
        let configs = WebhookManager::from_env();
        assert_eq!(configs.len(), 1);
        assert_eq!(configs[0].url, "http://example.com/webhook");
        assert_eq!(configs[0].secret, Some("secret123".to_string()));
        assert!(configs[0].events.contains(&"message_received".to_string()));
        assert!(configs[0].events.contains(&"message_sent".to_string()));
        
        // Clean up
        std::env::remove_var("WEBHOOK_URLS");
    }
    
    #[tokio::test]
    async fn test_webhook_from_env_multiple() {
        // Temporarily set environment variable
        std::env::set_var("WEBHOOK_URLS", "http://example1.com/webhook:secret1:message_received,http://example2.com/webhook::message_sent");
        
        let configs = WebhookManager::from_env();
        assert_eq!(configs.len(), 2);
        
        // First webhook
        assert_eq!(configs[0].url, "http://example1.com/webhook");
        assert_eq!(configs[0].secret, Some("secret1".to_string()));
        assert!(configs[0].events.contains(&"message_received".to_string()));
        
        // Second webhook (no secret)
        assert_eq!(configs[1].url, "http://example2.com/webhook");
        assert_eq!(configs[1].secret, None);
        assert!(configs[1].events.contains(&"message_sent".to_string()));
        
        // Clean up
        std::env::remove_var("WEBHOOK_URLS");
    }
}