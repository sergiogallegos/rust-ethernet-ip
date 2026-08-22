# Rust EtherNet/IP for Python

`rust-ethernet-ip` is a thin Python interface to the Rust EtherNet/IP core for
Allen-Bradley CompactLogix and ControlLogix controllers. It fits data
acquisition, analytics, scientific workflows, ML feature collection, historian
feeds, and lightweight API services that need direct Logix tag access.

## Release Status

- Latest published PyPI package: `1.2.0`
- Repository development line: `1.2.1` (not published yet)
- Supported Python versions: 3.10–3.12
- Native ABI used by 1.2.0: version `2`

The wheel contains the native Rust library. The 1.2.0 real-hardware gate
validated the Python wrapper alongside Rust, C#, and C/C++ on a CompactLogix
5069-L330ERM firmware 38, including controller/program paths, batches, scalar
types, built-in/custom STRING members, arrays, and nested UDT-member paths.

## Install

```bash
python -m pip install rust-ethernet-ip==1.2.0
```

Optional example dependencies:

```bash
python -m pip install 'rust-ethernet-ip[analytics]==1.2.0'  # pandas
python -m pip install 'rust-ethernet-ip[api]==1.2.0'        # FastAPI
python -m pip install 'rust-ethernet-ip[mqtt]==1.2.0'       # MQTT
```

## Start Here

Use a context manager so the EtherNet/IP session is always unregistered:

```python
from rust_ethernet_ip import Client

with Client("192.168.0.10:44818") as plc:
    count = plc.read_tag("ProductionCount")
    temperature = plc.read_tag("TankTemperature")
    running = plc.read_tag("MachineRunning")
    recipe = plc.read_string("RecipeName")

    plc.write_tag("ProductionSetpoint", 1250)
    plc.write_tag("TemperatureSetpoint", 72.5)
    plc.write_tag("EnableCommand", True)
    plc.write_tag("RecipeName", "PRODUCT_A")
```

Python values infer these common Logix types:

- `bool` → `BOOL`
- 32-bit-range `int` → `DINT`
- larger signed `int` → `LINT`
- `float` → `REAL`
- `str` → `STRING`

Use `value_type` when inference is ambiguous or a narrower/unsigned type is
required:

```python
plc.write_tag("SmallCounter", 123, value_type="INT")
plc.write_tag("UnsignedCount", 4_000_000_000, value_type="UDINT")
plc.write_tag("PrecisionValue", 1.23456789, value_type="LREAL")
```

## STRING Support in 1.2.0

STRING writes are handle-aware. The same `write_tag` call supports top-level
built-in STRING tags and built-in/custom STRING members addressed by full path:

```python
with Client("192.168.0.10:44818") as plc:
    plc.write_tag("RecipeName", "PRODUCT_A")
    plc.write_tag("Mixer.Description", "Primary mixer")
    plc.write_tag("Motors[0].Description", "Infeed conveyor")

    print(plc.read_string("Motors[0].Description"))
```

Real hardware confirms built-in `STRING`, custom `Str82`, and custom `Str400`
members on 5069-L330ERM firmware 38. A 600-byte custom string has simulator
fragmentation coverage; qualify very large custom strings on the exact target
controller and firmware before production use.

## Controller and Program-Scoped Tags

Known program tags use the Logix symbolic prefix directly:

```python
with Client("192.168.0.10:44818") as plc:
    controller_value = plc.read_tag("ProductionCount")
    program_value = plc.read_tag("Program:MainProgram.ProductionCount")
    plc.write_tag("Program:MainProgram.ProductionSetpoint", 1250)
```

The Python 1.2.0 wrapper does not expose tag discovery or metadata APIs. It can
read and write known controller/program paths, arrays, bits, and UDT members.
Do not assume that program enumeration is available merely because direct
program paths work.

## Batch Reads and Writes

```python
from rust_ethernet_ip import BatchReadError, BatchWriteItem, Client

with Client("192.168.0.10:44818") as plc:
    try:
        values = plc.read_tags([
            "ProductionCount",
            "TankTemperature",
            "Program:MainProgram.MachineRunning",
        ])
    except BatchReadError as exc:
        print("Successful values:", exc.partial_values)
        print("Per-tag errors:", exc.errors)
        raise

    results = plc.write_tags([
        BatchWriteItem("ProductionSetpoint", 1250),
        BatchWriteItem("TemperatureSetpoint", 72.5),
        BatchWriteItem("EnableCommand", True),
        BatchWriteItem("RecipeName", "PRODUCT_A"),
        BatchWriteItem("SmallCounter", 123, value_type="INT"),
    ])

    for tag, result in results.items():
        print(tag, "ok" if result.success else result.error)
```

`read_tags` uses the native batch-read path. On validated ControlLogix
hardware, `write_tags` intentionally submits writes sequentially so each tag
retains accurate success/error reporting.

## ControlLogix Routing

Connect to the Ethernet module and supply the CPU backplane slot:

```python
from rust_ethernet_ip import Client, RoutePath

route = RoutePath(slots=[0])
with Client("192.168.0.20:44818", route_path=route) as plc:
    print(plc.read_tag("ProductionCount"))
```

For an ordered multi-hop route, use explicit hops:

```python
from rust_ethernet_ip import Client, RouteHop, RoutePath

route = RoutePath(hops=[
    RouteHop.backplane(slot=3),
    RouteHop.ethernet("192.168.10.20", port=2),
    RouteHop.backplane(slot=0),
])

with Client("192.168.0.20:44818", route_path=route) as plc:
    print(plc.read_tag("ProductionCount"))
```

CompactLogix controllers with built-in Ethernet normally do not need a route.

## Health and Diagnostics

```python
with Client("192.168.0.10:44818") as plc:
    plc.read_tag("ProductionCount")
    print("healthy:", plc.check_health())

    snapshot = plc.get_diagnostics_snapshot(detailed=True)
    print("reads:", snapshot.operations.total_reads)
    print("failed reads:", snapshot.operations.failed_reads)
    print("average latency:", snapshot.performance.avg_read_latency_ms)
    print("last error:", snapshot.errors.last_error_message)
```

CPU and memory values are placeholders in this release. Connection,
operation, error, latency, and verified-health metrics are the useful fields.

## Error Handling

```python
from rust_ethernet_ip import (
    BatchReadError,
    NativeLibraryLoadError,
    PlcConnectionError,
    PlcOperationError,
)

try:
    with Client("192.168.0.10:44818") as plc:
        plc.write_tag("ReadOnlyTag", 42)
except NativeLibraryLoadError as exc:
    print("Native package problem:", exc)
except PlcConnectionError as exc:
    print("Connection problem:", exc)
except BatchReadError as exc:
    print(exc.partial_values, exc.errors)
except PlcOperationError as exc:
    print("PLC/CIP operation problem:", exc)
```

`PlcOperationError` includes the native last-error reason when available.

## Example Catalog

Core wrapper examples:

- [`read_single_tag.py`](examples/read_single_tag.py)
- [`write_single_tag.py`](examples/write_single_tag.py)
- [`read_batch_tags.py`](examples/read_batch_tags.py)
- [`write_batch_tags.py`](examples/write_batch_tags.py)
- [`program_scoped_tags.py`](examples/program_scoped_tags.py)
- [`control_logix_route.py`](examples/control_logix_route.py)
- [`diagnostics_snapshot.py`](examples/diagnostics_snapshot.py)

Data and application examples:

- [`log_tags_to_csv.py`](examples/log_tags_to_csv.py)
- [`log_tags_to_sqlite.py`](examples/log_tags_to_sqlite.py)
- [`pandas_dataframe_example.py`](examples/pandas_dataframe_example.py)
- [`collector_service.py`](examples/collector_service.py)
- [`fastapi_service_example.py`](examples/fastapi_service_example.py)
- [`mqtt_publisher_example.py`](examples/mqtt_publisher_example.py)

Set `RUST_ETHERNET_IP_PLC_ADDRESS` for the examples. Routed examples also use
`RUST_ETHERNET_IP_PLC_SLOT`.

```bash
export RUST_ETHERNET_IP_PLC_ADDRESS=192.168.0.10:44818
PYTHONPATH=python python3 python/examples/read_single_tag.py
```

## Collector, API, and MQTT Examples

```bash
PYTHONPATH=python python3 python/examples/collector_service.py \
  --config python/examples/collector_config.example.json --once

PYTHONPATH=python python3 python/examples/mqtt_publisher_example.py \
  --config python/examples/mqtt_publisher_config.example.json --once
```

The collector writes timestamped batch snapshots to CSV or SQLite. The MQTT
example publishes normalized snapshots to
`factory/{site}/plc/{plc_name}/snapshot`. A Docker example stack is available:

```bash
docker compose -f docker/python-stack/docker-compose.yml up --build
```

## Local Repository Development

From the repository root:

```bash
cargo build --features ffi --example python_test_simulator
PYTHONPATH=python python3 -m unittest discover -s python/tests
RUST_ETHERNET_IP_START_SIM=1 PYTHONPATH=python \
  python3 -m unittest discover -s python/tests
```

If the native library is outside the usual `target/debug` or `target/release`
location, set `RUST_ETHERNET_IP_NATIVE_LIB` to its absolute path.

## Current Boundaries

- Python is intentionally synchronous and thin; Rust owns protocol behavior.
- Program-scoped enumeration, subscriptions, and tag-group APIs are not exposed
  in Python 1.2.0.
- Whole UDT-array-element writes are not supported; write known members by
  their full paths.
- This package targets CompactLogix and ControlLogix EtherNet/IP tag access,
  not Modbus TCP or a general OPC server.

See the [integration guide](../docs/INTEGRATION_AND_DEPLOYMENT.md),
[hardware matrix](../docs/HARDWARE_COMPATIBILITY.md), and
[1.2.0 validation record](../docs/validation/2026-07-08_release-1.2.0-gate_cross-binding_5069-L330ERM_fw38.md).

Use [GitHub Issues](https://github.com/sergiogallegos/rust-ethernet-ip/issues)
for reproducible defects and
[GitHub Discussions](https://github.com/sergiogallegos/rust-ethernet-ip/discussions)
for integration questions. The package is MIT licensed.
