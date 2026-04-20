# REST and MQTT Adapter Boundaries

Date: 2026-04-19

## Summary

This document defines the recommended boundaries for REST and MQTT adapters built on top of `rust-ethernet-ip`.

Goal:

- clarify what belongs in the core library
- clarify what belongs in wrappers
- clarify what belongs in service adapters

The key rule is:

- direct PLC access remains in the Rust core and its thin wrappers
- REST and MQTT are adapter layers above that access layer

## Why This Matters

The repo now has three service-adjacent reference points:

- a Rust web backend example
- a Python FastAPI example
- a Python collector example

Without clear boundaries, these can drift into overlapping mini-platforms.

Papers 4, 14, and 15 all point to the same conclusion:

- request/response access and pub/sub distribution solve different problems
- structured semantics should remain stable across those boundaries
- messaging layers should not replace the core data-access layer

## Core Rule

### What the Rust Core Owns

- EtherNet/IP connection management
- route-path behavior
- read/write semantics
- batch operations
- tag/group/subscription semantics already supported by the library
- metadata and schema export

### What Wrappers Own

- ergonomic language bindings
- resource lifetime management
- exceptions and error translation
- convenient examples for language users

### What Adapters Own

- HTTP request/response translation
- MQTT topic/payload publication
- persistence workflow orchestration
- auth, deployment, and service configuration

## REST Boundary

REST is for:

- on-demand reads
- explicit writes
- snapshots
- management and status endpoints

REST is not for:

- high-rate event streaming as the primary transport
- replacing the batch-read semantics in the library

### Recommended REST Surface

MVP shape:

- `GET /health`
- `GET /tags/{name}`
- `POST /tags/{name}`
- `POST /snapshot`
- `GET /schema`

Where:

- `GET /tags/{name}` maps to `read_tag`
- `POST /tags/{name}` maps to `write_tag`
- `POST /snapshot` maps to batch reads on a requested or configured tag set
- `GET /schema` maps to `export_schema_json`

### REST Payload Recommendation

Keep payloads explicit and close to the wrapper model.

Example read response:

```json
{
  "tag_name": "REAL_TAG",
  "value": 6.5,
  "value_type": "REAL"
}
```

Example snapshot response:

```json
{
  "timestamp_utc": "2026-04-19T18:00:00Z",
  "values": {
    "DINT_TAG": 1234,
    "REAL_TAG": 6.5
  },
  "errors": {}
}
```

### REST Recommendation

- REST should expose snapshots and command-style interactions
- use batch reads for snapshot endpoints
- do not create many single-tag backend round-trips when one batch read would do

## MQTT Boundary

MQTT is for:

- publishing sampled data outward
- decoupling collectors from downstream consumers
- feeding UNS-style, historian, analytics, or event-driven pipelines

MQTT is not for:

- becoming the primary internal representation of PLC state
- replacing schema export or direct read/write APIs

### Recommended MQTT Surface

The first MQTT adapter should sit on top of the collector output, not directly on raw PLC calls.

Recommended flow:

1. collector batch reads tags
2. collector normalizes rows or snapshot objects
3. MQTT adapter publishes those normalized objects

This avoids:

- duplicated polling logic
- duplicated reconnect logic
- duplicated error handling

### Recommended MQTT Topic Shape

Conservative topic shape:

```text
factory/{site}/plc/{plc_name}/tag/{tag_name}
```

Or for snapshots:

```text
factory/{site}/plc/{plc_name}/snapshot
```

### Recommended MQTT Payload Shape

Payloads should preserve semantics:

```json
{
  "timestamp_utc": "2026-04-19T18:00:00Z",
  "tag_name": "REAL_TAG",
  "value": 6.5,
  "value_type": "REAL",
  "quality": "ok"
}
```

For snapshot messages:

```json
{
  "timestamp_utc": "2026-04-19T18:00:00Z",
  "values": {
    "REAL_TAG": 6.5,
    "BOOL_TAG": true
  },
  "errors": {}
}
```

### MQTT Recommendation

- start with snapshot publication or row publication, not both
- keep one polling engine and one publisher
- preserve `timestamp`, `value`, `value_type`, and error context

## Relationship Between Collector, REST, and MQTT

Recommended dependency direction:

1. Rust core
2. thin wrappers
3. collector
4. REST or MQTT adapters

Meaning:

- the collector is the first reusable service boundary
- REST and MQTT should reuse collector semantics or wrapper semantics, not bypass them with new protocol code

## What Not To Do

- do not add HTTP-specific logic into the Rust core crate
- do not add MQTT client dependencies into the Rust core crate for the first adapter iteration
- do not implement one poll loop for REST and another for MQTT if both can share a collector/service layer
- do not let topic names or REST routes become the schema source of truth

## Recommended Next Implementation Order

1. keep the current Python collector as the reference polling service
2. expose a slightly cleaner collector output model if needed
3. add a REST adapter that serves health, read, write, snapshot, and schema
4. add an MQTT publisher example that publishes normalized snapshot data

## Short-Term Recommendation

For this repo:

- keep the FastAPI example small and request/response oriented
- keep the collector focused on polling and sinks
- add MQTT only after the collector output model is settled

## Long-Term Recommendation

When the repo grows more service examples:

- schema export should become the shared structure reference
- collectors should remain the polling source
- REST and MQTT should become transport adapters, not competing service cores
