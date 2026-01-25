// test_helpers.rs
// =========================================================================
// Common test helpers for PLC integration tests
//
// Provides utilities for:
// - Getting PLC address from environment variables
// - Checking if PLC tests should be skipped
// - Connecting to PLC with proper error handling
// =========================================================================

// Note: This module is used by test files in the tests/ directory.
// Each test file is a separate crate, so this module must be declared
// in each test file that uses it: `mod test_helpers;`

use rust_ethernet_ip::{EipClient, RoutePath};
use std::env;
use std::time::Duration;
use tokio::time::timeout;

/// Get the test PLC address from environment variable or use default
///
/// Priority:
/// 1. `TEST_PLC_ADDRESS` environment variable
/// 2. Default: `192.168.0.1:44818`
///
/// # Example
///
/// ```bash
/// # Set custom PLC address
/// export TEST_PLC_ADDRESS=192.168.1.100:44818
/// cargo test
///
/// # Or inline
/// TEST_PLC_ADDRESS=192.168.1.100:44818 cargo test
/// ```
pub fn get_test_plc_address() -> String {
    env::var("TEST_PLC_ADDRESS").unwrap_or_else(|_| "192.168.0.1:44818".to_string())
}

/// Get the CPU slot from environment variable or use default
///
/// # Example
///
/// ```bash
/// export TEST_PLC_SLOT=1
/// cargo test
/// ```
pub fn get_test_plc_slot() -> u8 {
    env::var("TEST_PLC_SLOT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0)
}

/// Check if PLC tests should be skipped
///
/// Returns `true` if `SKIP_PLC_TESTS` environment variable is set to any value
///
/// # Example
///
/// ```bash
/// # Skip all PLC-dependent tests
/// SKIP_PLC_TESTS=1 cargo test
///
/// # Or
/// export SKIP_PLC_TESTS=1
/// cargo test
/// ```
pub fn should_skip_plc_tests() -> bool {
    env::var("SKIP_PLC_TESTS").is_ok()
}

/// Connect to PLC with timeout and proper error handling
///
/// Returns `Ok(client)` if connection succeeds, or skips the test gracefully
/// if connection fails or times out.
///
/// # Arguments
///
/// * `address` - PLC address (e.g., "192.168.1.100:44818")
/// * `timeout_secs` - Connection timeout in seconds (default: 10)
///
/// # Returns
///
/// * `Some(client)` - Connected client
/// * `None` - Connection failed or timed out (test should be skipped)
pub async fn connect_to_plc(address: &str, timeout_secs: u64) -> Option<EipClient> {
    match timeout(
        Duration::from_secs(timeout_secs),
        EipClient::connect(address),
    )
    .await
    {
        Ok(Ok(client)) => Some(client),
        Ok(Err(e)) => {
            tracing::debug!("Skipping test - PLC not available at {}: {}", address, e);
            None
        }
        Err(_) => {
            tracing::debug!("Skipping test - Connection timeout to {}", address);
            None
        }
    }
}

/// Connect to PLC with route path
///
/// Similar to `connect_to_plc` but uses route path for ControlLogix/CompactLogix
pub async fn connect_to_plc_with_route(
    address: &str,
    route: RoutePath,
    timeout_secs: u64,
) -> Option<EipClient> {
    match timeout(
        Duration::from_secs(timeout_secs),
        EipClient::with_route_path(address, route),
    )
    .await
    {
        Ok(Ok(client)) => Some(client),
        Ok(Err(e)) => {
            tracing::debug!("Skipping test - PLC not available at {}: {}", address, e);
            None
        }
        Err(_) => {
            tracing::debug!("Skipping test - Connection timeout to {}", address);
            None
        }
    }
}

/// Check if a PLC is available at the given address
///
/// Returns `true` if connection succeeds within timeout, `false` otherwise
pub async fn is_plc_available(address: &str, timeout_secs: u64) -> bool {
    connect_to_plc(address, timeout_secs).await.is_some()
}

/// Get test configuration from environment variables
///
/// Returns a tuple of (address, slot, should_skip)
pub fn get_test_config() -> (String, u8, bool) {
    (
        get_test_plc_address(),
        get_test_plc_slot(),
        should_skip_plc_tests(),
    )
}
