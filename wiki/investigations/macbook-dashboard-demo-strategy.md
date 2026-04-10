# MacBook Dashboard Demo Strategy

## Summary

For a MacBook-hosted demo aimed at showing live PLC visibility, operator-style status, and manager-facing machine KPIs, the strongest current recommendation is a web app. Use the existing web patterns in this repo and keep the PLC-facing backend on the same machine as the browser demo.

## Current Understanding

- `confirmed`: A web app is the best fit for the stated audience because it works on macOS immediately and can also be shown to directors, managers, or IT stakeholders from other devices without rebuilding a desktop client.
- `confirmed`: The repo already contains two viable web-oriented starting points:
  - a Rust backend with React frontend in `examples/web_app/`
  - an ASP.NET Core API example using the C# wrapper in `examples/AspNetExample/`
- `confirmed`: The Rust core and the C# wrapper were both validated on real CompactLogix and routed ControlLogix targets on `2026-04-07` for the exercised read/write, batch, subscription, health-check, and route-path flows.
- `confirmed`: For a capability demo, batch reads, batch writes, and mixed execute are the strongest features to highlight because they have the clearest real-hardware throughput evidence.
- `needs-care`: Direct writes to standalone `STRING` tags, direct writes to `STRING` members inside UDTs, and direct writes to UDT array element members should not be central to the dashboard demo because they remain documented controller limitations.
- `confirmed`: The current Rust web example has now been extended into a manager-facing dashboard shape with route-path connection support, controller identity lookup, live batch snapshot panels, trend charts, a benchmark panel, and local traceability-event persistence.
- `confirmed`: The current dashboard leans on the already-created `gTest*` and `Program:TestProgram.*` fixture tags instead of introducing a separate demo PLC schema.
- `confirmed`: The current persistence layer for the demo is local JSON storage, which keeps the example portable on a MacBook without introducing database setup overhead.

## Recommendation

### Preferred Demo Shape

- Use a browser-based dashboard on the MacBook.
- Keep the PLC connection in a backend service, not in frontend JavaScript.
- Use the existing `gTest*` controller and program tags as the demo data contract.
- Prefer program-scoped tags for richer seeded data on the validated ControlLogix target when visual impact matters.

### Backend Choice

- Use the Rust backend if the goal is to prove the native library directly and show the strongest "pure Rust" story.
- Use the ASP.NET Core backend with the C# wrapper if the goal is to show how a .NET MES/OEE-style application would integrate in a realistic enterprise stack.

### What To Show In The UI

- Live connection state and controller identity
- Poll-cycle latency and last successful refresh time
- Batch-read KPI cards for throughput, machine state, counts, and rates
- A read/write panel for supported primitive tags
- A side-by-side "single vs batch" performance panel using the validated benchmark tag set
- A limitations panel that explicitly marks unsupported direct `STRING` and UDT-array-member writes as controller behavior, not library regressions

## Practical Guidance

- For ControlLogix via Ethernet module, use route-path connection with slot `0` in the current validated topology.
- For CompactLogix with integrated Ethernet, use direct connection unless routing is specifically required.
- Prefer polling/tag-group style updates for dashboard views instead of many ad hoc single reads.
- Prefer whole-structure read-modify-write if the demo needs to update UDT-backed values that include restricted members.
- Keep the first demo focused on stable primitives and supported batch workflows; add full OEE/MES semantics later as a second layer.
- Use local file-backed persistence first for the demo story; move to SQLite only when the example needs stronger querying or multi-session durability.

## Evidence

- [../../wiki/releases/0.7.0-validation-synthesis.md](../releases/0.7.0-validation-synthesis.md)
- [../../wiki/wrapper-parity/rust-vs-csharp.md](../wrapper-parity/rust-vs-csharp.md)
- [../../wiki/limitations/string-and-udt-write-behavior.md](../limitations/string-and-udt-write-behavior.md)
- [../../docs/validation/2026-04-07_real_plc_5069-L320ERMS3_fw35.md](../../docs/validation/2026-04-07_real_plc_5069-L320ERMS3_fw35.md)
- [../../docs/validation/2026-04-07_real_plc_1756-L81ES_via_1756-EN3TR_slot0.md](../../docs/validation/2026-04-07_real_plc_1756-L81ES_via_1756-EN3TR_slot0.md)
- [../../docs/validation/2026-04-07_csharp_wrapper_real_plc_5069-L320ERMS3_fw35.md](../../docs/validation/2026-04-07_csharp_wrapper_real_plc_5069-L320ERMS3_fw35.md)
- [../../docs/validation/2026-04-07_csharp_wrapper_real_plc_1756-L81ES_via_1756-EN3TR_slot0.md](../../docs/validation/2026-04-07_csharp_wrapper_real_plc_1756-L81ES_via_1756-EN3TR_slot0.md)
- [../../examples/web_app/README.md](../../examples/web_app/README.md)
- [../../examples/AspNetExample/README.md](../../examples/AspNetExample/README.md)
- [../../docs/PLC_TEST_TAG_DEFINITIONS.md](../../docs/PLC_TEST_TAG_DEFINITIONS.md)

## Open Questions

- Whether the repo should standardize on one primary manager-facing demo stack instead of maintaining parallel Rust-web and ASP.NET-web example paths.
- Whether a dedicated OEE/MES demo dataset should be added to the PLC fixtures instead of reusing the generic `gTest*` validation tags.
- Whether controller identity lookup should move into the core Rust library API instead of remaining example-specific raw CIP logic in the web backend.

## Related Pages

- [../releases/0.7.0-validation-synthesis.md](../releases/0.7.0-validation-synthesis.md)
- [../wrapper-parity/rust-vs-csharp.md](../wrapper-parity/rust-vs-csharp.md)
- [../limitations/string-and-udt-write-behavior.md](../limitations/string-and-udt-write-behavior.md)
- [../protocol/route-path-behavior.md](../protocol/route-path-behavior.md)
