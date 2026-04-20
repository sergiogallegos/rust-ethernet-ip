# Metadata and Schema Export Design

Date: 2026-04-19

## Summary

This document defines the recommended direction for metadata and schema export in `rust-ethernet-ip`.

Goal:

- expose deterministic, tool-friendly controller metadata
- keep Rust as the semantic source of truth
- support C#, Python, and future service adapters without duplicating discovery logic

This is a design document, not an implementation commitment.

Current status:

- Rust-side export structs are now implemented
- `EipClient::export_schema()` and `EipClient::export_schema_json()` now exist
- focused unit coverage now exists for export assembly and JSON shape
- wrapper exposure and real-PLC validation remain follow-up work

## Why This Matters

The repo already supports important discovery and introspection primitives:

- `discover_tags()`
- `discover_tags_detailed()`
- `discover_program_tags()`
- `get_tag_attributes()`
- `get_tag_metadata()`
- `get_udt_definition()`

What is still missing is a stable export shape that lets downstream tooling answer questions like:

- what tags exist
- what type each tag has
- which tags are arrays
- which tags are UDTs
- what UDT members exist
- which pieces are controller scope vs program scope

The research papers most relevant here are:

- paper 2: automatic configuration and discovery
- paper 11: asset-model organization

Those support a deterministic, sectioned export shape rather than ad hoc discovery calls from each wrapper.

## Design Goals

- reuse existing Rust discovery and introspection APIs
- avoid introducing a second metadata model in wrappers
- keep export stable and JSON-friendly
- separate current confirmed data from optional future enrichments
- support partial availability instead of pretending all controllers expose everything

## Non-Goals

- do not turn the core library into an OPC UA server
- do not build a digital twin layer into the protocol crate
- do not require discovery support in every wrapper before the core export shape exists
- do not promise full PLC-wide introspection parity on every controller family immediately

## Recommended Export Shape

Use an explicit top-level document with sections.

Recommended JSON shape:

```json
{
  "schema_version": "0.1",
  "generated_at_utc": "2026-04-19T18:00:00Z",
  "library": {
    "name": "rust-ethernet-ip",
    "version": "0.8.0-dev"
  },
  "target": {
    "address": "192.168.0.101:44818",
    "route_path": [1, 0],
    "controller_family": null,
    "firmware_revision": null
  },
  "capabilities": {
    "tag_discovery": true,
    "tag_attributes": true,
    "udt_definitions": true,
    "program_tags": false
  },
  "tags": [
    {
      "name": "ProductionCount",
      "scope": {
        "kind": "controller",
        "program": null
      },
      "data_type": {
        "cip_code": 196,
        "name": "DINT",
        "kind": "primitive"
      },
      "dimensions": [],
      "size_bytes": 4,
      "permissions": "read_write",
      "template_instance_id": null,
      "udt_name": null
    }
  ],
  "udts": [
    {
      "name": "MotorData",
      "template_instance_id": 123,
      "size_bytes": 64,
      "members": [
        {
          "name": "Speed",
          "offset_bytes": 0,
          "data_type": {
            "cip_code": 202,
            "name": "REAL",
            "kind": "primitive"
          },
          "dimensions": []
        }
      ]
    }
  ]
}
```

## Section Meanings

### `schema_version`

- version the export contract independently from crate version
- allows future additive evolution

### `target`

- identifies the inspected PLC connection context
- should remain conservative
- include route path only when explicitly used
- leave unknown controller-identification fields as `null` until there is validated support

### `capabilities`

- declares what this export actually observed or could populate
- avoids fake completeness

### `tags`

- one entry per discovered tag
- use normalized scope and normalized type section
- include `template_instance_id` when known
- include `udt_name` only when derivable with confidence

### `udts`

- separate template-level structure from tag instances
- supports reuse when many tags share the same template

## Recommended Rust Types

Do not export internal structs directly.

Instead, add explicit export structs such as:

```rust
pub struct SchemaExport {
    pub schema_version: String,
    pub generated_at_utc: String,
    pub library: SchemaLibraryInfo,
    pub target: SchemaTargetInfo,
    pub capabilities: SchemaCapabilities,
    pub tags: Vec<SchemaTag>,
    pub udts: Vec<SchemaUdt>,
}
```

Reason:

- internal structs are optimized for runtime use, not long-term contract stability
- explicit export structs let the repo evolve caching and parsing internals safely

## Mapping from Current APIs

### Current sources to reuse

- `discover_tags_detailed()` for the main tag list
- `get_tag_attributes()` for targeted enrichment or fallback lookups
- `get_udt_definition()` for UDT templates
- `get_tag_metadata()` only as lightweight cache-oriented support, not as the export backbone

### Recommended assembly flow

1. call `discover_tags_detailed()`
2. build `SchemaTag` entries from returned `TagAttributes`
3. collect unique `template_instance_id` or UDT names
4. resolve UDT definitions once per unique template
5. populate `udts`
6. emit a single stable export object

## API Recommendation

Add one high-level Rust method rather than exposing many partial export helpers:

```rust
pub async fn export_schema(&mut self) -> crate::error::Result<SchemaExport>
```

Optional follow-up methods later:

```rust
pub async fn export_schema_json(&mut self) -> crate::error::Result<String>
pub async fn export_tag_schema(&mut self, tag_name: &str) -> crate::error::Result<SchemaTag>
```

Recommendation:

- implement `export_schema()` first
- keep wrapper-specific JSON generation outside the wrappers if possible

## FFI and Wrapper Direction

Do not expose schema export to Python MVP immediately.

Recommended order:

1. implement stable Rust export structs
2. add a JSON-returning FFI surface
3. expose it in C# and Python once the contract is stable

Example future FFI:

```text
eip_export_schema_json(client_id, result_buffer, max_size)
```

This is better than trying to marshal nested trees field-by-field through FFI.

## Known Gaps and Constraints

Current constraints from the repo:

- `eip_discover_tags` in `src/ffi.rs` is still a stub
- `eip_get_tag_metadata` in `src/ffi.rs` is still a stub
- wrapper metadata parity is uneven today
- some discovery behaviors remain controller-profile dependent

This means:

- schema export should begin in Rust first
- wrappers should consume a stabilized export, not invent one

## Phased Plan

## Phase 1

- define export structs
- implement `export_schema()` in Rust
- generate JSON from Rust
- document exact contract

## Phase 2

- add fixture-based tests for export shape
- add simulator-backed export tests where discovery coverage exists
- add one real-PLC validation note once hardware is available

## Phase 3

- expose JSON export via FFI
- add C# and Python wrapper methods
- add one example showing schema dump to file

## Recommended Validation

- unit tests for serialization shape
- deterministic tests for type and scope mapping
- compatibility check that repeated exports are stable in field naming and layout
- real-PLC validation before presenting it as production-ready discovery for all controllers

## Recommendation

This should be the next design-level feature after the current Python MVP work.

It is valuable because it helps:

- Python tooling
- C# tooling
- future collectors and service adapters
- docs and troubleshooting

without moving the repo away from its core identity as the Rust EtherNet/IP implementation.
