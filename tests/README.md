# Test Configuration Guide

This guide explains how to configure and run tests for the `rust-ethernet-ip` library.

## Environment Variables

### `TEST_PLC_ADDRESS`

Specifies the PLC IP address and port for integration tests.

**Default:** `192.168.0.1:44818`

**Examples:**
```bash
# Set custom PLC address
export TEST_PLC_ADDRESS=192.168.1.100:44818
cargo test

# Or inline
TEST_PLC_ADDRESS=192.168.1.100:44818 cargo test

# Windows PowerShell
$env:TEST_PLC_ADDRESS="192.168.1.100:44818"
cargo test

# Windows CMD
set TEST_PLC_ADDRESS=192.168.1.100:44818
cargo test
```

### `TEST_PLC_SLOT`

Specifies the CPU slot number for ControlLogix/CompactLogix PLCs.

**Default:** `0` (CompactLogix)

**Examples:**
```bash
# For ControlLogix with CPU in slot 1
export TEST_PLC_SLOT=1
cargo test

# For CompactLogix (default slot 0)
export TEST_PLC_SLOT=0
cargo test
```

### `SKIP_PLC_TESTS`

When set to any value, all PLC-dependent tests will be skipped gracefully.

**Use cases:**
- Running tests without a physical PLC available
- Running only unit tests
- CI/CD environments without PLC access

**Examples:**
```bash
# Skip all PLC tests
export SKIP_PLC_TESTS=1
cargo test

# Or inline
SKIP_PLC_TESTS=1 cargo test

# Windows PowerShell
$env:SKIP_PLC_TESTS="1"
cargo test

# Windows CMD
set SKIP_PLC_TESTS=1
cargo test
```

## Running Tests

### Run All Tests (with PLC)

```bash
# Using default PLC address (192.168.0.1:44818)
cargo test

# Using custom PLC address
TEST_PLC_ADDRESS=192.168.1.100:44818 cargo test
```

### Run Tests Without PLC

```bash
# Skip all PLC-dependent tests
SKIP_PLC_TESTS=1 cargo test

# This will run only unit tests and skip integration tests
```

### Run Specific Test Categories

```bash
# Run only unit tests (no PLC required)
cargo test --lib

# Run integration tests (requires PLC)
cargo test --test integration_test

# Run with ignored tests (requires PLC)
cargo test -- --ignored
```

### Run Tests with Custom Configuration

```bash
# Full custom configuration
TEST_PLC_ADDRESS=192.168.1.100:44818 TEST_PLC_SLOT=1 cargo test

# Skip PLC tests
SKIP_PLC_TESTS=1 cargo test

# Run specific test file
TEST_PLC_ADDRESS=192.168.1.100:44818 cargo test --test route_path_operations_tests
```

## Test Helper Functions

The `test_helpers` module provides convenient functions for tests:

- `get_test_plc_address()` - Get PLC address from env or default
- `get_test_plc_slot()` - Get CPU slot from env or default
- `should_skip_plc_tests()` - Check if tests should be skipped
- `connect_to_plc(address, timeout)` - Connect with timeout handling
- `connect_to_plc_with_route(address, route, timeout)` - Connect with route path
- `is_plc_available(address, timeout)` - Check PLC availability
- `get_test_config()` - Get all config as tuple (address, slot, should_skip)

## Example Test Usage

```rust
use test_helpers::{connect_to_plc, get_test_config, should_skip_plc_tests};

#[tokio::test]
async fn test_example() {
    // Skip if SKIP_PLC_TESTS is set
    if should_skip_plc_tests() {
        tracing::debug!("Skipping test - SKIP_PLC_TESTS is set");
        return;
    }

    // Get configuration
    let (plc_address, slot, _) = get_test_config();
    
    // Connect with timeout handling
    let mut client = match connect_to_plc(&plc_address, 10).await {
        Some(client) => client,
        None => return, // Gracefully skip if PLC unavailable
    };

    // Run your test...
}
```

## Troubleshooting

### Tests are being skipped unexpectedly

- Check if `SKIP_PLC_TESTS` is set: `echo $SKIP_PLC_TESTS` (Linux/Mac) or `echo %SKIP_PLC_TESTS%` (Windows)
- Verify PLC is accessible: `ping <PLC_IP>`
- Check PLC address: `echo $TEST_PLC_ADDRESS` (Linux/Mac) or `echo %TEST_PLC_ADDRESS%` (Windows)

### Connection timeouts

- Increase timeout in test code (default is 10 seconds)
- Verify network connectivity to PLC
- Check firewall settings
- Ensure PLC is powered on and connected

### Wrong PLC slot

- Set `TEST_PLC_SLOT` to the correct slot number
- For CompactLogix, use slot 0
- For ControlLogix, check the CPU slot number
