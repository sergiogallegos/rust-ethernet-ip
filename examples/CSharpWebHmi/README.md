# C# Web HMI Demo

This example is a complete first application for `rust-ethernet-ip` 1.2.1. It
combines:

- the Rust EtherNet/IP core and native C ABI;
- the maintained `RustEtherNetIp` C# wrapper;
- an ASP.NET Core `.NET 10` API;
- a React 19 + TypeScript frontend built with Vite;
- the same controller, program-scope, array, BOOL, STRING, and UDT tag shapes
  used by the repository's full-coverage hardware tests.

The interface is a neutral high-performance HMI demonstration. It contains no
company identity and makes no claim that generic test tags represent a real
production process.

## What You Can Do

- Start in simulation mode without a PLC.
- Connect directly to a CompactLogix or through a communication module and
  backplane slot to a ControlLogix.
- Monitor 39 values across controller arrays, a UDT, and program-scoped tags.
- See read quality, acquisition time, route information, ABI version, and the
  native Rust library version.
- Optionally pulse and restore one allowlisted BOOL test tag. Writes are off by
  default.

## Architecture

```text
React + TypeScript dashboard
          │ HTTP / JSON
          ▼
ASP.NET Core minimal API (.NET 10)
          │ typed C# calls
          ▼
RustEtherNetIp C# wrapper 1.2.1
          │ C ABI v3 / P-Invoke
          ▼
rust-ethernet-ip native library 1.2.1
          │ EtherNet/IP + CIP explicit messaging
          ▼
CompactLogix or ControlLogix PLC
```

See [ARCHITECTURE.md](ARCHITECTURE.md) for the request flow and exact source
locations.

## 1. PC Requirements

Install these tools before cloning or building the source example:

| Tool | Minimum for this repository | Check |
|---|---:|---|
| Git | Current supported release | `git --version` |
| Rust | 1.88 or newer | `rustc --version` |
| .NET SDK | 10.0 | `dotnet --version` |
| Node.js | 20.19+, 22.12+, or a newer supported line | `node --version` |
| npm | Included with Node.js | `npm --version` |

Installation sources:

- Git: <https://git-scm.com/downloads>
- Rust: <https://rustup.rs/>
- .NET 10 SDK: <https://dotnet.microsoft.com/download/dotnet/10.0>
- Node.js: <https://nodejs.org/en/download>

The source checkout builds the Rust native library locally. The published
NuGet package also contains native runtimes, but this repository example uses a
`ProjectReference` so contributors exercise the exact checked-out 1.2.1 C# and
Rust sources together.

## 2. Clone and Verify the Tools

```bash
git clone https://github.com/sergiogallegos/rust-ethernet-ip.git
cd rust-ethernet-ip

rustc --version
dotnet --version
node --version
npm --version
```

The expected library version is recorded in the repository `VERSION` file:

```bash
cat VERSION
# 1.2.1
```

## 3. Run Without a PLC First

Simulation is the default and uses the same JSON contract and tag names as live
mode. This is the fastest way to verify the PC toolchain and interface.

### macOS or Linux

```bash
cargo build --release --features ffi --locked
cd examples/CSharpWebHmi/frontend
npm ci
npm run build
cd ..
dotnet run -c Release
```

### Windows PowerShell

```powershell
cargo build --release --features ffi --locked
Set-Location examples/CSharpWebHmi/frontend
npm ci
npm run build
Set-Location ..
dotnet run -c Release
```

Open <http://127.0.0.1:5071>. The header and footer must say `Simulation` or
`Simulated`, and all 39 signals should have good quality.

## 4. Create the PLC Tags

For live mode, create the exact minimal tag set in
[PLC_TAGS.md](PLC_TAGS.md). It explains:

- the controller-scoped arrays and STRING;
- the five-member `TEST_UDT` definition and `gTestUDT` instance;
- the `TestProgram` program and its program-scoped tags;
- suggested initial values;
- the network and controller access checks to perform before starting the app.

The demo needs only a small subset of the full validation layout. A controller
that already follows
[`docs/PLC_TEST_TAG_DEFINITIONS.md`](../../docs/PLC_TEST_TAG_DEFINITIONS.md)
already has the array and UDT shapes; add `gTest_STRING` at controller and
`TestProgram` scope if they are not present.

## 5. Choose Direct or Routed Connection

Use a direct connection for a CompactLogix CPU with its own Ethernet port:

```bash
export HMI_MODE=live
export HMI_PLC_ADDRESS=192.168.1.10:44818
export HMI_USE_ROUTE=false
```

Use a routed connection when the address belongs to a ControlLogix Ethernet
module and the processor is reached through the backplane:

```bash
export HMI_MODE=live
export HMI_PLC_ADDRESS=192.168.1.20:44818
export HMI_USE_ROUTE=true
export HMI_PLC_SLOT=0
```

PowerShell uses `$env:` instead of `export`:

```powershell
$env:HMI_MODE = "live"
$env:HMI_PLC_ADDRESS = "192.168.1.20:44818"
$env:HMI_USE_ROUTE = "true"
$env:HMI_PLC_SLOT = "0"
```

`HMI_PLC_ADDRESS` is the controller or communication-module address, including
port `44818`. `HMI_PLC_SLOT` is the processor slot—not the Ethernet-module
slot.

Optionally provide the identity shown in the dashboard. These labels do not
affect the connection:

```bash
export HMI_CONTROLLER_NAME="ControlLogix 1756-L75/B"
export HMI_CONTROLLER_FIRMWARE="33.011"
```

If omitted, the page uses the neutral labels `Logix controller` and
`User configured` rather than guessing the user's hardware.

## 6. Run Against the PLC

Build once as shown in the simulation section, set the live environment
variables in the same terminal, and run:

```bash
cd examples/CSharpWebHmi
dotnet run -c Release
```

Expected live indicators:

- header state: `Connected`;
- data source: `Live PLC`;
- native core: `v1.2.1` and C ABI `3`;
- signal quality: `39 / 39 good`;
- controller arrays, UDT structure, program scope, and routed target all
  `Healthy`.

If the live connection fails and `HMI_FALLBACK_TO_SIMULATION` is not `false`,
the page intentionally changes to `Fallback` and shows the connection error.
For strict live-only testing:

```bash
export HMI_FALLBACK_TO_SIMULATION=false
```

## 7. Optional Safe Write Demonstration

The backend rejects every write unless this flag is explicitly set:

```bash
export HMI_ALLOW_WRITES=true
```

The `Test Pulse` button is allowlisted to `gTestArray_BOOL[0]`. It:

1. reads the original BOOL value;
2. writes the inverse value;
3. waits 300 ms;
4. writes the original value back.

Do not enable writes against an operational controller unless this tag is a
dedicated test tag and changing it cannot command equipment. The demo does not
expose an arbitrary tag-write endpoint.

## Development Mode

For frontend hot reload, run the backend and Vite in separate terminals.

Terminal 1:

```bash
cd examples/CSharpWebHmi
ASPNETCORE_URLS=http://127.0.0.1:5071 dotnet run
```

Terminal 2:

```bash
cd examples/CSharpWebHmi/frontend
npm run dev
```

Open <http://127.0.0.1:5173>. Vite proxies `/api` requests to the ASP.NET
backend on port `5071`.

## Production-Style Single-Server Build

`npm run build` writes the frontend bundle into `examples/CSharpWebHmi/wwwroot`.
ASP.NET Core then serves both the API and the compiled React application:

```bash
cd examples/CSharpWebHmi/frontend
npm ci
npm run build
cd ..
dotnet publish -c Release -o publish
cd publish
dotnet CSharpWebHmi.dll
```

The generated `wwwroot`, `bin`, `obj`, and `node_modules` directories are
ignored. The npm lockfile is committed for repeatable frontend installation.

## Where the C# Wrapper Does the Work

The backend service in
[`Services/PlcDashboardService.cs`](Services/PlcDashboardService.cs) is the
important integration example:

- `Connect(...)` handles a direct controller endpoint.
- `ConnectWithRoute(...)` plus `RoutePath.AddSlot(...)` handles a routed
  ControlLogix processor.
- `ReadDint`, `ReadReal`, `ReadBool`, `ReadInt`, and `ReadString` read the
  configured atomic, STRING, UDT-member, and program-scoped paths.
- `CheckHealth()` decides whether an existing native session can be reused.
- `NativeRuntime.LibraryVersion` and `NativeRuntime.AbiVersion` expose the
  loaded Rust library contract.
- `WriteBool(...)` is used only by the guarded pulse-and-restore demonstration.
- `Dispose()` unregisters the EtherNet/IP session and releases native state.

The React application never talks to the PLC directly. It receives normalized
JSON from `/api/dashboard`; this keeps browser code isolated from industrial
network credentials, native binaries, and CIP details.

## Troubleshooting

### The page says `Fallback`

Read the warning in `Communication Status`. Verify the address, port, route
choice, processor slot, controller run state, and network path. Set
`HMI_FALLBACK_TO_SIMULATION=false` when diagnosing so a live failure is not
visually replaced by simulation data.

### Native library cannot be loaded

Run this from the repository root before `dotnet run`:

```bash
cargo build --release --features ffi --locked
```

The project file copies the platform library from `target/release` into the
.NET output directory.

### Some signals are bad

Compare the failing tag shown in the backend log with [PLC_TAGS.md](PLC_TAGS.md).
Tag names, capitalization, scope, array dimensions, and Logix data types must
match exactly.

### Browser displays API errors during frontend development

Confirm the backend is running on `http://127.0.0.1:5071`. The proxy target is
defined in `frontend/vite.config.ts`.

### Port 44818 is unreachable

Verify normal IP connectivity and that firewalls/VLAN rules permit TCP 44818
between the PC and PLC. Do not expose a controller directly to the public
internet.

## Safety and Scope

This is an integration and visualization example, not a safety HMI. It must not
be used for safety functions, emergency stops, guarding, or motion protection.
Production systems require authentication, authorization, audit logging,
network segmentation, write interlocks, command confirmation, and an
application-specific hazard review.
