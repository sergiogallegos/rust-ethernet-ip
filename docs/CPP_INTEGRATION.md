# C and C++ Integration

The checked-in C ABI is the stable boundary for C and C++ consumers. Build the
native library with:

```sh
cargo build --release --features ffi --locked
```

Then include `include/rust_ethernet_ip.h` and link to the generated dynamic
library:

- Linux: `target/release/librust_ethernet_ip.so`
- macOS: `target/release/librust_ethernet_ip.dylib`
- Windows: `target/release/rust_ethernet_ip.dll` plus
  `target/release/rust_ethernet_ip.dll.lib` for MSVC linking

Ship the dynamic library next to the executable or in a directory covered by
the platform loader path.

## ABI Handshake

Call `eip_abi_version()` at startup and require
`RUST_ETHERNET_IP_ABI_VERSION`. `eip_capabilities()` returns a bitmask of
optional features such as ordered route hops, batch execution, diagnostics JSON,
tag-group subscription support, and `eip_get_last_error`.

Every operation returns `0` on success unless documented otherwise. Connect
functions return a positive client id on success. On failure, call
`eip_get_last_error(client_id, buffer, len)` for the most recent native error
message when a client id is available.

## Threading Model

Every FFI call is blocking. Internally, each call uses `block_on` against the
library's global Tokio runtime. Qt applications must not call the driver from
the GUI thread.

Use one owning worker object per native client handle and run that worker on a
dedicated `QThread`. Publish read results back to the UI with signals, and send
writes to the worker with queued invocations.

```cpp
class PlcWorker : public QObject {
    Q_OBJECT

public slots:
    void connectToPlc(QString address);
    void pollOnce();
    void writeSetpoint(QString tag, int value);

signals:
    void valueChanged(QString tag, QVariant value);
    void fault(QString message);

private:
    std::optional<rust_ethernet_ip::EipClient> client_;
};
```

Treat a client id as single-owner state. The native registry serializes access
to the handle table, but it is not a substitute for an application-level owner
that controls when operations are issued and when disconnect happens.

## Example

`examples/cpp/` contains a dependency-free header-only RAII wrapper and a CMake
smoke demo. After building the native library:

```sh
cmake -S examples/cpp -B target/cpp \
  -DRUST_ETHERNET_IP_NATIVE_LIB="$PWD/target/release/librust_ethernet_ip.so"
cmake --build target/cpp
ctest --test-dir target/cpp --output-on-failure
```

The smoke starts the checked-in simulator, connects through the C ABI,
round-trips DINT, REAL, and STRING tags, then runs batch read/write calls.
