# Research Papers for Industrial Platform Roadmap

## Summary

A curated set of ten industrial/IIoT/digital-twin/ML references was added first, followed by five more targeted papers on asset modeling, edge deployment, low-latency collection, and semantic pub/sub patterns.

The most useful conclusion is that the papers are more valuable for:

- data modeling
- discovery
- gateway/service design
- analytics-oriented data flows

than for low-level EtherNet/IP protocol correctness itself.

## Current Understanding

- Papers on OPC UA, IIoT gateways, and protocol comparison are useful for shaping service layers and wrapper/API direction.
- Papers on predictive maintenance and ICS anomaly detection are useful for Python/data examples and future observability work.
- Digital twin papers are useful for long-term schema and structured-object direction, but should not distract from the repo’s core identity as a Rust EtherNet/IP library.
- Some initially proposed links were incorrect, especially several `arXiv` references; the curated reading list corrects that.
- The most actionable additions from the second pass are:
  - Asset Administration Shell modeling for future schema/metadata export
  - edge-computing architecture for collector-service boundaries
  - low-latency flat IIoT architecture for batching and service topology decisions
  - OPC UA pub/sub and MQTT comparisons for future adapter and event-stream designs

## Evidence

- [docs/research/CURATED_INDUSTRIAL_RESEARCH_READING_LIST.md](../../docs/research/CURATED_INDUSTRIAL_RESEARCH_READING_LIST.md)
- <https://www.mdpi.com/2624-831X/3/4/27>
- <https://www.mdpi.com/2079-9292/8/6/600>
- <https://www.mdpi.com/1424-8220/24/7/2072>
- <https://www.mdpi.com/1999-5903/11/3/66>
- <https://www.sciencedirect.com/science/article/pii/S2542660521000846>
- <https://colab.ws/articles/10.1109/access.2020.2998358>
- <https://www.mdpi.com/2071-1050/12/19/8211>
- <https://www.mdpi.com/2078-2489/16/9/737>
- <https://link.springer.com/article/10.1186/s42400-021-00095-5>
- <https://www.mdpi.com/1424-8220/21/6/2004>
- <https://www.sciencedirect.com/science/article/pii/S1877050919309317>
- <https://www.sciencedirect.com/science/article/pii/S1383762122001564>
- <https://www.sciencedirect.com/science/article/pii/S2542660525002483>
- <https://pmc.ncbi.nlm.nih.gov/articles/PMC9606965/>

## Open Questions

- Whether the repo should store local copies of these papers or only curated references and summaries.
- Which paper-driven ideas should become concrete roadmap items first:
  - discovery/schema work
  - data collector service
  - MQTT/REST examples
  - anomaly/monitoring hooks
- Whether a future schema/export layer should align more with lightweight repo-specific models or with AAS-style structured envelopes.

## Related Pages

- [ecosystem-platform-patterns-2026-04-19.md](ecosystem-platform-patterns-2026-04-19.md)
- [python-wrapper-strategy-2026-04-19.md](python-wrapper-strategy-2026-04-19.md)
- [python-mvp-surface-2026-04-19.md](python-mvp-surface-2026-04-19.md)
