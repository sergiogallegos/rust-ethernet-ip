# Go Wrapper Comprehensive Test

This test verifies that the Go wrapper can correctly read and write all tags from `PLC_TEST_TAG_DEFINITIONS.md`.

## Prerequisites

1. Ensure the `rust_ethernet_ip.dll` (Windows) or `rust_ethernet_ip.so` (Linux) is available in the library path

2. Ensure all tags from `PLC_TEST_TAG_DEFINITIONS.md` exist in your PLC

3. Update `PLC_ADDRESS` and `CPU_SLOT` constants in `main.go` if needed

## Running the Test

```bash
cd examples/GoWrapperTest
go run main.go
```

## What It Tests

1. **Step 1**: Reads initial values of all ~392 tags
2. **Step 2**: Writes new test values to all tags
3. **Step 3**: Reads back and verifies the writes were successful

## Expected Results

- ✅ **~333 tags** should pass (84.9% success rate)
- ❌ **~59 tags** will fail due to PLC firmware limitations:
  - 55 tags: UDT array element member writes (Error 0x2107)
  - 2 tags: Simple STRING tag writes (Error 0x2107)
  - 2 tags: STRING member writes in UDTs (Error 0x2107)

These failures are expected and documented as PLC firmware limitations, not library bugs.

