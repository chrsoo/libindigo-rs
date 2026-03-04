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
    TestHarness::initialize()?;

    if !TestHarness::is_available() {
        return Err("INDIGO server not available".into());
    }

    TestHarness::reset_for_test().await?;

    Ok(TestHarness::server_address()?)
}
