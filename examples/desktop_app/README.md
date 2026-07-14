# Rust EtherNet/IP Desktop Application

A native Rust desktop application built with `egui` for testing and demonstrating the Rust EtherNet/IP library capabilities.

## Features

- **Connection Management**: Connect to PLCs with optional RoutePath support for ControlLogix systems
- **Tag Operations**: Read and write tags of various types (DINT, REAL, BOOL, INT, STRING, UDT)
- **Array Operations**: Read and write individual array elements
- **UDT Operations**: Read UDTs and UDT members
- **Activity Log**: Real-time logging of all operations

## Running

```bash
cd examples/desktop_app
cargo run --release
```

## Usage

1. **Connect to PLC**:
   - Enter PLC IP address (e.g., `192.168.1.100` or `192.168.1.100:44818`)
   - For ControlLogix systems, check "Use Route Path" and set CPU slot (0-31)
   - Click "Connect"

2. **Tag Operations**:
   - Enter tag name
   - Select data type
   - Enter value (for writes)
   - Click "Read" or "Write"

3. **Array Operations**:
   - Enter array name (e.g., `gTestArray_DINT`)
   - Set array index
   - Enter value (for writes)
   - Click "Read Element" or "Write Element"

4. **UDT Operations**:
   - Enter UDT name (e.g., `gTestUDT`)
   - Click "Read UDT" to read the entire UDT
   - Enter member path (e.g., `gTestUDT.Member1_DINT`)
   - Click "Read UDT Member" to read a specific member

## Known Limitations

- **UDT Array Element Member Writes**: Cannot write directly to UDT array element members (e.g., `gTestUDT_Array[0].Member1_DINT`). This is a PLC firmware limitation, not a library bug.
- **UDT Member Parsing**: Full UDT member parsing from raw bytes requires UDT definition conversion which is not fully implemented in the desktop app. Direct tag access (e.g., `gTestUDT.Member1_DINT`) should work for most cases.

## Requirements

- Rust 1.88+
- Tokio runtime
- egui/eframe for GUI

## Building

```bash
cargo build --release
```

The executable will be in `target/release/desktop_app` (or `target/release/desktop_app.exe` on Windows).
