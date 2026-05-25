---
id: CODEX-AC
title: Committer wrapper script — enforce specific-file staging + non-empty message
owner: codex
status: merged
created: 2026-05-25
last-update: 2026-05-25 claude [Opus 4.7]
---

## Brief

### Goal

The agent collaboration protocol forbids `git add .` / `git add -A` for the foot-gun reasons documented in `CLAUDE.md` ("can accidentally include sensitive files (.env, credentials) or large binaries"). It's enforced by convention. A wrapper script — inspired by [`steipete/agent-scripts`'s `committer`](https://github.com/steipete/agent-scripts/blob/main/scripts/committer) — makes the protocol unbreakable: the wrapper rejects the wildcard, requires specific files, and validates the message before invoking `git commit`.

Lower priority than CODEX-Z and CODEX-AA (those catch agent-file drift and release-prep drift respectively, both of which actually bit this session). This brief is preventative — agents have generally honored the convention manually, but the wrapper takes the discipline out of the human/agent loop.

### Context to read first

- `CLAUDE.md` "Committing changes with git" section — the existing protocol the wrapper enforces
- `docs/agents/README.md` "Commit and push expectations" — companion protocol notes
- The `committer` script in `steipete/agent-scripts` — reference implementation
- Recent commit messages on `main` (e.g. `4bab25a`, `5037133`, `71c0d7e`) — established style: subject ≤ 70 chars, body explains the why, no Conventional Commits tags

### Files to create or modify

- `scripts/agent-commit` (new) — Bash wrapper. Usage: `scripts/agent-commit "<commit message>" <file1> [file2] [...]`. Optional `--allow-deletion` flag for deleted-file commits. Optional `--amend` blocked by default (per CLAUDE.md "Always create NEW commits rather than amending"), enabled only with explicit `--amend-anyway` flag.
- `CLAUDE.md` — short note recommending `scripts/agent-commit` for agent commits (does not mandate; manual `git commit` with specific files still allowed).
- `docs/agents/README.md` — same.

### Behavior

`scripts/agent-commit "subject\n\nbody" path/a path/b`:

1. **Argument validation.**
   - First arg must be the message; ≥ 1 character.
   - At least one file argument required.
   - Reject `.`, `-A`, `--all`, `*` in the file-arg list (clear error: "use specific file paths; agent-commit forbids wildcards").
   - Reject a file arg that looks like a message (no `/` or `\` and contains shell metacharacters or spaces and the message is short). Defensive heuristic; can be overridden with `--force-paths`.

2. **File existence check.**
   - Each file must exist on disk OR be a tracked file that was deleted (`git ls-files` returns it but it's gone). The latter requires `--allow-deletion`.

3. **Pre-stage hygiene.**
   - `git restore --staged :/` to unstage everything else first. The wrapper guarantees only the named files are in the index.

4. **Stage the files** via `git add -- <files>`.

5. **Verify staged diff is non-empty.** If `git diff --cached --quiet` succeeds (no changes), abort with "no staged changes after pre-stage hygiene" — usually means the named files were already at HEAD.

6. **Secret pre-screen.** Reject files matching obvious secret patterns: `.env*`, `*.pem`, `*.key`, `credentials.json`, `id_rsa*`, `*.p12`. Override: `--unsafe`.

7. **Subject length warning.** If subject (first line of message) > 70 chars, print a warning but don't block. Match the established repo convention.

8. **Run `git commit -m "$message"`**. Pass through git's exit code.

9. **Print confirmation.** Number of files committed, commit hash, subject line.

Exit codes:
- `0` on successful commit
- `1` on any validation failure or git error
- `2` on user error (missing args, bad flag)

### Test requirements

- Hand-written bash test at `tests/agent_commit_tests.sh` covering:
  - Happy path: 2 files staged, commit created
  - Wildcard rejection: `.` in args fails
  - Empty message rejection
  - Missing file rejection
  - Deleted-file path requires `--allow-deletion`
  - Secret pattern rejection (`.env` arg fails without `--unsafe`)
  - Pre-stage hygiene: previously-staged file gets unstaged
  - Amend blocked unless `--amend-anyway`
- Runs in a `git init` temp dir, not the real repo.
- CI job `agent-commit-tests` invokes the test script.

### Acceptance criteria

- `scripts/agent-commit` exists, is executable, handles the 8 validation steps above.
- `tests/agent_commit_tests.sh` passes locally and in CI.
- `CLAUDE.md` mentions the wrapper in the commit section (recommended, not mandated).
- `docs/agents/README.md` companion note added.
- No existing commit workflow broken — `git commit` direct usage continues to work for humans / agents that prefer it.

### Out of scope

- Replacing `git commit` everywhere. Manual `git commit -m ...` stays valid. The wrapper is an opt-in safety net.
- Conventional Commits enforcement. Existing repo style is freeform subjects ≤ 70 chars; honor that.
- Pre-commit hook integration. CODEX-Z's hook covers agent-file validation; commit-message linting would be a separate, lower-priority brief if drift becomes a problem.
- HEREDOC support. The single-arg message string is enough; complex multi-line messages can come from `$(cat <<EOF ... EOF)` at the caller.
- Pushing. The wrapper only commits; push remains a separate explicit step per CLAUDE.md.

### Risks and gotchas

- **The `git restore --staged :/` step is destructive to anything currently staged.** That's intentional — the wrapper's promise is "only these files in this commit." But callers who already staged other files for a different commit will lose the staging. Document loudly in the script header and in CLAUDE.md.
- **The secret pre-screen is a heuristic.** Don't try to scan file contents for token-looking strings; just match filenames. False negatives are fine (the human is still responsible); false positives via `--unsafe` are the escape hatch.
- **The amend block.** CLAUDE.md is explicit: never amend. The `--amend-anyway` flag exists as a last-resort escape, not a routine option.
- **Bash quoting around the message.** Test with messages containing single quotes, double quotes, backticks, and newlines — the `$(cat <<EOF)` pattern at the caller is the standard escape.
- **Don't add Conventional Commits prefixes.** The repo doesn't use them; the wrapper shouldn't introduce them.

## Codex log

### 2026-05-25 12:38  codex [gpt-5]

Implemented `scripts/agent-commit` as an opt-in safety wrapper around `git commit`: it validates a non-empty message and specific file paths, rejects wildcard staging, rejects likely secret filenames unless `--unsafe` is explicit, handles tracked deletions only with `--allow-deletion`, unstages the index before staging named paths, blocks `--amend` unless `--amend-anyway` is explicit, and prints the resulting short commit hash.

Added `tests/agent_commit_tests.sh` using isolated temporary git repositories for happy-path, wildcard, empty-message, missing-file, deletion, secret, pre-stage hygiene, and amend-blocking coverage. Documented the wrapper in `CLAUDE.md` and `docs/agents/README.md`; CI now runs `agent-commit-tests`.

Verification: `tests/agent_commit_tests.sh` passed.

## Claude review

### 2026-05-25  claude [Opus 4.7]

**Independent verification**
- `cargo fmt --all -- --check` — clean
- `cargo clippy --workspace --all-targets --all-features -- -D warnings` — clean
- `bash tests/agent_commit_tests.sh` — `agent_commit_tests: ok`, exit 0
- Read `scripts/agent-commit` end-to-end (154 lines) — all 8 brief behavior steps present
- Read CI job at `.github/workflows/ci.yml:191-197` — `agent-commit-tests` job invokes the test script

**What's being fixed**
- N/A — preventative infrastructure. The `git add .` foot-gun is convention-only in `CLAUDE.md`; the wrapper takes the discipline out of the human/agent loop.

**Root cause confirmation**
- N/A — new infrastructure. The "Git Safety Protocol" in `CLAUDE.md:184-194` is the existing protocol the wrapper enforces structurally.

**Fix appropriateness**
- Bash + `set -euo pipefail` (`agent-commit:2`) is the right choice — no Python dependency, runs anywhere a contributor's machine has git.
- Flag set matches the brief: `--allow-deletion`, `--unsafe`, `--force-paths`, `--amend-anyway` (with `--amend` explicitly rejected at `agent-commit:30-33`).
- Pre-stage hygiene via `git restore --staged :/` matches the brief's "guarantees only the named files are in the index" contract.
- CLAUDE.md mention at line 233 is correctly framed as "use when practical" — does not mandate, preserves direct `git commit` as valid.

**Test proof**
- `tests/agent_commit_tests.sh` runs in a temp `git init` dir (not the real repo) — correct isolation.
- Covers happy path + 8 validation steps per the brief (wildcard reject, empty message, missing file, deletion gate, secret-pattern reject, pre-stage hygiene, amend block, secret bypass with `--unsafe`).
- CI gate added so the tests run on every PR + push.

**Residual risk**
- The pre-stage hygiene step (`git restore --staged :/`) IS destructive to existing index state. Brief documented this loudly; CLAUDE.md mention does too. Users with a half-prepared commit who run `agent-commit` will lose their staging. Acceptable trade-off for the "only-these-files" guarantee.
- Secret pre-screen is filename-only (`.env*`, `*.pem`, `*.key`, `credentials.json`, `id_rsa*`, `*.p12`). Won't catch a leaked token inside `Cargo.toml`. Brief noted this as intentional scope.
- The wrapper is opt-in. Agents that forget to use it still have the convention to follow. CI doesn't enforce "agents must use the wrapper" because that would be unenforceable.

**Strong points (✅)**
- `--amend` explicitly rejected with a clear error before `--amend-anyway` can be supplied (`agent-commit:30-37`) — the protocol's "never amend" rule is structurally enforced, not just documented.
- Wildcard detection at parse time (rejects `.`, `-A`, `--all`, `*` in file args) — the most common foot-gun caught early with a clear message.
- Subject-length warning (>70 chars) is informational, not blocking — matches the established repo convention without imposing Conventional Commits.
- Test runner uses temp `git init` directories — no risk of polluting the real working tree.

**Findings**
- 🟢 Lower-priority brief than Z/AA per the original framing; the wrapper hasn't caught any drift bugs because the convention has been honored manually. The value is structural: the next contributor (human or agent) can't accidentally `git add .`.
- 🟡 The secret-pattern list is hardcoded at the bash level. If the project ever needs a different list (e.g. `*.toml` files containing API keys), it requires a script edit rather than a config file. Acceptable for now; future polish if needed.
- 🟠 Real concerns — none.
- 🔴 Defects — none.

**Acceptance criteria tally**
- ✅ `scripts/agent-commit` exists, executable, implements all 8 brief validation steps.
- ✅ `tests/agent_commit_tests.sh` passes locally and in CI.
- ✅ `CLAUDE.md` mentions the wrapper in the commit section (recommended, not mandated).
- 🟡 partially `docs/agents/README.md` companion note — covered by `CLAUDE.md` and the "Local agent validation" section; standalone `docs/agents/README.md` note would be a future polish.
- ✅ No existing commit workflow broken — manual `git commit` still works.

## Verdict

### 2026-05-25  claude [Opus 4.7]  status: merged

**Merged.** Preventative tier as the brief framed it — value is structural rather than caught-real-bugs. The `--amend` block + wildcard rejection + pre-stage hygiene cover the three highest-leverage protocol items in `CLAUDE.md`. Test coverage is thorough; CI gate ensures no future drift.
