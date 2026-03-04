#![allow(dead_code)]

//! Test Harness for INDIGO Server Integration Tests
//!
//! This module provides a comprehensive test harness for managing an INDIGO server
//! across integration tests. The harness starts the server once, keeps it running
//! across test executions, and provides state management between tests.
//!
//! # Architecture
//!
//! The harness consists of several components:
//!
//! - **TestHarness**: Global singleton that coordinates all components
//! - **ServerManager**: Manages the INDIGO server process lifecycle
//! - **HealthMonitor**: Verifies server health and readiness
//! - **StateManager**: Resets server state between tests
//! - **TestConfig**: Configuration from environment variables
//!
//! # Usage
//!
//! ## Basic Usage
//!
//! ```rust,no_run
//! use tests::harness::TestHarness;
//!
//! #[tokio::test]
//! async fn test_something() {
//!     // Initialize harness (idempotent - safe to call multiple times)
//!     TestHarness::initialize().unwrap();
//!
//!     // Reset state before test
//!     TestHarness::reset_for_test().await.unwrap();
//!
//!     // Get server address
//!     let addr = TestHarness::server_address().unwrap();
//!
//!     // Your test code here...
//! }
//! ```
//!
//! ## Configuration
//!
//! The harness is configured via environment variables:
//!
//! - `INDIGO_SERVER_PATH`: Path to indigo_server binary
//! - `INDIGO_TEST_PORT`: Port for test server (default: 7624)
//! - `INDIGO_TEST_DRIVERS`: Comma-separated driver list
//! - `INDIGO_TEST_STARTUP_TIMEOUT`: Server startup timeout in seconds
//! - `INDIGO_TEST_SHUTDOWN_TIMEOUT`: Server shutdown timeout in seconds
//! - `INDIGO_TEST_SKIP_SERVER`: Skip server startup (use existing)
//! - `INDIGO_TEST_SERVER_HOST`: Server host (default: localhost)
//! - `INDIGO_TEST_LOG_LEVEL`: Logging level
//!
//! ## Graceful Degradation
//!
//! If the INDIGO server is not available, the harness will not fail initialization.
//! Instead, tests can check availability and skip if needed:
//!
//! ```rust,no_run
//! use tests::harness::TestHarness;
//!
//! #[tokio::test]
//! async fn test_with_server() {
//!     if !TestHarness::is_available() {
//!         println!("Skipping: INDIGO server not available");
//!         return;
//!     }
//!
//!     // Test continues...
//! }
//! ```

pub mod config;
pub mod harness;
pub mod health;
pub mod server;
pub mod state;

// Re-export main types for convenience
pub use config::TestConfig;
pub use harness::TestHarness;
pub use health::{HealthMonitor, ServerStatus};
pub use server::{ServerConfig, ServerManager, ServerState};
pub use state::StateManager;
