# Python MVP API and FFI Mapping

This document turns the Python wrapper strategy into a concrete MVP design.

Date: 2026-04-19

## Summary

The Python MVP should use a small subset of the existing Rust FFI boundary and prefer generic JSON-based operations over many typed entrypoints.

Recommended approach:

- use the existing `cdylib`
- load it from Python through a thin binding layer
- keep the public Python API narrow
- use generic FFI calls where possible:
  - `eip_connect`
  - `eip_disconnect`
  - `eip_read_tag`
  - `eip_write_tags_batch` for both single and multi-write
  - `eip_read_tags_batch`
  - `eip_check_health`

This keeps the Python surface smaller and avoids encoding a large amount of type-specific marshaling logic in Python.

## Proposed Public Python API

```python
from rust_ethernet_ip import Client

with Client("192.168.0.10:44818") as plc:
    value = plc.read_tag("Program:Main.Counter")
    plc.write_tag("Program:Main.Counter", 42, value_type="DINT")

    results = plc.read_tags([
        "Tag1",
        "Tag2",
        "Program:Main.Tag3",
    ])

    plc.write_tags([
        {"tag_name": "Tag1", "value": 123, "value_type": "DINT"},
        {"tag_name": "Tag2", "value": True, "value_type": "BOOL"},
    ])

    healthy = plc.check_health()
```

## Public API Shape

Recommended initial API:

- `Client(address: str, *, auto_connect: bool = True)`
- `Client.connect() -> None`
- `Client.disconnect() -> None`
- `Client.close() -> None`
- `Client.__enter__() / __exit__()`
- `Client.read_tag(tag_name: str) -> object`
- `Client.write_tag(tag_name: str, value: object, *, value_type: str) -> None`
- `Client.read_tags(tag_names: list[str]) -> dict[str, object]`
- `Client.write_tags(items: list[dict]) -> dict[str, WriteResult]`
- `Client.check_health() -> bool`

Optional but not MVP:

- route-path connect
- subscriptions
- tag groups
- UDT metadata helpers

## Recommended Python Types

Keep Python-facing types lightweight:

- normal Python scalars for basic values
- `dict[str, Any]` for generic JSON-decoded values
- small dataclasses only for batch results and exceptions if needed

Suggested helper shapes:

```python
@dataclass
class WriteResult:
    tag_name: str
    success: bool
    error: str | None = None
```

## FFI Mapping

## Connection Lifecycle

Python API:

- `Client.connect()`
- `Client.disconnect()`

FFI calls:

- `eip_connect(ip_address: *const c_char) -> c_int`
- `eip_disconnect(client_id: c_int) -> c_int`

Notes:

- store the returned `client_id` in the Python client object
- `client_id < 0` means connection failure

## Single Read

Python API:

- `Client.read_tag(tag_name)`

FFI call:

- `eip_read_tag(client_id, tag_name, result_buffer, max_size)`

Reason to prefer this:

- it returns JSON-encoded `PlcValue` content
- it avoids Python needing many typed read functions
- it supports richer tag forms than a type-specific scalar-only path

Python side:

- allocate a buffer
- call `eip_read_tag`
- decode returned JSON
- map into Python values

## Single Write

Python API:

- `Client.write_tag(tag_name, value, value_type=...)`

Recommended FFI call:

- `eip_write_tags_batch(client_id, tag_values_json, tag_count, result_buffer, max_size)`

Reason to prefer this even for one write:

- the current FFI has no generic `eip_write_tag` JSON entrypoint
- using batch write for a single item gives per-item result detail
- it already accepts a JSON payload and supports multiple types

Payload shape:

```json
[
  {
    "tag_name": "Program:Main.Counter",
    "value_type": "DINT",
    "value": 42
  }
]
```

## Batch Read

Python API:

- `Client.read_tags([...])`

FFI call:

- `eip_read_tags_batch(client_id, tag_names, tag_count, result_buffer, max_size)`

Expected Python behavior:

- return a dictionary keyed by tag name
- raise only on call-level failure
- preserve per-tag failures in a structured result or exception policy

Recommended MVP behavior:

- return `dict[str, object]` when all tags succeed
- raise a `BatchReadError` carrying per-tag errors when any tag fails

## Batch Write

Python API:

- `Client.write_tags([...])`

FFI call:

- `eip_write_tags_batch(client_id, tag_values_json, tag_count, result_buffer, max_size)`

Recommended result:

- `dict[str, WriteResult]`

## Health Check

Python API:

- `Client.check_health()`

FFI call:

- `eip_check_health(client_id, out is_healthy)`

Practical note:

- `eip_check_health_detailed` currently delegates to the same implementation
- for the Python MVP, do not expose a separate `check_health_detailed()` yet

## Not Recommended for MVP

Avoid in the first Python cut:

- typed scalar FFI calls like `eip_read_dint`, `eip_write_bool`, etc. as the primary Python surface
- metadata and discovery wrappers as part of the required MVP
- route-path connect in the initial API unless it falls out naturally from implementation
- free-form batch execute as the first public batch surface

## FFI Gaps and Friction Points

These are the main gaps found while mapping the MVP to the current FFI:

## Gap 1: No Generic Single-Write Entry Point

Current state:

- there is `eip_read_tag`
- there is no symmetrical generic `eip_write_tag` FFI that accepts JSON or a generic payload

Impact:

- Python should use `eip_write_tags_batch` for single writes in the MVP

Recommendation:

- acceptable for MVP
- later consider adding a real generic `eip_write_tag` FFI helper for symmetry and simpler bindings

## Gap 2: Health Check Is Shallow

Current state:

- `eip_check_health` only checks whether the client ID exists in the registry
- `eip_check_health_detailed` currently delegates to the same logic

Impact:

- Python `check_health()` should be documented as a lightweight native-session check, not a guaranteed PLC round-trip

Recommendation:

- acceptable for MVP
- later improve the native health model before exposing a richer Python health surface

## Gap 3: Metadata Discovery Stubs

Current state:

- `eip_discover_tags` returns success without real implementation
- `eip_get_tag_metadata` returns `-1`

Impact:

- do not build Python metadata/discovery MVP features on top of these stubs

Recommendation:

- explicitly exclude these from Python MVP

## Gap 4: Error Surface Is Numeric + JSON, Not Versioned

Current state:

- many FFI functions return `0` / `-1`
- some richer error detail only exists inside JSON batch results

Impact:

- Python exceptions should wrap the existing model, not pretend more precision exists

Recommendation:

- define a narrow Python exception hierarchy
- later consider a more explicit native error channel if wrapper ergonomics become a priority

## Gap 5: Historical Wrapper Docs Are Stale

Current state:

- docs reference `pywrapper/` and older wrapper experiments that are not present in the live tree

Impact:

- new Python work must not assume those paths or architectures still exist

Recommendation:

- treat them as historical only
- reframe or archive them after the new Python path is established

## Recommended Python MVP Checklist

- [ ] create `python/rust_ethernet_ip/` package skeleton
- [ ] implement native-library loader in `bindings.py`
- [ ] implement `Client` lifecycle over `eip_connect` / `eip_disconnect`
- [ ] implement `read_tag()` over `eip_read_tag`
- [ ] implement `write_tag()` over one-item `eip_write_tags_batch`
- [ ] implement `read_tags()` over `eip_read_tags_batch`
- [ ] implement `write_tags()` over `eip_write_tags_batch`
- [ ] implement `check_health()` over `eip_check_health`
- [ ] add smoke tests against the simulator-backed path if feasible
- [ ] add CSV / SQLite / pandas-oriented examples

## Design Guardrail

The Python wrapper should make the Rust core easier to adopt in analytics, AI, MES, and service workflows.

It should not become a second protocol implementation or a separate product center.
