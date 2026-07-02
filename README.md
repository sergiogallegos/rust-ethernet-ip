<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="https://raw.githubusercontent.com/sergiogallegos/rust-ethernet-ip/main/images/brand/logo-dark.png">
    <source media="(prefers-color-scheme: light)" srcset="https://raw.githubusercontent.com/sergiogallegos/rust-ethernet-ip/main/images/brand/logo-light.png">
    <img src="https://raw.githubusercontent.com/sergiogallegos/rust-ethernet-ip/main/images/brand/logo-light.png" alt="rust-ethernet-ip logo" width="420" />
  </picture>
</p>

[![Crates.io](https://img.shields.io/crates/v/rust-ethernet-ip.svg)](https://crates.io/crates/rust-ethernet-ip)
[![Rust](https://img.shields.io/badge/rust-1.96+-orange.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Documentation](https://docs.rs/rust-ethernet-ip/badge.svg)](https://docs.rs/rust-ethernet-ip)

Production-focused EtherNet/IP library for **Allen-Bradley CompactLogix and ControlLogix PLCs**.

## Why this project exists

### Why Rust for the core

EtherNet/IP runs on factory floors where a dropped packet or an out-of-bounds parse can stop a production line. Rust was chosen for the core because it provides:

- memory safety with no garbage collector — no GC pauses during high-rate scan loops
- predictable latency and low overhead, important for sub-100 ms tag polling
- a strong type system that pushes wire-protocol mistakes to compile time instead of to runtime in front of a real PLC
- a single statically-linked binary that drops into industrial PCs and edge gateways without a managed runtime

The same library can therefore serve both the embedded edge — where C and C++ have historically dominated — and higher-level integrations, without rewriting the protocol layer for each consumer.

### Why a C# wrapper

The Allen-Bradley world is overwhelmingly a Windows and .NET world: HMIs, MES integrations, SCADA front-ends, OPC servers, and integrator-built operator software are usually written in C#. Most engineers on the plant floor are not going to write Rust, and they should not have to. The NuGet-packaged `RustEtherNetIp` wrapper lets those teams consume the Rust core through a familiar API (`client.ReadDint("Tag")`) while the protocol work still runs in the native layer.

### Why a Python wrapper

Data engineering, analytics, historian ingestion, MES bridges, and machine learning on the plant floor are predominantly Python. A Python wrapper means a data scientist or integration engineer can pull live PLC data into pandas, into a Kafka producer, or into a Docker-deployed collector service, without rewriting the protocol stack or routing through OPC.

### Vision and open source

There is no widely-adopted, modern, open-source EtherNet/IP library for Allen-Bradley PLCs that is production-credible across the Rust, .NET, and Python ecosystems at the same time. Existing options tend to be closed-source vendor SDKs with restrictive licensing, aging C libraries with thin or stale language bindings, or per-team rewrites that never get hardened against real PLC firmware quirks.

This project exists to fill that gap with a single, MIT-licensed protocol implementation the industrial automation community can build on, audit, and extend — and to make the firmware-imposed limitations (STRING writes, UDT array element writes, route-path quirks) explicit and documented rather than rediscovered by every new integrator.

## Version Status

- Current stable release: `1.1.0` (tagged 2026-06-19, crates.io + NuGet + PyPI)
- Previous stable release: `1.0.0`
- Earlier stable release: `0.7.0`
- Real-hardware validation evidence is included for the release

Release snapshot:
- `1.1.0` is a non-breaking correctness + cleanup + feature release: client-side bit access (read-modify-write), C# UDT-write/Dispose/finalizer fixes, `RoutePath::add_port` fix, a Python wheel that bundles the native library, richer errors via `eip_get_last_error` (`CAP_LAST_ERROR`) surfaced through C# `PlcException` and Python, a C# async API, `thiserror` 2.0 + trimmed `tokio` features, and automated crates.io/PyPI/multi-RID-NuGet release packaging. The public Rust API and the C ABI (version 1) are unchanged. See [`CHANGELOG.md`](CHANGELOG.md).
- Rust + C# + Python full-coverage hardware exercisers all pass against CompactLogix 5069-L330ERM fw38 (2026-06-19): byte-identical 2299/2299 reads, 2206/2206 writes, 2206/2206 verify, 60 firmware-blocked-as-expected, 0 unexpected anomalies.
- crates.io ships five workspace artifacts at `1.1.0`: `rust-ethernet-ip-types`, `rust-ethernet-ip-tag-path`, `rust-ethernet-ip-protocol`, `rust-ethernet-ip-udt`, and the top-level `rust-ethernet-ip`. NuGet ships `RustEtherNetIp 1.1.0` and PyPI ships `rust-ethernet-ip 1.1.0` from the GitHub release workflow on tag push.

## Project Focus

- Rust core library
- C# wrapper via NuGet (`RustEtherNetIp`)
- Python wrapper for data collection, analytics, and service integrations
- Industrial PC applications, with current NuGet packaging focused on Windows `win-x64`
- Deterministic behavior and regression safety

## Key Capabilities

- Native support for all 13 common AB data types: `BOOL`, `SINT`, `INT`, `DINT`, `LINT`, `USINT`, `UINT`, `UDINT`, `ULINT`, `REAL`, `LREAL`, `STRING`, `UDT`
- Advanced tag addressing: program-scoped tags, array indexing, bit access, nested UDT paths
- Route path support for backplane/slot routing (ControlLogix)
- Batch operations (`read_tags_batch`, `write_tags_batch`, `execute_batch`)
- Tag-group polling API (`upsert_tag_group`, `read_tag_group_once`, `subscribe_tag_group`)
- UDT discovery and metadata access
- Real-time subscriptions and health-check APIs
- Schema export and diagnostics snapshot surfaces
- C# wrapper for .NET integration
- Python wrapper and service/data-pipeline examples

## Known PLC/Firmware Limitations

Some write behaviors are restricted by PLC firmware (not library protocol implementation):

- Direct writes to standalone `STRING` tags can fail on some controllers
- Direct writes to `STRING` members inside UDTs can fail on some controllers
- Direct writes to UDT array element members (for example `MyUdtArray[0].Member`) can fail

Real-hardware note from the `0.7.0` release validation:
- Validated on `5069-L320ERMS3`, firmware `35`, at `192.168.0.1:44818`
- Validated on `1756-L81ES`, firmware `37`, via `1756-EN3TR` slot `0` at `192.168.0.101:44818`
- On that CompactLogix target, normal reads/writes, route-path access, subscriptions, UDT reads, and batch operations are working
- On that ControlLogix target, the same main read/write, route-path, subscription, UDT-read, and batch paths are working
- On that same target, the remaining observed firmware-imposed limits are:
  - direct `STRING` writes, which can surface as batch-level `0x1E` or extended `0x2107`
  - direct writes to UDT array element members, which surface as `0x2107`

Recommended pattern for restricted cases: **read-modify-write the full UDT/array element**.

Detailed technical background and examples:
- [AB String/UDT write limitations](docs/AB_String_UDT_Write_Limitations.md)
- [CompactLogix real-PLC validation record](docs/validation/2026-04-07_real_plc_5069-L320ERMS3_fw35.md)
- [CompactLogix C# wrapper validation record](docs/validation/2026-04-07_csharp_wrapper_real_plc_5069-L320ERMS3_fw35.md)
- [ControlLogix real-PLC validation record](docs/validation/2026-04-16_real_plc_1756-L81ES_via_1756-EN3TR_slot0.md)
- [ControlLogix C# wrapper validation record](docs/validation/2026-04-16_csharp_wrapper_real_plc_1756-L81ES_via_1756-EN3TR_slot0.md)

## Installation

### Rust

```toml
[dependencies]
rust-ethernet-ip = "1.1.0"
tokio = { version = "1", features = ["full"] }
```

### C#

```xml
<PackageReference Include="RustEtherNetIp" Version="1.1.0" />
```

Or from the CLI:

```bash
dotnet add package RustEtherNetIp --version 1.1.0
```

Current NuGet packaging note:
- `RustEtherNetIp` `1.1.0` is published on NuGet
- the package bundles native runtimes for `win-x64`, `linux-x64`, and `osx-arm64`
- the managed package currently targets `.NET 10`

### Python

```bash
pip install rust-ethernet-ip==1.1.0
```

The wheel bundles the native library, so a plain `pip install` works with no separate build. (The Rust and C# wrappers ship alongside it from the same release.)

See:

- [python/README.md](python/README.md)
- [Integration and deployment guide](docs/INTEGRATION_AND_DEPLOYMENT.md)
- [docs/PYTHON_WRAPPER_STRATEGY.md](docs/PYTHON_WRAPPER_STRATEGY.md)
- [docs/DOCKER_EXAMPLE_STACKS.md](docs/DOCKER_EXAMPLE_STACKS.md)

## Integration and Deployment

If you are evaluating the library for production use, start here:

- [Integration and deployment guide](docs/INTEGRATION_AND_DEPLOYMENT.md)

That guide covers:

- when to use Rust vs C# vs Python
- step-by-step integration into each stack
- native runtime deployment expectations
- routed ControlLogix usage
- troubleshooting and rollout checks

## Quick Start (Rust)

```rust
use rust_ethernet_ip::{EipClient, PlcValue, RoutePath};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Direct connect
    let mut client = EipClient::connect("192.168.1.100:44818").await?;

    // Or routed connect (example: ControlLogix slot 3)
    let route = RoutePath::new().add_slot(3);
    let mut routed = EipClient::with_route_path("192.168.1.100:44818", route).await?;

    let running = client.read_tag("Program:Main.MotorRunning").await?;
    client
        .write_tag("Program:Main.SetPoint", PlcValue::Dint(1500))
        .await?;

    let tags = vec!["Program:Main.Temp", "Program:Main.Pressure"];
    let batch = routed.read_tags_batch(&tags).await?;

    println!("running={running:?}, batch={batch:?}");
    Ok(())
}
```

## Quick Start (C#)

```csharp
using RustEtherNetIp;

using var client = new EtherNetIpClient();
if (client.Connect("192.168.1.100:44818"))
{
    bool running = client.ReadBool("Program:Main.MotorRunning");
    int count = client.ReadDint("Program:Main.ProductionCount");

    client.WriteBool("Program:Main.Start", true);
    client.WriteDint("Program:Main.SetPoint", 1500);

    Console.WriteLine($"running={running}, count={count}");
}
```

## Batch Operations

```rust
use rust_ethernet_ip::{BatchOperation, PlcValue};

// Batch write
let writes = vec![
    ("SetPoint1", PlcValue::Real(72.5)),
    ("SetPoint2", PlcValue::Real(74.0)),
    ("Enable", PlcValue::Bool(true)),
];
let write_results = client.write_tags_batch(&writes).await?;

// Mixed batch
let ops = vec![
    BatchOperation::Read { tag_name: "ActualTemp".into() },
    BatchOperation::Write { tag_name: "SetPoint1".into(), value: PlcValue::Real(73.0) },
];
let mixed_results = client.execute_batch(&ops).await?;
```

Notes:
- `read_tags_batch(...)` and `write_tags_batch(...)` preserve tag association in their return values.
- `execute_batch(...)` may regroup mixed operations for packet optimization, so correlate results by the returned operation metadata rather than assuming strict mixed-input ordering.

## Tag Group Event Handling

### Rust

```rust
use rust_ethernet_ip::{EipClient, TagGroupEventKind};

let mut client = EipClient::connect("192.168.1.100:44818").await?;
client
    .upsert_tag_group(
        "cell_1",
        vec!["Program:Main.Temp".into(), "Program:Main.Pressure".into()],
        250,
    )
    .await?;

let sub = client.subscribe_tag_group("cell_1").await?;
while let Some(event) = sub.wait_for_update().await {
    match event.kind {
        TagGroupEventKind::Data => {
            // All tags read successfully
        }
        TagGroupEventKind::PartialError => {
            // Some tags failed; inspect per-tag `snapshot.values[*].error`
        }
        TagGroupEventKind::ReadFailure => {
            // Full cycle failed; inspect `event.error` and `event.failure`
        }
    }
}
```

### C#

```csharp
client.UpsertTagGroup("cell_1", new[] { "DINT_TAG", "PressureTag" }, updateRateMs: 250);
var group = client.SubscribeToTagGroup("cell_1");

group.PollingEvent += (_, evt) =>
{
    switch (evt.Kind)
    {
        case TagGroupEventKind.Data:
            // All tags good
            break;
        case TagGroupEventKind.PartialError:
            // Mixed quality; inspect evt.Errors per tag
            break;
        case TagGroupEventKind.ReadFailure:
            // Entire cycle failed; inspect evt.ErrorMessage + evt.Failure
            break;
    }
};
```

## Build and Test

```bash
cargo fmt
cargo clippy -p rust-ethernet-ip --lib -- -D warnings
cargo test --workspace --all-targets
dotnet test csharp/RustEtherNetIp.Tests/RustEtherNetIp.Tests.csproj -v minimal
```

## Examples

### .NET

```bash
cd examples/WpfExample && dotnet run
cd examples/WinFormsExample && dotnet run
cd examples/AspNetExample && dotnet run
```

### Rust

```bash
cargo run --example comprehensive_terminal_demo
cargo run --example stream_injection_example
cargo run --example test_discover_and_verify
```

### Python

```bash
PYTHONPATH=python python3 python/examples/read_single_tag.py
PYTHONPATH=python python3 python/examples/collector_service.py --config python/examples/collector_config.example.json --once
docker compose -f docker/python-stack/docker-compose.yml up --build
```

## Documentation

- [API docs (docs.rs)](https://docs.rs/rust-ethernet-ip)
- [Programmer manual (Rust + C#)](docs/programmer_manual.md)
- [Integration and deployment guide](docs/INTEGRATION_AND_DEPLOYMENT.md)
- [Python wrapper guide](python/README.md)
- [Official sources traceability](docs/OFFICIAL_SOURCES.md)
- [PLC/simulator compatibility matrix (0.7.0)](docs/compat/0.7.0_plc_simulator_compatibility_matrix.md)
- [C# wrapper guide](csharp/RustEtherNetIp/README.md)
- [Tag introspection](docs/tag_introspection.md)
- [Troubleshooting](docs/TROUBLESHOOTING.md)
- [Changelog](CHANGELOG.md)

## Community and Support

- [GitHub Issues](https://github.com/sergiogallegos/rust-ethernet-ip/issues)
- [GitHub Discussions](https://github.com/sergiogallegos/rust-ethernet-ip/discussions)
- [Discord](https://discord.gg/uzaM3tua)
- [Sponsor development](https://github.com/sponsors/sergiogallegos)

Project collaboration is open for:

- priority issue handling
- priority feature sponsorship
- integration support for real deployments
- OEM and system-integrator feedback
- companies willing to provide specific hardware access for validation

If your team wants to collaborate on one of those paths, start with a GitHub Discussion or issue and describe:

- controller model and firmware
- direct vs routed topology
- target application type
- required feature set and timeline

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md).

## License

MIT. See [LICENSE](LICENSE).

## Safety Notice

This software is provided "AS IS". Validate thoroughly in your own environment before production deployment, especially for industrial control systems.
