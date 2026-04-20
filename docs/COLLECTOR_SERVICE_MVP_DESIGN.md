# Collector Service MVP Design

Date: 2026-04-19

## Summary

This document defines the recommended MVP for a collector service built around `rust-ethernet-ip`.

Purpose:

- poll PLC tags on a configured interval
- emit timestamped snapshots
- write those snapshots to simple sinks such as CSV or SQLite
- keep the service layer separate from the Rust protocol core

This is an MVP service design, not a commitment to a final production architecture.

Current status:

- the first collector example now exists in `python/examples/collector_service.py`
- a starter config exists in `python/examples/collector_config.example.json`
- focused runtime validation against a real PLC is still pending

## Why This Comes Next

The current repo now has:

- a stable Rust core
- a C# wrapper
- a Python wrapper MVP
- analytics-oriented Python examples
- a schema export direction

The next useful system-level piece is not MQTT or a large platform. It is a collector service that turns PLC reads into consistent time-series records for downstream systems.

This aligns with:

- paper 3: gateway architecture
- paper 12: edge/CPPS architecture
- paper 13: low-latency collection patterns

## Core Guardrail

The collector service must remain:

- a thin service layer above the Rust core
- configuration-driven
- explicit about polling and output sinks

It must not move protocol logic, tag parsing semantics, or controller-specific behavior out of the Rust library.

## MVP Goals

- connect to one PLC
- poll a configured tag set on a fixed interval
- use batch reads by default
- timestamp every sample
- write to one sink per run
- fail clearly and predictably

## Non-Goals

- no built-in MQTT in the first collector MVP
- no built-in cloud stack
- no multi-PLC orchestration in the first iteration
- no complicated rule engine
- no historian replacement claims

## Recommended Placement

Recommended first location:

- `python/examples/collector_service.py`

Reason:

- the Python path is already positioned for analytics and data workflows
- the repo now has Python examples for CSV, SQLite, pandas, and API use
- the service can stay small and dependency-light for the first iteration

Longer-term, this can evolve into either:

- a Python package example under `python/examples/`
- a dedicated example app under `examples/collector_service/`

## Recommended Runtime Shape

MVP architecture:

1. configuration loader
2. PLC client lifecycle
3. batch poll loop
4. sink writer
5. simple status/error logging

Recommended flow:

1. load config
2. connect to PLC
3. on each interval:
   - batch read configured tags
   - capture `timestamp_utc`
   - normalize into rows
   - write to sink
4. stop cleanly on signal or fatal connection failure

## Recommended Config Shape

Use a small JSON config first.

Example:

```json
{
  "plc": {
    "address": "192.168.0.10:44818"
  },
  "polling": {
    "interval_ms": 1000
  },
  "tags": [
    "DINT_TAG",
    "REAL_TAG",
    "BOOL_TAG",
    "STRING_TAG"
  ],
  "sink": {
    "kind": "sqlite",
    "path": "data/plc_samples.sqlite"
  }
}
```

## Sink Options for MVP

### CSV

Best for:

- quick export
- manual inspection
- simple demo flows

Recommended row shape:

- `timestamp_utc`
- `tag_name`
- `value_json`
- `value_type`

### SQLite

Best for:

- lightweight local persistence
- later querying
- notebooks and batch analysis

Recommended table:

```sql
CREATE TABLE IF NOT EXISTS plc_samples (
    timestamp_utc TEXT NOT NULL,
    tag_name TEXT NOT NULL,
    value_json TEXT NOT NULL,
    value_type TEXT NOT NULL
);
```

## Recommended Data Shape

Normalize every poll result into rows rather than storing one giant opaque object.

Recommended internal record:

```python
{
    "timestamp_utc": "...",
    "tag_name": "REAL_TAG",
    "value": 6.5,
    "value_type": "float"
}
```

For persistence, serialize `value` as JSON-compatible text where needed.

## Error Handling Recommendation

### Batch Read Behavior

Use batch read first.

If per-tag failures occur:

- keep successful values
- write failure records or log them explicitly
- do not silently drop errors

MVP recommendation:

- continue polling after partial failures
- log failed tags per cycle
- only stop on repeated connection-level failure

### Connection Failure

For MVP:

- log the failure
- retry connection with a bounded backoff
- stop after a configured max retry count or user interrupt

## CLI Recommendation

Use a simple CLI entry shape:

```bash
PYTHONPATH=python python3 python/examples/collector_service.py --config collector.json
```

Optional later flags:

- `--once`
- `--duration-seconds`
- `--stdout`

## Relationship to Other Repo Pieces

### Python Wrapper

- collector service should use `Client.read_tags()`
- do not bypass the wrapper to call FFI directly from the service

### Schema Export

- future collector versions can use schema export to enrich rows with data type metadata
- that should be additive, not required for MVP

### Web Backend

- the existing Rust `web_app` backend is a dashboard/demo service
- the collector should stay simpler and narrower than that backend

## Validation Recommendation

For the first collector implementation:

- unit test config parsing if extracted into helpers
- run it against the auto-start Python simulator path where possible
- validate CSV and SQLite output files are created and populated

Real-PLC validation later should confirm:

- timing stability
- long-running reconnect behavior
- data quality under repeated polls

## Recommended MVP Deliverables

1. `python/examples/collector_service.py`
2. `python/examples/collector_config.example.json`
3. README section for running the collector
4. one CSV mode
5. one SQLite mode

## Recommendation

Implement the collector as a Python example service first.

That is the right level for the repo today because it:

- supports the Python/data/AI direction
- reuses the current wrapper
- keeps the Rust core clean
- creates a real service-layer reference for future MQTT and REST designs
