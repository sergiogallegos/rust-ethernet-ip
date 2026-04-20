# Ecosystem Platform Patterns

## Summary

Public Rockwell, Ignition-community, and Design Group repositories suggest that the strongest role for this repo is not a full SCADA/MES platform, but the data-access layer underneath those systems.

The durable conclusion is:

- keep Rust as the PLC/protocol core
- expose thin wrapper layers
- grow toward reusable data services and templates

## Current Understanding

- Rockwell's public GitHub material emphasizes CI/CD and tooling around Logix projects rather than a modern open software-access SDK.
- Ignition-community repositories emphasize modular extensions, gateway APIs, and integration layers.
- Design Group repositories emphasize practical delivery patterns such as Dockerized development environments, CI/CD support, REST-facing modules, and utility scripts.
- Python is strategically important because it fits data collection, analytics, AI, and lightweight service workflows better than .NET alone.

## Evidence

- Rockwell Automation GitHub: <https://github.com/RockwellAutomation>
- `ra-logix-cicd`: <https://github.com/RockwellAutomation/ra-logix-cicd>
- Ignition Module Development Community:
  - <https://github.com/IgnitionModuleDevelopmentCommunity/ignition-extensions>
  - <https://github.com/IgnitionModuleDevelopmentCommunity/IgnitionNode-RED>
- Design Group:
  - <https://github.com/design-group>
  - <https://github.com/design-group/ignition-docker>
  - <https://github.com/design-group/ignition-tag-cicd-module>
- [docs/PLATFORM_EXPANSION_BACKLOG.md](../../docs/PLATFORM_EXPANSION_BACKLOG.md)
- [docs/CODEX_PYTHON_PLATFORM_EXPANSION_PROMPT.md](../../docs/CODEX_PYTHON_PLATFORM_EXPANSION_PROMPT.md)

## Open Questions

- Whether the current external Rust boundary is already clean enough for the first Python wrapper, or should be formalized further first.
- How quickly the repo should move from a wrapper-only step into higher-level services such as collectors and REST/MQTT examples.

## Related Pages

- [software-architecture-map.md](software-architecture-map.md)
- [../wrapper-parity/rust-vs-csharp.md](../wrapper-parity/rust-vs-csharp.md)
- [rust-toolchain-baseline-2026-04-19.md](rust-toolchain-baseline-2026-04-19.md)
