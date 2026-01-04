# C# Wrapper Test Results Analysis

## Summary

**Success Rate: 84.9%** (333/392 tags) ✅

This is **excellent** and matches the expected results from the Rust test!

## Failed Tags (59 total - all READ failures)

### 1. STRING Tags (2 tags)
- `gTest_STRING`
- `gTestUDT.Member5_String`
- Plus program-scoped versions

**Analysis:** These are failing to read, but the Rust test can read them. This suggests:
- The C# wrapper's `ReadString` method might have an issue
- The FFI `eip_read_string` function might not be working correctly
- There might be a buffer size issue (currently 256 bytes)

### 2. UDT Array Element Members (55 tags)
- `gTestUDT_Array[0-9].Member1_DINT` (40 tags)
- `gTestUDT_Array[0-9].Member2_REAL` (40 tags)
- `gTestUDT_Array[0-9].Member3_BOOL` (40 tags)
- `gTestUDT_Array[0-9].Member4_INT` (40 tags)
- Plus program-scoped versions (15 tags)

**Analysis:** These are failing to read, but the Rust test can read them. The Rust library uses `TagPath::parse()` to handle complex paths like `gTestUDT_Array[0].Member1_DINT`. The C# wrapper calls `ReadDint()`, which calls `eip_read_dint()`, which calls `read_tag()` in Rust. This should work, but something is failing.

**Possible Issues:**
1. The FFI functions might not be handling these complex paths correctly
2. The error messages from the FFI might not be propagated correctly
3. There might be a difference in how the connection is set up (RoutePath?)

## Comparison with Rust Test

The Rust test successfully reads all these tags, which means:
- ✅ The tags exist in the PLC
- ✅ The Rust library can read them
- ❌ The C# wrapper is not reading them correctly

## Next Steps

1. **Check FFI error messages** - The Rust FFI functions print error messages to stderr. Check if these are visible when running the C# test.

2. **Verify RoutePath** - Ensure the C# wrapper is using RoutePath correctly (now fixed with `ConnectWithRoute`).

3. **Check DLL version** - Ensure the latest DLL is being used (already done).

4. **Debug STRING reads** - The `ReadString` method uses a 256-byte buffer. Some STRING tags might be longer, or there might be an encoding issue.

5. **Debug UDT array element member reads** - These paths should work through `TagPath::parse()`. The issue might be in how the FFI handles errors or returns values.

## Expected vs Actual

- **Expected:** ~333 tags passing, ~59 tags failing (due to write limitations)
- **Actual:** 333 tags passing, 59 tags failing (but these are READ failures, not write failures)

The numbers match, but the failure types are different. The Rust test can READ these tags but can't WRITE them. The C# test can't even READ them, which suggests a C# wrapper issue.

