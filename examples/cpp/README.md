# C++ Examples

These examples use the stable C ABI in `include/rust_ethernet_ip.h`. The
header-only `eip_client.hpp` is a small optional RAII layer.

| Target | Source | Purpose |
|---|---|---|
| `cpp_smoke_demo` | [`demo.cpp`](demo.cpp) | RAII connection, DINT, REAL, STRING, batch read/write |
| `cpp_route_and_diagnostics` | [`route_and_diagnostics.cpp`](route_and_diagnostics.cpp) | ControlLogix backplane route, typed read, diagnostics JSON |
| `cpp_discovery` | [`discovery.cpp`](discovery.cpp) | Controller tag discovery and correct result cleanup |
| `cpp_udt_and_scope` | [`udt_and_scope.cpp`](udt_and_scope.cpp) | Controller/program paths, whole-UDT reads, member writes, STRINGs |
| `cpp_full_coverage` | [`full_coverage.cpp`](full_coverage.cpp) | Maintainer real-hardware parity runner |
| `ffi_header_link_check` | [`parity_link_check.cpp`](parity_link_check.cpp) | Compile/link check for every declared C export |

Build from the repository root:

```bash
cargo build --release --features ffi --locked
cmake -S examples/cpp -B target/cpp \
  -DRUST_ETHERNET_IP_NATIVE_LIB="$PWD/target/release/librust_ethernet_ip.so"
cmake --build target/cpp
```

On macOS use the `.dylib`; on Windows use `rust_ethernet_ip.dll`. The CMake
project resolves the platform default automatically when the explicit option
is omitted.

Run against dedicated test tags, not uncontrolled production outputs:

```bash
target/cpp/cpp_discovery 192.168.0.10:44818
target/cpp/cpp_route_and_diagnostics 192.168.0.20:44818 0
target/cpp/cpp_udt_and_scope 192.168.0.10:44818 MainProgram
```

The smoke demo is run automatically against the checked-in simulator by
`ctest`. The route and discovery programs require a real controller and are
compile/link checked in CI without being executed there.

See the full [C/C++ integration guide](../../docs/CPP_INTEGRATION.md) for ABI
handshaking, loader setup, errors, direct program tag paths, threading, and the
boundary between the full C ABI and the smaller RAII example.
