#![allow(dead_code)]

//! Configuration for the test harness
//!
//! This module provides configuration management for the INDIGO server test harness,
//! including environment variable parsing, default values, and validation.

use std::path::PathBuf;
use std::time::Duration;

/// Configuration for the INDIGO server test harness
#[derive(Debug, Clone)]
pub struct TestConfig {
    /// Path to the indigo_server binary
    pub server_binary_path: Option<PathBuf>,

    /// Port for the test server
    pub port: u16,

    /// List of drivers to load
    pub drivers: Vec<String>,

    /// Server startup timeout
    pub startup_timeout: Duration,

    /// Server shutdown timeout
    pub shutdown_timeout: Duration,

    /// Skip server startup (use existing server)
    pub skip_server_startup: bool,

    /// Server host (if using existing server)
    pub server_host: String,

    /// Logging level
    pub log_level: String,

    /// State reset timeout
    pub state_reset_timeout: Duration,
}

impl Default for TestConfig {
    fn default() -> Self {
        Self {
            server_binary_path: None,
            port: 7624,
            drivers: vec![
                "indigo_ccd_simulator".to_string(),
                "indigo_mount_simulator".to_string(),
            ],
            startup_timeout: Duration::from_secs(10),
            shutdown_timeout: Duration::from_secs(5),
            skip_server_startup: false,
            server_host: "localhost".to_string(),
            log_level: "info".to_string(),
            state_reset_timeout: Duration::from_secs(2),
        }
    }
}

impl TestConfig {
    /// Create configuration from environment variables
    pub fn from_env() -> Self {
        let mut config = Self::default();

        // INDIGO_SERVER_PATH
        if let Ok(path) = std::env::var("INDIGO_SERVER_PATH") {
            config.server_binary_path = Some(PathBuf::from(path));
        }

        // INDIGO_TEST_PORT
        if let Ok(port_str) = std::env::var("INDIGO_TEST_PORT") {
            if let Ok(port) = port_str.parse::<u16>() {
                config.port = port;
            }
        }

        // INDIGO_TEST_DRIVERS
        if let Ok(drivers_str) = std::env::var("INDIGO_TEST_DRIVERS") {
            config.drivers = drivers_str
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
        }

        // INDIGO_TEST_STARTUP_TIMEOUT
        if let Ok(timeout_str) = std::env::var("INDIGO_TEST_STARTUP_TIMEOUT") {
            if let Ok(secs) = timeout_str.parse::<u64>() {
                config.startup_timeout = Duration::from_secs(secs);
            }
        }

        // INDIGO_TEST_SHUTDOWN_TIMEOUT
        if let Ok(timeout_str) = std::env::var("INDIGO_TEST_SHUTDOWN_TIMEOUT") {
            if let Ok(secs) = timeout_str.parse::<u64>() {
                config.shutdown_timeout = Duration::from_secs(secs);
            }
        }

        // INDIGO_TEST_SKIP_SERVER
        if let Ok(skip_str) = std::env::var("INDIGO_TEST_SKIP_SERVER") {
            config.skip_server_startup = skip_str.to_lowercase() == "true" || skip_str == "1";
        }

        // INDIGO_TEST_SERVER_HOST
        if let Ok(host) = std::env::var("INDIGO_TEST_SERVER_HOST") {
            config.server_host = host;
        }

        // INDIGO_TEST_LOG_LEVEL
        if let Ok(level) = std::env::var("INDIGO_TEST_LOG_LEVEL") {
            config.log_level = level;
        }

        // INDIGO_TEST_STATE_RESET_TIMEOUT
        if let Ok(timeout_str) = std::env::var("INDIGO_TEST_STATE_RESET_TIMEOUT") {
            if let Ok(secs) = timeout_str.parse::<u64>() {
                config.state_reset_timeout = Duration::from_secs(secs);
            }
        }

        config
    }

    /// Get the server address
    pub fn server_address(&self) -> String {
        format!("{}:{}", self.server_host, self.port)
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<(), String> {
        // Validate port
        if self.port == 0 {
            return Err("Port cannot be 0".to_string());
        }

        // Validate drivers
        if self.drivers.is_empty() {
            return Err("At least one driver must be specified".to_string());
        }

        // Validate timeouts
        if self.startup_timeout.as_secs() == 0 {
            return Err("Startup timeout must be greater than 0".to_string());
        }

        if self.shutdown_timeout.as_secs() == 0 {
            return Err("Shutdown timeout must be greater than 0".to_string());
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = TestConfig::default();
        assert_eq!(config.port, 7624);
        assert_eq!(config.drivers.len(), 2);
        assert_eq!(config.server_host, "localhost");
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_server_address() {
        let config = TestConfig::default();
        assert_eq!(config.server_address(), "localhost:7624");
    }

    #[test]
    fn test_validation() {
        let mut config = TestConfig::default();

        // Valid config
        assert!(config.validate().is_ok());

        // Invalid port
        config.port = 0;
        assert!(config.validate().is_err());
        config.port = 7624;

        // Invalid drivers
        config.drivers.clear();
        assert!(config.validate().is_err());
    }
}
