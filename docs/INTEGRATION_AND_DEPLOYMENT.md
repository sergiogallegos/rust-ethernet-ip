# Integration and Deployment Guide

This guide explains how to integrate `rust-ethernet-ip` into Rust, C#, and Python projects and how to ship the required runtime artifacts.

Use this file as the active deployment reference.

For older Windows-only DLL notes, see [DLL_DEPLOYMENT.md](DLL_DEPLOYMENT.md), which is now historical reference material.

## Choose Your Track

- Use the Rust crate if your application is already Rust-native or if you want the lowest-level API surface and full control.
- Use the C# wrapper if you are building a `.NET` HMI, SCADA, dashboard, service, or MES/OEE integration.
- Use the Python wrapper if you need polling, data collection, analytics, API adapters, or quick service integration.

## What Gets Deployed

- Rust: your compiled Rust application only.
- C#: your `.NET` application plus the native `rust_ethernet_ip` library beside the app when the package/runtime does not already provide it for your target.
- Python: your Python code plus the native `rust_ethernet_ip` library in a location the wrapper can load.

Current packaging state:

- Rust crate: `1.2.0` published on crates.io
- C# NuGet package: `RustEtherNetIp 1.2.0` published with native runtime assets
- Python package: `rust-ethernet-ip 1.2.0` published on PyPI with native wheels
- C/C++: stable ABI v2 header and CMake example ship in the source repository

## Rust Integration

### 1. Add the crate

```toml
[dependencies]
rust-ethernet-ip = "1.2.0"
tokio = { version = "1", features = ["full"] }
```

For source builds from `main`, the crate manifests and four sibling crates are aligned at `1.2.0`.

### 2. Connect and read/write

```rust
use rust_ethernet_ip::{EipClient, PlcValue, RoutePath};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let route = RoutePath::new().add_slot(0);
    let mut client = EipClient::with_route_path("192.168.0.101:44818", route).await?;

    let current = client.read_tag("Program:Main.Counter").await?;
    client
        .write_tag("Program:Main.SetPoint", PlcValue::Dint(1500))
        .await?;

    println!("Current value: {current:?}");
    Ok(())
}
```

### 3. Build for deployment

```bash
cargo build --release
```

Ship the compiled executable the same way you would ship any other Rust application for the target OS.

### Good fit

- Rust services
- industrial gateways
- protocol adapters
- performance-sensitive applications

## C# Integration

### 1. Add the NuGet package

```bash
dotnet add package RustEtherNetIp --version 1.2.0
```

Or in the project file:

```xml
<PackageReference Include="RustEtherNetIp" Version="1.2.0" />
```

### 2. Use the wrapper

```csharp
using RustEtherNetIp;

using var client = new EtherNetIpClient();
if (client.Connect("192.168.0.101:44818"))
{
    bool running = client.ReadBool("Program:Main.MotorRunning");
    int count = client.ReadDint("Program:Main.Counter");
    client.WriteDint("Program:Main.SetPoint", 1500);
}
```

For routed ControlLogix access:

```csharp
using RustEtherNetIp;

var route = new RoutePath();
route.AddSlot(0);

using var client = new EtherNetIpClient();
bool ok = client.Connect("192.168.0.101:44818", route);
```

### 3. Build and publish

Typical local build:

```bash
dotnet build
```

Typical deployment build:

```bash
dotnet publish -c Release
```

### 4. Native runtime notes

The wrapper P/Invokes the native library as `rust_ethernet_ip`.

Current practical guidance:

- The published NuGet package is the simplest path for Windows `win-x64` consumers.
- If you are building directly from source, build the Rust native library and make sure it is copied next to your application output:

```bash
cargo build --release
```

Expected native library names:

- Windows: `rust_ethernet_ip.dll`
- macOS: `librust_ethernet_ip.dylib`
- Linux: `librust_ethernet_ip.so`

For source-based builds, keep the native library beside the `.exe` / `.dll` produced by `dotnet build` or `dotnet publish`.

### 5. Shipping checklist

- confirm your target runtime and architecture
- confirm the native library is present in the publish output
- validate one real PLC connection path before shipping
- document any route-path requirements for the deployment site

### Good fit

- WPF or WinForms HMIs
- SCADA front ends
- background Windows services
- ASP.NET or worker-service integrations

## Python Integration

The Python wrapper is currently used directly from this repository.

### 1. Build the native library

```bash
cargo build --release
```

### 2. Run Python from the repo

```bash
PYTHONPATH=python python3
```

### 3. Basic usage

```python
from rust_ethernet_ip import Client, RoutePath

with Client("192.168.0.101:44818", route_path=RoutePath(slots=[0])) as plc:
    value = plc.read_tag("gTestArray_DINT[0]")
    print(value)
```

### 4. If the native library is not in a default path

Set:

```bash
export RUST_ETHERNET_IP_NATIVE_LIB=/absolute/path/to/librust_ethernet_ip.dylib
```

On Windows, point that variable to `rust_ethernet_ip.dll`.
On Linux, point it to `librust_ethernet_ip.so`.

### 5. Example integration paths

- one-shot polling:

```bash
PYTHONPATH=python python3 python/examples/read_single_tag.py
```

- batch polling:

```bash
PYTHONPATH=python python3 python/examples/read_batch_tags.py
```

- collector:

```bash
PYTHONPATH=python python3 python/examples/collector_service.py \
  --config python/examples/collector_config.example.json \
  --once
```

### Good fit

- data collectors
- lightweight APIs
- SQLite / CSV logging
- analytics and pandas workflows
- MQTT publishing

## ControlLogix Routing

For routed ControlLogix access, pass the CPU slot rather than trying to encode the backplane path into the tag name.

Examples:

- Rust: `RoutePath::new().add_slot(0)`
- C#: `new RoutePath().AddSlot(0)`
- Python: `RoutePath(slots=[0])`

Validated routed target examples are recorded in:

- [validation/2026-04-16_real_plc_1756-L81ES_via_1756-EN3TR_slot0.md](validation/2026-04-16_real_plc_1756-L81ES_via_1756-EN3TR_slot0.md)
- [validation/2026-04-16_csharp_wrapper_real_plc_1756-L81ES_via_1756-EN3TR_slot0.md](validation/2026-04-16_csharp_wrapper_real_plc_1756-L81ES_via_1756-EN3TR_slot0.md)
- [validation/2026-04-21_python_wrapper_real_plc_1756-L81ES_via_1756-EN3TR_slot0.md](validation/2026-04-21_python_wrapper_real_plc_1756-L81ES_via_1756-EN3TR_slot0.md)

## Known Live-Limit Cases

These are controller/firmware limitations, not general installation failures:

- standalone standard `STRING` tags require the validated Logix structure encoding
- direct writes to `STRING` members inside UDTs reject with `0x2107` under the current member encoding
- scalar UDT array element members are writeable on 5069-L330ERM fw38 with corrected paths

Recommended workaround for rejected STRING members:

- read the full structure or array element
- modify it in memory
- write the full value back

## Troubleshooting

### C# or Python native library not found

- confirm the native library exists for the target OS
- confirm it is next to the app output or referenced through `RUST_ETHERNET_IP_NATIVE_LIB` where applicable
- confirm architecture matches the process architecture

### Route path problems

- verify the PLC IP is the front-facing Ethernet module or CPU Ethernet port you actually reach from the PC
- verify the CPU slot is correct
- do not encode route-path data into the tag string

### Real PLC acceptance before shipping

Before production rollout, validate at least:

- direct connect or routed connect
- one read
- one write
- one batch read
- one diagnostics or health check path

## Support and Collaboration

Community channels:

- [GitHub Issues](https://github.com/sergiogallegos/rust-ethernet-ip/issues) for bugs and reproducible regressions
- [GitHub Discussions](https://github.com/sergiogallegos/rust-ethernet-ip/discussions) for integration questions and design discussion
- [Discord](https://discord.gg/uzaM3tua) for lightweight community interaction

The project is open to:

- priority issue handling
- priority feature sponsorship
- integration support for real projects
- OEM or system-integrator deployment feedback
- hardware-backed validation partnerships

If your company wants deeper validation coverage, it is also helpful to provide:

- specific PLC hardware access
- firmware/version details
- routing topology details
- test-tag fixtures or representative project requirements

That kind of collaboration is one of the fastest ways to expand the public support matrix and real-hardware evidence.
