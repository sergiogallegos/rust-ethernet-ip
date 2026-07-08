# C++ Consumer Support

## Summary

`confirmed`: Current mainline has a first-class C/C++ consumption path through
the existing C ABI: checked-in header, export parity gate, and a simulator-backed
C++ smoke example. It is packaging and integration support, not a second native
API surface.

## Current Understanding

- `confirmed`: `include/rust_ethernet_ip.h` declares the exported handle-based
  C ABI and excludes the three CODEX-AS raw-pointer compatibility functions
  that are no longer C ABI symbols.
- `confirmed`: The parity gate has two checks: a C++ link-check executable takes
  addresses of every header declaration, and
  `scripts/check-ffi-header-parity.py` compares exported `eip_*` symbols from
  the dynamic library to header declarations.
- `confirmed`: `examples/cpp` contains a dependency-free RAII wrapper and a
  CTest smoke that starts the simulator, round-trips DINT/REAL/STRING, and
  runs batch read/write calls.
- `confirmed`: Qt guidance is documentation-only: keep blocking FFI calls off
  the GUI thread, with one worker object owning one client handle.

## Evidence

- [../../include/rust_ethernet_ip.h](../../include/rust_ethernet_ip.h)
- [../../examples/cpp/CMakeLists.txt](../../examples/cpp/CMakeLists.txt)
- [../../examples/cpp/eip_client.hpp](../../examples/cpp/eip_client.hpp)
- [../../scripts/check-ffi-header-parity.py](../../scripts/check-ffi-header-parity.py)
- [../../docs/CPP_INTEGRATION.md](../../docs/CPP_INTEGRATION.md)
- [../../docs/agents/tasks/CODEX-AU-cpp-consumer-support.md](../../docs/agents/tasks/CODEX-AU-cpp-consumer-support.md)

## Open Questions

- Whether future C++ demand justifies vcpkg/conan packaging.
- Whether the C++ smoke should later run on macOS CI as well as Ubuntu/Windows.

## Related Pages

- [rust-vs-csharp.md](rust-vs-csharp.md)
- [../protocol/abi-contract.md](../protocol/abi-contract.md)
