# Monitoring and Diagnostics Plan

## Summary

The repo now has a concrete plan for strengthening monitoring and diagnostics before any anomaly-oriented features.

The key conclusion is:

- improve operational telemetry in the Rust core first
- expose that telemetry upward through thin wrappers
- avoid inventing separate diagnostics models in service examples

## Current Understanding

- The current monitoring module now has a first Rust-side `DiagnosticsSnapshot`, explicit `HealthCheckMode`, and first-pass `ErrorCategory` classification.
- That diagnostics snapshot is now exposed through FFI and mapped into thin C# and Python wrapper types.
- Some metrics are still placeholders, and real-PLC validation of the diagnostics surfaces is still pending.
- The existing real-PLC validation records already show controller-specific error patterns that should inform diagnostics classification.
- `check_health` and `check_health_detailed` need a clearer shared diagnostics model behind them.
- Paper 10 is useful here mainly as a sequencing reminder: reliable telemetry comes before anomaly detection.

## Evidence

- [docs/MONITORING_DIAGNOSTICS_IMPROVEMENT_PLAN.md](../../docs/MONITORING_DIAGNOSTICS_IMPROVEMENT_PLAN.md)
- [src/monitoring.rs](../../src/monitoring.rs)
- [src/lib.rs](../../src/lib.rs)
- [src/ffi.rs](../../src/ffi.rs)
- [csharp/RustEtherNetIp/EthernetNetIpClient.Diagnostics.cs](../../csharp/RustEtherNetIp/EthernetNetIpClient.Diagnostics.cs)
- [python/rust_ethernet_ip/client.py](../../python/rust_ethernet_ip/client.py)
- [docs/RESEARCH_FEATURE_MAP.md](../../docs/RESEARCH_FEATURE_MAP.md)
- [docs/validation/REAL_PLC_TESTING.md](../../docs/validation/REAL_PLC_TESTING.md)
- [docs/validation/2026-04-07_real_plc_5069-L320ERMS3_fw35.md](../../docs/validation/2026-04-07_real_plc_5069-L320ERMS3_fw35.md)

## Open Questions

- Whether the next diagnostics iteration should include a rolling recent-error ring buffer or remain aggregate-only.
- How much real-PLC validation should be gathered before promoting diagnostics as a supported wrapper contract.

## Related Pages

- [research-feature-map-2026-04-19.md](research-feature-map-2026-04-19.md)
- [rest-mqtt-adapter-boundaries-2026-04-19.md](rest-mqtt-adapter-boundaries-2026-04-19.md)
- [metadata-schema-export-design-2026-04-19.md](metadata-schema-export-design-2026-04-19.md)
