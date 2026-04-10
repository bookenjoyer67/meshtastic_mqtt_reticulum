use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use std::sync::Arc;
use tokio::sync::Mutex;
use std::fs::{File, OpenOptions};
use std::io::{Write, BufWriter};
use std::path::PathBuf;

/// Log levels for structured logging
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum LogLevel {
    DEBUG,
    INFO,
    WARN,
    ERROR,
    CRITICAL,
}

impl From<log::Level> for LogLevel {
    fn from(level: log::Level) -> Self {
        match level {
            log::Level::Error => LogLevel::ERROR,
            log::Level::Warn => LogLevel::WARN,
            log::Level::Info => LogLevel::INFO,
            log::Level::Debug => LogLevel::DEBUG,
            log::Level::Trace => LogLevel::DEBUG,
        }
    }
}

/// Structured log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructuredLogEntry {
    pub timestamp: DateTime<Utc>,
    pub level: LogLevel,
    pub module: String,
    pub message: String,
    pub component: String,
    pub correlation_id: Option<String>,
    pub user_id: Option<String>,
    pub peer_id: Option<String>,
    pub channel: Option<String>,
    pub message_id: Option<String>,
    pub file_transfer_id: Option<String>,
    pub duration_ms: Option<u64>,
    pub bytes_transferred: Option<u64>,
    pub error_details: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

impl StructuredLogEntry {
    pub fn new(level: LogLevel, module: &str, message: &str, component: &str) -> Self {
        Self {
            timestamp: Utc::now(),
            level,
            module: module.to_string(),
            message: message.to_string(),
            component: component.to_string(),
            correlation_id: None,
            user_id: None,
            peer_id: None,
            channel: None,
            message_id: None,
            file_transfer_id: None,
            duration_ms: None,
            bytes_transferred: None,
            error_details: None,
            metadata: None,
        }
    }

    pub fn with_correlation_id(mut self, correlation_id: &str) -> Self {
        self.correlation_id = Some(correlation_id.to_string());
        self
    }

    pub fn with_user_id(mut self, user_id: &str) -> Self {
        self.user_id = Some(user_id.to_string());
        self
    }

    pub fn with_peer_id(mut self, peer_id: &str) -> Self {
        self.peer_id = Some(peer_id.to_string());
        self
    }

    pub fn with_channel(mut self, channel: &str) -> Self {
        self.channel = Some(channel.to_string());
        self
    }

    pub fn with_message_id(mut self, message_id: &str) -> Self {
        self.message_id = Some(message_id.to_string());
        self
    }

    pub fn with_file_transfer_id(mut self, file_transfer_id: &str) -> Self {
        self.file_transfer_id = Some(file_transfer_id.to_string());
        self
    }

    pub fn with_duration(mut self, duration_ms: u64) -> Self {
        self.duration_ms = Some(duration_ms);
        self
    }

    pub fn with_bytes_transferred(mut self, bytes: u64) -> Self {
        self.bytes_transferred = Some(bytes);
        self
    }

    pub fn with_error_details(mut self, error: &str) -> Self {
        self.error_details = Some(error.to_string());
        self
    }

    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = Some(metadata);
        self
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(|_| "{}".to_string())
    }

    pub fn to_json_pretty(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_else(|_| "{}".to_string())
    }
}

/// Structured logger that writes to file and optionally to console
pub struct StructuredLogger {
    file_writer: Option<Arc<Mutex<BufWriter<File>>>>,
    log_to_console: bool,
    #[allow(dead_code)]
    log_to_file: bool,
    log_file_path: PathBuf,
}

impl StructuredLogger {
    pub fn new(log_to_console: bool, log_to_file: bool, log_file_path: Option<PathBuf>) -> Self {
        let default_path = PathBuf::from("meshtastic_bridge.log");
        let log_file_path = log_file_path.unwrap_or(default_path);
        
        let file_writer = if log_to_file {
            match OpenOptions::new()
                .create(true)
                .append(true)
                .open(&log_file_path)
            {
                Ok(file) => Some(Arc::new(Mutex::new(BufWriter::new(file)))),
                Err(e) => {
                    eprintln!("Failed to open log file {}: {}", log_file_path.display(), e);
                    None
                }
            }
        } else {
            None
        };

        Self {
            file_writer,
            log_to_console,
            log_to_file,
            log_file_path,
        }
    }

    pub async fn log(&self, entry: StructuredLogEntry) {
        let json_string = entry.to_json();
        
        // Log to console if enabled
        if self.log_to_console {
            let level_str = match entry.level {
                LogLevel::DEBUG => "DEBUG",
                LogLevel::INFO => "INFO",
                LogLevel::WARN => "WARN",
                LogLevel::ERROR => "ERROR",
                LogLevel::CRITICAL => "CRITICAL",
            };
            println!("[{}] [{}] [{}] {}",
                entry.timestamp.format("%Y-%m-%d %H:%M:%S%.3f"),
                level_str,
                entry.component,
                entry.message
            );
        }

        // Log to file if enabled
        if let Some(writer) = &self.file_writer {
            let mut writer_lock = writer.lock().await;
            if let Err(e) = writeln!(writer_lock, "{}", json_string) {
                eprintln!("Failed to write to log file: {}", e);
            }
        }
    }

    pub async fn flush(&self) {
        if let Some(writer) = &self.file_writer {
            let mut writer_lock = writer.lock().await;
            if let Err(e) = writer_lock.flush() {
                eprintln!("Failed to flush log file: {}", e);
            }
        }
    }

    pub fn get_log_file_path(&self) -> &PathBuf {
        &self.log_file_path
    }
}

/// Convenience functions for common logging scenarios
#[derive(Clone)]
pub struct LogHelper {
    logger: Arc<StructuredLogger>,
    component: String,
}

impl LogHelper {
    pub fn new(logger: Arc<StructuredLogger>, component: &str) -> Self {
        Self {
            logger,
            component: component.to_string(),
        }
    }

    pub async fn debug(&self, module: &str, message: &str) {
        let entry = StructuredLogEntry::new(LogLevel::DEBUG, module, message, &self.component);
        self.logger.log(entry).await;
    }

    pub async fn info(&self, module: &str, message: &str) {
        let entry = StructuredLogEntry::new(LogLevel::INFO, module, message, &self.component);
        self.logger.log(entry).await;
    }

    pub async fn warn(&self, module: &str, message: &str) {
        let entry = StructuredLogEntry::new(LogLevel::WARN, module, message, &self.component);
        self.logger.log(entry).await;
    }

    pub async fn error(&self, module: &str, message: &str) {
        let entry = StructuredLogEntry::new(LogLevel::ERROR, module, message, &self.component);
        self.logger.log(entry).await;
    }

    pub async fn critical(&self, module: &str, message: &str) {
        let entry = StructuredLogEntry::new(LogLevel::CRITICAL, module, message, &self.component);
        self.logger.log(entry).await;
    }

    pub async fn message_sent(&self, message_id: &str, channel: &str, peer_id: Option<&str>, duration_ms: u64) {
        let mut entry = StructuredLogEntry::new(
            LogLevel::INFO,
            "messaging",
            &format!("Message sent to channel {}", channel),
            &self.component,
        )
        .with_message_id(message_id)
        .with_channel(channel)
        .with_duration(duration_ms);

        if let Some(peer_id) = peer_id {
            entry = entry.with_peer_id(peer_id);
        }

        self.logger.log(entry).await;
    }

    pub async fn message_received(&self, message_id: &str, channel: &str, peer_id: Option<&str>) {
        let mut entry = StructuredLogEntry::new(
            LogLevel::INFO,
            "messaging",
            &format!("Message received from channel {}", channel),
            &self.component,
        )
        .with_message_id(message_id)
        .with_channel(channel);

        if let Some(peer_id) = peer_id {
            entry = entry.with_peer_id(peer_id);
        }

        self.logger.log(entry).await;
    }

    pub async fn file_transfer_started(&self, transfer_id: &str, file_name: &str, file_size: u64, peer_id: Option<&str>) {
        let mut entry = StructuredLogEntry::new(
            LogLevel::INFO,
            "file_transfer",
            &format!("File transfer started: {} ({} bytes)", file_name, file_size),
            &self.component,
        )
        .with_file_transfer_id(transfer_id)
        .with_bytes_transferred(file_size);

        if let Some(peer_id) = peer_id {
            entry = entry.with_peer_id(peer_id);
        }

        self.logger.log(entry).await;
    }

    pub async fn file_transfer_completed(&self, transfer_id: &str, file_name: &str, duration_ms: u64, bytes_transferred: u64) {
        let entry = StructuredLogEntry::new(
            LogLevel::INFO,
            "file_transfer",
            &format!("File transfer completed: {} ({} bytes in {} ms)", file_name, bytes_transferred, duration_ms),
            &self.component,
        )
        .with_file_transfer_id(transfer_id)
        .with_duration(duration_ms)
        .with_bytes_transferred(bytes_transferred);

        self.logger.log(entry).await;
    }

    pub async fn connection_established(&self, component: &str, endpoint: &str, duration_ms: u64) {
        let entry = StructuredLogEntry::new(
            LogLevel::INFO,
            "connection",
            &format!("Connection established to {}", endpoint),
            component,
        )
        .with_duration(duration_ms);

        self.logger.log(entry).await;
    }

    pub async fn connection_lost(&self, component: &str, endpoint: &str, error: &str) {
        let entry = StructuredLogEntry::new(
            LogLevel::WARN,
            "connection",
            &format!("Connection lost to {}: {}", endpoint, error),
            component,
        )
        .with_error_details(error);

        self.logger.log(entry).await;
    }

    pub async fn security_event(&self, event_type: &str, severity: LogLevel, details: &str, user_id: Option<&str>, peer_id: Option<&str>) {
        let mut entry = StructuredLogEntry::new(
            severity,
            "security",
            &format!("Security event: {} - {}", event_type, details),
            &self.component,
        );

        if let Some(user_id) = user_id {
            entry = entry.with_user_id(user_id);
        }

        if let Some(peer_id) = peer_id {
            entry = entry.with_peer_id(peer_id);
        }

        self.logger.log(entry).await;
    }

    pub async fn rate_limit_hit(&self, component: &str, identifier: &str, limit_type: &str) {
        let entry = StructuredLogEntry::new(
            LogLevel::WARN,
            "rate_limit",
            &format!("Rate limit hit for {}: {}", limit_type, identifier),
            component,
        );

        self.logger.log(entry).await;
    }
}

/// Initialize structured logging
pub fn init_structured_logging(log_to_console: bool, log_to_file: bool, log_file_path: Option<PathBuf>) -> Arc<StructuredLogger> {
    Arc::new(StructuredLogger::new(log_to_console, log_to_file, log_file_path))
}