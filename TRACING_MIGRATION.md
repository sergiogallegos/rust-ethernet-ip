# Tracing Migration Guide

This document describes the migration from `println!` debugging to the `tracing` crate for structured, configurable logging.

## What Changed

- Added `tracing` and `tracing-subscriber` dependencies to `Cargo.toml`
- Added `init_tracing()` and `try_init_tracing()` functions to initialize tracing
- Replaced `println!` statements with appropriate tracing macros:
  - `println!("[DEBUG] ...")` → `tracing::debug!("...")`
  - `println!("[WARN] ...")` → `tracing::warn!("...")`
  - `println!("✅ ...")` → `tracing::info!("...")`
  - `println!("❌ ...")` → `tracing::error!("...")`
  - Very verbose debug → `tracing::trace!("...")`
- Replaced `eprintln!` statements with `tracing::error!` or appropriate levels

## Files Updated

- ✅ `src/lib.rs` - Added tracing initialization functions and replaced ALL println! statements (~191 statements)
- ✅ `src/tag_manager.rs` - Replaced all println! with tracing macros
- ✅ `src/ffi.rs` - Replaced all eprintln! with tracing macros
- ✅ `src/tag_path.rs` - All println! statements replaced
- ✅ All test files - Replaced all println! and eprintln! with tracing macros

## How to Use

### Basic Usage

```rust
use rust_ethernet_ip::init_tracing;

fn main() {
    // Initialize tracing (reads RUST_LOG environment variable)
    init_tracing();
    
    // Your code here - tracing macros will now work
}
```

### Setting Log Levels

Use the `RUST_LOG` environment variable to control logging:

```bash
# Show all debug and above
RUST_LOG=debug cargo run

# Show only errors
RUST_LOG=error cargo run

# Show debug for this crate only
RUST_LOG=rust_ethernet_ip=debug cargo run

# Show trace (most verbose) for this crate
RUST_LOG=rust_ethernet_ip=trace cargo run

# Show info for this crate, debug for dependencies
RUST_LOG=info,rust_ethernet_ip=debug cargo run
```

### Log Levels

- `trace` - Most verbose (all events, including very detailed debug info)
- `debug` - Debug information (development and troubleshooting)
- `info` - Informational messages (default, normal operation)
- `warn` - Warnings (potential issues)
- `error` - Errors (failures)

### In Your Code

```rust
use tracing;

// Debug information
tracing::debug!("Connecting to PLC at {}", address);

// Informational messages
tracing::info!("Session registration successful");

// Warnings
tracing::warn!("Unexpected service reply, attempting to parse anyway");

// Errors
tracing::error!("Failed to read tag: {}", error);

// Very verbose (trace level)
tracing::trace!("Raw packet data: {:02X?}", packet);
```

## Benefits

1. **Configurable**: Control log verbosity without recompiling
2. **Performance**: Can be compiled out in release builds
3. **Structured**: Better integration with logging frameworks
4. **Filtering**: Filter by module, crate, or log level
5. **Production Ready**: Standard approach for Rust applications

## Migration Complete

All `println!` and `eprintln!` statements have been successfully migrated to tracing macros:
- ✅ All source files (`src/*.rs`) - Complete
- ✅ All test files (`tests/*.rs`) - Complete
- ✅ Proper log levels assigned (trace, debug, info, warn, error)
- ✅ Code compiles successfully

## Testing

When running tests, you can control logging:

```bash
# Run tests with debug logging
RUST_LOG=debug cargo test

# Run tests with trace logging for specific module
RUST_LOG=rust_ethernet_ip::tag_manager=trace cargo test
```

## Migration Pattern

When migrating remaining `println!` statements:

1. Identify the log level:
   - `[DEBUG]` or debug info → `tracing::debug!`
   - `[WARN]` or warnings → `tracing::warn!`
   - Success messages (✅) → `tracing::info!`
   - Errors (❌) → `tracing::error!`
   - Very verbose/raw data → `tracing::trace!`

2. Remove emoji and prefixes (tracing handles formatting)
3. Update format strings if needed

Example:
```rust
// Before
println!("[DEBUG] Parsed {} tags from response", tags.len());

// After
tracing::debug!("Parsed {} tags from response", tags.len());
```
