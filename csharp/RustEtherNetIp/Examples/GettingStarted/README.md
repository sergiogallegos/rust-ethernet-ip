# C# Getting-Started Examples

These are copy-ready `.NET 10` console programs for the published
`RustEtherNetIp` 1.2.0 NuGet package. Work through them in order:

| Example | Learn |
|---|---|
| [`01_ScalarAndString`](01_ScalarAndString/Program.cs) | Connect, dispose, typed scalar I/O, built-in/custom STRING paths, errors |
| [`02_BatchOperations`](02_BatchOperations/Program.cs) | Batch read/write and mixed-operation result handling |
| [`03_DiscoveryAndProgramTags`](03_DiscoveryAndProgramTags/Program.cs) | Controller tag discovery, attributes, known program-scoped paths |
| [`04_ControlLogixRouting`](04_ControlLogixRouting/Program.cs) | Route through an Ethernet module to a ControlLogix CPU slot |
| [`05_Diagnostics`](05_Diagnostics/Program.cs) | Health, operation counts, latency, and last-error metrics |
| [`06_Subscriptions`](06_Subscriptions/Program.cs) | Polling subscription and one-shot tag-group acquisition |

Create a project, add the stable package, and replace its `Program.cs` with one
of the examples:

```bash
dotnet new console --framework net10.0 -n PlcQuickStart
cd PlcQuickStart
dotnet add package RustEtherNetIp --version 1.2.0
```

Set the PLC address instead of editing every program:

```bash
export PLC_ADDRESS=192.168.0.10:44818
dotnet run
```

From a source checkout, each directory is also directly buildable and runnable:

```bash
cargo build --release --features ffi --locked
dotnet run --project \
  csharp/RustEtherNetIp/Examples/GettingStarted/01_ScalarAndString/ScalarAndString.csproj
```

On PowerShell:

```powershell
$env:PLC_ADDRESS = "192.168.0.10:44818"
dotnet run
```

The sample tag names are placeholders. Create matching test tags or replace
them with known tags in your Logix project. Review every write target before
running an example against production equipment; start with a non-production
controller or a dedicated test program.

For source-repository builds, reference the project instead of NuGet and build
the native library first:

```bash
cargo build --release --features ffi --locked
dotnet add reference ../path/to/csharp/RustEtherNetIp/RustEtherNetIp.csproj
```

See the [C# wrapper guide](../../README.md) for data types, 1.2.0 STRING
behavior, limitations, deployment, and real-hardware evidence.
