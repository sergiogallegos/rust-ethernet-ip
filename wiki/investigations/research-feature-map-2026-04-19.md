# Research Feature Map

## Summary

The curated research papers now have an explicit feature map tying them to repo components and timing.

The strongest near-term paper-driven moves are:

- Python data-collection examples
- metadata and schema-export design
- collector-service planning

The papers do not justify shifting protocol logic out of the Rust core.

## Current Understanding

- Papers 8, 9, and 13 support the current Python-wrapper direction because they emphasize reliable data collection and analytics-friendly export, not protocol reinvention.
- Papers 2 and 11 are the strongest support for a future metadata or schema-export layer.
- Papers 3, 12, 14, and 15 support collector, REST, and MQTT adapter work, but as layers above the core library.
- Papers 6 and 7 are useful as architectural guardrails more than direct feature triggers.

## Evidence

- [docs/RESEARCH_FEATURE_MAP.md](../../docs/RESEARCH_FEATURE_MAP.md)
- [docs/research/CURATED_INDUSTRIAL_RESEARCH_READING_LIST.md](../../docs/research/CURATED_INDUSTRIAL_RESEARCH_READING_LIST.md)

## Open Questions

- Whether schema export should remain a repo-specific JSON contract or align partially with AAS-like sectioning.
- Whether the first service-layer artifact after Python should be a collector, a REST bridge, or an MQTT publisher.

## Related Pages

- [research-papers-industrial-platform-roadmap-2026-04-19.md](research-papers-industrial-platform-roadmap-2026-04-19.md)
- [python-wrapper-strategy-2026-04-19.md](python-wrapper-strategy-2026-04-19.md)
- [python-mvp-surface-2026-04-19.md](python-mvp-surface-2026-04-19.md)
- [software-architecture-map.md](software-architecture-map.md)
