//! Common test utilities
//!
//! This module provides shared utilities for integration tests,
//! including fixtures, assertions, and test data builders.

use std::error::Error;

/// Setup function for integration tests that use the test harness.
///
/// This function:
/// 1. Initializes the test harness (idempotent)
/// 2. Checks if the INDIGO server is available
/// 3. Resets the server state for the test
/// 4. Returns the server address
///
/// # Returns
///
/// - `Ok(String)` - The server address to connect to
/// - `Err(Box<dyn Error>)` - If setup fails (server unavailable, etc.)
///
/// # Example
///
/// ```rust
/// let addr = match common::setup_test().await {
///     Ok(addr) => addr,
///     Err(e) => {
///         eprintln!("Skipping test: {}", e);
///         return;
///     }
/// };
/// ```
pub async fn setup_test() -> Result<String, Box<dyn Error>> {
    use crate::harness::TestHarness;

    // Initialize the test harness
    // This will fail gracefully if the server is not available
    TestHarness::initialize().await?;

    if !TestHarness::is_available() {
        return Err("INDIGO server not available".into());
    }

    TestHarness::reset_for_test().await?;

    Ok(TestHarness::server_address()?)
}

/// Setup function with strict state verification.
///
/// This function performs the same setup as `setup_test()` but also:
/// - Verifies pre-test state is clean
/// - Returns a cleanup guard that verifies post-test state
///
/// # Returns
///
/// - `Ok((String, TestGuard))` - The server address and cleanup guard
/// - `Err(Box<dyn Error>)` - If setup fails or state is not clean
///
/// # Example
///
/// ```rust
/// let (addr, _guard) = match common::setup_test_with_verification().await {
///     Ok(result) => result,
///     Err(e) => {
///         eprintln!("Skipping test: {}", e);
///         return;
///     }
/// };
/// // Test code here...
/// // Guard automatically verifies state on drop
/// ```
pub async fn setup_test_with_verification() -> Result<(String, TestGuard), Box<dyn Error>> {
    use crate::harness::TestHarness;

    // Initialize the test harness
    TestHarness::initialize().await?;

    if !TestHarness::is_available() {
        return Err("INDIGO server not available".into());
    }

    // Reset state
    TestHarness::reset_for_test().await?;

    // Verify pre-test state is clean
    TestHarness::verify_pre_test_state().await?;

    let addr = TestHarness::server_address()?;
    let guard = TestGuard::new();

    Ok((addr, guard))
}

/// Setup function with lenient state verification.
///
/// This function is similar to `setup_test_with_verification()` but:
/// - Warns about dirty state instead of failing
/// - Forces clean state if needed
/// - Still verifies post-test state
///
/// Use this when you need verification but want to be more forgiving
/// of pre-existing state issues.
///
/// # Example
///
/// ```rust
/// let (addr, _guard) = common::setup_test_lenient().await?;
/// ```
pub async fn setup_test_lenient() -> Result<(String, TestGuard), Box<dyn Error>> {
    use crate::harness::TestHarness;

    TestHarness::initialize().await?;

    if !TestHarness::is_available() {
        return Err("INDIGO server not available".into());
    }

    // Reset state
    TestHarness::reset_for_test().await?;

    // Try to verify pre-test state, but don't fail
    if let Err(e) = TestHarness::verify_pre_test_state().await {
        eprintln!("⚠️  Warning: Pre-test state verification failed: {}", e);
        eprintln!("⚠️  Forcing clean state...");
        TestHarness::force_clean_state().await?;
    }

    let addr = TestHarness::server_address()?;
    let guard = TestGuard::new();

    Ok((addr, guard))
}

/// Guard that verifies test state on drop.
///
/// This ensures that tests clean up properly by checking state
/// when the guard is dropped (typically at the end of the test).
pub struct TestGuard {
    verify_on_drop: bool,
}

impl TestGuard {
    /// Create a new test guard
    pub fn new() -> Self {
        Self {
            verify_on_drop: true,
        }
    }

    /// Disable verification on drop (for tests that intentionally leave state dirty)
    pub fn disable_verification(&mut self) {
        self.verify_on_drop = false;
    }

    /// Manually verify state (useful for mid-test checks)
    pub async fn verify_state(&self) -> Result<(), Box<dyn Error>> {
        use crate::harness::TestHarness;
        TestHarness::verify_post_test_state().await?;
        Ok(())
    }
}

impl Drop for TestGuard {
    fn drop(&mut self) {
        if !self.verify_on_drop {
            return;
        }

        // We can't use async in Drop, so we just log a warning
        // The actual verification happens in the test harness reset
        use crate::harness::TestHarness;

        // Get current state info
        if let Ok(stats) = TestHarness::get_state_statistics() {
            if stats.active_connections > 0 || stats.tracked_devices > 0 {
                eprintln!("⚠️  TestGuard: Potential resource leak detected!");
                eprintln!("    Active connections: {}", stats.active_connections);
                eprintln!("    Tracked devices: {}", stats.tracked_devices);
                eprintln!("    Tests should clean up all resources before completion.");
            }
        }
    }
}

impl Default for TestGuard {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper to track a connection for test isolation
pub fn track_connection() -> Result<(), Box<dyn Error>> {
    use crate::harness::TestHarness;
    TestHarness::track_connection()?;
    Ok(())
}

/// Helper to untrack a connection
pub fn untrack_connection() -> Result<(), Box<dyn Error>> {
    use crate::harness::TestHarness;
    TestHarness::untrack_connection()?;
    Ok(())
}

/// Helper to track a device interaction
pub fn track_device(device: &str) -> Result<(), Box<dyn Error>> {
    use crate::harness::TestHarness;
    TestHarness::track_device(device)?;
    Ok(())
}

/// RAII guard for connection tracking
///
/// Automatically tracks and untracks a connection.
///
/// # Example
///
/// ```rust
/// {
///     let _conn = ConnectionGuard::new()?;
///     // Connection is tracked
///     // ... test code ...
/// } // Connection is automatically untracked
/// ```
pub struct ConnectionGuard;

impl ConnectionGuard {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        track_connection()?;
        Ok(Self)
    }
}

impl Drop for ConnectionGuard {
    fn drop(&mut self) {
        let _ = untrack_connection();
    }
}

/// RAII guard for device tracking
///
/// Automatically tracks a device interaction.
///
/// # Example
///
/// ```rust
/// {
///     let _device = DeviceGuard::new("CCD Simulator")?;
///     // Device is tracked
///     // ... test code ...
/// }
/// ```
pub struct DeviceGuard {
    device: String,
}

impl DeviceGuard {
    pub fn new(device: &str) -> Result<Self, Box<dyn Error>> {
        track_device(device)?;
        Ok(Self {
            device: device.to_string(),
        })
    }

    pub fn device(&self) -> &str {
        &self.device
    }
}
