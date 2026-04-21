# Version 0.6.0 Changelog

> Historical reference: this changelog captures the `0.6.0` release state only. Do not treat its status language as the current repo or release state.

**Release Date:** January 2026  
**Status:** ✅ Production Ready

## 🎉 Major Changes

### 🔧 Generic UDT Format
- **New `UdtData` struct**: Replaces HashMap-based UDT representation
  - Contains `symbol_id` (template instance ID) and raw `data` bytes
  - Works with any UDT without requiring prior knowledge of member structure
  - Enables parsing UDT members using UDT definitions when needed
  - Supports reading and writing UDTs generically

### ✅ Library Health
- **All 31 unit tests passing**: Core library is production-ready
- **Comprehensive examples updated**: All examples now use new `UdtData` API
- **Integration tests updated**: Tests updated for new UDT format
- **Code quality**: Fixed clippy warnings and improved code consistency

### 🚀 C# Wrapper Enhancements
- **Batch Operations**: `ReadTagsBatch()` and `WriteTagsBatch()` for high-performance multi-tag operations
- **TagGroup**: Periodic polling with event-driven updates (`TagGroup` class with `DataChanged` event)
- **Performance Statistics**: `ClientStatistics` class tracking read/write counts, errors, and average response times
- **Data Quality & Timestamp**: `TagReadResult` with `Quality`, `TimeStamp`, and detailed error information
- **Value Scaling**: `ValueScaling` utility class with `ScaleLinear()` and `ScaleSquareRoot()` methods
- **Enhanced Error Handling**: Detailed error messages with quality indicators and timestamps

### 🔧 Connection & RoutePath Fixes
- **WinForms Application**: Fixed connection to use `ConnectWithRoute()` when RoutePath is enabled
- **WPF Application**: Fixed connection to use `ConnectWithRoute()` when RoutePath is enabled
- **ASP.NET Application**: Updated `PlcService.Connect()` to accept and use RoutePath parameters
- **Connection Verification**: Added automatic connection tests after successful connection
- **Error Handling**: Improved error messages and exception handling across all example applications

### 🐛 Bug Fixes
- **TagReadResult Duplicate**: Renamed internal `TagReadResult` to `TagReadResultBatch` to resolve naming conflicts
- **Nullability Warnings**: Fixed nullable reference type warnings in `PlcValue`, `TagSubscription`, `UdtData`, and `EthernetNetIpClient`
- **DLL Deployment**: Fixed DLL path in `RustEtherNetIp.csproj` to ensure `rust_ethernet_ip.dll` is correctly copied

## 📊 Breaking Changes

### UDT API Changes
- **Before (v0.5.x):**
  ```rust
  if let PlcValue::Udt(members) = value {
      let count = members.len();
      if let Some(val) = members.get("Member1") {
          // ...
      }
  }
  ```

- **After (v0.6.0):**
  ```rust
  if let PlcValue::Udt(udt_data) = value {
      let size = udt_data.data.len();
      let symbol_id = udt_data.symbol_id;
      // To access members, parse using UDT definition:
      let members = udt_data.parse(&udt_definition)?;
  }
  ```

## 🔄 Migration Guide

### For Users of UDT Features

1. **Reading UDTs**: The return type is now `UdtData` instead of `HashMap<String, PlcValue>`
2. **Accessing Members**: Use `udt_data.parse(&udt_definition)` to get parsed members
3. **Writing UDTs**: Read first to get `symbol_id`, then modify and write back

### Example Migration

```rust
// Old way (v0.5.x)
let udt_value = client.read_tag("MyUDT").await?;
if let PlcValue::Udt(members) = udt_value {
    if let Some(val) = members.get("Member1") {
        println!("Member1: {:?}", val);
    }
}

// New way (v0.6.0)
let udt_value = client.read_tag("MyUDT").await?;
if let PlcValue::Udt(udt_data) = udt_value {
    // Get UDT definition
    let udt_def = client.get_udt_definition("MyUDT").await?;
    
    // Parse members
    let members = udt_data.parse(&udt_def)?;
    if let Some(val) = members.get("Member1") {
        println!("Member1: {:?}", val);
    }
}
```

## 📝 Updated Files

### Core Library
- `src/lib.rs`: Updated UDT handling to use `UdtData` struct
- `src/udt.rs`: Enhanced UDT parsing and serialization

### Examples (All Updated)
- `examples/test_udt_structure.rs`
- `examples/rust_examples/test_gtracking_udt.rs`
- `examples/rust_examples/test_part_data_enhanced.rs`
- `examples/rust_examples/enhanced_udt_demo.rs`
- `examples/rust_examples/generic_udt_demo.rs`
- `examples/rust_examples/data_types_showcase.rs`
- `examples/rust_examples/test_udt_multiple_members.rs`
- `examples/rust_examples/test_part_data_chunked.rs`
- `examples/rust_examples/test_part_data_udt.rs`
- `examples/rust_examples/test_real_udt.rs`
- `examples/rust_examples/test_udt_chunked.rs`
- `examples/rust_examples/test_program_tag_out_fuse.rs`

### Tests (All Updated)
- `tests/integration_test.rs`
- `tests/comprehensive_test.rs`
- `tests/udt_enhanced_parsing_tests.rs`
- `tests/udt_enhanced_tests.rs`
- `tests/udt_data_tests.rs`

### Documentation
- `README.md`: Updated with v0.6.0 features and status
- `docs/LIBRARY_CHECK_SUMMARY.md`: New document with library health status

## 🎯 Benefits

1. **Universal UDT Support**: Works with any UDT without hardcoding member names
2. **Better Performance**: Raw bytes format is more efficient for large UDTs
3. **Future-Proof**: Easy to extend for new UDT types
4. **Type Safety**: Clear separation between raw data and parsed members

## ⚠️ Known Limitations

- Some integration tests may need PLC connection to run (marked with `#[ignore]`)
- UDT array element member writes are limited by PLC firmware (Error 0x2107)
- See `docs/UDT_ARRAY_ELEMENT_MEMBER_WRITE_ANALYSIS.md` for details

## 🔗 Related Documentation

- `docs/LIBRARY_CHECK_SUMMARY.md`: Library health and test status
- `docs/UDT_ARRAY_ELEMENT_MEMBER_WRITE_ANALYSIS.md`: UDT array limitations
- `docs/PLC_TEST_TAG_DEFINITIONS.md`: Test tag definitions

---

**Next Steps:**
- Continue improving UDT parsing performance
- Add more comprehensive examples
- Enhance error messages for UDT operations
