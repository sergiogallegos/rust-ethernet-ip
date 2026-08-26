---
name: code-change-verification
description: Select and run repository verification for code, wrapper, ABI, build, packaging, or behavior changes before handoff. Do not use for documentation-only edits with no executable or release impact.
---

# Code Change Verification

Verify the changed surface with the smallest command set that proves the task's
acceptance criteria, then report exactly what ran and what remains unverified.

## Workflow

1. Inspect `git status --short` and the relevant diff. Preserve unrelated work.
2. Read task-specific acceptance criteria and any matching page under
   `docs/agents/notes/`.
3. Run focused tests while iterating, then the required surface checks below.
4. Treat a skipped hardware test as unverified, never as a pass.
5. Report command, result, environment limitation, and residual risk.

## Required Routing

- Rust source or tests:
  - `cargo fmt --all -- --check`
  - `cargo clippy --workspace --all-targets --all-features -- -D warnings`
  - `SKIP_PLC_TESTS=1 cargo test --workspace --all-features --locked`
  - `cargo test --test plc_sim_tests --locked`
- FFI or C# changes: build the release FFI artifact, run
  `scripts/check-ffi-header-parity.py`, and run the relevant .NET tests.
- Python changes: compile the Python sources and run the relevant unit or
  simulator integration tests documented in `python/README.md`.
- C/C++ changes: run the CMake build and matching `ctest` targets.
- Release metadata or packaging: run `scripts/check-release-readiness` with the
  intended version and its script tests.
- `docs/agents/` changes: run `scripts/validate-agent-files` and
  `tests/validate_agent_files_tests.sh`.
- `wiki/` changes: run `$wiki-ingest-and-lint`.

Use task-specific gates such as `scripts/schema-change-gate` or
`scripts/run-cross-binding-feature-gate.sh` only when their documented surface
matches the change. Live PLC commands require explicit maintainer authorization.

Do not broaden a failing check into unrelated cleanup. Distinguish failures
introduced by the change from environment or pre-existing failures with
evidence.
