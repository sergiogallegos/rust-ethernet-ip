# Quick Test Guide for UDT Changes

## Quick Start

### 1. Run Unit Tests (No PLC Required)
```bash
cargo test --test udt_data_tests --lib
```

### 2. Run Integration Tests (Requires PLC)
```bash
# Set your PLC address
export PLC_ADDRESS=192.168.1.100:44818

# Run all integration tests
cargo test --test udt_data_tests --ignored
```

### 3. Run Example
```bash
# Set your PLC address and UDT tag name
export PLC_ADDRESS=192.168.1.100:44818
export UDT_TAG_NAME=Part_Data

# Run the example
cargo run --example test_udt_data_format
```

## What Changed?

### Before (Old Format)
```rust
// Required hardcoded member names
let mut members = HashMap::new();
members.insert("Speed".to_string(), PlcValue::Dint(1500));
let value = PlcValue::Udt(members);
```

### After (New Format)
```rust
// Generic - works with any UDT
let value = client.read_tag("MotorData").await?;
if let PlcValue::Udt(udt_data) = value {
    println!("Symbol ID: {}", udt_data.symbol_id);
    println!("Raw data: {:02X?}", udt_data.data);
}
```

## Key Test Points

✅ **Reading UDTs** - Returns `UdtData` with `symbol_id` and raw bytes  
✅ **Writing UDTs** - Uses `symbol_id` (auto-reads if 0)  
✅ **Generic** - Works with any UDT, no hardcoded names  
✅ **Parsing** - Use `UdtData::parse()` with UDT definition when needed  

## Test Checklist

- [ ] Unit tests pass
- [ ] Integration tests pass (with PLC)
- [ ] Example runs successfully
- [ ] UDT read returns valid `symbol_id` (> 0)
- [ ] UDT write works
- [ ] No hardcoded member names in code

## Troubleshooting

**Issue**: `symbol_id` is 0  
**Fix**: Ensure tag exists and is a valid UDT. Read tag attributes first.

**Issue**: Tests fail to compile  
**Fix**: Make sure you're using the latest code with `UdtData` struct.

**Issue**: Write fails  
**Fix**: Check if tag is read-only. Ensure `symbol_id` is valid.

## More Information

See `TESTING_UDT_CHANGES.md` for detailed testing guide and migration instructions.

