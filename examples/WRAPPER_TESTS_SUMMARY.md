# Wrapper Comprehensive Tests - Summary

## Overview

Three comprehensive test programs have been created to verify each wrapper (C#, Go, Python) can correctly read and write all ~392 tags from `PLC_TEST_TAG_DEFINITIONS.md`. These tests should be run **before** testing the example applications to ensure each wrapper is working correctly at the library level.

## Test Files Created

### 1. C# Wrapper Test
- **File:** `examples/CSharpWrapperTest/Program.cs`
- **Project:** `examples/CSharpWrapperTest/CSharpWrapperTest.csproj`
- **Status:** ✅ Compiles successfully
- **RoutePath:** Note added (ConnectWithRoute may be needed for full RoutePath support)

### 2. Go Wrapper Test
- **File:** `examples/GoWrapperTest/main.go`
- **Module:** `examples/GoWrapperTest/go.mod`
- **Status:** ✅ Ready to run
- **RoutePath:** Handles temporarily disabled RoutePath gracefully

### 3. Python Wrapper Test
- **File:** `examples/PythonWrapperTest/test_all_tags.py`
- **Status:** ✅ Ready to run
- **RoutePath:** Uses `connect_with_route` method correctly

## Test Structure

All three tests follow the same structure as `examples/test_plc_test_tag_definitions.rs`:

1. **STEP 1: Reading Initial Values**
   - Reads all ~392 tags
   - Tracks read failures
   - Tags that fail to read are skipped in subsequent steps

2. **STEP 2: Writing Test Values**
   - Writes new test values to all successfully-read tags
   - For STRING types, immediately reads back to verify write
   - Tracks write failures with detailed error messages

3. **STEP 3: Reading Back and Verifying Writes**
   - Reads back all tags that were successfully written
   - Compares values to verify writes were successful
   - Provides detailed mismatch reporting for STRING types

## Tag Coverage

All tests include the same ~392 tags:

- **Controller-Scoped Arrays:**
  - `gTestArray_DINT[0-9]` (10 tags)
  - `gTestArray_REAL[0-9]` (10 tags)
  - `gTestArray_BOOL[0-9]` (10 tags)
  - `gTestArray_INT[0-9]` (10 tags)
  - `gTestArray_Large[100,200,300,500,999]` (5 tags)

- **Controller-Scoped Simple Tags:**
  - `gTest_STRING` (1 tag)

- **Controller-Scoped UDT Members:**
  - `gTestUDT.Member1_DINT` through `Member5_String` (5 tags)
  - `gTestUDT.Array_DINT[0-9]` (10 tags)
  - `gTestUDT_Array[0-9].Member1_DINT` through `Member4_INT` (40 tags)
  - `gTestUDT_Array[0-9].Array_REAL[0-4]` (50 tags)

- **Program-Scoped Arrays:**
  - `Program:TestProgram.gTestArray_DINT[0-9]` (10 tags)
  - `Program:TestProgram.gTestArray_REAL[0-9]` (10 tags)
  - `Program:TestProgram.gTestArray_BOOL[0-9]` (10 tags)

- **Program-Scoped Simple Tags:**
  - `Program:TestProgram.gTest_STRING` (1 tag)

- **Program-Scoped UDT Members:**
  - `Program:TestProgram.gTestUDT.Member1_DINT` through `Member5_String` (5 tags)
  - `Program:TestProgram.gTestUDT.Array_DINT[0-9]` (10 tags)
  - `Program:TestProgram.gTestUDT_Array[0-4].Member1_DINT` through `Member3_BOOL` (15 tags)

**Total: ~392 tags**

## Expected Results

All three tests should produce similar results:

- ✅ **~333 tags** (84.9%) should pass successfully
- ❌ **~59 tags** will fail due to PLC firmware limitations:
  - **55 tags**: UDT array element member writes (Error 0x2107)
    - Pattern: `gTestUDT_Array[*].Member*` and `Program:TestProgram.gTestUDT_Array[*].Member*`
  - **2 tags**: Simple STRING tag writes (Error 0x2107)
    - `gTest_STRING` and `Program:TestProgram.gTest_STRING`
  - **2 tags**: STRING member writes in UDTs (Error 0x2107)
    - `gTestUDT.Member5_String` and `Program:TestProgram.gTestUDT.Member5_String`

## Failure Summary Features

Each test provides a detailed failure summary that:
- Groups failures by error type
- Identifies known PLC limitations (Error 0x2107)
- Categorizes failures into:
  - UDT array element member writes
  - STRING tag writes
  - STRING member writes in UDTs
- Shows affected tags with pattern grouping (e.g., `gTestUDT_Array[*].Member1_DINT`)

## Running the Tests

### C# Test
```bash
cd examples/CSharpWrapperTest
dotnet run
```

### Go Test
```bash
cd examples/GoWrapperTest
go run main.go
```

### Python Test
```bash
cd examples/PythonWrapperTest
python test_all_tags.py
```

## Next Steps

1. **Run Rust test first:** `cargo run --example test_plc_test_tag_definitions`
   - This verifies the core Rust library works correctly

2. **Run wrapper tests:**
   - C# wrapper test
   - Go wrapper test
   - Python wrapper test

3. **Compare results:**
   - All tests should produce similar results
   - If a wrapper test shows different failures than the Rust test, investigate the wrapper implementation

4. **Test example applications:**
   - Once wrapper tests pass, test WinForms, WPF, ASP.NET, and gonextjs examples

## Notes

- All tests use the same tag definitions for consistency
- RoutePath support may vary by wrapper (C# may need ConnectWithRoute, Go is temporarily disabled)
- Known failures are documented as PLC firmware limitations, not library bugs
- Tests provide detailed error messages to help identify wrapper-specific issues

