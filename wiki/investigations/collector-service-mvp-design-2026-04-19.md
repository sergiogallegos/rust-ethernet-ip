# Collector Service MVP Design

## Summary

The repo now has a concrete collector-service MVP design.

The key recommendation is:

- implement the collector first as a Python example service
- use batch polling and simple CSV or SQLite sinks
- keep service behavior above the Rust core, not inside it

## Current Understanding

- The collector is the right next service-layer move after the Python MVP and schema export work.
- The repo already has the necessary building blocks:
  - Python `Client.read_tags()`
  - CSV and SQLite examples
  - optional API example
  - a larger Rust backend example that demonstrates service shape, but at a broader scope
- The collector should remain narrower than the web backend and should not absorb protocol logic.
- The first collector implementation now exists as a Python example driven by JSON config, batch polling, and CSV/SQLite sinks.

## Evidence

- [docs/COLLECTOR_SERVICE_MVP_DESIGN.md](../../docs/COLLECTOR_SERVICE_MVP_DESIGN.md)
- [docs/RESEARCH_FEATURE_MAP.md](../../docs/RESEARCH_FEATURE_MAP.md)
- [python/examples/log_tags_to_csv.py](../../python/examples/log_tags_to_csv.py)
- [python/examples/log_tags_to_sqlite.py](../../python/examples/log_tags_to_sqlite.py)
- [python/examples/collector_service.py](../../python/examples/collector_service.py)
- [examples/web_app/backend/src/main.rs](../../examples/web_app/backend/src/main.rs)

## Open Questions

- Whether the first implementation should be a single-file example or a small example package directory.
- Whether the collector should persist per-tag errors as rows or only log them for MVP.
- Whether reconnect policy should be fixed in the example or config-driven from the start.

## Related Pages

- [python-wrapper-strategy-2026-04-19.md](python-wrapper-strategy-2026-04-19.md)
- [research-feature-map-2026-04-19.md](research-feature-map-2026-04-19.md)
- [metadata-schema-export-design-2026-04-19.md](metadata-schema-export-design-2026-04-19.md)
