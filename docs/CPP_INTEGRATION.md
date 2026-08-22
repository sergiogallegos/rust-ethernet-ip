# C and C++ Integration Guide

The checked-in C ABI is the stable native boundary for C, C++, robotics,
computer-vision, edge-compute, and Qt applications. It provides direct
EtherNet/IP access to CompactLogix and ControlLogix tags without requiring an
OPC client.

## Release Status

- Latest published library line: `1.2.0`
- Repository development line: `1.2.1` (not released yet)
- C ABI version: `2`
- Language level used by the examples: C++17

The 1.2.0 real-hardware gate exercised the C ABI alongside Rust, C#, and Python
on CompactLogix 5069-L330ERM firmware 38. C/C++ completed 2,338 reads and 2,319
writes plus read-back verification with zero unexpected anomalies.

## Build and Link

Build the dynamic library:

```bash
cargo build --release --features ffi --locked
```

Include [`include/rust_ethernet_ip.h`](../include/rust_ethernet_ip.h) and link
the platform artifact:

- Linux: `target/release/librust_ethernet_ip.so`
- macOS: `target/release/librust_ethernet_ip.dylib`
- Windows: `target/release/rust_ethernet_ip.dll` and the MSVC import library

Build every checked-in example:

```bash
cmake -S examples/cpp -B target/cpp \
  -DRUST_ETHERNET_IP_NATIVE_LIB="$PWD/target/release/librust_ethernet_ip.so"
cmake --build target/cpp
ctest --test-dir target/cpp --output-on-failure
```

The CMake project copies the native library beside each executable. For your
application, ship it beside the executable or configure the platform loader
path explicitly.

## ABI Handshake

Fail early when the header and native library do not match:

```cpp
#include "rust_ethernet_ip.h"

#include <cstdint>
#include <stdexcept>

void verify_native_runtime()
{
    if (eip_abi_version() != RUST_ETHERNET_IP_ABI_VERSION)
        throw std::runtime_error("rust-ethernet-ip C ABI mismatch");

    const std::uint64_t capabilities = eip_capabilities();
    if ((capabilities & RUST_ETHERNET_IP_CAP_LAST_ERROR) == 0)
        throw std::runtime_error("native library lacks last-error support");
    if ((capabilities & RUST_ETHERNET_IP_CAP_SCHEMA_REFRESH) == 0)
        throw std::runtime_error("native library lacks schema-refresh support");
}
```

`eip_library_version()` reports the library version. The returned metadata
strings have process lifetime and must not be freed.

## Error Pattern

Connect functions return a positive client ID. Other operations return `0` on
success. Retrieve native detail immediately after a failure:

```cpp
#include <array>
#include <string>

std::string last_error(int client_id)
{
    std::array<char, 1024> buffer {};
    int written = eip_get_last_error(
        client_id, buffer.data(), static_cast<int>(buffer.size()));
    return written > 0
        ? std::string(buffer.data(), static_cast<std::size_t>(written))
        : std::string {};
}
```

Use RAII or one cleanup path so every successful client ID reaches
`eip_disconnect`.

## C++ RAII Quick Start

[`examples/cpp/eip_client.hpp`](../examples/cpp/eip_client.hpp) provides a
small move-only owner for direct connections, DINT/REAL/STRING access, and
batches:

```cpp
#include "eip_client.hpp"

#include <iostream>

using rust_ethernet_ip::EipClient;
using rust_ethernet_ip::require_ok;
using rust_ethernet_ip::require_value;

int main()
{
    auto connected = EipClient::connect("192.168.0.10:44818");
    if (!connected) {
        std::cerr << connected.error.message << "\n";
        return 1;
    }

    EipClient plc = std::move(connected.value);
    std::cout << require_value(plc.read_dint("ProductionCount"), "read") << "\n";
    require_ok(plc.write_real("TemperatureSetpoint", 72.5), "write");
    require_ok(plc.write_string("RecipeName", "PRODUCT_A"), "write STRING");
}
```

The RAII class disconnects in its destructor and cannot be copied.

## Choose Single, Batch, or Structure Access

| Need | Best starting API | Why |
|---|---|---|
| One value, command, or occasional setpoint | Typed `eip_read_*` / `eip_write_*` call | Direct return code and no JSON parsing |
| Several independent tags in one scan | `eip_read_tags_batch` / `eip_write_tags_batch` | Packet-size-aware grouping and per-tag JSON results |
| One known UDT member | Typed call using its complete symbolic path | Avoids transferring or rebuilding the whole structure |
| Inspect a whole UDT | `eip_read_udt_chunked` | Caller-owned JSON buffer; supports fragmented large replies |
| Change an entire UDT | Usually do not; write members individually | `eip_write_udt` requires the exact template-compatible symbol ID and raw bytes |

A batch reduces protocol round trips but is not an atomic controller
transaction; parse every per-tag result. Use a typed single call for one tag.
Use whole-UDT reads for snapshots or inspection and member paths for ordinary
control, configuration, and recipe changes.

## STRINGs and UDT Member Paths in 1.2.0

`eip_write_string` is handle-aware. Supply the complete symbolic path for a
top-level built-in STRING, a custom STRING member, or a STRING member inside a
UDT array element:

```cpp
eip_write_string(client_id, "RecipeName", "PRODUCT_A");
eip_write_string(client_id, "Mixer.Description", "Primary mixer");
eip_write_string(client_id, "Motors[0].Description", "Infeed conveyor");

char value[512] {};
if (eip_read_string(client_id, "Motors[0].Description", value, sizeof(value)) != 0)
    throw std::runtime_error(last_error(client_id));
```

The built-in Studio 5000 `STRING` contains a 4-byte `LEN` and
`SINT DATA[82]`, plus alignment, so its text capacity is **82 bytes**. The ABI
accepts UTF-8, where a non-ASCII character may occupy multiple bytes. A custom
string type uses its declared `DATA[N]` capacity and a different structure
handle; the library discovers that handle.

This supersedes older notes that treated all direct UDT STRING-member writes
as firmware-blocked. Real hardware confirms built-in STRING and custom
`Str82`/`Str400` members on 5069-L330ERM firmware 38. A measured unconnected
CIP write on that target fits about 494 bytes total, including service and tag
path overhead, so this is not a 494-character limit. Version 1.2.0 uses CIP
fragmented services when the value will not fit one packet. A 600-byte custom
string is simulator-confirmed; qualify very large types on the intended target.
Size caller-owned read buffers for the largest expected text plus a null byte.

## Controller and Program Tag Paths

Controller tags use their normal names. Known program tags include the
`Program:` prefix:

```cpp
int controller_count = 0;
int program_count = 0;

eip_read_dint(client_id, "ProductionCount", &controller_count);
eip_read_dint(
    client_id,
    "Program:MainProgram.ProductionCount",
    &program_count);
```

The 1.2.0 C ABI exposes controller-scoped discovery but not program-scoped
enumeration. Known program paths are fully usable for reads, writes, and
batches.

Controller-scoped paths are simply `TagName`. A program-scoped tag belongs to
one Logix program and uses `Program:<program-name>.TagName`; the same typed C
function is used for either scope.

## Whole UDT Reads and Member Writes

```cpp
std::array<char, 8192> udt_json {};
if (eip_read_udt_chunked(client_id, "Mixer", udt_json.data(), udt_json.size()) != 0)
    throw std::runtime_error(last_error(client_id));

double speed = 0.0;
eip_read_real(client_id, "Mixer.SpeedFeedback", &speed);
eip_write_real(client_id, "Mixer.SpeedSetpoint", 60.0);
eip_write_bool(client_id, "Mixer.Enabled", 1);
eip_write_string(client_id, "Mixer.Description", "Primary mixer");

// Reading a whole array element works; write its members individually.
eip_read_udt_chunked(client_id, "Motors[0]", udt_json.data(), udt_json.size());
eip_write_dint(client_id, "Motors[0].CommandSpeed", 1250);
```

The returned whole-UDT JSON is either decoded member data or the raw structure
representation needed by the ABI. Do not invent raw bytes or a member map for
a whole write. Whole UDT-array-element writes are not supported in 1.2.0.

## Batch Read and Write

The RAII example returns the native JSON result so the application can parse it
with its preferred JSON library:

```cpp
auto reads = plc.read_tags_batch({
    "ProductionCount",
    "TankTemperature",
    "Program:MainProgram.MachineRunning",
});

if (!reads)
    throw std::runtime_error(reads.error.message);

const std::string writes = R"([
  {"tag_name":"ProductionSetpoint","value_type":"DINT","value":1250},
  {"tag_name":"TemperatureSetpoint","value_type":"REAL","value":72.5},
  {"tag_name":"EnableCommand","value_type":"BOOL","value":true},
  {"tag_name":"RecipeName","value_type":"STRING","value":"PRODUCT_A"}
])";

auto result = plc.write_tags_batch(writes, 4);
if (!result)
    throw std::runtime_error(result.error.message);
```

Inspect every per-tag result. Packet-size-aware batching in 1.2.0 splits large
requests instead of relying only on operation count.

## ControlLogix Backplane Route

Connect to the Ethernet module and supply an ordered backplane hop to the CPU:

```cpp
const std::uint8_t hop_types[] = { 1 }; // 1 = backplane
const std::uint8_t ports[] = { 1 };
const std::uint8_t slots[] = { 0 };
const char *addresses[] = { nullptr };

int client_id = eip_connect_with_route_hops(
    "192.168.0.20:44818",
    hop_types,
    ports,
    slots,
    addresses,
    1);
```

For multi-hop networks, append Ethernet hops (`hop_type = 2`, usually port 2,
with an address) and further backplane hops in traversal order. See the
build-checked [`route_and_diagnostics.cpp`](../examples/cpp/route_and_diagnostics.cpp).

CompactLogix controllers with built-in Ethernet normally use `eip_connect`
without a route.

## Controller Tag Discovery

Owned discovery results must be released with the matching free function:

```cpp
EipTagDiscoveryResult result {};
if (eip_discover_tags_detailed_by_id(client_id, &result) != 0 || !result.success)
    throw std::runtime_error(result.error_message != nullptr
        ? result.error_message
        : last_error(client_id));

for (int i = 0; i < result.tag_count; ++i) {
    const EipTagAttributes &tag = result.tags[i];
    std::cout << tag.name << " " << tag.data_type_name << " " << tag.size << "\n";
}

eip_free_tag_discovery_result(&result);
```

The complete program is
[`examples/cpp/discovery.cpp`](../examples/cpp/discovery.cpp).

## Health and Diagnostics

```cpp
int healthy = 0;
if (eip_check_health_detailed(client_id, &healthy) != 0)
    throw std::runtime_error(last_error(client_id));

char *json = nullptr;
if (eip_get_diagnostics_json(client_id, 1, &json) != 0 || json == nullptr)
    throw std::runtime_error(last_error(client_id));

std::cout << json << "\n";
eip_free_string(json);
```

Diagnostics JSON contains connection, operation, latency, error-category,
verified-health, schema-generation, cache-hit/miss/eviction, contradiction,
and bounded-recovery metrics. CPU and memory values are placeholders.

For an online tag replacement or controller download, pause application
writes, complete the controller change, call `eip_refresh_schema(client_id)`
(or `client.refreshSchema()` through the C++ convenience layer), optionally
rediscover and verify critical reads, and only then resume writes. The call
invalidates schema-derived caches without reconnecting.

## Threading and Qt

Every C ABI call is blocking. Do not poll the PLC on a Qt GUI thread or a
computer-vision capture thread.

Use one owning worker per client ID on a dedicated `QThread`, and communicate
with the UI through queued signals/slots:

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

Treat the client ID as single-owner state. The native registry protects its
handle table, but application ownership must prevent operations racing with
disconnect.

## Full C ABI Versus the Convenience Class

[`include/rust_ethernet_ip.h`](../include/rust_ethernet_ip.h) is the complete
native contract. It covers every scalar type, generic JSON reads, arrays, UDTs,
controller discovery, metadata, ordered routes, diagnostics, health, and
batches.

`eip_client.hpp` is intentionally smaller. Missing methods in that sample class
do not mean the C ABI lacks the feature. Applications may extend the RAII class
or call the C functions directly.

## Checked-In Examples

- [`demo.cpp`](../examples/cpp/demo.cpp): RAII scalar/STRING/batch smoke
- [`route_and_diagnostics.cpp`](../examples/cpp/route_and_diagnostics.cpp): routed ControlLogix and diagnostics
- [`discovery.cpp`](../examples/cpp/discovery.cpp): controller discovery and cleanup
- [`udt_and_scope.cpp`](../examples/cpp/udt_and_scope.cpp): controller/program paths, whole-UDT reads, member writes, and STRINGs
- [`full_coverage.cpp`](../examples/cpp/full_coverage.cpp): maintainer hardware matrix runner
- [`examples/cpp/README.md`](../examples/cpp/README.md): build and run index

The header/link check and all example targets compile on Ubuntu, Windows, and
macOS in the 1.2.1 preparation line. The simulator smoke runs in CI; route and
discovery examples require real hardware and are compile/link checked only.

## Current Boundaries

- Program-scoped enumeration is not exposed in the C ABI; use known full paths.
- Whole UDT-array-element writes are not supported; write members individually.
- Offset-based UDT member calls remain compatibility exports, but maintained
  high-level APIs and examples use symbolic member paths instead.
- Installable CMake package metadata, `pkg-config` files, standalone native SDK
  archives, and generated C reference pages remain future packaging work.
- The protocol scope is EtherNet/IP Logix tag access for CompactLogix and
  ControlLogix, not Modbus TCP.

See the [hardware compatibility matrix](HARDWARE_COMPATIBILITY.md),
[integration/deployment guide](INTEGRATION_AND_DEPLOYMENT.md), and
[1.2.0 hardware gate](validation/2026-07-08_release-1.2.0-gate_cross-binding_5069-L330ERM_fw38.md).

The project is MIT licensed. Contributions that add controller/firmware
evidence or extend the native ergonomics are welcome.
