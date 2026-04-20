# REST and MQTT Adapter Boundaries

## Summary

The repo now has a concrete design note for REST and MQTT adapter boundaries.

The key conclusion is:

- REST and MQTT are transport adapters above the core data-access layer
- they should not become parallel protocol implementations

## Current Understanding

- The existing Rust web backend example is broader than the desired REST MVP boundary.
- The Python FastAPI example is currently the cleanest small request/response adapter reference.
- The new Python collector is the right source layer for a future MQTT publisher because it already owns polling and normalization.
- The first MQTT example now follows that boundary by publishing normalized batch snapshots above the wrapper instead of embedding broker logic into the Rust core.
- Papers 4, 14, and 15 support this split: request/response, snapshot, and pub/sub should be separate concerns built on shared semantics.

## Evidence

- [docs/REST_MQTT_ADAPTER_BOUNDARIES.md](../../docs/REST_MQTT_ADAPTER_BOUNDARIES.md)
- [docs/RESEARCH_FEATURE_MAP.md](../../docs/RESEARCH_FEATURE_MAP.md)
- [python/examples/fastapi_service_example.py](../../python/examples/fastapi_service_example.py)
- [python/examples/collector_service.py](../../python/examples/collector_service.py)
- [python/examples/mqtt_publisher_example.py](../../python/examples/mqtt_publisher_example.py)
- [examples/web_app/backend/src/main.rs](../../examples/web_app/backend/src/main.rs)

## Open Questions

- Whether the first REST adapter should stay as a Python example or also get a Rust example beyond the current dashboard backend.
- Whether the next MQTT iteration should publish per-tag rows as an additional mode beyond the current snapshot-first example.
- Whether schema export should be wired into REST before MQTT arrives, or both should wait for more real-PLC validation.

## Related Pages

- [collector-service-mvp-design-2026-04-19.md](collector-service-mvp-design-2026-04-19.md)
- [metadata-schema-export-design-2026-04-19.md](metadata-schema-export-design-2026-04-19.md)
- [research-feature-map-2026-04-19.md](research-feature-map-2026-04-19.md)
