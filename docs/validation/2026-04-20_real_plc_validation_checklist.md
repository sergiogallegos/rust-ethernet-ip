# 2026-04-20 Real PLC Validation Checklist

Status: planned

Use this checklist for the next real-PLC validation session on the current `0.8.0` draft line.

## Goals

Validate the newer work added beyond the already-completed Rust/C# ControlLogix pass:

- schema export
- Python wrapper
- collector service
- diagnostics snapshots
- MQTT publisher

## Prerequisites

- reachable PLC address, for example `192.168.x.x:44818`
- route-path details if using ControlLogix through ENxT/backplane
- Rust toolchain `1.95`
- `.NET 10 SDK`
- Python `3.10+`
- optional reachable MQTT broker if validating publisher end to end

## Environment

Set as needed:

```bash
export TEST_PLC_ADDRESS=192.168.x.x:44818
export RUST_ETHERNET_IP_PLC_ADDRESS=192.168.x.x:44818
export RUST_ETHERNET_IP_MQTT_HOST=127.0.0.1
```

## Rust Core

1. Schema export

```bash
cargo test schema:: --lib
```

Then run a small manual schema-export probe against the PLC:

- connect
- call `export_schema()`
- save JSON output
- confirm:
  - top-level tags are populated
  - UDTs are populated when present
  - route-path is preserved when expected
  - warnings are understandable

2. Health behavior

- verify `check_health()`
- verify `check_health_detailed()`
- verify routed connection behavior if applicable

## Python Wrapper

1. Baseline

```bash
PYTHONPATH=python python3 -m unittest discover -s python/tests
```

2. Manual wrapper smoke

```bash
PYTHONPATH=python python3 python/examples/read_single_tag.py
PYTHONPATH=python python3 python/examples/read_batch_tags.py
```

3. Diagnostics snapshot

- call `get_diagnostics_snapshot()`
- call `get_diagnostics_snapshot(detailed=True)`
- confirm:
  - mapping is valid
  - `health_mode` changes as expected
  - timestamps are populated sensibly

## Collector Service

Run once:

```bash
PYTHONPATH=python python3 python/examples/collector_service.py \
  --config python/examples/collector_config.example.json \
  --once
```

Confirm:

- SQLite or CSV output file is created
- rows are written
- partial batch errors are surfaced if present

## MQTT Publisher

With a reachable broker:

```bash
PYTHONPATH=python python3 python/examples/mqtt_publisher_example.py \
  --config python/examples/mqtt_publisher_config.example.json \
  --once
```

Confirm:

- publisher connects
- snapshot is published
- topic matches `factory/{site}/plc/{plc_name}/snapshot`
- payload includes `timestamp_utc`, `values`, and `errors`

## Docker Stack

In a network-enabled environment:

```bash
docker compose -f docker/python-stack/docker-compose.yml up --build
docker compose -f docker/python-stack/docker-compose.yml --profile mqtt up --build
```

Confirm:

- API starts
- collector starts
- optional MQTT stack starts
- env overrides work as intended

## C# Wrapper

Run wrapper regression:

```bash
dotnet build csharp/RustEtherNetIp/RustEtherNetIp.csproj -c Release
dotnet test csharp/RustEtherNetIp.Tests/RustEtherNetIp.Tests.csproj -c Release
```

Manual checks:

- `CheckHealth()`
- `CheckHealthDetailed()`
- `GetDiagnosticsSnapshot()`
- `GetDiagnosticsSnapshotDetailed()`

## Record Results

When done:

- update the appropriate `docs/validation/*.md` records
- update `CHANGELOG.md` or release notes if needed
- update `wiki/` if the validation produces durable engineering conclusions
