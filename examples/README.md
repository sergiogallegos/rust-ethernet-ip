# Examples Index

This folder contains demo apps and runnable samples for `rust-ethernet-ip`.

Current release state:
- Stable published line: `0.7.0`
- Previous stable line: `0.6.3`

## Directory Map

- `rust_examples/`: Rust demo programs
- `WpfExample/`: WPF desktop UI demo
- `WinFormsExample/`: WinForms desktop UI demo
- `AspNetExample/`: ASP.NET Core API + web dashboard
- `web_app/`: Rust backend + frontend web demo
- `desktop_app/`: Rust native desktop app demo
- `csharp_examples/`: C# focused validation utilities
- `CSharpFFITest/`: low-level C# FFI connectivity check
- `CSharpWrapperTest/`: wrapper behavior/integration checks

## Quick Start

### Rust demos

```bash
cargo run --example comprehensive_terminal_demo
cargo run --example stream_injection_example
cargo run --example test_discover_and_verify
```

### WPF (Windows)

```bash
cd examples/WpfExample
dotnet run
```

### WinForms (Windows)

```bash
cd examples/WinFormsExample
dotnet run
```

### ASP.NET demo

```bash
cd examples/AspNetExample
dotnet run
```

### web_app demo

```bash
cd examples/web_app/backend
cargo run
```

## Notes

- Most demos default to `192.168.0.1:44818` unless configured otherwise.
- Use a simulator or test PLC first before connecting to production controllers.
- For release-line behavior and limitations, always cross-check root docs:
  - `README.md`
  - `CHANGELOG.md`
  - `docs/AB_String_UDT_Write_Limitations.md`
