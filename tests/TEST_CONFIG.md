# Test Configuration Quick Reference

## Environment Variables

### `TEST_PLC_ADDRESS`
Set the PLC IP address and port for tests.

**Default:** `192.168.0.1:44818`

**Examples:**
```bash
# Linux/Mac
export TEST_PLC_ADDRESS=192.168.1.100:44818
cargo test

# Windows PowerShell
$env:TEST_PLC_ADDRESS="192.168.1.100:44818"
cargo test

# Windows CMD
set TEST_PLC_ADDRESS=192.168.1.100:44818
cargo test

# Inline (all platforms)
TEST_PLC_ADDRESS=192.168.1.100:44818 cargo test
```

### `TEST_PLC_SLOT`
Set the CPU slot number (0 for CompactLogix, 1+ for ControlLogix).

**Default:** `0`

**Examples:**
```bash
export TEST_PLC_SLOT=1
cargo test
```

### `SKIP_PLC_TESTS`
Skip all PLC-dependent tests (run only unit tests).

**Examples:**
```bash
# Skip all PLC tests
SKIP_PLC_TESTS=1 cargo test

# Windows PowerShell
$env:SKIP_PLC_TESTS="1"
cargo test
```

## Common Usage Patterns

### Run tests with your PLC
```bash
TEST_PLC_ADDRESS=192.168.1.100:44818 cargo test
```

### Run tests without PLC (unit tests only)
```bash
SKIP_PLC_TESTS=1 cargo test
```

### Run specific test file
```bash
TEST_PLC_ADDRESS=192.168.1.100:44818 cargo test --test route_path_operations_tests
```

### Full configuration
```bash
TEST_PLC_ADDRESS=192.168.1.100:44818 TEST_PLC_SLOT=1 cargo test
```
