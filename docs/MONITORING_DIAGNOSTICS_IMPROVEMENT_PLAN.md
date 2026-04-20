# Monitoring and Diagnostics Improvement Plan

Date: 2026-04-19

## Summary

This document defines the next monitoring and diagnostics improvements for `rust-ethernet-ip`.

The goal is not to turn the core crate into an observability platform.

The goal is to make the Rust core expose reliable operational signals that:

- reflect real PLC communication behavior
- support wrapper and service troubleshooting
- provide a sound base for later anomaly-oriented work

This follows the repo guardrail:

- Rust remains the protocol and semantics source of truth
- wrappers remain thin
- higher-level services consume diagnostics rather than inventing their own PLC-health model

## Current State

The current monitoring module already has:

- connection counters
- read/write counters
- coarse latency aggregates
- coarse error counters
- a computed health status

It is useful as an internal sketch, but it is not yet a strong operational contract.

## Current Gaps

### 1. Placeholder System Metrics

`memory_usage_mb` and `cpu_usage_percent` are currently hard-coded placeholders.

Implication:

- these values are not trustworthy enough for user-facing health surfaces
- wrappers and services should not treat them as meaningful telemetry

### 2. Error Taxonomy Is Too Coarse

Current error buckets are:

- network
- protocol
- timeout
- tag_not_found
- data_type

This is not enough to distinguish important real-world EtherNet/IP cases such as:

- path/routing failures
- session or connection invalidation
- embedded service failures
- known controller write-limit cases
- retryable vs non-retryable failures

### 3. Health Is Library-Centric, Not Operation-Centric

The current `overall_health` calculation is based mostly on:

- active connection count
- total failure count
- aggregate read/write error rate

This misses more actionable signals such as:

- recent successful operation timestamp
- recent failed operation timestamp
- last retriable error
- health by connection target
- batch partial-failure rate

### 4. Monitoring Is Not Yet a Stable Export Surface

The repo has:

- `check_health`
- `check_health_detailed`
- wrapper-level exception and diagnostics patterns

But there is not yet one explicit Rust-side diagnostics snapshot contract for reuse by:

- C#
- Python
- collector/API/MQTT examples

### 5. Monitoring and Real Validation Are Not Tied Together Enough

The repo already has validated real-PLC error shapes in `docs/validation/`.

The monitoring model should align with those known behaviors so diagnostics reflect:

- real CompactLogix/ControlLogix failure patterns
- known direct STRING/UDT write limitations
- route/path issues that matter in enterprise deployments

## Recommended Direction

## Phase 1. Tighten Core Semantics

Low-risk cleanup inside the current module:

- remove unnecessary async paths in internal metrics updates
- keep lock hold times narrow
- document which metrics are authoritative and which are placeholders

Status:

- the unnecessary async `record_error` path has now been removed

## Phase 2. Define a Stable Diagnostics Snapshot

Add a Rust-side snapshot contract, separate from tracing/logging, that can be reused by wrappers and adapters.

Recommended shape:

- `DiagnosticsSnapshot`
- `ConnectionDiagnostics`
- `OperationDiagnostics`
- `ErrorDiagnostics`
- `HealthDiagnostics`

Recommended fields:

- last successful read/write timestamps
- last failed read/write timestamps
- recent latency summaries
- retryable/non-retryable failure counts
- batch partial-failure counts
- last known PLC-facing error classification
- whether health is based on passive state or active verification

This should be:

- serializable
- cheap to read
- explicit about unknown or placeholder values

Status:

- implemented in the Rust core with `DiagnosticsSnapshot`
- current snapshot includes explicit placeholder signaling for system CPU/memory metrics
- FFI and wrapper exposure remains follow-up work

## Phase 3. Align Error Classification with Real PLC Behavior

Introduce a stronger error classification layer that maps existing native errors into operational categories.

Candidate categories:

- network
- timeout
- session
- route_path
- cip_protocol
- batch_embedded_service
- known_controller_limitation
- data_type
- not_found
- unknown

The point is not to hide the original error string.

The point is to add a stable operational category that wrappers and services can reason about.

Status:

- first-pass `ErrorCategory` classification is implemented in the Rust core
- current mapping covers timeout, network, session, route-path, embedded-service, known controller limitation, data-type, not-found, and generic CIP protocol cases
- real-PLC validation and wrapper exposure remain follow-up work

## Phase 4. Improve Health Surfaces

Health should distinguish between:

- passive connected state
- recent successful operation state
- active verified health check state

Recommended contract additions:

- `health_mode`: `passive` or `verified`
- `last_verified_at`
- `last_success_at`
- `last_failure_at`
- `consecutive_failures`
- `last_error_category`

This will make `check_health` vs `check_health_detailed` easier to explain and consume in wrappers.

## Phase 5. Expose Diagnostics Above the Core

After the Rust-side snapshot is stable:

- expose it through FFI
- map it into C# types
- expose it in Python as a thin structured object or dict

Service layers should then reuse this, rather than invent their own health schema.

Status:

- implemented through a JSON-based FFI export
- exposed in C# as thin DTOs and in Python as thin dataclasses
- real-PLC validation for the diagnostics surfaces remains follow-up work

## What Not To Do

- do not add heavy observability dependencies into the Rust core for this step
- do not turn placeholder CPU/memory values into fake production promises
- do not duplicate diagnostics logic independently in C# and Python
- do not add ML or anomaly logic before operational telemetry is trustworthy

## Suggested Implementation Order

1. define Rust-side diagnostics snapshot types
2. classify native errors into stable operational categories
3. improve health snapshot semantics
4. add unit tests for error classification and health-state transitions
5. expose diagnostics via FFI for wrapper reuse
6. add real-PLC validation notes for diagnostics behavior

## Recommendation for 0.8.x

The best near-term target is:

- stronger diagnostics snapshot
- stronger error classification
- wrapper reuse of Rust diagnostics

That gives the project better operational quality without changing its core mission.

## Related Documents

- [SOFTWARE_ARCHITECTURE.md](SOFTWARE_ARCHITECTURE.md)
- [RESEARCH_FEATURE_MAP.md](RESEARCH_FEATURE_MAP.md)
- [REST_MQTT_ADAPTER_BOUNDARIES.md](REST_MQTT_ADAPTER_BOUNDARIES.md)
- [METADATA_SCHEMA_EXPORT_DESIGN.md](METADATA_SCHEMA_EXPORT_DESIGN.md)
