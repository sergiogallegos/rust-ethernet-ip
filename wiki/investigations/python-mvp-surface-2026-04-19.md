# Python MVP Surface

## Summary

The current Python MVP should use a very small native surface and prefer generic JSON-oriented FFI calls instead of building on many typed scalar entrypoints.

## Current Understanding

- `eip_read_tag` should be the primary single-read primitive for Python.
- `eip_write_tags_batch` should be used for both single and multi-write operations in the MVP because there is no generic `eip_write_tag` FFI helper.
- `eip_read_tags_batch` is the correct MVP batch-read primitive.
- `eip_check_health` is usable for a lightweight native-session check, but not as a strong PLC round-trip health signal.
- metadata and discovery should not be in Python MVP because current native stubs are not ready.
- The current `python/` package implements this surface with:
  - `Client.read_tag()`
  - `Client.write_tag()` and `Client.write_tags()`
  - `Client.read_tags()`
  - `Client.check_health()`
  - context-managed connect/disconnect
- The current pure-Python tests run with `python3 -m unittest discover -s python/tests`.
- The integration test skeleton covers simulator-backed connect/read/write/batch/health paths when `SIM_PLC_ADDRESS` is set.
- The Python test suite can now auto-launch the in-repo simulator with `RUST_ETHERNET_IP_START_SIM=1`.
- Current simulator-backed validation now covers mixed `DINT`, `REAL`, `BOOL`, and `STRING` batch flows after:
  - adding `0x00CE` Allen-Bradley STRING support to batch-result parsing
  - changing Python float inference to default to PLC `REAL`

## Evidence

- [src/ffi.rs](../../src/ffi.rs)
- [docs/PYTHON_WRAPPER_STRATEGY.md](../../docs/PYTHON_WRAPPER_STRATEGY.md)
- [docs/PYTHON_MVP_API_AND_FFI_MAPPING.md](../../docs/PYTHON_MVP_API_AND_FFI_MAPPING.md)
- [python/rust_ethernet_ip/client.py](../../python/rust_ethernet_ip/client.py)
- [python/tests/test_client_value_mapping.py](../../python/tests/test_client_value_mapping.py)
- [python/tests/test_import.py](../../python/tests/test_import.py)

## Open Questions

- Whether a generic `eip_write_tag` FFI helper should be added before or after the first Python MVP.
- Whether the Python batch-read API should return partial results by default or raise a batch-level exception carrying per-tag detail.
- Whether health checking should gain a stronger round-trip FFI surface before the Python wrapper presents it as more than a session check.
- Whether the Python wrapper should eventually expose a stronger typed value layer instead of relying on inference plus optional `value_type`.

## Related Pages

- [python-wrapper-strategy-2026-04-19.md](python-wrapper-strategy-2026-04-19.md)
- [ecosystem-platform-patterns-2026-04-19.md](ecosystem-platform-patterns-2026-04-19.md)
