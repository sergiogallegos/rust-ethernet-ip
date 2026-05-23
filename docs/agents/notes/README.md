# Maintainer notes

Per-surface decision pages. These are intentional load-bearing choices about *this* codebase — the kind of decisions that look removable until a future agent removes them and breaks production.

## What goes here vs. elsewhere

- **`notes/<surface>.md`** (this directory) — durable decisions about a code surface that survive across tasks. Format: "Do X. Reason: Y. How to apply: Z." Not a task lifecycle, not a tutorial.
- **`tasks/<id>.md`** — one-shot work items with a brief, log, review, and verdict. Closes when merged.
- **`board.md`** / **`log.md`** — index of in-flight tasks and append-only event log.
- **`CLAUDE.md`** (repo root) — always-loaded conventions. Cross-surface, not surface-specific. Should *point at* notes pages, not duplicate their content.
- **`docs/`** (repo root) — user-facing documentation. Different audience.

## When to read a notes page

Before reviewing or modifying the surface it covers. Each PR that touches a surface should be checked against the matching note. A reviewer who hasn't read the note can't reliably catch the regressions the note exists to prevent.

## When to add a notes page

When you find yourself writing the same "don't do X, here's why" explanation in a task log or PR review more than once, lift it into a note. A note is cheap to add and costs almost nothing to keep — the cost shows up only when one is missing and someone makes the same mistake again.

## Format

Mirror openclaw's `.agents/maintainer-notes/` style:

1. Title — `# <Surface> Maintainer Decisions`
2. One-sentence lead-in: who reads this and when.
3. Optional verification footer: `Verified against <upstream>, <date>.` Useful when the surface mirrors a third-party contract (CIP spec, .NET P/Invoke, firmware).
4. Sections per sub-topic. Bullets are imperatives. State the rule, then a short "why" clause.

Keep each page under ~80 lines. If it grows past that, split by sub-surface.

## Voice

Same neutral framing as the rest of `docs/agents/` — no first-person, no maintainer profiling. See the Voice section of `docs/agents/README.md`.
