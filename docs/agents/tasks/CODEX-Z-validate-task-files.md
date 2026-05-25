---
id: CODEX-Z
title: Validate agent task file frontmatter + board/log consistency on pre-commit
owner: codex
status: open
created: 2026-05-25
last-update: 2026-05-25 claude [Opus 4.7]
---

## Brief

### Goal

The agent-collaboration protocol in `CLAUDE.md` and `docs/agents/README.md` is enforced by convention only. Several drift bugs this session (status not bumped after merge, `publish = false` reference lingering after manifest change, stale Done table rows) would have been caught immediately by a mechanical validator. Add one.

Inspired by the `validate-skills` pattern in [`steipete/agent-scripts`](https://github.com/steipete/agent-scripts) — a single script that walks the agent files, enforces the schema, and fails the commit if anything's wrong.

### Context to read first

- `docs/agents/README.md` — full collaboration protocol
- `docs/agents/tasks/CODEX-X-bool-array-rmw-dword-offset.md` and `docs/agents/tasks/CODEX-Y-nested-bool-udt-array-element.md` — current canonical task-file shape
- `docs/agents/board.md` — Open table + Done table conventions, plus the merge-commit-ref convention
- `docs/agents/log.md` — append-only event log format (`YYYY-MM-DD HH:MM  <author>  [<model>]  <task-id-or-->  <event>`)
- `CLAUDE.md` agent collaboration appendix — the source of truth for the conventions

### Files to create or modify

- `scripts/validate-agent-files` (new) — Bash script (Ruby acceptable if cleaner for YAML parsing; pick one and justify in `## Codex log`). Walks `docs/agents/tasks/CODEX-*.md`, parses YAML frontmatter, runs cross-checks, prints failures to stderr, exits non-zero on failure.
- `.githooks/pre-commit` (new) — Shell script that runs `scripts/validate-agent-files` when any file under `docs/agents/` is staged. Skips the check when no agent files are touched.
- `scripts/install-hooks` (new) — One-line helper that runs `git config core.hooksPath .githooks` so contributors can opt into the hook without symlink fiddling. Document in `CLAUDE.md` and `docs/agents/README.md`.
- `docs/agents/README.md` — short new section explaining how to enable the hook and what it checks.
- `.github/workflows/ci.yml` — add a `validate-agent-files` job that runs the same script on PR + push. CI is the backstop for contributors who haven't installed the hook locally.

### Behavior

For each `docs/agents/tasks/CODEX-*.md`:

1. **Frontmatter shape.** Must be a valid YAML block between `---` delimiters at the start of the file. Required keys (all non-empty strings unless noted): `id`, `title`, `owner`, `status`, `created`, `last-update`. Extra keys allowed.

2. **`id` must match filename.** `id: CODEX-X` requires filename `CODEX-X-*.md`.

3. **`status` is enum.** Must be one of: `open`, `in-progress`, `submitted`, `under-review`, `merged`, `rejected`.

4. **`created` and `last-update` date format.** ISO `YYYY-MM-DD`. `last-update` line additionally must include the author + model tag in the format `YYYY-MM-DD <author> [<model>]` per the convention (e.g. `2026-05-25 claude [Opus 4.7]`).

5. **Section presence.** Every task file must contain top-level `## Brief`, `## Codex log`, `## Claude review`, and `## Verdict` sections in that order. Missing or out-of-order sections fail.

Cross-checks against `docs/agents/board.md`:

6. **Every task file with `status` in {open, in-progress, submitted, under-review} must have a row in the Open table** whose `Id` column matches the task's `id`.
7. **Every task file with `status: merged` must have a row in the Done table** with the matching `id`. The merge-commit column must be a 7+ hex char string or the literal `_(merge commit pending)_` (which gets a warning, not a failure — allows the backfill commit pattern).
8. **Conversely**, every Open table row's `Id` must point to an existing task file whose `status` is not `merged`/`rejected`. Every Done table row's `Id` must point to a task file with `status: merged`.

Cross-check against `docs/agents/log.md`:

9. **Log lines must parse** as `YYYY-MM-DD  <author>  [<model>]?  <task-id-or-->  <event-text>` (the model tag is optional only for lines dated before the 2026-05-17 convention introduction; everything from that date on requires it). Lines that don't parse are flagged.

Error format (stderr):

```
docs/agents/tasks/CODEX-Q-service-layer.md:5: status "submited" is not one of: open, in-progress, submitted, under-review, merged, rejected
docs/agents/board.md: CODEX-Q has status "merged" in task file but does not appear in Done table
docs/agents/log.md:142: line does not parse (expected YYYY-MM-DD format in column 1)
```

Exit codes: `0` if everything passes, `1` if any failure. Warnings (e.g. `_(merge commit pending)_`) print to stderr but don't change the exit code.

### Test requirements

- `scripts/validate-agent-files` must succeed on the current `main` tree (it represents the known-good state; failing on it means the validator's schema is wrong).
- Add a smoke test fixture under `tests/agent_files_fixtures/`:
  - `valid_task.md` — minimal compliant task file; validator passes
  - `missing_status.md` — frontmatter without `status`; validator exits 1 with a specific message
  - `wrong_filename.md` — `id: CODEX-Q` in a file named `CODEX-WRONG-...md`; validator exits 1
  - `out_of_order_sections.md` — `## Verdict` before `## Claude review`; validator exits 1
  - `bad_log_line.md` (board-level) — see how cross-checks fail
- A 5-line shell test that runs the validator against each fixture and asserts the expected exit code + greppable error fragment.
- CI job runs the validator on the actual repo state on every PR and push to `main`.

### Acceptance criteria

- `scripts/validate-agent-files` exists, is executable, and passes against `main` at the time of merge.
- `.githooks/pre-commit` runs the validator only when files under `docs/agents/` are staged.
- `scripts/install-hooks` is a single-line wrapper around `git config core.hooksPath .githooks`.
- `docs/agents/README.md` has a "Local validation" section explaining the hook + how to opt in.
- CI gates on the validator (job name `validate-agent-files`).
- Smoke-test fixtures and the shell test runner are in place.
- `cargo fmt -- --check`, `cargo clippy -- -D warnings`, full test matrix all stay green.

### Out of scope

- Validating wiki, CHANGELOG, or any non-agent docs. CODEX-AA covers release-readiness; this brief is just the agent collaboration surface.
- Enforcing prose conventions (no first-person, no maintainer profiling, etc.) — those are stylistic and don't lend to mechanical validation without false positives.
- Auto-fixing failures. The validator reports; humans fix.
- Replacing the existing CI matrix. This is a new job, parallel to the existing ones.

### Risks and gotchas

- **YAML parsing in Bash is painful.** If pure Bash gets ugly past ~50 lines, switch to Ruby (already in the macOS / Linux base image) or Python. Justify the choice in the Codex log.
- **The hook fires only when contributors opt in.** That's intentional — the CI job is the actual gate. Local hook is a fast-feedback bonus.
- **Append-only log convention.** The validator must accept the pre-2026-05-17 entries that don't have the model tag; only require it on entries dated 2026-05-17 or later. Don't lint pre-existing log lines into compliance.
- **Cross-check 6/7 (board ↔ task file consistency) is the load-bearing one.** Most of the drift bugs this session were here. If anything, prefer slightly noisier output on this check than silent passes.

## Codex log

_(append work entries here)_

## Claude review

_(append review entries here)_

## Verdict

_(final disposition)_
