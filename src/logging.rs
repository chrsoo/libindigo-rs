//! Logging configuration for libindigo.
//!
//! Uses the `tracing` framework with configurable output destinations and log levels.
//! By default, logs are written to stderr at INFO level.
//!
//! # Log Level Rules
//! - **ERROR**: Application integrity compromised
//! - **WARN**: Graceful recovery possible
//! - **INFO**: Meaningful user information
//! - **DEBUG**: Additional troubleshooting info
//! - **TRACE**: Detailed application logic tracing

use std::path::PathBuf;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

/// Log level configuration matching the tracing framework levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    /// Application integrity compromised
    Error,
    /// Graceful recovery possible
    Warn,
    /// Meaningful user information (default)
    Info,
    /// Additional troubleshooting info
    Debug,
    /// Detailed application logic tracing
    Trace,
}

impl Default for LogLevel {
    fn default() -> Self {
        LogLevel::Info
    }
}

impl From<LogLevel> for tracing::Level {
    fn from(level: LogLevel) -> Self {
        match level {
            LogLevel::Error => tracing::Level::ERROR,
            LogLevel::Warn => tracing::Level::WARN,
            LogLevel::Info => tracing::Level::INFO,
            LogLevel::Debug => tracing::Level::DEBUG,
            LogLevel::Trace => tracing::Level::TRACE,
        }
    }
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogLevel::Error => write!(f, "error"),
            LogLevel::Warn => write!(f, "warn"),
            LogLevel::Info => write!(f, "info"),
            LogLevel::Debug => write!(f, "debug"),
            LogLevel::Trace => write!(f, "trace"),
        }
    }
}

/// Configuration for logging output.
#[derive(Debug, Clone)]
pub struct LogConfig {
    /// The minimum log level to output. Defaults to INFO.
    pub level: LogLevel,
    /// Optional file path for log output. If set, logs are written to this file
    /// in addition to stderr.
    pub log_file: Option<PathBuf>,
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            level: LogLevel::default(),
            log_file: None,
        }
    }
}

impl LogConfig {
    /// Create a new LogConfig with the specified level.
    pub fn with_level(mut self, level: LogLevel) -> Self {
        self.level = level;
        self
    }

    /// Set the log file path.
    pub fn with_log_file(mut self, path: PathBuf) -> Self {
        self.log_file = Some(path);
        self
    }
}

/// Initialize the logging system with the given configuration.
///
/// This should be called once at application startup. If called multiple times,
/// subsequent calls will be ignored (tracing subscriber can only be set once).
///
/// # Arguments
/// * `config` - The logging configuration to use
///
/// # Returns
/// `Ok(())` if initialization succeeded, or an error if the subscriber could not be set.
pub fn init_logging(config: &LogConfig) -> Result<(), Box<dyn std::error::Error>> {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        EnvFilter::new(format!(
            "libindigo={level},libindigo_rs={level},libindigo_ffi={level}",
            level = config.level
        ))
    });

    let stderr_layer = fmt::layer()
        .with_writer(std::io::stderr)
        .with_target(true)
        .with_level(true)
        .with_ansi(true);

    if let Some(ref log_path) = config.log_file {
        let file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_path)?;

        let file_layer = fmt::layer()
            .with_writer(std::sync::Mutex::new(file))
            .with_target(true)
            .with_level(true)
            .with_ansi(false);

        tracing_subscriber::registry()
            .with(env_filter)
            .with(stderr_layer)
            .with(file_layer)
            .try_init()
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;
    } else {
        tracing_subscriber::registry()
            .with(env_filter)
            .with(stderr_layer)
            .try_init()
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;
    }

    Ok(())
}
