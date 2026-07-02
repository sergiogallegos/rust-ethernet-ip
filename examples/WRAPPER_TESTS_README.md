# Wrapper Comprehensive Tests

This directory contains comprehensive test programs for each wrapper (C#, Go, Python) that test reading and writing all ~392 tags from `PLC_TEST_TAG_DEFINITIONS.md`.

## Test Programs

### 1. C# Wrapper Test
**Location:** `examples/CSharpWrapperTest/`

**Run:**
```bash
cd examples/CSharpWrapperTest
dotnet run
```

**What it tests:**
- Reads all ~392 tags (controller and program-scoped)
- Writes new test values to all tags
- Reads back and verifies writes were successful
- Provides detailed failure summary grouped by error type

### 2. Go Wrapper Test
**Location:** `examples/GoWrapperTest/`

**Run:**
```bash
cd examples/GoWrapperTest
go run main.go
```

**What it tests:**
- Reads all ~392 tags (controller and program-scoped)
- Writes new test values to all tags
- Reads back and verifies writes were successful
- Provides detailed failure summary grouped by error type

### 3. Python Wrapper Test
**Location:** `examples/PythonWrapperTest/`

**Run:**
```bash
cd examples/PythonWrapperTest
python test_all_tags.py
```

**What it tests:**
- Reads all ~392 tags (controller and program-scoped)
- Writes new test values to all tags
- Reads back and verifies writes were successful
- Provides detailed failure summary grouped by error type

## Prerequisites

1. **All tags from `PLC_TEST_TAG_DEFINITIONS.md` must exist in your PLC**
   - Controller-scoped tags: `gTestArray_DINT[0-9]`, `gTestUDT`, `gTestUDT_Array[0-9]`, etc.
   - Program-scoped tags: `Program:TestProgram.gTestArray_DINT[0-9]`, etc.

2. **PLC must be accessible**
   - Default address: `192.168.0.1:44818` (update in each test file if needed)
   - ControlLogix CPU Slot: `0` (update if needed)

3. **Wrapper dependencies:**
   - **C#**: Requires `rust_ethernet_ip.dll` in output directory (copied automatically)
   - **Go**: Requires `rust_ethernet_ip.dll` (Windows) or `rust_ethernet_ip.so` (Linux) in library path
   - **Python**: Install with `pip install -e ../../pywrapper`

## Expected Results

All three tests should produce similar results:

- ✅ **~335 tags** should pass successfully
- ❌ **~57 tags** will fail due to PLC firmware limitations (not library bugs):
  - **55 tags**: UDT array element member writes (Error 0x2107)
    - Example: `gTestUDT_Array[0].Member1_DINT`
  - **2 tags**: STRING member writes in UDTs (Error 0x2107)
    - Example: `gTestUDT.Member5_String`

## Test Structure

Each test follows the same 3-step process:

1. **STEP 1: Reading Initial Values**
   - Reads all tags to establish baseline
   - Tags that fail to read are skipped in subsequent steps

2. **STEP 2: Writing Test Values**
   - Writes new test values to all successfully-read tags
   - For STRING types, immediately reads back to verify write

3. **STEP 3: Reading Back and Verifying Writes**
   - Reads back all tags that were successfully written
   - Compares values to verify writes were successful
   - Provides detailed mismatch reporting for STRING types

## Failure Summary

Each test provides a detailed failure summary that:
- Groups failures by error type
- Identifies known PLC limitations (Error 0x2107)
- Categorizes failures:
  - UDT array element member writes
  - STRING member writes in UDTs
- Shows affected tags with pattern grouping (e.g., `gTestUDT_Array[*].Member1_DINT`)

## Usage

Run these tests **before** testing the example applications (WinForms, WPF, ASP.NET, gonextjs) to verify that each wrapper is working correctly at the library level.

If a wrapper test fails unexpectedly (not due to known PLC limitations), investigate the wrapper implementation before testing the example applications.

## Notes

- These tests use the same tag definitions as `examples/test_plc_test_tag_definitions.rs` (Rust test)
- All tests should produce similar results, confirming wrapper correctness
- Known failures (Error 0x2107) are documented as PLC firmware limitations, not library bugs

