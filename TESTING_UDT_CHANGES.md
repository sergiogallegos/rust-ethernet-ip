# Testing Guide for UDT Changes

This document outlines how to test the new generic UDT handling implementation.

## Overview of Changes

The UDT implementation has been refactored from:
- **Old**: `PlcValue::Udt(HashMap<String, PlcValue>)` - required hardcoded member names
- **New**: `PlcValue::Udt(UdtData)` - generic format with `symbol_id` and raw bytes

## Test Plan

### 1. Unit Tests (No PLC Required)

Run the unit tests that don't require a PLC connection:

```bash
cargo test --test udt_data_tests --lib test_udt_data_parse_with_definition
cargo test --test udt_data_tests --lib test_udt_data_from_hash_map
cargo test --test udt_data_tests --lib test_udt_data_round_trip
```

### 2. Integration Tests (Requires PLC)

#### Prerequisites
- A PLC available at `127.0.0.1:44818` (or modify address in tests)
- At least one UDT tag (e.g., "Part_Data", "MotorData", "TestUDT")

#### Run Integration Tests

```bash
# Test UDT reading returns UdtData format
cargo test --test udt_data_tests --ignored test_udt_read_returns_udt_data

# Test UDT writing with symbol_id
cargo test --test udt_data_tests --ignored test_udt_write_with_symbol_id

# Test auto-read of symbol_id when writing
cargo test --test udt_data_tests --ignored test_udt_write_auto_reads_symbol_id

# Test generic UDT handling (works with any UDT)
cargo test --test udt_data_tests --ignored test_udt_generic_any_udt
```

### 3. Manual Testing

#### Test 1: Read a UDT

```rust
use rust_ethernet_ip::{EipClient, PlcValue};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = EipClient::connect("192.168.1.100:44818").await?;
    
    // Read UDT - should return UdtData
    let value = client.read_tag("Part_Data").await?;
    
    if let PlcValue::Udt(udt_data) = value {
        println!("Symbol ID: {}", udt_data.symbol_id);
        println!("Data size: {} bytes", udt_data.data.len());
        println!("Raw data: {:02X?}", udt_data.data);
    }
    
    Ok(())
}
```

**Expected Results:**
- ✅ Returns `PlcValue::Udt(UdtData { ... })`
- ✅ `symbol_id` > 0 (valid template instance ID)
- ✅ `data` contains raw bytes from PLC

#### Test 2: Write a UDT (with symbol_id)

```rust
use rust_ethernet_ip::{EipClient, PlcValue, UdtData};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = EipClient::connect("192.168.1.100:44818").await?;
    
    // First, read to get symbol_id
    let read_value = client.read_tag("Part_Data").await?;
    let udt_data = if let PlcValue::Udt(data) = read_value {
        data
    } else {
        return Err("Not a UDT".into());
    };
    
    // Modify data (example: flip first byte)
    let mut modified_data = udt_data.data.clone();
    if !modified_data.is_empty() {
        modified_data[0] = if modified_data[0] == 0 { 0xFF } else { 0x00 };
    }
    
    // Write back with same symbol_id
    let modified_udt = UdtData {
        symbol_id: udt_data.symbol_id,
        data: modified_data,
    };
    
    client.write_tag("Part_Data", PlcValue::Udt(modified_udt)).await?;
    
    Ok(())
}
```

**Expected Results:**
- ✅ Write succeeds
- ✅ Data is written to PLC correctly

#### Test 3: Write a UDT (auto-read symbol_id)

```rust
use rust_ethernet_ip::{EipClient, PlcValue, UdtData};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = EipClient::connect("192.168.1.100:44818").await?;
    
    // Create UdtData with symbol_id = 0 (will trigger auto-read)
    let udt_data = UdtData {
        symbol_id: 0, // Will be read automatically
        data: vec![0x00, 0x00, 0x00, 0x00], // Your data
    };
    
    // Write - should automatically read symbol_id
    client.write_tag("Part_Data", PlcValue::Udt(udt_data)).await?;
    
    Ok(())
}
```

**Expected Results:**
- ✅ Write succeeds
- ✅ `symbol_id` is automatically read from tag attributes

#### Test 4: Parse UDT with Definition

```rust
use rust_ethernet_ip::{EipClient, PlcValue, UdtData};
use rust_ethernet_ip::udt::{UserDefinedType, UdtMember};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = EipClient::connect("192.168.1.100:44818").await?;
    
    // Read UDT as raw data
    let value = client.read_tag("Part_Data").await?;
    let udt_data = if let PlcValue::Udt(data) = value {
        data
    } else {
        return Err("Not a UDT".into());
    };
    
    // Get UDT definition from PLC
    let definition = client.get_udt_definition("Part_Data").await?;
    
    // Parse raw data into member values
    let members = udt_data.parse(&definition)?;
    
    // Access individual members
    for (name, value) in &members {
        println!("{}: {:?}", name, value);
    }
    
    Ok(())
}
```

**Expected Results:**
- ✅ Parsing succeeds
- ✅ Members are correctly extracted from raw bytes

### 4. Backward Compatibility Testing

Some existing code may use the old `HashMap` format. Test migration:

#### Old Code (won't compile):
```rust
let mut members = HashMap::new();
members.insert("Speed".to_string(), PlcValue::Dint(1500));
client.write_tag("MotorData", PlcValue::Udt(members)).await?;
```

#### New Code (works):
```rust
// Option 1: Read first to get symbol_id, then modify
let read_value = client.read_tag("MotorData").await?;
let udt_data = if let PlcValue::Udt(data) = read_value {
    data
} else {
    return Err("Not a UDT".into());
};

// Modify data based on UDT definition
let definition = client.get_udt_definition("MotorData").await?;
let mut members = HashMap::new();
members.insert("Speed".to_string(), PlcValue::Dint(1500));
let modified_udt = UdtData::from_hash_map(&members, &definition, udt_data.symbol_id)?;

client.write_tag("MotorData", PlcValue::Udt(modified_udt)).await?;
```

### 5. Error Cases

Test error handling:

```rust
// Test 1: Write UDT without symbol_id when tag doesn't exist
let udt_data = UdtData {
    symbol_id: 0,
    data: vec![0x00],
};
// Should fail with "UDT template instance ID not found"
let result = client.write_tag("NonExistentUDT", PlcValue::Udt(udt_data)).await;
assert!(result.is_err());

// Test 2: Parse UDT with wrong definition
let wrong_definition = UserDefinedType::new("WrongUDT".to_string());
let result = udt_data.parse(&wrong_definition);
// May fail or return incorrect values
```

## Test Checklist

- [ ] Unit tests pass (no PLC required)
- [ ] Integration tests pass (with PLC)
- [ ] UDT read returns `UdtData` with valid `symbol_id`
- [ ] UDT write works with provided `symbol_id`
- [ ] UDT write auto-reads `symbol_id` when 0
- [ ] UDT parsing works with UDT definition
- [ ] UDT serialization works with UDT definition
- [ ] Round-trip conversion (HashMap → UdtData → HashMap) works
- [ ] Generic UDT handling works for any UDT (no hardcoded names)
- [ ] Error cases are handled correctly

## Running All Tests

```bash
# Run all unit tests
cargo test --test udt_data_tests --lib

# Run all integration tests (requires PLC)
cargo test --test udt_data_tests --ignored

# Run with verbose output
cargo test --test udt_data_tests -- --nocapture
```

## Troubleshooting

### Issue: Tests fail with "symbol_id is 0"
**Solution**: Ensure the PLC tag exists and is a valid UDT. The `symbol_id` comes from tag attributes.

### Issue: Parsing fails
**Solution**: Ensure you have the correct UDT definition. Use `get_udt_definition()` to get it from the PLC.

### Issue: Write fails with "template instance ID not found"
**Solution**: The tag might not exist or might not be a UDT. Check tag name and ensure it's a UDT type.

## Migration Guide

For existing code using the old `HashMap` format:

1. **Reading UDTs**: Change from expecting `HashMap` to expecting `UdtData`
2. **Writing UDTs**: 
   - Read the UDT first to get `symbol_id`
   - Use `UdtData::from_hash_map()` to convert HashMap to UdtData
   - Or modify raw bytes directly
3. **Accessing Members**: 
   - Get UDT definition using `get_udt_definition()`
   - Use `UdtData::parse()` to convert to HashMap when needed

See examples in `tests/udt_data_tests.rs` for reference implementations.

