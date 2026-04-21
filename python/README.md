# Python Wrapper

This package is the current thin Python wrapper for the `rust-ethernet-ip` native library.

Current scope:

- thin bindings over the existing Rust FFI boundary
- Pythonic client lifecycle
- optional route-path connection support for routed ControlLogix targets
- generic read/write and batch operations

It is intentionally light and keeps Rust as the protocol and performance core.

## Local Development

Build the native library first from the repo root:

```bash
cargo build --release
```

Run the lightweight Python tests from the repo root:

```bash
PYTHONPATH=python python3 -m unittest discover -s python/tests
```

Run the optional simulator-backed integration tests by setting `SIM_PLC_ADDRESS` to a reachable deterministic simulator instance:

```bash
SIM_PLC_ADDRESS=127.0.0.1:44818 PYTHONPATH=python python3 -m unittest discover -s python/tests
```

Or ask the Python tests to launch the in-repo simulator example automatically:

```bash
RUST_ETHERNET_IP_START_SIM=1 PYTHONPATH=python python3 -m unittest discover -s python/tests
```

You can also launch the simulator example directly and point tests or examples at it:

```bash
cargo run --quiet --example python_test_simulator
```

Load the package directly from the repo during development:

```bash
PYTHONPATH=python python3
```

If the native library is not in a default repo build output location, set:

```bash
export RUST_ETHERNET_IP_NATIVE_LIB=/absolute/path/to/librust_ethernet_ip.dylib
```

On Windows, point that variable to `rust_ethernet_ip.dll`.

## Current MVP Surface

- `Client(address, route_path=None, auto_connect=True)`
- `connect()`
- `disconnect()`
- `read_tag(name)`
- `write_tag(name, value, value_type=None)`
- `read_tags(names)`
- `write_tags(items)`
- `check_health()`
- `get_diagnostics_snapshot(detailed=False)`

Python `float` values default to PLC `REAL` in the wrapper.

If you need `LREAL`, pass it explicitly with `value_type="LREAL"`.

Diagnostics snapshots are exposed as thin Python dataclasses with nested metrics sections.

For routed ControlLogix access, pass `route_path=RoutePath(slots=[cpu_slot])`.

On validated ControlLogix hardware, `write_tags(...)` currently executes writes sequentially in the Python wrapper so per-tag success and error reporting stays accurate on live PLCs.

## Examples

- `python/examples/read_single_tag.py`
- `python/examples/read_batch_tags.py`
- `python/examples/log_tags_to_csv.py`
- `python/examples/log_tags_to_sqlite.py`
- `python/examples/pandas_dataframe_example.py`
- `python/examples/fastapi_service_example.py`
- `python/examples/collector_service.py`
- `python/examples/mqtt_publisher_example.py`

Optional example dependencies:

- analytics examples: `pip install '.[analytics]'`
- API examples: `pip install '.[api]'`
- MQTT example: `pip install '.[mqtt]'`

Collector service example:

```bash
PYTHONPATH=python python3 python/examples/collector_service.py \
  --config python/examples/collector_config.example.json \
  --once
```

The collector uses batch reads and writes timestamped rows to either CSV or SQLite based on the config file.
Set `RUST_ETHERNET_IP_PLC_ADDRESS` to override the PLC address from the config file.
Set `RUST_ETHERNET_IP_PLC_SLOT` to inject a simple slot-only route path for routed ControlLogix targets.

MQTT publisher example:

```bash
PYTHONPATH=python python3 python/examples/mqtt_publisher_example.py \
  --config python/examples/mqtt_publisher_config.example.json \
  --once
```

The MQTT example publishes normalized batch snapshots to a broker topic shaped like
`factory/{site}/plc/{plc_name}/snapshot`.
Set `RUST_ETHERNET_IP_MQTT_HOST` to override the broker host from the config file.
Set `RUST_ETHERNET_IP_PLC_SLOT` to inject a simple slot-only route path for routed ControlLogix targets.

The FastAPI example also supports:

- `RUST_ETHERNET_IP_PLC_ADDRESS`
- `RUST_ETHERNET_IP_PLC_SLOT`
- `RUST_ETHERNET_IP_API_HOST`
- `RUST_ETHERNET_IP_API_PORT`

Docker example stack:

```bash
docker compose -f docker/python-stack/docker-compose.yml up --build
```

## Scope Guardrail

This package is an enablement layer for Python users.

The Rust library remains the source of truth for:

- EtherNet/IP behavior
- correctness
- performance
- protocol semantics

## Current Validation Status

- pure-Python unit tests run with the standard library `unittest`
- integration tests exist for simulator-backed connect/read/write/batch flows
- simulator-backed tests are skipped automatically when `SIM_PLC_ADDRESS` is not set
- integration tests can auto-launch the in-repo simulator with `RUST_ETHERNET_IP_START_SIM=1`
- simulator-backed batch coverage includes mixed `DINT`, `REAL`, `BOOL`, and `STRING` flows
