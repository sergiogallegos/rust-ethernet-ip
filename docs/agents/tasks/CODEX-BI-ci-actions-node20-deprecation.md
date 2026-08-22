---
id: CODEX-BI
title: Bump CI actions off deprecated Node.js 20 runtime
owner: codex
status: open
created: 2026-08-22
last-update: 2026-08-22 claude [Sonnet 5]
---

## Brief

### Priority and dependency

Non-blocking maintenance task. No dependency on any other open task.

GitHub's runners currently force Node.js 20 actions to run on Node 24 as a
compatibility shim (per the 2026-08-22 CI run
[32595551867](https://github.com/sergiogallegos/rust-ethernet-ip/actions/runs/32595551867)):
every job annotation reads `Node.js 20 is deprecated ... forced to run on
Node.js 24`. The shim keeps jobs passing today, but GitHub can remove it on
its own schedule, at which point every workflow using an affected action
version would start failing with no local-repo code change to explain why.

### Context to read first

- `.github/workflows/ci.yml`
- `.github/workflows/release.yml`
- The linked CI run's annotations (lists every affected action per job).

### Required implementation

1. Identify every `uses: actions/...@vN` reference in both workflow files
   still pinned to a major version that ships a Node 20 runtime. As of this
   writing that includes at least `actions/checkout@v4`,
   `actions/setup-python@v5`, `actions/setup-dotnet@v4`,
   `actions/upload-artifact@v4`, and `actions/download-artifact@v4` — verify
   against the current CI run's annotations and each action's own release
   notes rather than trusting this list blindly, since versions may have
   moved since this brief was written.
2. Bump each to the current major version that runs on Node 24 natively (not
   via the forced-compatibility shim). Check each action's changelog for
   breaking input/output changes before bumping — these are typically
   drop-in but confirm rather than assume.
3. Re-run the full CI matrix (push to a branch or open a draft PR) and
   confirm the deprecation annotations are gone and every job stays green.

### Test requirements

- CI run on the updated workflows shows no Node.js 20 deprecation
  annotations on any job.
- Full existing job matrix (Rust test/clippy/fmt across platforms, C#,
  Python, C++, semver-checks, audit, release-readiness, agent-file
  validation) stays green — this is a pin bump, not a behavior change, so no
  job should newly fail.

### Acceptance criteria

- No `actions/*` reference in `.github/workflows/*.yml` targets a Node 20
  runtime.
- CI is green on the commit that lands this change.
- CHANGELOG entry only if the maintainer wants CI-only changes logged
  there (check existing convention — recent CI-only changes in this repo's
  history to decide); not required for acceptance either way.

### Out of scope

- Non-`actions/*` third-party actions/steps, unless the same CI run's
  annotations flag them too.
- Any workflow *behavior* change beyond the version bumps (job matrix,
  triggers, permissions, secrets).

### Risks / gotchas

- `actions/upload-artifact`/`download-artifact` had a breaking major version
  jump in the past (v3 → v4 changed artifact merge behavior) — read the
  changelog for the version being bumped to, not just the version number.
- Keep the bump mechanical and reviewable: one logical change (action
  version pins), not bundled with unrelated workflow edits.

## Codex log

## Claude review

## Verdict
