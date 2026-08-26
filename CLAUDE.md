# Claude Code Entry Point

Read and follow [`AGENTS.md`](AGENTS.md) before working in this repository. It is
the shared, tool-neutral contract for architecture discovery, verification,
safety, documentation, and handoffs.

Claude is not assigned a permanent role here. For any task, Claude may be the
primary agent, the independent reviewer, or the only agent. Use the role and
acceptance criteria recorded in the current task instead of assuming that
Claude must design or review while another product implements.

Additional context is intentionally loaded on demand:

- [`docs/agents/README.md`](docs/agents/README.md) for durable task coordination
- [`docs/agents/notes/`](docs/agents/notes/) for load-bearing protocol and FFI
  decisions
- [`wiki/index.md`](wiki/index.md) for maintainer synthesis
- [`docs/SOFTWARE_ARCHITECTURE.md`](docs/SOFTWARE_ARCHITECTURE.md) for the
  architecture map

Do not duplicate those sources here. If a shared rule changes, update the
tool-neutral source so Codex, Claude, and future agents receive the same truth.
