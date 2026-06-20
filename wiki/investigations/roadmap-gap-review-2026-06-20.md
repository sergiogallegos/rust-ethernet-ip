# Roadmap Gap Review 2026-06-20

## Summary

`docs/ROADMAP.md` already covered the urgent 1.1.0 follow-ups: documentation
cleanup, missing platform artifacts, data-type-table deduplication, FFI registry
hardening, SemVer-major public-surface cleanup, C# true async, and multi-chassis
routing validation.

This review added missing roadmap items for public API positioning, wrapper
parity, unsupported API decisions, diagnostics placeholders, passive config,
wrapper maintainability, test quality, supply-chain policy, simulator coverage,
and post-publish registry smoke checks.

## Current Understanding

- `confirmed`: Rust now exposes `Client`, `RetryClient`, `Fleet`, connection
  events, and service-layer helpers alongside `EipClient`, but current public
  docs and examples still mostly teach `EipClient`.
- `confirmed`: Python is installable and covers the 1.1.0 MVP path, but it does
  not expose several richer FFI surfaces that C# already uses, including detailed
  discovery, tag attributes, UDT definitions, and array-range helpers.
- `confirmed`: C# batch configuration methods are public but intentionally throw
  `NotSupportedException`; the native batch-config FFI exports are placeholders.
- `confirmed`: legacy `eip_discover_tags` / `eip_get_tag_metadata` exports are
  placeholders even though newer detailed discovery / attributes APIs exist.
- `confirmed`: diagnostics JSON intentionally marks system metrics as
  placeholders, so future operational positioning should either implement real
  metrics or keep that limitation explicit.
- `confirmed`: `ProductionConfig` is a richer configuration model than the
  current runtime actually consumes.
- `confirmed`: `EtherNetIpClient.cs` remains a large mixed-responsibility wrapper
  file even after async and native-method partials exist.
- `confirmed`: multiple ignored hardware-oriented tests still need a quality pass
  so configured runs assert behavior instead of returning early, and
  `tests/TEST_COVERAGE_SUMMARY.md` is stale.
- `confirmed`: CI has `cargo-audit`, but no checked-in `cargo-deny` policy for
  license, source, duplicate-version, or banned-dependency checks.
- `likely`: simulator expansion is the best next step before more real-hardware
  work for metadata, UDT discovery, restricted writes, and wrapper decoding.

## Evidence

- Updated roadmap: [`../../docs/ROADMAP.md`](../../docs/ROADMAP.md)
- Public Rust exports: [`../../src/lib.rs`](../../src/lib.rs)
- Actor client and service helpers: [`../../src/client/actor.rs`](../../src/client/actor.rs),
  [`../../src/client/service_layer.rs`](../../src/client/service_layer.rs)
- Fleet API: [`../../src/fleet.rs`](../../src/fleet.rs)
- FFI surface and placeholders: [`../../src/ffi.rs`](../../src/ffi.rs)
- Diagnostics placeholders: [`../../src/monitoring.rs`](../../src/monitoring.rs)
- Passive config surface: [`../../src/config.rs`](../../src/config.rs)
- C# wrapper surface: [`../../csharp/RustEtherNetIp/EthernetNetIpClient.cs`](../../csharp/RustEtherNetIp/EthernetNetIpClient.cs),
  [`../../csharp/RustEtherNetIp/README.md`](../../csharp/RustEtherNetIp/README.md)
- Python wrapper surface: [`../../python/rust_ethernet_ip/client.py`](../../python/rust_ethernet_ip/client.py),
  [`../../python/rust_ethernet_ip/bindings.py`](../../python/rust_ethernet_ip/bindings.py)
- Ignored hardware tests and stale coverage summary: [`../../tests`](../../tests),
  [`../../tests/TEST_COVERAGE_SUMMARY.md`](../../tests/TEST_COVERAGE_SUMMARY.md)
- CI supply-chain baseline: [`../../.github/workflows/ci.yml`](../../.github/workflows/ci.yml)
- Prior wiki synthesis: [`client-actor-service-retry-2026-05-24.md`](client-actor-service-retry-2026-05-24.md),
  [`fleet-api-2026-05-24.md`](fleet-api-2026-05-24.md),
  [`python-wrapper-strategy-2026-04-19.md`](python-wrapper-strategy-2026-04-19.md),
  [`test-coverage-strength-2026-05-18.md`](test-coverage-strength-2026-05-18.md)

## Open Questions

- Should `Client` / `Fleet` become the recommended Rust-first API for new apps,
  or stay advanced while `EipClient` remains the primary teaching path?
- Should Python aim for C# parity around discovery/UDT/schema APIs in 1.2.0, or
  should parity be staged after the documentation refresh?
- Should public C# batch-config methods be implemented end-to-end or deprecated
  before the 2.0 removal window?
- Should legacy metadata FFI exports be implemented as compatibility shims over
  detailed discovery, or marked unsupported and removed in 2.0?
- Should post-publish registry smoke checks run as a scheduled/manual workflow,
  or remain a release-manager checklist?

## Sequencing

Recommended order after 1.1.0:

1. Documentation refresh plus Rust API positioning.
2. Placeholder / passive-surface decisions, including batch config, legacy
   metadata exports, diagnostics placeholders, and passive `ProductionConfig`
   fields.
3. Python parity expansion on top of honest native surfaces.
4. Platform/package coverage and internal refactors as capacity allows.

## Related Pages

- [`client-actor-service-retry-2026-05-24.md`](client-actor-service-retry-2026-05-24.md)
- [`fleet-api-2026-05-24.md`](fleet-api-2026-05-24.md)
- [`python-wrapper-strategy-2026-04-19.md`](python-wrapper-strategy-2026-04-19.md)
- [`test-coverage-strength-2026-05-18.md`](test-coverage-strength-2026-05-18.md)
