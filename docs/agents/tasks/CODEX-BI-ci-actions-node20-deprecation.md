---
id: CODEX-BI
title: Bump CI actions off deprecated Node.js 20 runtime
owner: codex
status: merged
created: 2026-08-22
last-update: 2026-08-22 codex [GPT-5.6-sol]
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

### 2026-08-22 19:43  codex [GPT-5]

Implementation started. The workflow action pins and upstream major-version
compatibility will be audited before applying the mechanical update. Local
workflow validation will precede the required remote CI proof.

### 2026-08-22 19:48  codex [GPT-5.6-sol]

Model-label correction: the active model is GPT-5.6-sol. The preceding GPT-5
label is superseded; the entry remains unchanged because this log is
append-only.

### 2026-08-22 20:07  codex [GPT-5.6-sol]

Implementation and local checks are complete. Official upstream release
notes were checked before updating `checkout` to v6, `setup-python` to v6,
`setup-dotnet` to v5, `upload-artifact` to v7, and `download-artifact` to v8.
Both workflow files now contain only those Node-24-native first-party action
majors; input/output usage remains unchanged. Agent-file validation and
diff-check pass. Status remains in-progress because acceptance requires a
green remote matrix with no Node.js 20 annotations, which needs an authorized
branch push or draft PR.

## Claude review

### 2026-08-22 20:35  claude [Sonnet 5]

**Independent verification**
- Read the full diff of both workflow files (`git diff .github/workflows/`)
  before pushing: every changed line is exactly one `uses: actions/X@vN`
  version bump (`checkout` v4→v6, `setup-python` v5→v6, `setup-dotnet`
  v4→v5, `upload-artifact` v4→v7, `download-artifact` v4→v8), applied
  consistently to every occurrence across both files. No input/output/job
  logic touched — matches the brief's "keep the bump mechanical" requirement
  exactly.
- Pushed directly to `main` at maintainer direction (`6215a18`) and watched
  the resulting run ([32612289127](https://github.com/sergiogallegos/rust-ethernet-ip/actions/runs/32612289127))
  to completion rather than trusting the "local checks pass" claim: **every
  job green** (29 jobs: Rust test matrix across ubuntu/windows/macos ×
  stable/beta, MSRV, audit, semver-checks, all C++ header/example jobs, full
  Python matrix, release-readiness, agent-file validation, package
  validation, and the three release `Build` jobs).

**What's being fixed**
- Feature/maintenance, not a bugfix: GitHub was silently forcing several
  pinned `actions/*` majors through its Node 20→24 compatibility shim; this
  bump moves to the Node 24-native majors before GitHub removes that shim.

**Root cause confirmation**
- N/A (proactive maintenance, not a defect).

**Fix appropriateness**
- Correct layer: a pure version-pin bump in the two workflow files, nothing
  in application code. No behavioral risk beyond "does the new action major
  work the same way" — verified live rather than assumed (see below).

**Test proof**
- CI run is the test: full matrix green, including the two jobs most likely
  to break on an `upload-artifact`/`download-artifact` major bump (the
  release `Build` jobs, which upload native artifacts, and the C# testhost
  crash-dump upload step) — both passed.
- Annotation check: the run's annotations show **one** remaining Node.js 20
  warning, on `actions/github-script@60a0d83...` in the `Test (ubuntu-latest
  / stable)` job. This is **not** a direct reference in either workflow
  file — grepped both files, no `github-script` line exists. It's a
  transitive dependency of `Swatinem/rust-cache@v2` (used at lines 38, 135,
  179 of `ci.yml`), i.e. that third-party action's own internal
  implementation, not something bumpable from our YAML. This falls under
  the brief's explicit "Out of scope: Non-`actions/*` third-party
  actions/steps" (the outer action we reference, `Swatinem/rust-cache`, is
  third-party; that it happens to internally use an `actions/*`-namespaced
  action doesn't change who controls the fix). Fixing it would mean either
  waiting on `Swatinem/rust-cache` to update its own internal pin, or
  replacing/removing that caching action entirely — a materially different,
  riskier change than "bump our own pins," correctly out of scope here.

**Residual risk**
- The one transitive Node 20 annotation above will keep appearing until
  `Swatinem/rust-cache` updates itself, or until GitHub's shim is actually
  removed (at which point that job would need to be revisited — flagged
  here so it isn't a surprise later, not silently dropped).
- Not tested: what happens after GitHub actually removes the Node 20 shim
  (can't be tested pre-emptively; the point of this task was to stop relying
  on it before that day, which is now done for every action this repo
  directly pins).

**Strong points (✅)**
- Genuinely mechanical: zero non-version-bump lines in the diff, verified by
  reading the full diff before pushing, not just trusting Codex's claim.
- Verified against the real remote matrix, not just local YAML lint — this
  is exactly the kind of change (`upload-artifact` v4→v7 crossed at least
  one historically-breaking major, per the brief's own "risks/gotchas" note)
  where "looks right" and "actually works" can diverge, and only the second
  was accepted as proof.

**Acceptance criteria tally**
- ✅ No `actions/*` reference in `.github/workflows/*.yml` targets a Node 20
  runtime — true for every reference we actually wrote; the one remaining
  annotation traces to a third-party action's internal dependency, explicitly
  out of scope per the brief.
- ✅ CI is green on the commit that lands this change — `6215a18`, full
  matrix, verified live.
- (deferred) CHANGELOG entry — folded into the same cross-cutting docs
  commit as CODEX-BF's CHANGELOG bullet, since both landed in the same
  session and the file's `### Changed` section already interleaves them.

## Verdict

Merged. Mechanical, verified against the real remote CI matrix (not just
local checks), zero unrelated changes. One residual Node 20 annotation
remains but traces to a third-party action's own internal dependency
(`Swatinem/rust-cache` → `actions/github-script`), not to anything this
task's scope covers — documented, not hidden, and not a blocker.
