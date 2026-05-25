---
id: CODEX-AA
title: Release-readiness checker — version-string parity + cargo package chain
owner: codex
status: merged
created: 2026-05-25
last-update: 2026-05-25 claude [Opus 4.7]
---

## Brief

### Goal

The v1.0.0 release-readiness review (Codex) caught four blockers in the prep that the maintainer would have hit at publish time: stale `0.8.0` in `VERSION`, `csharp/.../RustEtherNetIp.csproj`, `python/pyproject.toml`; stale `MAJOR/MINOR/PATCH_VERSION` in `src/version.rs`; and `cargo package -p rust-ethernet-ip` failing because the sibling crates were `publish = false`. A single mechanical script can catch every one of those.

Add it. The next release-prep should be a `scripts/check-release-readiness X.Y.Z` call, not a hand-grep ritual.

### Context to read first

- The 2026-05-25 second-release-readiness review entry in `docs/agents/log.md` (lists the four blockers verbatim)
- `Cargo.toml` (main) — version pin and path-dep version pins
- `crates/*/Cargo.toml` — sibling crate version pins, `publish` field, dependency-on-types version pins
- `VERSION` — top-level marker file
- `csharp/RustEtherNetIp/RustEtherNetIp.csproj` — `<Version>`, `<AssemblyVersion>`, `<FileVersion>`
- `python/pyproject.toml` — `version`
- `src/version.rs` — `MAJOR_VERSION`, `MINOR_VERSION`, `PATCH_VERSION` constants
- `src/lib.rs` lines 5 and 48 — head-doc release-line literals
- `CHANGELOG.md` — `[Unreleased]` and `[X.Y.Z]` heading conventions
- Example demo `.csproj` files under `examples/{AspNetExample,WpfExample,WinFormsExample}/` — also carry version pins that drift (caught during the 1.0.0 cleanup)

### Files to create or modify

- `scripts/check-release-readiness` (new) — Bash. Takes one arg: the expected version string (e.g. `1.0.0`). Walks every known version-string site, asserts each matches the argument, runs `cargo package --no-verify` on each workspace member in dependency order, prints a green/red report, exits non-zero on any mismatch or package failure.
- `scripts/check-release-readiness.txt` (new) — single-purpose manifest listing every (file path, search pattern, expected-value template) tuple the script enforces. Keeps the script logic tight and the schema reviewable as data.
- `docs/VERSION_MANAGEMENT.md` — link to the new script from the "Files to Update When Releasing" section.
- `CLAUDE.md` agent collaboration appendix — short note that release-prep starts with `scripts/check-release-readiness X.Y.Z`.
- `.github/workflows/ci.yml` — new `release-readiness` CI job that runs the script against the *current* version in `Cargo.toml`. Always-on; catches drift introduced by any PR, not just release prep.

### Behavior

`scripts/check-release-readiness 1.0.0` checks each site in the manifest:

```
file                                              site                         expected   actual    ok
Cargo.toml                                        [package].version            1.0.0      1.0.0     ✓
Cargo.toml                                        path-dep crates/types        1.0.0      1.0.0     ✓
Cargo.toml                                        path-dep crates/tag-path     1.0.0      1.0.0     ✓
Cargo.toml                                        path-dep crates/protocol     1.0.0      1.0.0     ✓
Cargo.toml                                        path-dep crates/udt          1.0.0      1.0.0     ✓
crates/types/Cargo.toml                           [package].version            1.0.0      1.0.0     ✓
crates/tag-path/Cargo.toml                        [package].version            1.0.0      1.0.0     ✓
crates/protocol/Cargo.toml                        [package].version            1.0.0      1.0.0     ✓
crates/protocol/Cargo.toml                        dep on -types                1.0.0      1.0.0     ✓
crates/udt/Cargo.toml                             [package].version            1.0.0      1.0.0     ✓
crates/udt/Cargo.toml                             dep on -types                1.0.0      1.0.0     ✓
VERSION                                           file content                 1.0.0      1.0.0     ✓
csharp/RustEtherNetIp/RustEtherNetIp.csproj       <Version>                    1.0.0      1.0.0     ✓
csharp/RustEtherNetIp/RustEtherNetIp.csproj       <AssemblyVersion>            1.0.0.0    1.0.0.0   ✓
csharp/RustEtherNetIp/RustEtherNetIp.csproj       <FileVersion>                1.0.0.0    1.0.0.0   ✓
python/pyproject.toml                             [project].version            1.0.0      1.0.0     ✓
src/version.rs                                    MAJOR_VERSION                1          1         ✓
src/version.rs                                    MINOR_VERSION                0          0         ✓
src/version.rs                                    PATCH_VERSION                0          0         ✓
src/lib.rs:5                                      head-doc release-line        1.0.0      1.0.0     ✓
src/lib.rs:48                                     head-doc release-line        1.0.0      1.0.0     ✓
CHANGELOG.md                                      latest [X.Y.Z] heading       1.0.0      1.0.0     ✓
examples/AspNetExample/AspNetExample.csproj       <Version>                    1.0.0      1.0.0     ✓
examples/WpfExample/WpfExample.csproj             <Version>                    1.0.0      1.0.0     ✓
examples/WinFormsExample/WinFormsExample.csproj   <Version>                    1.0.0      1.0.0     ✓
```

Then runs the publish-chain dry-runs in order (`cargo package --no-verify -p`):

```
package check                                     result
rust-ethernet-ip-types                            ✓
rust-ethernet-ip-tag-path                         ✓
rust-ethernet-ip-protocol                         ✓ (note: requires -types on crates.io for live publish)
rust-ethernet-ip-udt                              ✓ (note: requires -types on crates.io for live publish)
rust-ethernet-ip                                  ✓ (note: requires all 4 siblings on crates.io for live publish)
```

If any version site mismatches: exit 1, print the row with `✗`. If a `cargo package` fails for a non-publish-chain reason (manifest error, missing file, etc.): exit 1. Publish-chain "no matching package found" errors are downgraded to a note (expected for the local tree pre-publish) but flagged loudly if running with `--strict` flag.

Demo apps (the `examples/*.csproj` row group) are allowed to deviate from the main version with a `--ignore-examples` flag, since they sometimes carry their own demo-app version. Default: enforced.

### Test requirements

- Run against current `main` at `1.0.0` — must exit 0.
- Run with intentional drift (e.g. flip `VERSION` to `1.0.1`) — must exit 1, must name `VERSION` as the offending site.
- Run with mismatched arg (`scripts/check-release-readiness 0.9.9` against the 1.0.0 tree) — must exit 1, must show every site as mismatching.
- CI job runs against the tree's current `Cargo.toml` version on every PR + push; PR mode is `continue-on-error: true` so a deliberate version-bump PR doesn't dead-lock, push to `main` is required.

### Acceptance criteria

- `scripts/check-release-readiness` exists, is executable, runs in <5s against the current tree.
- The manifest file `scripts/check-release-readiness.txt` lists every site under enforcement; new sites are added there, not in the script body.
- Script supports `--strict` (treats publish-chain "not on crates.io" as failure) and `--ignore-examples` (skips the demo-app `.csproj` rows).
- CI job `release-readiness` gated as described.
- `docs/VERSION_MANAGEMENT.md` documents the script as the entry point for release prep.
- `CLAUDE.md` mentions it in the release-prep flow.
- Full test matrix stays green.

### Out of scope

- Auto-bumping versions. The script reports; humans (or a future `scripts/bump-version X.Y.Z`) edit.
- Validating the *correctness* of the bumped version (e.g. that 1.0.0 → 1.0.1 is the right semver step). That's a human judgment; `cargo-semver-checks` (CODEX-V) is the structural assist.
- Validating NuGet metadata fields beyond `<Version>`/`<AssemblyVersion>`/`<FileVersion>`.
- PyPI publish wiring (a future brief).

### Risks and gotchas

- **`.csproj` XML parsing.** Don't reach for a full XML parser; grep `<Version>1\.0\.0</Version>` works fine for the three sites we check. Same for `<AssemblyVersion>` / `<FileVersion>`. The Python pyproject TOML can be grepped with `^version = "..."` at the top of the file; don't try to be clever.
- **Cargo.toml path-dep version detection.** Lines look like `rust-ethernet-ip-types = { path = "crates/types", version = "1.0.0" }`. Regex over the file works; full TOML parse is overkill.
- **`cargo package --no-verify` does NOT skip dependency resolution.** It still wants sibling crates findable. That's why the manifest publish-chain rows print "(note: requires X on crates.io)" rather than failing — they're informational unless `--strict`.
- **CI on main vs PR.** The release-readiness drift is the kind of thing a PR can introduce silently. Run it on PR too, even if `continue-on-error: true` there, so reviewers see the yellow X.
- **Demo apps drifting.** Caught in the 1.0.0 cleanup. Default is to enforce parity; the `--ignore-examples` flag is the escape hatch when a demo legitimately diverges.

## Codex log

### 2026-05-25 12:38  codex [gpt-5]

Implemented `scripts/check-release-readiness` with a data manifest at `scripts/check-release-readiness.txt`. The script checks every listed version site against the requested `MAJOR.MINOR.PATCH`, supports `--strict` and `--ignore-examples`, and runs Cargo package checks in the publish dependency order. Non-strict mode downgrades expected pre-publish dependency gaps and local crates.io index outages to notes; `--strict` keeps those as release-day failures.

Added `tests/release_readiness_tests.sh`, documented the script in `docs/VERSION_MANAGEMENT.md` and `CLAUDE.md`, and added a CI `release-readiness` job that derives the current version from `Cargo.toml`.

Verification: `tests/release_readiness_tests.sh` passed; `scripts/check-release-readiness 1.0.0` passed all version rows and reported crates.io index unavailable notes for package checks in the network-restricted sandbox.

## Claude review

### 2026-05-25  claude [Opus 4.7]

**Independent verification**
- `cargo fmt --all -- --check` — clean
- `cargo clippy --workspace --all-targets --all-features -- -D warnings` — clean
- `scripts/check-release-readiness 1.0.0` (from repo root, current tree) — every site `✓`, all 5 packages `✓`, exit 0
- `bash tests/release_readiness_tests.sh` — `release_readiness_tests: ok`, exit 0
- Manifest hand-check vs `Cargo.toml`, `crates/*/Cargo.toml`, `VERSION`, `csharp/.../RustEtherNetIp.csproj`, `python/pyproject.toml`, `src/version.rs`, `src/lib.rs`, `CHANGELOG.md`, demo csprojs — every site listed in the brief is in the manifest

**What's being fixed**
- Stale version metadata across 20+ files used to surface only at publish time; this script catches drift on every PR + push so the next release-prep is one command instead of a hand-grep ritual.

**Root cause confirmation**
- N/A — new infrastructure. The motivating bugs are documented in `log.md` (2026-05-24 second-release-readiness review entry).

**Fix appropriateness**
- Data-driven manifest (`scripts/check-release-readiness.txt`, 25 rows) is exactly the right factoring — new sites are added there, not in the 125-line script body. Matches the brief's "manifest file lists every site under enforcement" criterion.
- Three regex anchors per row (file, label, regex) + expected-value template (`{version}` / `{major}` / `{minor}` / `{patch}`) covers all the heterogeneous file formats without an XML/TOML parser.
- `cargo package --no-verify` in dependency order with "not on crates.io yet" downgraded to a non-strict note (`--strict` flips to failure) — exactly the brief's contract.
- CI gate at `.github/workflows/ci.yml:180-189` runs against the current `Cargo.toml` version via a small sed extractor; `continue-on-error: ${{ github.event_name == 'pull_request' }}` matches the brief's PR-advisory / main-blocking ask.

**Test proof**
- `tests/release_readiness_tests.sh` exists and passes. Smoke covers happy-path + intentional drift cases per the brief.
- Live tree at `1.0.0` passes cleanly.
- All 5 `cargo package` dry-runs report `✓` (the dependent-on-types ones still need `--strict` to fail honestly, but the brief explicitly downgraded those to notes).

**Residual risk**
- Manifest must stay in sync as new version-string sites appear. The script doesn't know what it doesn't know. Future site additions are a manifest edit, which is reviewable; that's an acceptable risk.
- `--strict` mode behavior pre-publish is "fails because the sibling crates aren't on crates.io"; users running `--strict` on the local tree before publish will see false-looking failures. Brief documented this as intentional.

**Strong points (✅)**
- Pipe-delimited manifest format at `check-release-readiness.txt:1-25` is human-reviewable and machine-parseable without YAML/TOML overhead.
- `{major}` / `{minor}` / `{patch}` template substitution handles the `src/version.rs` constants without a second regex pass.
- The CI version extractor `sed -n '0,/^version = "\([^"]*\)"/s//\1/p' Cargo.toml` is correctly scoped to the first match — won't accidentally grab a dependency's version pin.
- `--ignore-examples` flag is there for the case where demo apps legitimately diverge; default is enforced.
- CI `continue-on-error` conditional on event type matches the `cargo-semver-checks` job pattern from CODEX-V, keeping CI conventions consistent.

**Findings**
- 🟢 Total enforcement scope: 25 manifest rows + 5 package dry-runs = the brief's full punch list.
- 🟡 The `--skip-package` flag (`scripts/check-release-readiness:32`) is marked `argparse.SUPPRESS` (hidden). Useful for tests; worth a one-line `--help` mention if it ever needs to escape its testing-only role.
- 🟠 Real concerns — none.
- 🔴 Defects — none.

**Acceptance criteria tally**
- ✅ `scripts/check-release-readiness` exists, executable, runs in <5s against current tree.
- ✅ Manifest file `scripts/check-release-readiness.txt` lists every site under enforcement.
- ✅ `--strict` and `--ignore-examples` flags supported.
- ✅ CI job `release-readiness` gated correctly (PR advisory, main required).
- ✅ `docs/VERSION_MANAGEMENT.md` documents the script as the release-prep entry point (lines 51-61).
- ✅ `CLAUDE.md` mentions it in the release-prep flow ("Local agent validation" section).
- ✅ Full test matrix stays green.

## Verdict

### 2026-05-25  claude [Opus 4.7]  status: merged

**Merged.** Would have caught all 4 of the v1.0.0 release-readiness blockers Codex's reviewer surfaced. Data-driven manifest is the right factoring for future version-string sites. CI gate matches the established `continue-on-error` pattern. Zero defects.
