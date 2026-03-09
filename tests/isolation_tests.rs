//! Test isolation verification tests
//!
//! These tests verify that the test harness properly isolates tests
//! from each other and detects resource leaks.

#[path = "harness/mod.rs"]
mod harness;

#[path = "common/mod.rs"]
mod common;

use harness::TestHarness;
use serial_test::serial;

/// Test that state verification detects dirty state
#[tokio::test]
#[serial]
async fn test_state_verification_detects_dirty_state() {
    // Initialize harness
    if let Err(_) = TestHarness::initialize().await {
        eprintln!("Skipping test: INDIGO server not available");
        return;
    }

    if !TestHarness::is_available() {
        eprintln!("Skipping test: INDIGO server not available");
        return;
    }

    // Reset to clean state
    TestHarness::reset_for_test().await.unwrap();

    // Make state dirty by tracking a connection
    TestHarness::track_connection().unwrap();

    // Verification should detect the dirty state
    let result = TestHarness::verify_post_test_state().await;
    assert!(result.is_err(), "Should detect dirty state");

    // Clean up
    TestHarness::force_clean_state().await.unwrap();
}

/// Test that reset clears tracked state
#[tokio::test]
#[serial]
async fn test_reset_clears_tracked_state() {
    if let Err(_) = TestHarness::initialize().await {
        eprintln!("Skipping test: INDIGO server not available");
        return;
    }

    if !TestHarness::is_available() {
        eprintln!("Skipping test: INDIGO server not available");
        return;
    }

    // Make state dirty
    TestHarness::track_connection().unwrap();
    TestHarness::track_device("CCD Simulator").unwrap();

    // Reset should clear state
    TestHarness::reset_for_test().await.unwrap();

    // State should now be clean
    let stats = TestHarness::get_state_statistics().unwrap();
    assert_eq!(stats.active_connections, 0, "Connections should be cleared");
    assert_eq!(stats.tracked_devices, 0, "Devices should be cleared");
}

/// Test that multiple resets work correctly
#[tokio::test]
#[serial]
async fn test_multiple_resets() {
    if let Err(_) = TestHarness::initialize().await {
        eprintln!("Skipping test: INDIGO server not available");
        return;
    }

    if !TestHarness::is_available() {
        eprintln!("Skipping test: INDIGO server not available");
        return;
    }

    let initial_stats = TestHarness::get_state_statistics().unwrap();
    let initial_resets = initial_stats.total_resets;

    // Perform multiple resets
    TestHarness::reset_for_test().await.unwrap();
    TestHarness::reset_for_test().await.unwrap();
    TestHarness::reset_for_test().await.unwrap();

    let stats = TestHarness::get_state_statistics().unwrap();
    assert_eq!(
        stats.total_resets,
        initial_resets + 3,
        "Should track reset count"
    );
}

/// Test that connection tracking works correctly
#[tokio::test]
#[serial]
async fn test_connection_tracking() {
    if let Err(_) = TestHarness::initialize().await {
        eprintln!("Skipping test: INDIGO server not available");
        return;
    }

    if !TestHarness::is_available() {
        eprintln!("Skipping test: INDIGO server not available");
        return;
    }

    // Reset to clean state
    TestHarness::reset_for_test().await.unwrap();

    // Track connections
    TestHarness::track_connection().unwrap();
    TestHarness::track_connection().unwrap();

    let stats = TestHarness::get_state_statistics().unwrap();
    assert_eq!(stats.active_connections, 2, "Should track 2 connections");

    // Untrack one
    TestHarness::untrack_connection().unwrap();

    let stats = TestHarness::get_state_statistics().unwrap();
    assert_eq!(stats.active_connections, 1, "Should have 1 connection left");

    // Clean up
    TestHarness::force_clean_state().await.unwrap();
}

/// Test that device tracking works correctly
#[tokio::test]
#[serial]
async fn test_device_tracking() {
    if let Err(_) = TestHarness::initialize().await {
        eprintln!("Skipping test: INDIGO server not available");
        return;
    }

    if !TestHarness::is_available() {
        eprintln!("Skipping test: INDIGO server not available");
        return;
    }

    // Reset to clean state
    TestHarness::reset_for_test().await.unwrap();

    let initial_stats = TestHarness::get_state_statistics().unwrap();
    let initial_devices = initial_stats.tracked_devices;

    // Track devices
    TestHarness::track_device("CCD Simulator Test").unwrap();
    TestHarness::track_device("Mount Simulator Test").unwrap();

    let stats = TestHarness::get_state_statistics().unwrap();
    assert_eq!(
        stats.tracked_devices,
        initial_devices + 2,
        "Should track 2 additional devices"
    );

    // Tracking same device again should not duplicate
    TestHarness::track_device("CCD Simulator Test").unwrap();

    let stats = TestHarness::get_state_statistics().unwrap();
    assert_eq!(
        stats.tracked_devices,
        initial_devices + 2,
        "Should still have same number of devices"
    );

    // Clean up
    TestHarness::force_clean_state().await.unwrap();
}

/// Test that force clean works even with dirty state
#[tokio::test]
async fn test_force_clean_state() {
    if let Err(_) = TestHarness::initialize().await {
        eprintln!("Skipping test: INDIGO server not available");
        return;
    }

    if !TestHarness::is_available() {
        eprintln!("Skipping test: INDIGO server not available");
        return;
    }

    // Make state very dirty
    TestHarness::track_connection().unwrap();
    TestHarness::track_connection().unwrap();
    TestHarness::track_device("CCD Simulator").unwrap();
    TestHarness::track_device("Mount Simulator").unwrap();

    // Force clean should clear everything
    TestHarness::force_clean_state().await.unwrap();

    let stats = TestHarness::get_state_statistics().unwrap();
    assert_eq!(stats.active_connections, 0, "Should clear connections");
    assert_eq!(stats.tracked_devices, 0, "Should clear devices");
}

/// Test using the common setup utilities
#[tokio::test]
async fn test_common_setup_test() {
    let result = common::setup_test().await;

    if result.is_err() {
        eprintln!("Skipping test: INDIGO server not available");
        return;
    }

    let addr = result.unwrap();
    assert!(!addr.is_empty(), "Should return server address");

    // Clean up
    TestHarness::force_clean_state().await.unwrap();
}

/// Test using the common setup with verification
#[tokio::test]
async fn test_common_setup_with_verification() {
    let result = common::setup_test_with_verification().await;

    if result.is_err() {
        eprintln!("Skipping test: INDIGO server not available");
        return;
    }

    let (addr, _guard) = result.unwrap();
    assert!(!addr.is_empty(), "Should return server address");

    // Guard will verify state on drop
}

/// Test using the common setup lenient mode
#[tokio::test]
async fn test_common_setup_lenient() {
    let result = common::setup_test_lenient().await;

    if result.is_err() {
        eprintln!("Skipping test: INDIGO server not available");
        return;
    }

    let (addr, _guard) = result.unwrap();
    assert!(!addr.is_empty(), "Should return server address");

    // Guard will verify state on drop
}

/// Test ConnectionGuard RAII pattern
#[tokio::test]
#[serial]
async fn test_connection_guard() {
    if let Err(_) = TestHarness::initialize().await {
        eprintln!("Skipping test: INDIGO server not available");
        return;
    }

    if !TestHarness::is_available() {
        eprintln!("Skipping test: INDIGO server not available");
        return;
    }

    TestHarness::reset_for_test().await.unwrap();

    {
        let _guard = common::ConnectionGuard::new().unwrap();

        let stats = TestHarness::get_state_statistics().unwrap();
        assert_eq!(stats.active_connections, 1, "Should track connection");
    } // Guard drops here

    let stats = TestHarness::get_state_statistics().unwrap();
    assert_eq!(stats.active_connections, 0, "Should untrack connection");
}

/// Test DeviceGuard RAII pattern
#[tokio::test]
#[serial]
async fn test_device_guard() {
    if let Err(_) = TestHarness::initialize().await {
        eprintln!("Skipping test: INDIGO server not available");
        return;
    }

    if !TestHarness::is_available() {
        eprintln!("Skipping test: INDIGO server not available");
        return;
    }

    TestHarness::reset_for_test().await.unwrap();

    {
        let guard = common::DeviceGuard::new("CCD Simulator").unwrap();
        assert_eq!(guard.device(), "CCD Simulator");

        let stats = TestHarness::get_state_statistics().unwrap();
        assert_eq!(stats.tracked_devices, 1, "Should track device");
    }

    // Clean up
    TestHarness::force_clean_state().await.unwrap();
}

/// Test that statistics track verification failures
#[tokio::test]
#[serial]
async fn test_statistics_track_failures() {
    if let Err(_) = TestHarness::initialize().await {
        eprintln!("Skipping test: INDIGO server not available");
        return;
    }

    if !TestHarness::is_available() {
        eprintln!("Skipping test: INDIGO server not available");
        return;
    }

    TestHarness::reset_for_test().await.unwrap();

    let initial_stats = TestHarness::get_state_statistics().unwrap();
    let initial_failures = initial_stats.verification_failures;

    // Make state dirty and try to verify
    TestHarness::track_connection().unwrap();

    // Verification should fail because state is dirty
    let verify_result = TestHarness::verify_pre_test_state().await;
    assert!(
        verify_result.is_err(),
        "Verification should fail with dirty state"
    );

    let stats = TestHarness::get_state_statistics().unwrap();
    assert_eq!(
        stats.verification_failures,
        initial_failures + 1,
        "Should track exactly one verification failure (initial: {}, current: {})",
        initial_failures,
        stats.verification_failures
    );

    // Clean up
    TestHarness::force_clean_state().await.unwrap();
}

/// Test that statistics track resource leaks
#[tokio::test]
#[serial]
async fn test_statistics_track_leaks() {
    if let Err(_) = TestHarness::initialize().await {
        eprintln!("Skipping test: INDIGO server not available");
        return;
    }

    if !TestHarness::is_available() {
        eprintln!("Skipping test: INDIGO server not available");
        return;
    }

    TestHarness::reset_for_test().await.unwrap();

    let initial_stats = TestHarness::get_state_statistics().unwrap();
    let initial_leaks = initial_stats.resource_leaks;

    // Make state dirty and verify post-test (should detect leak)
    TestHarness::track_connection().unwrap();
    let _ = TestHarness::verify_post_test_state().await;

    let stats = TestHarness::get_state_statistics().unwrap();
    assert!(
        stats.resource_leaks > initial_leaks,
        "Should track resource leaks"
    );

    // Clean up
    TestHarness::force_clean_state().await.unwrap();
}

/// Test isolation between sequential tests
///
/// This test simulates two sequential tests to verify they don't interfere
#[tokio::test]
async fn test_isolation_between_sequential_tests() {
    if let Err(_) = TestHarness::initialize().await {
        eprintln!("Skipping test: INDIGO server not available");
        return;
    }

    if !TestHarness::is_available() {
        eprintln!("Skipping test: INDIGO server not available");
        return;
    }

    // Simulate first test
    {
        TestHarness::reset_for_test().await.unwrap();
        TestHarness::track_connection().unwrap();
        TestHarness::track_device("CCD Simulator").unwrap();
        // First test ends with dirty state (simulating a leak)
    }

    // Simulate second test
    {
        TestHarness::reset_for_test().await.unwrap();

        // State should be clean for second test
        let stats = TestHarness::get_state_statistics().unwrap();
        assert_eq!(
            stats.active_connections, 0,
            "Second test should start with clean connections"
        );
        assert_eq!(
            stats.tracked_devices, 0,
            "Second test should start with clean devices"
        );
    }

    // Clean up
    TestHarness::force_clean_state().await.unwrap();
}
