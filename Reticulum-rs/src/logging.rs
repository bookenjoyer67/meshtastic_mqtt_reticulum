//! Comprehensive logging utilities for Reticulum-rs
//!
//! This module provides structured logging with different log levels,
//! context-aware logging, and performance monitoring.

use log::{Level, LevelFilter, Log, Metadata, Record, SetLoggerError};
use std::collections::HashMap;
use std::fmt;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Log levels specific to Reticulum networking operations
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ReticulumLogLevel {
    /// Critical errors that prevent the system from functioning
    Critical = 0,
    /// Errors that affect functionality but may be recoverable
    Error = 1,
    /// Warnings about potential issues
    Warning = 2,
    /// General information about system operation
    Info = 3,
    /// Detailed information for debugging
    Debug = 4,
    /// Very detailed tracing information
    Trace = 5,
    /// Packet-level tracing
    Packet = 6,
}

impl From<ReticulumLogLevel> for LevelFilter {
    fn from(level: ReticulumLogLevel) -> Self {
        match level {
            ReticulumLogLevel::Critical => LevelFilter::Error,
            ReticulumLogLevel::Error => LevelFilter::Error,
            ReticulumLogLevel::Warning => LevelFilter::Warn,
            ReticulumLogLevel::Info => LevelFilter::Info,
            ReticulumLogLevel::Debug => LevelFilter::Debug,
            ReticulumLogLevel::Trace => LevelFilter::Trace,
            ReticulumLogLevel::Packet => LevelFilter::Trace,
        }
    }
}

impl fmt::Display for ReticulumLogLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ReticulumLogLevel::Critical => write!(f, "CRITICAL"),
            ReticulumLogLevel::Error => write!(f, "ERROR"),
            ReticulumLogLevel::Warning => write!(f, "WARNING"),
            ReticulumLogLevel::Info => write!(f, "INFO"),
            ReticulumLogLevel::Debug => write!(f, "DEBUG"),
            ReticulumLogLevel::Trace => write!(f, "TRACE"),
            ReticulumLogLevel::Packet => write!(f, "PACKET"),
        }
    }
}

/// Context for structured logging
#[derive(Debug, Clone)]
pub struct LogContext {
    /// Component name (e.g., "transport", "interface", "crypto")
    pub component: String,
    /// Optional operation ID for correlating logs
    pub operation_id: Option<String>,
    /// Optional peer address
    pub peer: Option<String>,
    /// Optional link ID
    pub link_id: Option<String>,
    /// Optional packet hash
    pub packet_hash: Option<String>,
    /// Additional context fields
    pub extra: HashMap<String, String>,
}

impl LogContext {
    pub fn new(component: &str) -> Self {
        Self {
            component: component.to_string(),
            operation_id: None,
            peer: None,
            link_id: None,
            packet_hash: None,
            extra: HashMap::new(),
        }
    }

    pub fn with_operation_id(mut self, id: &str) -> Self {
        self.operation_id = Some(id.to_string());
        self
    }

    pub fn with_peer(mut self, peer: &str) -> Self {
        self.peer = Some(peer.to_string());
        self
    }

    pub fn with_link_id(mut self, link_id: &str) -> Self {
        self.link_id = Some(link_id.to_string());
        self
    }

    pub fn with_packet_hash(mut self, hash: &str) -> Self {
        self.packet_hash = Some(hash.to_string());
        self
    }

    pub fn with_extra(mut self, key: &str, value: &str) -> Self {
        self.extra.insert(key.to_string(), value.to_string());
        self
    }
}

/// Performance measurement for operations
pub struct PerformanceTimer {
    start: Instant,
    operation: String,
    context: LogContext,
}

impl PerformanceTimer {
    pub fn start(operation: &str, context: LogContext) -> Self {
        Self {
            start: Instant::now(),
            operation: operation.to_string(),
            context,
        }
    }

    pub fn stop(self) -> Duration {
        let duration = self.start.elapsed();
        log::debug!(
            "Operation '{}' completed in {:?}",
            self.operation,
            duration
        );
        duration
    }
}

/// Structured logger for Reticulum
pub struct ReticulumLogger {
    level: ReticulumLogLevel,
    include_timestamp: bool,
    include_thread_id: bool,
    include_file_line: bool,
    /// In-memory log buffer for recent logs
    buffer: Arc<Mutex<Vec<String>>>,
    buffer_size: usize,
}

impl ReticulumLogger {
    pub fn new(level: ReticulumLogLevel) -> Self {
        Self {
            level,
            include_timestamp: true,
            include_thread_id: false,
            include_file_line: true,
            buffer: Arc::new(Mutex::new(Vec::new())),
            buffer_size: 1000,
        }
    }

    pub fn with_timestamp(mut self, include: bool) -> Self {
        self.include_timestamp = include;
        self
    }

    pub fn with_thread_id(mut self, include: bool) -> Self {
        self.include_thread_id = include;
        self
    }

    pub fn with_file_line(mut self, include: bool) -> Self {
        self.include_file_line = include;
        self
    }

    pub fn with_buffer_size(mut self, size: usize) -> Self {
        self.buffer_size = size;
        self
    }

    pub fn init(self) -> Result<(), SetLoggerError> {
        let max_level = LevelFilter::from(self.level);
        log::set_max_level(max_level);
        log::set_boxed_logger(Box::new(self))
    }

    pub fn get_recent_logs(&self) -> Vec<String> {
        self.buffer.lock().unwrap().clone()
    }

    pub fn clear_logs(&self) {
        self.buffer.lock().unwrap().clear();
    }

    fn format_record(&self, record: &Record) -> String {
        let mut parts = Vec::new();

        if self.include_timestamp {
            let now = chrono::Local::now();
            parts.push(format!("[{}]", now.format("%Y-%m-%d %H:%M:%S%.3f")));
        }

        parts.push(format!("[{}]", record.level()));

        if self.include_thread_id {
            if let Some(thread_id) = std::thread::current().name() {
                parts.push(format!("[thread:{}]", thread_id));
            } else {
                parts.push(format!("[thread:{:?}]", std::thread::current().id()));
            }
        }

        if self.include_file_line {
            if let (Some(file), Some(line)) = (record.file(), record.line()) {
                parts.push(format!("[{}:{}]", file, line));
            }
        }

        parts.push(record.args().to_string());

        parts.join(" ")
    }
}

impl Log for ReticulumLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        let reticulum_level = match self.level {
            ReticulumLogLevel::Critical => Level::Error,
            ReticulumLogLevel::Error => Level::Error,
            ReticulumLogLevel::Warning => Level::Warn,
            ReticulumLogLevel::Info => Level::Info,
            ReticulumLogLevel::Debug => Level::Debug,
            ReticulumLogLevel::Trace => Level::Trace,
            ReticulumLogLevel::Packet => Level::Trace,
        };
        metadata.level() <= reticulum_level
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let formatted = self.format_record(record);
            println!("{}", formatted);
            
            // Add to buffer
            let mut buffer = self.buffer.lock().unwrap();
            buffer.push(formatted);
            if buffer.len() > self.buffer_size {
                buffer.remove(0);
            }
        }
    }

    fn flush(&self) {}
}

/// Convenience macros for structured logging
#[macro_export]
macro_rules! log_critical {
    ($ctx:expr, $($arg:tt)*) => {
        log::error!(target: &$ctx.component, "[{}] {}", $ctx.component, format_args!($($arg)*))
    };
}

#[macro_export]
macro_rules! log_error {
    ($ctx:expr, $($arg:tt)*) => {
        log::error!(target: &$ctx.component, "[{}] {}", $ctx.component, format_args!($($arg)*))
    };
}

#[macro_export]
macro_rules! log_warning {
    ($ctx:expr, $($arg:tt)*) => {
        log::warn!(target: &$ctx.component, "[{}] {}", $ctx.component, format_args!($($arg)*))
    };
}

#[macro_export]
macro_rules! log_info {
    ($ctx:expr, $($arg:tt)*) => {
        log::info!(target: &$ctx.component, "[{}] {}", $ctx.component, format_args!($($arg)*))
    };
}

#[macro_export]
macro_rules! log_debug {
    ($ctx:expr, $($arg:tt)*) => {
        log::debug!(target: &$ctx.component, "[{}] {}", $ctx.component, format_args!($($arg)*))
    };
}

#[macro_export]
macro_rules! log_trace {
    ($ctx:expr, $($arg:tt)*) => {
        log::trace!(target: &$ctx.component, "[{}] {}", $ctx.component, format_args!($($arg)*))
    };
}

#[macro_export]
macro_rules! log_packet {
    ($ctx:expr, $($arg:tt)*) => {
        log::trace!(target: "packet", "[PACKET][{}] {}", $ctx.component, format_args!($($arg)*))
    };
}

/// Initialize logging system
pub fn init_logging(level: ReticulumLogLevel) -> Result<(), SetLoggerError> {
    let logger = ReticulumLogger::new(level)
        .with_timestamp(true)
        .with_file_line(true)
        .with_buffer_size(1000);
    
    logger.init()?;
    
    log::info!("Logging initialized at level: {}", level);
    Ok(())
}

/// Initialize logging from environment variable
pub fn init_logging_from_env() -> Result<(), SetLoggerError> {
    let level = std::env::var("RETICULUM_LOG_LEVEL")
        .unwrap_or_else(|_| "info".to_string())
        .to_lowercase();
    
    let reticulum_level = match level.as_str() {
        "critical" => ReticulumLogLevel::Critical,
        "error" => ReticulumLogLevel::Error,
        "warning" => ReticulumLogLevel::Warning,
        "info" => ReticulumLogLevel::Info,
        "debug" => ReticulumLogLevel::Debug,
        "trace" => ReticulumLogLevel::Trace,
        "packet" => ReticulumLogLevel::Packet,
        _ => ReticulumLogLevel::Info,
    };
    
    init_logging(reticulum_level)
}