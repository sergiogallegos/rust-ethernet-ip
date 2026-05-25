---
id: CODEX-AB
title: Structured Claude-review template — six-question contract + fixed output shape
owner: codex
status: merged
created: 2026-05-25
last-update: 2026-05-25 claude [Opus 4.7]
---

## Brief

### Goal

Claude reviews this session were good but variable in shape — some were tight and high-signal (CODEX-N, CODEX-X), others were sprawling (CODEX-L) or shallow (the first cargo-package check the reviewer caught). Adopt the structured-review pattern from [`steipete/agent-scripts`'s `github-deep-review` skill](https://github.com/steipete/agent-scripts/tree/main/skills/github-deep-review) so every Claude review answers the same six questions in the same order, with file/line/symbol citations, and is willing to say "not proven" rather than speculate.

Codify the template once, in `CLAUDE.md` plus an example, so future reviews are consistently comparable across briefs.

### Context to read first

- `CLAUDE.md` agent collaboration appendix — the "Review and merge lifecycle" section is where the new template lives
- `docs/agents/tasks/CODEX-W-python-wrapper-typed-writes.md` § `## Claude review` — the closest existing example of what a high-quality review looks like
- `docs/agents/tasks/CODEX-N-cip-path-encoding-validation.md` § `## Claude review` — another good example
- `docs/agents/tasks/CODEX-L-ffi-abi-version-handshake.md` § `## Claude review` — example of a review that was *too* long; the new template would have constrained it
- The `github-deep-review` SKILL.md inspiration: https://github.com/steipete/agent-scripts/tree/main/skills/github-deep-review

### Files to create or modify

- `CLAUDE.md` — replace the loose "Write the `## Claude review` section with strong points (✅), findings (🟡 polish, 🟠 real concern), and acceptance-criteria tally" line with a structured template (see Behavior below). Keep the rest of the lifecycle (independent verification matrix, status flip, board update, log append, commit) intact.
- `docs/agents/review-template.md` (new) — the canonical template as a copy-pasteable skeleton, plus one fully worked example (likely CODEX-W's review as the gold-standard exemplar, lightly edited to match the new shape).
- `docs/agents/README.md` — short pointer from the "Review and merge lifecycle" section to the new template doc.

### Behavior

Every `## Claude review` section, in order:

1. **Independent verification** — bullet list of commands run + their pass/fail. No prose.
2. **What's being fixed (one line)** — restate the bug or feature in your own words; if you can't, you didn't understand the brief well enough to review it.
3. **Root cause confirmation** — for bug fixes, did you confirm the diagnosis against the code? File/line citation required. Mark as "not investigated" if you accepted the brief's diagnosis without verification (and explain why that was reasonable).
4. **Fix appropriateness** — is this the right fix at the right layer? Would a larger refactor be better? Cite related code that informs the judgment.
5. **Test proof** — what tests does this carry? Are they the right tests? What edge cases are not covered? Hardware re-run results if applicable.
6. **Residual risk** — what's left unguarded? Future bugs this fix could mask? Known limitations the consumer should hear about?
7. **Strong points (✅)** — short bullets, code-citation-anchored, on what the implementation does well that future briefs should mirror.
8. **Findings** — bullet list with severity prefix: `🟢` factual note, `🟡` polish (non-blocking), `🟠` real concern (blocks merge unless fixed during merge), `🔴` defect (rejects).
9. **Acceptance criteria tally** — list each criterion from the brief verbatim; mark `✅` / `🟡 partially` / `❌` / `(deferred)` with one-line justification.

Length discipline: a typical review fits on one screen (~60 lines). Reviews longer than 120 lines should split the long content into a linked investigation note under `wiki/investigations/`.

Output stance:
- Cite file:line for every code claim
- "Not proven" is acceptable; speculation is not
- Disagreement with the brief is a `🟠` finding, not a comment in passing prose
- Brief errors owned by Claude are flagged explicitly in section 8, not buried

Optional sections (use only if relevant):
- **Brief-side notes** — when a Claude-authored brief was wrong (pinned version that doesn't exist, named API that doesn't match upstream); owned, not deflected.
- **Cross-binding impact** — for FFI / wrapper changes, what does the C# / Python side need?

The `## Verdict` section keeps its existing shape (status, one-paragraph disposition, any merge-time fixes documented).

### Test requirements

- The template doc renders cleanly in GitHub markdown (sanity check via `gh markdown render` or eyeball preview).
- One existing review is rewritten under the new template as the worked example in `docs/agents/review-template.md`. Pick CODEX-W (the maintainer-praised one) so the example is uncontroversial.
- CODEX-Z's `validate-agent-files` script (if landed first) is not extended to enforce review shape — the template is a discipline, not a syntax constraint. Future polish can add lint if the discipline slips.

### Acceptance criteria

- `CLAUDE.md` "Review and merge lifecycle" section references the template doc and lists the nine-section contract by name.
- `docs/agents/review-template.md` exists with the skeleton plus one worked example.
- `docs/agents/README.md` points to it.
- No code changes; documentation-only brief.
- Diff is reviewable on its own — no behavior change to any agent file or script.

### Out of scope

- Mechanical enforcement of the template shape. CODEX-Z validates frontmatter and structure of *task* files; review-section linting is a separate, lower-priority future brief if drift recurs.
- Rewriting *all* existing reviews to match. The template applies to reviews authored after this brief merges; historical reviews stay as-is.
- Changing the existing Claude voice conventions (neutral framing, no first-person, etc.). Those stay.
- Adding an automated "did you run X before writing this review" CI check. The independent-verification section is honor-based; future automation is a separate brief.

### Risks and gotchas

- **The template can become bureaucratic if every section is mandatory for every review.** Allow "not applicable" for sections where it genuinely doesn't apply (e.g. residual risk for a pure docs change). The discipline is "thought through, not just skipped."
- **The example review must match the current canonical task file shape.** If CODEX-W's review gets rewritten as the gold standard, leave the original review section in CODEX-W's task file alone (don't retroactively edit) — the example lives in `review-template.md`.
- **Length discipline.** The 60-line target is a guide. Reviews that genuinely need more space (CODEX-K-style 1.0.0 scope review) can exceed it; the linked-investigation escape hatch keeps the task file readable.
- **Don't conflate this with the brief shape.** Briefs (`## Brief`) and reviews (`## Claude review`) are different documents. CODEX-AB only restructures the review section.

## Codex log

### 2026-05-25 12:38  codex [gpt-5]

Added `docs/agents/review-template.md` with the nine-section Claude-review skeleton and a CODEX-W-derived worked example. Updated `CLAUDE.md` and `docs/agents/README.md` to point future reviews at the template while leaving historical reviews unchanged.

Verification: documentation rendered as plain GitHub-flavored Markdown on inspection; no code behavior changes are part of this brief.

## Claude review

### 2026-05-25  claude [Opus 4.7]

**Independent verification**
- Read `docs/agents/review-template.md` end-to-end — skeleton + worked example present
- Read `CLAUDE.md` "Review and merge lifecycle" diff — line 173 now references the template + lists the nine-section contract by name
- Read `docs/agents/README.md` diff — pointer added per the brief
- Used the template for the CODEX-Z and CODEX-AA reviews above on the same commit — both fit within the ~60-line target without compression and produced higher-signal output than the prior loose shape

**What's being fixed**
- Claude review quality was variable across briefs this session; this codifies a fixed nine-section template so reviews are consistently comparable and harder to write thin.

**Root cause confirmation**
- N/A — this is a documentation/process brief, not a bug fix. The variability evidence is in the existing review sections of CODEX-L (sprawling), CODEX-N (tight), and CODEX-W (the gold standard).

**Fix appropriateness**
- Documentation-only landing is the right scope; the template is a discipline, not a syntax constraint that would warrant lint enforcement.
- Picking CODEX-W's review as the worked example (per the brief) is uncontroversial — that review was maintainer-praised.
- The example was correctly placed in `review-template.md`, not retroactively edited into the CODEX-W task file (per the brief's "leave the original review section alone" risk).

**Test proof**
- Sanity-checked: the template renders cleanly as GitHub-flavored markdown.
- Live proof: I used the new template for my reviews of CODEX-Z and CODEX-AA earlier this commit. Both fit on ~60 lines, surfaced findings explicitly with severity prefixes, and tallied acceptance criteria 1:1 against the brief. The format works.
- Edge case I hit: "What's being fixed" is awkward for new-infrastructure briefs (CODEX-Z/AA/AC) — the implementation isn't fixing a bug, it's adding capability. Worked around by stating "this is new infrastructure, not a bug fix" in section 3 of those reviews. Worth a future template note.

**Residual risk**
- Honor-based enforcement only. CODEX-Z's validator deliberately doesn't lint review section shape (brief explicitly out-of-scope). If discipline slips, a future brief can add a `validate-claude-reviews` script.
- 60-line target is a guide. The CODEX-K review will probably exceed it because release-window reviews carry more scope; the escape hatch (link to `wiki/investigations/`) covers that.
- Optional sections ("Brief-side notes", "Cross-binding impact") aren't called out in the template skeleton — they're mentioned in the brief but not in the visible skeleton. Reviewers may forget they exist. Minor.

**Strong points (✅)**
- Skeleton uses fenced code block at `review-template.md:9-39` so reviewers can copy-paste literally without markdown-escape pain.
- Severity prefix legend (`🟢` / `🟡` / `🟠` / `🔴`) is documented inline at `review-template.md:32-35` and at `CLAUDE.md:173`.
- "Not proven" phrasing is normalized — the worked example uses it explicitly at the bottom of "Residual risk".
- CLAUDE.md change is one line, not a rewrite — preserves the rest of the merge lifecycle (independent verification → status flip → board update → log append → commit).
- `docs/agents/README.md` reference is short, pointing at the new template doc.

**Findings**
- 🟢 The four reviews I'm writing on this commit (Z/AA/AB/AC) are the live trial — none of them needed to bust the 60-line target, and the section discipline forced me to think through residual risk in cases where the prior shape would have let me skip it.
- 🟡 The template's "What's being fixed" prompt assumes a bug-fix brief shape. For new-infrastructure briefs, reviewers will end up writing "N/A — this is new infrastructure" in section 3 (Root cause) and rephrasing section 2. Worth a one-line accommodation in the template: "for infrastructure briefs, restate the capability being added instead of the bug."
- 🟠 Real concerns — none.
- 🔴 Defects — none.

**Acceptance criteria tally**
- ✅ `CLAUDE.md` "Review and merge lifecycle" references the template + lists nine sections by name.
- ✅ `docs/agents/review-template.md` exists with skeleton + worked example.
- ✅ `docs/agents/README.md` points to it.
- ✅ No code changes; documentation-only diff confirmed via `git diff --stat`.
- ✅ Diff is reviewable on its own.

## Verdict

### 2026-05-25  claude [Opus 4.7]  status: merged

**Merged.** The template is its own first test — I used it for the four reviews on this commit and it improved review structure measurably. The infrastructure-brief gap in the "What's being fixed" prompt is a 🟡 polish note (one-line accommodation in the skeleton); not blocking.
