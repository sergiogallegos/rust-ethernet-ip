# Metadata and Schema Export Design

## Summary

The repo now has a concrete design proposal for metadata and schema export.

The key recommendation is:

- build export structs in Rust first
- expose one stable JSON-friendly schema document
- add wrapper bindings only after the Rust contract is stable

## Current Understanding

- The repo already has enough discovery primitives to support a first export design:
  - `discover_tags_detailed()`
  - `get_tag_attributes()`
  - `get_udt_definition()`
- The current FFI metadata surfaces are not yet the right foundation for this; some are still stubs.
- Research papers 2 and 11 support a deterministic, sectioned export contract rather than ad hoc wrapper discovery logic.
- The Rust implementation now exists in the core crate as `export_schema()` and `export_schema_json()`.
- Current target metadata remains conservative: route path is included when present, but connection address and controller identity are still omitted.
- Focused unit coverage now exists for schema type classification, tag/UDT mapping, and top-level JSON serialization shape.

## Evidence

- [docs/METADATA_SCHEMA_EXPORT_DESIGN.md](../../docs/METADATA_SCHEMA_EXPORT_DESIGN.md)
- [src/schema.rs](../../src/schema.rs)
- [docs/RESEARCH_FEATURE_MAP.md](../../docs/RESEARCH_FEATURE_MAP.md)
- [docs/tag_introspection.md](../../docs/tag_introspection.md)
- [src/lib.rs](../../src/lib.rs)
- [src/ffi.rs](../../src/ffi.rs)

## Open Questions

- Whether `udt_name` should remain tag-name-derived for now or gain a more precise template/type name mapping later.
- When the JSON export should be exposed through FFI for C# and Python.
- How much controller-identification metadata should be included before there is validated real-hardware coverage.

## Related Pages

- [research-feature-map-2026-04-19.md](research-feature-map-2026-04-19.md)
- [python-wrapper-strategy-2026-04-19.md](python-wrapper-strategy-2026-04-19.md)
- [software-architecture-map.md](software-architecture-map.md)
