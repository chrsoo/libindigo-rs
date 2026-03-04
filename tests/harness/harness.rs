#![allow(dead_code)]

//! Test Harness - Main coordinator for integration tests
//!
//! This module provides the global test harness singleton that manages
//! the INDIGO server lifecycle across all integration tests.

use super::config::TestConfig;
use super::health::HealthMonitor;
use super::server::{ServerConfig, ServerManager, ServerState};
use super::state::StateManager;
use once_cell::sync::Lazy;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

/// Global flag indicating if harness is available
static HARNESS_AVAILABLE: AtomicBool = AtomicBool::new(false);

/// Global test harness instance
static TEST_HARNESS: Lazy<Arc<Mutex<Option<TestHarnessInner>>>> =
    Lazy::new(|| Arc::new(Mutex::new(None)));

/// Inner test harness structure
struct TestHarnessInner {
    server_manager: ServerManager,
    health_monitor: HealthMonitor,
    state_manager: StateManager,
    config: TestConfig,
}

/// Test harness - provides global access to server management
pub struct TestHarness;

impl TestHarness {
    /// Initialize the test harness
    ///
    /// This should be called once before running integration tests.
    /// It will start the INDIGO server and wait for it to be ready.
    pub fn initialize() -> Result<(), String> {
        // Check if already initialized
        {
            let harness = TEST_HARNESS.lock().unwrap();
            if harness.is_some() {
                return Ok(());
            }
        }

        // Try to initialize, but don't fail if server is unavailable
        match Self::try_initialize() {
            Ok(()) => {
                HARNESS_AVAILABLE.store(true, Ordering::SeqCst);
                Ok(())
            }
            Err(e) => {
                eprintln!("Warning: Failed to initialize test harness: {}", e);
                eprintln!("Integration tests will be skipped.");
                eprintln!("To run integration tests, ensure INDIGO server is available.");
                HARNESS_AVAILABLE.store(false, Ordering::SeqCst);
                // Return Ok to allow tests to run (they will skip if harness unavailable)
                Ok(())
            }
        }
    }

    /// Try to initialize the harness (internal)
    fn try_initialize() -> Result<(), String> {
        // Load configuration
        let config = TestConfig::from_env();
        config.validate()?;

        // Skip server startup if configured
        if config.skip_server_startup {
            let health_monitor = HealthMonitor::new(config.server_address());
            let state_manager =
                StateManager::new(config.server_address()).with_timeout(config.state_reset_timeout);

            // Create a dummy server manager (no process)
            let server_config = ServerConfig {
                binary_path: std::path::PathBuf::from(""),
                port: config.port,
                drivers: config.drivers.clone(),
                startup_timeout: config.startup_timeout,
                shutdown_timeout: config.shutdown_timeout,
            };
            let server_manager = ServerManager::new(server_config);

            let inner = TestHarnessInner {
                server_manager,
                health_monitor,
                state_manager,
                config,
            };

            *TEST_HARNESS.lock().unwrap() = Some(inner);
            return Ok(());
        }

        // Discover server binary
        let binary_path = if let Some(path) = config.server_binary_path.clone() {
            path
        } else {
            ServerManager::discover_binary()?
        };

        // Create server configuration
        let server_config = ServerConfig {
            binary_path,
            port: config.port,
            drivers: config.drivers.clone(),
            startup_timeout: config.startup_timeout,
            shutdown_timeout: config.shutdown_timeout,
        };

        // Create server manager and start server
        let mut server_manager = ServerManager::new(server_config);
        server_manager.start()?;

        // Create health monitor
        let health_monitor = HealthMonitor::new(server_manager.address());

        // Wait for server to be ready
        let runtime = tokio::runtime::Runtime::new()
            .map_err(|e| format!("Failed to create tokio runtime: {}", e))?;

        runtime.block_on(async { health_monitor.wait_for_ready(config.startup_timeout).await })?;

        // Create state manager
        let state_manager =
            StateManager::new(server_manager.address()).with_timeout(config.state_reset_timeout);

        // Store in global
        let inner = TestHarnessInner {
            server_manager,
            health_monitor,
            state_manager,
            config,
        };

        *TEST_HARNESS.lock().unwrap() = Some(inner);

        Ok(())
    }

    /// Check if harness is available
    pub fn is_available() -> bool {
        HARNESS_AVAILABLE.load(Ordering::SeqCst)
    }

    /// Get server address
    pub fn server_address() -> Result<String, String> {
        if !Self::is_available() {
            return Err("Test harness not available".to_string());
        }

        let harness = TEST_HARNESS.lock().unwrap();
        if let Some(ref inner) = *harness {
            Ok(inner.server_manager.address())
        } else {
            Err("Test harness not initialized".to_string())
        }
    }

    /// Reset state between tests
    pub async fn reset_for_test() -> Result<(), String> {
        if !Self::is_available() {
            return Err("Test harness not available".to_string());
        }

        let harness = TEST_HARNESS.lock().unwrap();
        if let Some(ref inner) = *harness {
            inner.state_manager.reset_state().await
        } else {
            Err("Test harness not initialized".to_string())
        }
    }

    /// Shutdown the harness
    pub fn shutdown() -> Result<(), String> {
        let mut harness = TEST_HARNESS.lock().unwrap();
        if let Some(mut inner) = harness.take() {
            inner.server_manager.stop()?;
        }
        HARNESS_AVAILABLE.store(false, Ordering::SeqCst);
        Ok(())
    }

    /// Get server state
    pub fn server_state() -> Result<ServerState, String> {
        if !Self::is_available() {
            return Err("Test harness not available".to_string());
        }

        let harness = TEST_HARNESS.lock().unwrap();
        if let Some(ref inner) = *harness {
            Ok(inner.server_manager.state())
        } else {
            Err("Test harness not initialized".to_string())
        }
    }

    /// Get server output (for debugging)
    pub fn server_output() -> Result<Vec<String>, String> {
        if !Self::is_available() {
            return Err("Test harness not available".to_string());
        }

        let harness = TEST_HARNESS.lock().unwrap();
        if let Some(ref inner) = *harness {
            Ok(inner.server_manager.get_output())
        } else {
            Err("Test harness not initialized".to_string())
        }
    }

    /// Get last N lines of server output (for debugging)
    pub fn tail_server_output(lines: usize) -> Result<Vec<String>, String> {
        if !Self::is_available() {
            return Err("Test harness not available".to_string());
        }

        let harness = TEST_HARNESS.lock().unwrap();
        if let Some(ref inner) = *harness {
            Ok(inner.server_manager.tail_output(lines))
        } else {
            Err("Test harness not initialized".to_string())
        }
    }
}

// Ensure cleanup on process exit
impl Drop for TestHarnessInner {
    fn drop(&mut self) {
        let _ = self.server_manager.stop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_harness_not_initialized() {
        // Before initialization, harness should not be available
        // (unless it was initialized by another test)
        let addr = TestHarness::server_address();
        if !TestHarness::is_available() {
            assert!(addr.is_err());
        }
    }
}
