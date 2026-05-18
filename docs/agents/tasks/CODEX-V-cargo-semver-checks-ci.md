---
id: CODEX-V
title: Add cargo-semver-checks to CI as the SemVer gate
owner: codex
status: open
created: 2026-05-18
last-update: 2026-05-18 claude [Opus 4.7]
---

## Brief

### Goal

Add `cargo-semver-checks` as a CI job that diffs the working-tree's public API against the latest crates.io release of `rust-ethernet-ip` and fails when an undeclared breaking change is introduced. This is defensive infrastructure for the v0.8.0 cut: with CODEX-K's release-window bundle, CODEX-N's possibly-new error variant, CODEX-P's behavioral actor refactor, and the `#[non_exhaustive]` sweep all landing in the same version, the chance of a SemVer-meaningful slip is high. `cargo-semver-checks` is the standard tool that catches them mechanically.

Driven by the architecture review at [`wiki/investigations/architecture-review-2026-05-18.md`](../../../wiki/investigations/architecture-review-2026-05-18.md). Small, contained, parallel-safe with every other brief.

### Context to read first

- `.github/workflows/ci.yml` end-to-end — current job layout, where the new job fits.
- The `cargo-semver-checks` docs at [`https://github.com/obi1kenobi/cargo-semver-checks`](https://github.com/obi1kenobi/cargo-semver-checks) — invocation flags, output format, baseline-fetching options.
- `Cargo.toml:7-9` — current version (`0.8.0`); the baseline is the most recent crates.io release (`0.7.0`).
- `CHANGELOG.md` — for the existing release cadence.

### Files to create or modify

- `.github/workflows/ci.yml` — add a new `semver-checks` job (ubuntu-latest, stable Rust).
- `docs/agents/board.md` — add CODEX-V to the Open table (this brief's own creation, in the same commit).

### Behavior

The job runs:

```bash
cargo install cargo-semver-checks --locked    # via taiki-e/install-action@v2 for speed
cargo semver-checks check-release --baseline-version 0.7.0
```

(or, after v0.8.0 ships, `--baseline-version 0.8.0`.)

Output:
- **On PR**: informational; failure does not block merge (set `continue-on-error: true`) so a deliberate SemVer-major change is not a CI deadlock.
- **On push to `main`**: required; failure blocks the run. Forces the maintainer to bump `Cargo.toml` version explicitly before pushing a breaking change.

The job should publish its findings as a GitHub annotation so the reviewer sees the offending public-item path in the PR UI.

### Test requirements

- The job runs successfully against the current `main` (since v0.7.0 → v0.8.0-draft is SemVer-minor for the most part) — verifies the harness works.
- Manually trigger a deliberate SemVer-major change in a throwaway branch (e.g. rename one public function) and confirm the job fails with a clear message identifying the renamed symbol.
- The existing CI matrix (`test`, `python`, `msrv`, `audit`, `package`, `version-check`, `build`) is unaffected.

### Acceptance criteria

- New job appears as a check on every PR and every push to `main`.
- Job uses `taiki-e/install-action@v2` for the `cargo-semver-checks` binary (prebuilt binary, no source compile).
- PR mode is `continue-on-error: true`; main-branch mode is required.
- A test-only branch demonstrating a deliberate API break is referenced in the `## Codex log` with the link to the failing run.
- `build` job's `needs:` array is updated to include `semver-checks` (so artifacts are only produced when the SemVer story is intact on main).
- The agent collaboration commit message follows the project voice convention (no agent attribution).

### Out of scope

- Configuring `cargo-semver-checks` advisories or per-item ignore lists — defer until the first false positive surfaces.
- Auto-generating the next version number from the detected diff. Manual bump is fine.
- Replacing the existing `version-check` job. The two are complementary: `version-check` verifies manifest version consistency; `semver-checks` verifies that the version bump matches the actual API diff.

### Risks and gotchas

- `cargo-semver-checks` requires building both the baseline crate from crates.io *and* the working-tree crate. On a cold cache this can be slow (~5 minutes); with `Swatinem/rust-cache@v2 { shared-key: semver }`, subsequent runs are fast.
- The baseline pin must be updated on every release tag. Either:
  - Use `--baseline-rev <previous-release-tag>` (preferred — auto-tracks via git) so the workflow doesn't need a `Cargo.toml` baseline bump every release.
  - Or accept the small maintenance cost of bumping the `--baseline-version` string in the workflow file at release time.
  Codex picks one; justify in the log.
- The tool currently does not detect every kind of behavioral change (e.g., changing the value returned by a `const fn` is not caught). It catches *signature* SemVer breaks reliably, which is the load-bearing 90%.
- A `continue-on-error: true` step still appears in the GitHub UI as a yellow/red annotation, which is what we want — visible but non-blocking on PRs.

## Codex log

_(append work entries here)_

## Claude review

_(append review entries here)_

## Verdict

_(final disposition)_
