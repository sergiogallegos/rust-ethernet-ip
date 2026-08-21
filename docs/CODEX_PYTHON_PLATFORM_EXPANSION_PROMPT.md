# Codex Prompt: Python and Data Platform Expansion

> **Historical planning prompt.** This captured the `1.0.0` expansion plan.
> The latest published release is `1.2.0`, and `1.2.1` is in preparation; use
> `README.md`, `docs/README.md`, and `docs/ROADMAP.md` for current status.

You are working inside the repository `rust-ethernet-ip`.

## Project Context

- This repo is a production-focused EtherNet/IP library for Allen-Bradley CompactLogix and ControlLogix PLCs.
- It already includes:
  - a Rust core library
  - a C# wrapper
  - tests, examples, docs, build scripts, and release notes
- The working line at the time of this prompt was `1.0.0`, not `0.8.0`.
- The Rust baseline is on Rust 2024 with workspace MSRV `rust-version = "1.88"`.
- The C# wrapper targets `.NET 10` and `C# 14`.

## Strategic Goal

We are evolving the project from:

- `Rust EtherNet/IP library`

to:

- `Industrial Data Access Layer for Modern Software Systems`

Important guardrail:

- this repository must remain the strong Rust core for EtherNet/IP
- wrappers exist to help users build their own C# or Python projects
- ecosystem additions should support adoption, not displace the repo's core-library identity

The repo should become a bridge between:

- Rockwell PLCs and OT systems
- enterprise software
- MES and OEE solutions
- analytics and data engineering workflows
- AI and ML workflows

## Core Product Direction

Keep this architecture principle:

1. Rust is the source of truth for protocol behavior and performance.
2. Wrappers stay thin.
3. Do not duplicate core PLC/protocol logic in higher-level languages.
4. Favor an external, reusable Rust boundary that can support more than one wrapper over time.

## Ecosystem Inspiration

This prompt is informed by patterns seen in these public sources:

- Rockwell Automation GitHub: <https://github.com/RockwellAutomation>
- `ra-logix-cicd`: <https://github.com/RockwellAutomation/ra-logix-cicd>
- Ignition Module Development Community:
  - <https://github.com/IgnitionModuleDevelopmentCommunity/ignition-extensions>
  - <https://github.com/IgnitionModuleDevelopmentCommunity/IgnitionNode-RED>
- Design Group:
  - <https://github.com/design-group>
  - <https://github.com/design-group/ignition-docker>
  - <https://github.com/design-group/ignition-tag-cicd-module>

Key interpretation:

- Rockwell publishes tooling, not a modern open software access SDK.
- Ignition-style platforms build modular integrations on top of a data-access layer.
- Integrators ship Docker stacks, CI/CD helpers, gateway utilities, and service patterns.
- Python is important for analytics, scripting, and lightweight services.

## Main Task

Design and implement a first-class Python path for this repo while preserving the existing Rust and C# foundation.

## Phase 1: Inspect the Real Repo First

Before making invasive changes:

1. Read:
  - `AGENTS.md`
  - `README.md`
  - `BUILD.md`
  - `Cargo.toml`
  - `docs/README.md`
  - `docs/SOFTWARE_ARCHITECTURE.md`
  - wrapper-related docs and source files
2. Inspect:
  - the Rust public API surface
  - the FFI boundary in `src/ffi.rs`
  - the C# wrapper implementation
  - release/build tooling relevant to packaging
3. Summarize:
  - what wrapper boundary already exists
  - whether Python can reuse the current external boundary safely
  - what changes would be needed for a stable Python-facing integration strategy

## Phase 2: Choose the Python Wrapper Strategy

Evaluate at least these two options:

### Option A

- formalize or reuse the external Rust FFI boundary
- implement Python as a thin wrapper on top of that boundary

### Option B

- use `PyO3` / `maturin` directly against Rust

Unless the real repo strongly points elsewhere, prefer Option A if it best supports:

- long-term multi-language support
- clean separation of core logic and wrappers
- future wrapper reuse beyond C# and Python

Explain the choice in terms of:

- maintainability
- API stability
- packaging complexity
- long-term platform direction

## Phase 3: Produce a Concrete Plan Before Coding

Create a short plan document or working notes that covers:

- chosen architecture
- files/modules to add
- files/modules to change
- proposed public Python API
- packaging strategy
- test strategy
- example set
- compatibility risks

Do not jump into a large implementation without this design pass.

## Phase 4: Implement a Python MVP

Implement a narrow but strong MVP.

Target capabilities:

- connect to a PLC
- disconnect / resource cleanup
- read one tag
- write one tag
- batch read multiple tags
- health check if available cleanly

Only expose subscriptions, tag groups, or UDT discovery in the MVP if they map cleanly and safely.
If they do not, document them as the next iteration instead of forcing a poor surface.

## Python API Direction

The API should feel natural for industrial Python users.

Example target shape:

```python
from rust_ethernet_ip import Client

with Client("192.168.0.1") as plc:
    value = plc.read_tag("MyTag")
    plc.write_tag("MyTag", 123)
    values = plc.read_tags(["Tag1", "Tag2", "Program:Main.Tag3"])
```

Design goals:

- clear names
- minimal surprise
- good error messages
- safe cleanup
- pragmatic typing where practical
- thin mapping to Rust semantics

## Phase 5: Add Data and Service Examples

Add practical examples that justify Python support for this repo.

At minimum, aim for examples like:

- `read_single_tag.py`
- `read_batch_tags.py`
- `log_tags_to_csv.py`
- `log_tags_to_sqlite.py`

Optional examples if clean and scoped:

- `pandas_dataframe_example.py`
- `fastapi_service_example.py`

Keep optional dependencies truly optional.

## Phase 6: Establish the Next Platform Pieces

Beyond the Python wrapper MVP, define how this repo can expand into a broader ecosystem.

Plan, and implement only if clearly in scope:

- a data collector service
- an MQTT publisher
- a small REST API service
- Docker examples for local integration stacks

These should be built on top of the existing core, not as parallel data-access implementations.

## Phase 7: Documentation and Positioning

Update the docs so the repo clearly communicates:

- Rust core
- .NET access
- Python access
- fit for MES, analytics, and AI/data workflows

Add or update:

- Python install/build instructions
- local development notes
- wrapper architecture explanation
- platform notes and limitations
- example usage documentation

## Phase 8: Validation

Run and summarize what you can validate:

- Rust checks/tests
- Python tests
- wrapper import tests
- example smoke tests

If something cannot be fully validated, say exactly what remains manual.

## Engineering Constraints

- Do not break the existing Rust public API unless absolutely necessary.
- Do not break the C# wrapper.
- Keep wrappers thin.
- Avoid duplicated protocol logic in Python.
- Prefer explicit, maintainable code over clever code.
- Minimize nonessential dependencies.
- Respect the repo’s existing style and documentation structure.

## Output Expectations

Work in this order:

1. inspect and summarize the real repo
2. choose and justify the Python wrapper strategy
3. create a concrete implementation plan
4. implement incrementally
5. validate what you can
6. document what changed and what remains

End with:

- what was implemented
- how to build and use the Python path
- what remains for the next iteration

Start by inspecting the current repo and proposing the wrapper architecture before making invasive changes.
