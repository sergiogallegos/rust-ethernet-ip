---
id: CODEX-AC
title: Committer wrapper script — enforce specific-file staging + non-empty message
owner: codex
status: open
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

_(append work entries here)_

## Claude review

_(append review entries here)_

## Verdict

_(final disposition)_
