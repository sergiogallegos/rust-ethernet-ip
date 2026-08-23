# Web HMI Architecture

## Component Boundaries

| Layer | Location | Responsibility |
|---|---|---|
| React UI | `frontend/src/App.tsx` | Renders the HMI, polls normalized snapshots, and sends the allowlisted pulse command |
| HMI styling | `frontend/src/styles.css` | Neutral high-performance HMI palette, layout, status semantics, typography, and touch sizing |
| ASP.NET endpoints | `Program.cs` | Hosts static files and exposes `/api/dashboard`, `/api/dashboard/pulse`, and `/api/health` |
| PLC integration | `Services/PlcDashboardService.cs` | Owns one client, serializes access, connects, reads typed tags, maps quality, and guards writes |
| C# wrapper | `../../csharp/RustEtherNetIp/` | Provides managed typed methods and the ABI/version contract |
| Rust core | repository root + `src/` | Implements EtherNet/IP sessions, CIP services, routing, codecs, caching, and diagnostics |

## Snapshot Request Flow

1. React requests `GET /api/dashboard` every 1.8 seconds.
2. ASP.NET resolves the singleton `PlcDashboardService`.
3. A semaphore ensures that one native client operation sequence runs at a
   time.
4. In live mode, `CheckHealth()` reuses a healthy session or reconnects.
5. Routed mode creates `RoutePath().AddSlot(slot)` and calls
   `ConnectWithRoute`; direct mode calls `Connect`.
6. Typed wrapper methods read the known Logix types. Each signal is converted
   to a JSON-safe value plus `Good` or `Bad` quality.
7. ASP.NET serializes the complete snapshot using camel-case JSON.
8. React updates metrics, the REAL-array profile, BOOL lamps, UDT rows,
   program-scope readouts, and communication notices.

One failed tag does not discard the other values. Its signal becomes `Bad`,
the aggregate connection state becomes `Degraded`, and the dashboard reports
the number of failures.

## Native Runtime Boundary

`RustEtherNetIp.csproj` loads the platform library and calls its stable C ABI
through P/Invoke. The dashboard proves which binary was loaded by displaying:

- `NativeRuntime.LibraryVersion` — expected `1.2.1`;
- `NativeRuntime.AbiVersion` — expected `3`.

The browser never loads the native library and never opens an EtherNet/IP
socket. All PLC operations remain on the server.

## Connection Modes

### Simulation

`HMI_MODE=simulation` avoids native calls and generates deterministic values
with the live schema. It is suitable for UI evaluation, training, automated
browser smoke tests, and initial setup.

### Direct PLC

`HMI_MODE=live` and `HMI_USE_ROUTE=false` call `EtherNetIpClient.Connect`.
This is normally appropriate when the configured IP belongs directly to the
processor.

### Routed ControlLogix

`HMI_MODE=live`, `HMI_USE_ROUTE=true`, and `HMI_PLC_SLOT=N` call
`ConnectWithRoute` using a backplane slot segment. This is appropriate when
the configured IP belongs to a chassis communication module.

## Write Boundary

The example intentionally has no arbitrary write API. `/api/dashboard/pulse`
has three server-side guards:

1. `HMI_ALLOW_WRITES` must be `true`;
2. `HMI_MODE` must be `live`;
3. the tag is compiled as `gTestArray_BOOL[0]` and cannot be supplied by the
   browser.

The operation reads the original value, writes its inverse, waits 300 ms, and
restores the original. Production applications should add authentication,
role authorization, command confirmation, audit trails, interlocks, and
application-specific allowlists.

## Why React with Vite Instead of Next.js

ASP.NET Core already owns the server, API, native runtime, and deployment
boundary. A client-side React application keeps the example focused and builds
to static files that ASP.NET can serve. Next.js would add a second server-side
runtime without improving this single-screen industrial demo.
