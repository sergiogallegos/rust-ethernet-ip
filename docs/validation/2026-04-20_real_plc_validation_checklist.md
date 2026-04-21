# 2026-04-20 Real PLC Validation Checklist

Status: routed ControlLogix release-gate pass captured

Use this checklist for the next real-PLC validation session on the current `0.8.0` draft line.

Target exercised on 2026-04-20:

- PLC: `1756-L81ES`
- Address: `192.168.0.101:44818`
- Topology: routed via `1756-EN3TR`, backplane slot `0`

## Current Results

### Final Release-Gate Pass

- PASS: a final serial release-gate run on 2026-04-20 revalidated the exercised Rust, C#, and Python surfaces against the same routed `1756-L81ES` target without cross-test interference
- CONFIRMED: live write-heavy checks must be run serially on this PLC/test-tag set; running the smoke, benchmark, and full validation app in parallel can perturb shared `gTestArray_DINT[5..7]` write targets

### Rust Core

- PASS: `cargo test schema:: --lib`
- PASS: `cargo test discovery_tests --lib`
- PASS: `TEST_PLC_ADDRESS=192.168.0.101:44818 TEST_PLC_SLOT=0 cargo test --test health_check_tests -- --ignored --nocapture`
- PASS: `TEST_PLC_ADDRESS=192.168.0.101:44818 TEST_PLC_SLOT=0 cargo test --test route_path_operations_tests -- --nocapture`
- PASS: routed read-only probe still passed `44/44`
- PASS: routed passive and detailed health checks both reported healthy
- PASS: routed diagnostics snapshots returned `Passive` and `Verified` health modes with `Healthy` overall status and `0` route-path errors
- PASS: routed `export_schema()` now succeeds after fixing the malformed request shape, adding paged Symbol Object discovery, and implementing paged Template Object reads for large UDT definitions
- PASS WITH WARNINGS: the live schema export on this target returned `43` top-level tags, `9` UDTs, preserved `route_path=[0]`, and reduced the warning list to a single target-address omission note
- CONFIRMED: routed Template Object reads on this target require `Template Read (0x4C)` request data in the form `offset:u32 + byte_count:u16`; size-only requests return `0x13 Not enough data`
- PASS: the 2026-04-21 rerun again produced `44/44` on `readonly_plc_probe`, `2/2` health checks, `5/5` route-path tests, `5/5` batch tests, `3/3` cache tests, `4/4` subscription tests, and the same `333/392` full-matrix result with `333` restored / `0` restore failures

### Python Wrapper

- PASS AFTER REBUILD: `cargo build` / `cargo build --release` refreshed the cdylib so the Python wrapper could load `eip_get_diagnostics_json`
- PASS: Python route-path support was added on top of the existing FFI `eip_connect_with_route` entry point
- PASS: routed Python `check_health()` and diagnostics snapshot calls succeeded against `192.168.0.101:44818`
- PASS: routed Python single-tag read succeeded for `gTestArray_DINT[0]`
- PASS: routed Python batch reads succeeded for valid `gTest*` tags
- PASS: routed Python `STRING` reads now decode live Logix `STRING` payloads to plain text, including `gTest_STRING="HELLO"` in both single and batch reads
- PASS: collector example wrote `4` rows to SQLite when run with `RUST_ETHERNET_IP_PLC_SLOT=0`
- PASS: `python/examples/collector_config.example.json` now uses the same validated `gTest*` tag set as the real-PLC smoke pass
- PASS AFTER FIX: the dedicated 2026-04-21 Python wrapper ControlLogix validation record confirmed the routed live write-status mismatch was fixed for the exercised `DINT` and `REAL` paths; one routed BOOL write path remains an explicit follow-up item

### C# Wrapper

- PASS: `TEST_PLC_ADDRESS=192.168.0.101:44818 TEST_PLC_SLOT=0 dotnet run --project examples/CSharpWrapperSmoke/CSharpWrapperSmoke.csproj`
- PASS: `TEST_PLC_ADDRESS=192.168.0.101:44818 TEST_PLC_SLOT=0 dotnet run --project examples/CSharpWrapperBenchmark/CSharpWrapperBenchmark.csproj -- --iterations 25`
- PASS: `TEST_PLC_ADDRESS=192.168.0.101:44818 TEST_PLC_SLOT=0 dotnet run --project examples/CSharpWrapperTest/CSharpWrapperTest.csproj`
- PASS: temporary routed diagnostics probe reported `CheckHealth=True`, `CheckHealthDetailed=True`, `Passive` and `Verified` diagnostics snapshots, `Healthy` overall status, and `0` route-path errors
- PASS WITH EXPECTED LIMITATIONS: when run in isolation, the routed full-tag validation app passed `333/392`; the `59` non-passes were limited to known firmware-limited STRING/UDT-array-member writes
- PASS: benchmark at `25` iterations reported about `550` single reads/sec, `467` single writes/sec, `5484` batch-read logical ops/sec, `1008` batch-write logical ops/sec, and `1918` mixed logical ops/sec on the final serial pass
- CONFIRMED: the earlier `gTestArray_DINT[5..7]` verify mismatches were caused by running the smoke, benchmark, and full validation app in parallel against the same live write targets; they do not reproduce when the full validation app is run alone
- PASS: the 2026-04-21 rerun again produced smoke pass + `333/392` full-matrix pass profile; the isolated benchmark rerun at `25` iterations reported about `673` single reads/sec, `493` single writes/sec, `5903` batch-read logical ops/sec, `1055` batch-write logical ops/sec, and `1993` mixed logical ops/sec
- INFO: launching the benchmark and full validation harness concurrently on 2026-04-21 caused a local `.pdb` file-lock conflict in `obj/Debug/net10.0`; rerunning the benchmark by itself resolved it

### MQTT Publisher

- PASS: a temporary Python virtualenv with `paho-mqtt 2.1.0` published one routed snapshot from the physical `1756-L81ES` target to a local Mosquitto broker
- PASS: the broker-backed subscriber observed topic `factory/lab/plc/controllogix-l81es/snapshot`
- PASS: the observed payload contained `timestamp_utc`, `plc_name`, `values`, and empty `errors`
- PASS: the published values matched the live PLC read on this pass, including `gTestArray_DINT[0]=1000`, `gTestArray_REAL[0]=10.0`, `gTestArray_BOOL[0]=false`, and `gTestUDT.Member1_DINT=7777`

### Optional In This Pass

- Docker stack validation remains optional and is not treated as a release blocker for the core Rust/C#/Python libraries

Blockers observed:
- none on the exercised Rust/C#/Python routed ControlLogix paths; remaining non-passes in the full C# matrix are the expected PLC firmware limitations only

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
