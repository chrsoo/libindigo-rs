#![allow(dead_code)]

//! State Manager for INDIGO server
//!
//! This module manages server state between tests, providing lightweight
//! state reset without requiring full server restart. It tracks connections,
//! devices, and properties to ensure clean state for each test.

use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::time::sleep;

/// Tracks state information for the server
#[derive(Debug, Clone, Default)]
pub struct StateInfo {
    /// Number of active connections (tracked by tests)
    pub active_connections: usize,
    /// List of devices that have been interacted with
    pub touched_devices: Vec<String>,
    /// Number of state resets performed
    pub reset_count: usize,
    /// Last reset timestamp
    pub last_reset: Option<std::time::Instant>,
    /// Number of state verification failures
    pub verification_failures: usize,
    /// Number of resource leaks detected
    pub resource_leaks: usize,
}

/// Result of state verification
#[derive(Debug, Clone, PartialEq)]
pub enum StateVerification {
    /// State is clean and ready for testing
    Clean,
    /// State has issues but can be cleaned
    Dirty {
        active_connections: usize,
        tracked_devices: usize,
    },
    /// State has critical issues
    Critical { reason: String },
}

/// Manages server state between tests
pub struct StateManager {
    address: String,
    reset_timeout: Duration,
    state_info: Arc<Mutex<StateInfo>>,
}

impl StateManager {
    /// Create a new state manager
    pub fn new(address: String) -> Self {
        Self {
            address,
            reset_timeout: Duration::from_secs(2),
            state_info: Arc::new(Mutex::new(StateInfo::default())),
        }
    }

    /// Set the reset timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.reset_timeout = timeout;
        self
    }

    /// Reset server state between tests
    ///
    /// This performs a lightweight reset without restarting the server:
    /// 1. Verify pre-reset state and detect leaks
    /// 2. Wait for any pending operations to complete
    /// 3. Allow time for connections to close
    /// 4. Clear tracked state information
    /// 5. Provide settling time for the server
    /// 6. Verify post-reset state is clean
    pub async fn reset_state(&self) -> Result<(), String> {
        eprintln!("[STATE] Resetting server state...");

        // Pre-reset verification - detect potential leaks
        let pre_verification = self.verify_state_internal().await;
        if let StateVerification::Dirty {
            active_connections,
            tracked_devices,
        } = pre_verification
        {
            eprintln!(
                "[STATE] Warning: Dirty state detected before reset - {} connections, {} devices",
                active_connections, tracked_devices
            );
            let mut info = self.state_info.lock().unwrap();
            info.resource_leaks += 1;
        }

        // Wait for pending operations to complete
        // This gives time for any in-flight messages to be processed
        sleep(Duration::from_millis(100)).await;

        // Clear our tracked state
        {
            let mut info = self.state_info.lock().unwrap();
            info.active_connections = 0;
            info.touched_devices.clear();
            info.reset_count += 1;
            info.last_reset = Some(std::time::Instant::now());
        }

        // In a full implementation, we would:
        // - Disconnect any active test clients
        // - Reset device properties to defaults
        // - Clear message buffers
        //
        // For now, we do a simple time-based reset which is sufficient
        // for most test scenarios since each test creates its own client

        // Additional settling time
        sleep(Duration::from_millis(100)).await;

        // Post-reset verification
        let post_verification = self.verify_state_internal().await;
        if post_verification != StateVerification::Clean {
            eprintln!(
                "[STATE] Warning: State not clean after reset: {:?}",
                post_verification
            );
            let mut info = self.state_info.lock().unwrap();
            info.verification_failures += 1;
        }

        eprintln!("[STATE] State reset complete");
        Ok(())
    }

    /// Reset state with custom timeout
    pub async fn reset_state_with_timeout(&self, timeout: Duration) -> Result<(), String> {
        eprintln!("[STATE] Resetting state with custom timeout: {:?}", timeout);

        sleep(timeout / 2).await;

        {
            let mut info = self.state_info.lock().unwrap();
            info.active_connections = 0;
            info.touched_devices.clear();
            info.reset_count += 1;
            info.last_reset = Some(std::time::Instant::now());
        }

        sleep(timeout / 2).await;

        Ok(())
    }

    /// Track a connection (for test isolation)
    pub fn track_connection(&self) {
        let mut info = self.state_info.lock().unwrap();
        info.active_connections += 1;
        eprintln!(
            "[STATE] Connection tracked (total: {})",
            info.active_connections
        );
    }

    /// Untrack a connection
    pub fn untrack_connection(&self) {
        let mut info = self.state_info.lock().unwrap();
        if info.active_connections > 0 {
            info.active_connections -= 1;
        }
        eprintln!(
            "[STATE] Connection untracked (remaining: {})",
            info.active_connections
        );
    }

    /// Track a device that has been interacted with
    pub fn track_device(&self, device: &str) {
        let mut info = self.state_info.lock().unwrap();
        if !info.touched_devices.contains(&device.to_string()) {
            info.touched_devices.push(device.to_string());
            eprintln!("[STATE] Device tracked: {}", device);
        }
    }

    /// Get current state information
    pub fn get_state_info(&self) -> StateInfo {
        self.state_info.lock().unwrap().clone()
    }

    /// Disconnect all clients (placeholder for future implementation)
    ///
    /// In a full implementation, this would track active connections
    /// and close them gracefully.
    pub async fn disconnect_all_clients(&self) -> Result<(), String> {
        eprintln!("[STATE] Disconnecting all clients...");

        let active = {
            let info = self.state_info.lock().unwrap();
            info.active_connections
        };

        if active > 0 {
            eprintln!(
                "[STATE] Warning: {} active connections tracked but cannot force disconnect",
                active
            );
        }

        // Placeholder - in practice, the INDIGO server manages connections
        // and they are closed when clients disconnect
        Ok(())
    }

    /// Reset device properties to defaults (placeholder for future implementation)
    ///
    /// This would send commands to reset specific device properties.
    pub async fn reset_device_properties(&self, device: &str) -> Result<(), String> {
        eprintln!("[STATE] Resetting properties for device: {}", device);

        // Placeholder - would require protocol-level commands
        // For now, tests are responsible for their own cleanup

        // Remove from tracked devices
        {
            let mut info = self.state_info.lock().unwrap();
            info.touched_devices.retain(|d| d != device);
        }

        Ok(())
    }

    /// Reset all tracked device properties
    pub async fn reset_all_device_properties(&self) -> Result<(), String> {
        let devices = {
            let info = self.state_info.lock().unwrap();
            info.touched_devices.clone()
        };

        eprintln!("[STATE] Resetting properties for {} devices", devices.len());

        for device in devices {
            self.reset_device_properties(&device).await?;
        }

        Ok(())
    }

    /// Clear any pending operations (placeholder for future implementation)
    pub async fn clear_pending_operations(&self) -> Result<(), String> {
        eprintln!("[STATE] Clearing pending operations...");
        // Placeholder - would require server-side support
        Ok(())
    }

    /// Internal state verification (returns detailed result)
    async fn verify_state_internal(&self) -> StateVerification {
        let info = self.get_state_info();

        // Check for active connections
        if info.active_connections > 0 {
            return StateVerification::Dirty {
                active_connections: info.active_connections,
                tracked_devices: info.touched_devices.len(),
            };
        }

        // Check for touched devices
        if !info.touched_devices.is_empty() {
            return StateVerification::Dirty {
                active_connections: 0,
                tracked_devices: info.touched_devices.len(),
            };
        }

        StateVerification::Clean
    }

    /// Verify state is clean (basic check)
    pub async fn verify_clean_state(&self) -> Result<bool, String> {
        let verification = self.verify_state_internal().await;

        match verification {
            StateVerification::Clean => {
                eprintln!("[STATE] State is clean");
                Ok(true)
            }
            StateVerification::Dirty {
                active_connections,
                tracked_devices,
            } => {
                eprintln!(
                    "[STATE] Warning: Dirty state - {} connections, {} devices",
                    active_connections, tracked_devices
                );
                Ok(false)
            }
            StateVerification::Critical { reason } => {
                eprintln!("[STATE] Critical state issue: {}", reason);
                Err(reason)
            }
        }
    }

    /// Verify state before test (strict check)
    pub async fn verify_pre_test_state(&self) -> Result<(), String> {
        eprintln!("[STATE] Verifying pre-test state...");

        let verification = self.verify_state_internal().await;

        match verification {
            StateVerification::Clean => {
                eprintln!("[STATE] ✓ Pre-test state is clean");
                Ok(())
            }
            StateVerification::Dirty {
                active_connections,
                tracked_devices,
            } => {
                let msg = format!(
                    "Pre-test state is dirty: {} active connections, {} tracked devices. \
                    This indicates incomplete cleanup from a previous test.",
                    active_connections, tracked_devices
                );
                eprintln!("[STATE] ✗ {}", msg);

                // Record the failure
                {
                    let mut info = self.state_info.lock().unwrap();
                    info.verification_failures += 1;
                }

                Err(msg)
            }
            StateVerification::Critical { reason } => {
                eprintln!("[STATE] ✗ Critical pre-test state issue: {}", reason);
                Err(reason)
            }
        }
    }

    /// Verify state after test (detect leaks)
    pub async fn verify_post_test_state(&self) -> Result<(), String> {
        eprintln!("[STATE] Verifying post-test state...");

        let verification = self.verify_state_internal().await;

        match verification {
            StateVerification::Clean => {
                eprintln!("[STATE] ✓ Post-test state is clean");
                Ok(())
            }
            StateVerification::Dirty {
                active_connections,
                tracked_devices,
            } => {
                let msg = format!(
                    "Post-test state is dirty: {} active connections, {} tracked devices. \
                    This indicates the test did not clean up properly.",
                    active_connections, tracked_devices
                );
                eprintln!("[STATE] ✗ {}", msg);

                // Record the leak
                {
                    let mut info = self.state_info.lock().unwrap();
                    info.resource_leaks += 1;
                }

                Err(msg)
            }
            StateVerification::Critical { reason } => {
                eprintln!("[STATE] ✗ Critical post-test state issue: {}", reason);
                Err(reason)
            }
        }
    }

    /// Wait for state to become clean
    pub async fn wait_for_clean_state(&self, timeout: Duration) -> Result<(), String> {
        let start = std::time::Instant::now();

        while start.elapsed() < timeout {
            if self.verify_clean_state().await? {
                return Ok(());
            }
            sleep(Duration::from_millis(100)).await;
        }

        Err(format!("State did not become clean within {:?}", timeout))
    }

    /// Get statistics about state management
    pub fn get_statistics(&self) -> StateStatistics {
        let info = self.state_info.lock().unwrap();
        StateStatistics {
            total_resets: info.reset_count,
            active_connections: info.active_connections,
            tracked_devices: info.touched_devices.len(),
            last_reset_elapsed: info.last_reset.map(|instant| instant.elapsed()),
            verification_failures: info.verification_failures,
            resource_leaks: info.resource_leaks,
        }
    }

    /// Force clean state (for recovery scenarios)
    ///
    /// This aggressively clears all tracked state without verification.
    /// Use only when normal reset fails or for test cleanup.
    pub async fn force_clean_state(&self) -> Result<(), String> {
        eprintln!("[STATE] Force cleaning state...");

        {
            let mut info = self.state_info.lock().unwrap();
            info.active_connections = 0;
            info.touched_devices.clear();
        }

        // Give extra time for any lingering operations
        sleep(Duration::from_millis(200)).await;

        eprintln!("[STATE] Force clean complete");
        Ok(())
    }

    /// Get server address
    pub fn address(&self) -> &str {
        &self.address
    }
}

/// Statistics about state management
#[derive(Debug, Clone)]
pub struct StateStatistics {
    pub total_resets: usize,
    pub active_connections: usize,
    pub tracked_devices: usize,
    pub last_reset_elapsed: Option<Duration>,
    pub verification_failures: usize,
    pub resource_leaks: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_manager_creation() {
        let manager = StateManager::new("localhost:7624".to_string());
        assert_eq!(manager.address(), "localhost:7624");
    }

    #[test]
    fn test_state_manager_with_timeout() {
        let manager =
            StateManager::new("localhost:7624".to_string()).with_timeout(Duration::from_secs(5));
        assert_eq!(manager.reset_timeout, Duration::from_secs(5));
    }

    #[tokio::test]
    async fn test_reset_state() {
        let manager = StateManager::new("localhost:7624".to_string());
        let result = manager.reset_state().await;
        assert!(result.is_ok());

        let info = manager.get_state_info();
        assert_eq!(info.reset_count, 1);
        assert!(info.last_reset.is_some());
    }

    #[tokio::test]
    async fn test_verify_clean_state() {
        let manager = StateManager::new("localhost:7624".to_string());
        let result = manager.verify_clean_state().await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), true);
    }

    #[test]
    fn test_connection_tracking() {
        let manager = StateManager::new("localhost:7624".to_string());

        manager.track_connection();
        let info = manager.get_state_info();
        assert_eq!(info.active_connections, 1);

        manager.track_connection();
        let info = manager.get_state_info();
        assert_eq!(info.active_connections, 2);

        manager.untrack_connection();
        let info = manager.get_state_info();
        assert_eq!(info.active_connections, 1);
    }

    #[test]
    fn test_device_tracking() {
        let manager = StateManager::new("localhost:7624".to_string());

        manager.track_device("CCD Simulator");
        let info = manager.get_state_info();
        assert_eq!(info.touched_devices.len(), 1);
        assert!(info.touched_devices.contains(&"CCD Simulator".to_string()));

        // Tracking same device again should not duplicate
        manager.track_device("CCD Simulator");
        let info = manager.get_state_info();
        assert_eq!(info.touched_devices.len(), 1);

        manager.track_device("Mount Simulator");
        let info = manager.get_state_info();
        assert_eq!(info.touched_devices.len(), 2);
    }

    #[test]
    fn test_statistics() {
        let manager = StateManager::new("localhost:7624".to_string());

        manager.track_connection();
        manager.track_device("CCD Simulator");

        let stats = manager.get_statistics();
        assert_eq!(stats.active_connections, 1);
        assert_eq!(stats.tracked_devices, 1);
        assert_eq!(stats.total_resets, 0);
    }

    #[tokio::test]
    async fn test_multiple_resets() {
        let manager = StateManager::new("localhost:7624".to_string());

        manager.reset_state().await.unwrap();
        manager.reset_state().await.unwrap();
        manager.reset_state().await.unwrap();

        let stats = manager.get_statistics();
        assert_eq!(stats.total_resets, 3);
    }

    #[tokio::test]
    async fn test_pre_test_verification_clean() {
        let manager = StateManager::new("localhost:7624".to_string());
        let result = manager.verify_pre_test_state().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_pre_test_verification_dirty() {
        let manager = StateManager::new("localhost:7624".to_string());

        // Make state dirty
        manager.track_connection();

        let result = manager.verify_pre_test_state().await;
        assert!(result.is_err());

        let stats = manager.get_statistics();
        assert_eq!(stats.verification_failures, 1);
    }

    #[tokio::test]
    async fn test_post_test_verification_detects_leaks() {
        let manager = StateManager::new("localhost:7624".to_string());

        // Simulate a test that didn't clean up
        manager.track_connection();
        manager.track_device("CCD Simulator");

        let result = manager.verify_post_test_state().await;
        assert!(result.is_err());

        let stats = manager.get_statistics();
        assert_eq!(stats.resource_leaks, 1);
    }

    #[tokio::test]
    async fn test_force_clean_state() {
        let manager = StateManager::new("localhost:7624".to_string());

        // Make state dirty
        manager.track_connection();
        manager.track_device("CCD Simulator");

        // Force clean
        manager.force_clean_state().await.unwrap();

        // Verify clean
        let result = manager.verify_clean_state().await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), true);
    }

    #[tokio::test]
    async fn test_reset_detects_pre_existing_leaks() {
        let manager = StateManager::new("localhost:7624".to_string());

        // Make state dirty
        manager.track_connection();

        // Reset should detect the leak
        manager.reset_state().await.unwrap();

        let stats = manager.get_statistics();
        assert_eq!(stats.resource_leaks, 1);
    }
}
