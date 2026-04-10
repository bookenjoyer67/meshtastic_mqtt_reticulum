use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use std::sync::Arc;

/// Rate limiter for message sending
/// Provides protection against DoS attacks and spam
pub struct RateLimiter {
    /// Map of source identifier to their message timestamps
    message_timestamps: Mutex<HashMap<String, Vec<Instant>>>,
    /// Maximum messages per time window
    max_messages: usize,
    /// Time window in seconds
    time_window_seconds: u64,
    /// Maximum burst size (allow some burst before strict limiting)
    max_burst: usize,
}

impl RateLimiter {
    /// Create a new rate limiter with default settings
    pub fn new() -> Self {
        Self::with_limits(10, 60, 5) // Default: 10 messages per minute, burst of 5
    }

    /// Create a new rate limiter with custom limits
    pub fn with_limits(max_messages: usize, time_window_seconds: u64, max_burst: usize) -> Self {
        Self {
            message_timestamps: Mutex::new(HashMap::new()),
            max_messages,
            time_window_seconds,
            max_burst,
        }
    }

    /// Check if a message from the given source should be allowed
    /// Returns Ok(()) if allowed, Err(message) if rate limited
    pub async fn check_rate_limit(&self, source_id: &str) -> Result<(), String> {
        let mut timestamps = self.message_timestamps.lock().await;
        
        // Clean up old timestamps first
        self.cleanup_old_timestamps(&mut timestamps, source_id);
        
        let source_timestamps = timestamps.entry(source_id.to_string()).or_insert_with(Vec::new);
        
        // Check if we're at or above the limit
        if source_timestamps.len() >= self.max_messages {
            // Check burst allowance
            if source_timestamps.len() >= self.max_messages + self.max_burst {
                return Err(format!(
                    "Rate limit exceeded: {} messages in {} seconds (max: {}, burst: {})",
                    source_timestamps.len(),
                    self.time_window_seconds,
                    self.max_messages,
                    self.max_burst
                ));
            }
            
            // For burst messages, add increasing delay
            let burst_count = source_timestamps.len() - self.max_messages;
            let delay_seconds = 2u64.pow(burst_count as u32); // Exponential backoff: 2, 4, 8, 16, 32 seconds...
            
            // Check if enough time has passed since last burst message
            if let Some(last_timestamp) = source_timestamps.last() {
                let elapsed = last_timestamp.elapsed();
                if elapsed < Duration::from_secs(delay_seconds) {
                    let wait_time = Duration::from_secs(delay_seconds) - elapsed;
                    return Err(format!(
                        "Burst rate limit: please wait {} seconds before sending another message",
                        wait_time.as_secs()
                    ));
                }
            }
        }
        
        // Record this message
        source_timestamps.push(Instant::now());
        
        Ok(())
    }

    /// Clean up timestamps older than the time window
    fn cleanup_old_timestamps(&self, timestamps: &mut HashMap<String, Vec<Instant>>, source_id: &str) {
        if let Some(source_timestamps) = timestamps.get_mut(source_id) {
            let cutoff = Instant::now() - Duration::from_secs(self.time_window_seconds);
            source_timestamps.retain(|&ts| ts > cutoff);
            
            // Remove empty vectors to free memory
            if source_timestamps.is_empty() {
                timestamps.remove(source_id);
            }
        }
    }

    /// Get current statistics for a source
    pub async fn get_stats(&self, source_id: &str) -> RateLimitStats {
        let timestamps = self.message_timestamps.lock().await;
        
        if let Some(source_timestamps) = timestamps.get(source_id) {
            let cutoff = Instant::now() - Duration::from_secs(self.time_window_seconds);
            let recent_messages = source_timestamps.iter().filter(|&&ts| ts > cutoff).count();
            
            RateLimitStats {
                source_id: source_id.to_string(),
                recent_messages,
                max_messages: self.max_messages,
                time_window_seconds: self.time_window_seconds,
                max_burst: self.max_burst,
                remaining_messages: self.max_messages.saturating_sub(recent_messages),
                is_limited: recent_messages >= self.max_messages,
            }
        } else {
            RateLimitStats {
                source_id: source_id.to_string(),
                recent_messages: 0,
                max_messages: self.max_messages,
                time_window_seconds: self.time_window_seconds,
                max_burst: self.max_burst,
                remaining_messages: self.max_messages,
                is_limited: false,
            }
        }
    }

    /// Reset rate limiting for a specific source
    pub async fn reset_source(&self, source_id: &str) {
        let mut timestamps = self.message_timestamps.lock().await;
        timestamps.remove(source_id);
    }

    /// Reset all rate limiting
    pub async fn reset_all(&self) {
        let mut timestamps = self.message_timestamps.lock().await;
        timestamps.clear();
    }
}

/// Statistics about rate limiting for a source
#[derive(Debug, Clone)]
pub struct RateLimitStats {
    pub source_id: String,
    pub recent_messages: usize,
    pub max_messages: usize,
    pub time_window_seconds: u64,
    pub max_burst: usize,
    pub remaining_messages: usize,
    pub is_limited: bool,
}

impl Default for RateLimiter {
    fn default() -> Self {
        Self::new()
    }
}

/// Configuration for rate limiting
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    pub max_messages: usize,
    pub time_window_seconds: u64,
    pub max_burst: usize,
    pub enabled: bool,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            max_messages: 10,
            time_window_seconds: 60,
            max_burst: 5,
            enabled: true,
        }
    }
}

/// Rate limiter that can be shared across threads
pub type SharedRateLimiter = Arc<RateLimiter>;

/// Create a shared rate limiter with default settings
pub fn create_shared_rate_limiter() -> SharedRateLimiter {
    Arc::new(RateLimiter::new())
}

/// Create a shared rate limiter with custom settings
pub fn create_shared_rate_limiter_with_config(config: RateLimitConfig) -> SharedRateLimiter {
    Arc::new(RateLimiter::with_limits(
        config.max_messages,
        config.time_window_seconds,
        config.max_burst,
    ))
}