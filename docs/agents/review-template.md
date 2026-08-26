# Agent Review Template

Use this template for `## Independent review`. Cite files, symbols, commands, or
artifacts for technical claims, and write `not proven` when evidence is absent.
The reviewer may be a second agent or the primary agent performing a clearly
separated fresh self-review.

```markdown
### YYYY-MM-DD HH:MM  <agent> [<model>] — <independent review|fresh self-review>

**Independent verification**
- `<command>` — pass/fail and one-line result

**What's being changed**
- One-line restatement of the problem or feature.

**Root cause or design confirmation**
- Confirmed/not investigated, with source citations.

**Appropriateness**
- Whether the change lands at the correct layer and preserves boundaries.

**Outcome proof**
- Tests, final-state checks, artifacts, and uncovered edge cases.

**Residual risk**
- Known limitations, unverified claims, hardware gaps, and follow-ups.

**Strong points**
- Evidence-backed choices worth preserving.

**Findings**
- 🟢 factual note
- 🟡 non-blocking improvement
- 🟠 concern that blocks approval until resolved
- 🔴 defect that rejects the submission

**Acceptance criteria tally**
- ✅ Criterion — result
- 🟡 partially Criterion — missing piece
- ❌ Criterion — failed
- (deferred) Criterion — explicit owner and timing
```

Do not approve solely because the primary reports that tests passed. Re-run the
highest-value checks when feasible and inspect the changed implementation plus
its tests. Prefer outcome-based findings over prescribing the exact sequence of
tool calls used to reach the result.
