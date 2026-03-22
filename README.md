<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="images/brand/logo-dark.png">
    <source media="(prefers-color-scheme: light)" srcset="images/brand/logo-light.png">
    <img src="images/brand/logo-light.png" alt="rust-ethernet-ip logo" width="420" />
  </picture>
</p>

[![Crates.io](https://img.shields.io/crates/v/rust-ethernet-ip.svg)](https://crates.io/crates/rust-ethernet-ip)
[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Documentation](https://docs.rs/rust-ethernet-ip/badge.svg)](https://docs.rs/rust-ethernet-ip)

Production-focused EtherNet/IP library for **Allen-Bradley CompactLogix and ControlLogix PLCs**.

## Version Status

- Latest published stable: `0.6.3`
- Current development on `main`: `0.7.0` (unreleased)
- We are hardening reliability and tests before publishing the next crate release

## Project Focus

- Rust core library
- C# wrapper (`RustEtherNetIp.dll`)
- Industrial PC applications (Windows/Linux/macOS)
- Deterministic behavior and regression safety

## Key Capabilities

- Native support for all 13 common AB data types: `BOOL`, `SINT`, `INT`, `DINT`, `LINT`, `USINT`, `UINT`, `UDINT`, `ULINT`, `REAL`, `LREAL`, `STRING`, `UDT`
- Advanced tag addressing: program-scoped tags, array indexing, bit access, nested UDT paths
- Route path support for backplane/slot routing (ControlLogix)
- Batch operations (`read_tags_batch`, `write_tags_batch`, `execute_batch`)
- Tag-group polling API (`upsert_tag_group`, `read_tag_group_once`, `subscribe_tag_group`)
- UDT discovery and metadata access
- Real-time subscriptions and health-check APIs
- C# wrapper for .NET integration

## Known PLC/Firmware Limitations

Some write behaviors are restricted by PLC firmware (not library protocol implementation):

- Direct writes to standalone `STRING` tags can fail on some controllers
- Direct writes to UDT array element members (for example `MyUdtArray[0].Member`) can fail

Recommended pattern for restricted cases: **read-modify-write the full UDT/array element**.

Detailed technical background and examples:
- [AB String/UDT write limitations](docs/AB_String_UDT_Write_Limitations.md)

## Installation

### Rust

```toml
[dependencies]
rust-ethernet-ip = "0.6.3"
tokio = { version = "1", features = ["full"] }
```

### C#

```xml
<PackageReference Include="RustEtherNetIp" Version="0.6.3" />
```

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

## Documentation

- [API docs (docs.rs)](https://docs.rs/rust-ethernet-ip)
- [Programmer manual (Rust + C#)](docs/programmer_manual.md)
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

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md).

## License

MIT. See [LICENSE](LICENSE).

## Safety Notice

This software is provided "AS IS". Validate thoroughly in your own environment before production deployment, especially for industrial control systems.
