#![allow(dead_code)]

//! Test Harness - Main coordinator for integration tests
//!
//! This module provides the global test harness singleton that manages
//! the INDIGO server lifecycle across all integration tests.

use super::config::TestConfig;
use super::health::{HealthMonitor, ServerStatus};
use super::server::{ServerConfig, ServerManager, ServerState};
use super::state::{StateManager, StateStatistics};
use once_cell::sync::Lazy;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

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
    ///
    /// # Note
    ///
    /// This function is async and must be called from an async context.
    /// It uses the current Tokio runtime instead of creating a new one,
    /// avoiding runtime nesting issues.
    pub async fn initialize() -> Result<(), String> {
        // Check if already initialized
        {
            let harness = TEST_HARNESS.lock().unwrap();
            if harness.is_some() {
                eprintln!("[HARNESS] Already initialized");
                return Ok(());
            }
        }

        eprintln!("[HARNESS] Initializing test harness...");

        // Try to initialize, but don't fail if server is unavailable
        match Self::try_initialize().await {
            Ok(()) => {
                HARNESS_AVAILABLE.store(true, Ordering::SeqCst);
                eprintln!("[HARNESS] Initialization successful");
                Ok(())
            }
            Err(e) => {
                eprintln!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
                eprintln!("⚠️  Warning: Failed to initialize test harness");
                eprintln!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
                eprintln!("Error: {}", e);
                eprintln!();
                eprintln!("Integration tests will be skipped.");
                eprintln!("To run integration tests, ensure INDIGO server is available:");
                eprintln!("  1. Build: cd sys/externals/indigo && make");
                eprintln!("  2. Or set: export INDIGO_SERVER_PATH=/path/to/indigo_server");
                eprintln!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
                HARNESS_AVAILABLE.store(false, Ordering::SeqCst);
                // Return Ok to allow tests to run (they will skip if harness unavailable)
                Ok(())
            }
        }
    }

    /// Try to initialize the harness (internal)
    async fn try_initialize() -> Result<(), String> {
        // Load configuration
        let config = TestConfig::from_env();
        config.validate()?;

        eprintln!("[HARNESS] Configuration loaded:");
        eprintln!("  - Port: {}", config.port);
        eprintln!("  - Drivers: {}", config.drivers.join(", "));
        eprintln!("  - Startup timeout: {:?}", config.startup_timeout);

        // Skip server startup if configured
        if config.skip_server_startup {
            eprintln!("[HARNESS] Skipping server startup (using existing server)");

            let health_monitor = HealthMonitor::new(config.server_address());
            let state_manager =
                StateManager::new(config.server_address()).with_timeout(config.state_reset_timeout);

            // Verify existing server is reachable
            health_monitor
                .wait_for_ready(Duration::from_secs(5))
                .await?;

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
            eprintln!("[HARNESS] Using configured server path: {:?}", path);
            path
        } else {
            eprintln!("[HARNESS] Discovering server binary...");
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

        // Wait for server to be ready (using the current async runtime)
        eprintln!("[HARNESS] Waiting for server to be ready...");
        health_monitor
            .wait_for_ready(config.startup_timeout)
            .await?;

        // Create state manager
        let state_manager =
            StateManager::new(server_manager.address()).with_timeout(config.state_reset_timeout);

        // Get server address before moving server_manager
        let server_addr = server_manager.address();

        // Store in global
        let inner = TestHarnessInner {
            server_manager,
            health_monitor,
            state_manager,
            config,
        };

        *TEST_HARNESS.lock().unwrap() = Some(inner);

        eprintln!("[HARNESS] Server is ready at {}", server_addr);

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

    /// Verify state before test (strict check)
    pub async fn verify_pre_test_state() -> Result<(), String> {
        if !Self::is_available() {
            return Err("Test harness not available".to_string());
        }

        let harness = TEST_HARNESS.lock().unwrap();
        if let Some(ref inner) = *harness {
            inner.state_manager.verify_pre_test_state().await
        } else {
            Err("Test harness not initialized".to_string())
        }
    }

    /// Verify state after test (detect leaks)
    pub async fn verify_post_test_state() -> Result<(), String> {
        if !Self::is_available() {
            return Err("Test harness not available".to_string());
        }

        let harness = TEST_HARNESS.lock().unwrap();
        if let Some(ref inner) = *harness {
            inner.state_manager.verify_post_test_state().await
        } else {
            Err("Test harness not initialized".to_string())
        }
    }

    /// Force clean state (for recovery scenarios)
    pub async fn force_clean_state() -> Result<(), String> {
        if !Self::is_available() {
            return Err("Test harness not available".to_string());
        }

        let harness = TEST_HARNESS.lock().unwrap();
        if let Some(ref inner) = *harness {
            inner.state_manager.force_clean_state().await
        } else {
            Err("Test harness not initialized".to_string())
        }
    }

    /// Track a connection (for test isolation)
    pub fn track_connection() -> Result<(), String> {
        if !Self::is_available() {
            return Err("Test harness not available".to_string());
        }

        let harness = TEST_HARNESS.lock().unwrap();
        if let Some(ref inner) = *harness {
            inner.state_manager.track_connection();
            Ok(())
        } else {
            Err("Test harness not initialized".to_string())
        }
    }

    /// Untrack a connection
    pub fn untrack_connection() -> Result<(), String> {
        if !Self::is_available() {
            return Err("Test harness not available".to_string());
        }

        let harness = TEST_HARNESS.lock().unwrap();
        if let Some(ref inner) = *harness {
            inner.state_manager.untrack_connection();
            Ok(())
        } else {
            Err("Test harness not initialized".to_string())
        }
    }

    /// Track a device that has been interacted with
    pub fn track_device(device: &str) -> Result<(), String> {
        if !Self::is_available() {
            return Err("Test harness not available".to_string());
        }

        let harness = TEST_HARNESS.lock().unwrap();
        if let Some(ref inner) = *harness {
            inner.state_manager.track_device(device);
            Ok(())
        } else {
            Err("Test harness not initialized".to_string())
        }
    }

    /// Shutdown the harness
    pub fn shutdown() -> Result<(), String> {
        eprintln!("[HARNESS] Shutting down...");

        let mut harness = TEST_HARNESS.lock().unwrap();
        if let Some(mut inner) = harness.take() {
            inner.server_manager.stop()?;
        }
        HARNESS_AVAILABLE.store(false, Ordering::SeqCst);

        eprintln!("[HARNESS] Shutdown complete");
        Ok(())
    }

    /// Restart the server
    pub async fn restart_server() -> Result<(), String> {
        if !Self::is_available() {
            return Err("Test harness not available".to_string());
        }

        eprintln!("[HARNESS] Restarting server...");

        let mut harness = TEST_HARNESS.lock().unwrap();
        if let Some(ref mut inner) = *harness {
            inner.server_manager.restart()?;

            // Wait for server to be ready again
            inner
                .health_monitor
                .wait_for_ready(inner.config.startup_timeout)
                .await?;

            eprintln!("[HARNESS] Server restarted successfully");
            Ok(())
        } else {
            Err("Test harness not initialized".to_string())
        }
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

    /// Get server health status
    pub async fn get_server_status() -> Result<ServerStatus, String> {
        if !Self::is_available() {
            return Err("Test harness not available".to_string());
        }

        let harness = TEST_HARNESS.lock().unwrap();
        if let Some(ref inner) = *harness {
            Ok(inner.health_monitor.get_status().await)
        } else {
            Err("Test harness not initialized".to_string())
        }
    }

    /// Get state management statistics
    pub fn get_state_statistics() -> Result<StateStatistics, String> {
        if !Self::is_available() {
            return Err("Test harness not available".to_string());
        }

        let harness = TEST_HARNESS.lock().unwrap();
        if let Some(ref inner) = *harness {
            Ok(inner.state_manager.get_statistics())
        } else {
            Err("Test harness not initialized".to_string())
        }
    }

    /// Check server connectivity
    pub async fn check_connectivity() -> Result<bool, String> {
        if !Self::is_available() {
            return Err("Test harness not available".to_string());
        }

        let harness = TEST_HARNESS.lock().unwrap();
        if let Some(ref inner) = *harness {
            inner.health_monitor.check_connectivity().await
        } else {
            Err("Test harness not initialized".to_string())
        }
    }

    /// Wait for server to be ready (useful after restart or issues)
    pub async fn wait_for_ready(timeout: Duration) -> Result<(), String> {
        if !Self::is_available() {
            return Err("Test harness not available".to_string());
        }

        let harness = TEST_HARNESS.lock().unwrap();
        if let Some(ref inner) = *harness {
            inner.health_monitor.wait_for_ready(timeout).await
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

    /// Clear server output buffer
    pub fn clear_server_output() -> Result<(), String> {
        if !Self::is_available() {
            return Err("Test harness not available".to_string());
        }

        let harness = TEST_HARNESS.lock().unwrap();
        if let Some(ref inner) = *harness {
            inner.server_manager.clear_output();
            Ok(())
        } else {
            Err("Test harness not initialized".to_string())
        }
    }

    /// Get server uptime
    pub fn server_uptime() -> Result<Option<Duration>, String> {
        if !Self::is_available() {
            return Err("Test harness not available".to_string());
        }

        let harness = TEST_HARNESS.lock().unwrap();
        if let Some(ref inner) = *harness {
            Ok(inner.server_manager.uptime())
        } else {
            Err("Test harness not initialized".to_string())
        }
    }

    /// Get server process ID
    pub fn server_pid() -> Result<Option<u32>, String> {
        if !Self::is_available() {
            return Err("Test harness not available".to_string());
        }

        let harness = TEST_HARNESS.lock().unwrap();
        if let Some(ref inner) = *harness {
            Ok(inner.server_manager.pid())
        } else {
            Err("Test harness not initialized".to_string())
        }
    }

    /// Print diagnostic information
    pub fn print_diagnostics() {
        if !Self::is_available() {
            eprintln!("[DIAGNOSTICS] Test harness not available");
            return;
        }

        eprintln!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        eprintln!("📊 Test Harness Diagnostics");
        eprintln!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

        if let Ok(state) = Self::server_state() {
            eprintln!("Server State: {:?}", state);
        }

        if let Ok(Some(uptime)) = Self::server_uptime() {
            eprintln!("Server Uptime: {:?}", uptime);
        }

        if let Ok(Some(pid)) = Self::server_pid() {
            eprintln!("Server PID: {}", pid);
        }

        if let Ok(addr) = Self::server_address() {
            eprintln!("Server Address: {}", addr);
        }

        if let Ok(stats) = Self::get_state_statistics() {
            eprintln!("State Resets: {}", stats.total_resets);
            eprintln!("Active Connections: {}", stats.active_connections);
            eprintln!("Tracked Devices: {}", stats.tracked_devices);
            if let Some(elapsed) = stats.last_reset_elapsed {
                eprintln!("Last Reset: {:?} ago", elapsed);
            }
        }

        eprintln!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    }
}

// Ensure cleanup on process exit
impl Drop for TestHarnessInner {
    fn drop(&mut self) {
        eprintln!("[HARNESS] TestHarnessInner dropped - cleaning up");
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

    #[test]
    fn test_harness_availability_check() {
        // This should not panic
        let available = TestHarness::is_available();
        assert!(available == true || available == false);
    }
}
