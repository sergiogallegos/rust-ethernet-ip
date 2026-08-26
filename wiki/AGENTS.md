# Wiki Maintenance Guide

This file applies to work under `wiki/`. The wiki is a maintainer-oriented
synthesis layer, not a replacement for product documentation or raw evidence.

## Source Layers and Authority

Use three distinct layers:

1. Raw sources: current code and tests, `docs/validation/`, `docs/audit/`,
   `docs/compat/`, official references, issues, and historical analyses.
2. Wiki: concise conclusions that connect those sources.
3. Schema: this file, `wiki/index.md`, and the checks enforced by
   `scripts/validate-wiki`.

When sources disagree, prefer:

1. current code and tests
2. current gates, audits, and validation records
3. official vendor or protocol references
4. historical project analysis
5. chat discussion or implicit assumptions

Flag conflicts and uncertainty instead of flattening them.

## Page Contract

Prefer this structure when it fits:

```markdown
# Title

## Summary

## Current Understanding

## Evidence

## Open Questions

## Related Pages
```

- Use Markdown and relative links for repository-local sources.
- Link authoritative evidence for non-trivial claims.
- Mark conclusions `confirmed`, `likely`, `unclear`, `superseded`, or another
  explicit confidence state.
- Include dates for validation- or release-specific behavior.
- Preserve stable page names and update existing synthesis instead of creating
  near-duplicates.
- Do not copy raw sources verbatim or use the wiki as a notes dump.

## Operations

When ingesting a source:

1. Read the source and affected existing pages.
2. Update or create the smallest useful synthesis.
3. Update `wiki/index.md` for added, renamed, or materially reframed pages.
4. Append an entry to `wiki/log.md`.
5. Run `scripts/validate-wiki`.

For a complex repository question, read `wiki/index.md` first, then the relevant
pages and underlying sources. If the answer creates durable engineering
synthesis, file back only the distilled result.

For a lint operation, check contradictions, superseded claims, weak evidence,
missing links, orphan pages, duplicate concepts, and findings that belong in
user-facing docs or tests instead.

## Index and Log

Each index entry contains a relative page link, a one-line summary, and an
optional status such as `confirmed`, `active`, `historical`, or `needs-review`.

`wiki/log.md` is append-only. Each entry starts with:

```text
## [YYYY-MM-DD] operation | short title
```

Allowed operations are `ingest`, `query`, `lint`, and `reframe`. Record pages
changed, the key outcome, and sources used.

## Boundary

If information is primarily for library users, update `README.md`, `CHANGELOG.md`,
or `docs/`. If it accumulates engineering understanding across sources, keep it
in the wiki. A wiki conclusion that reveals a code, test, or product-doc gap
must call out that gap explicitly.
