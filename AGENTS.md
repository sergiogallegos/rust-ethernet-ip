# AGENTS.md

This file defines how an LLM agent should maintain the repository knowledge wiki in this project.

The goal is not to replace the main project documentation. The goal is to maintain a persistent synthesis layer for engineering knowledge that accumulates over time.

## Purpose

This repository already has strong human-facing documentation:

- `README.md` for product overview and quick start
- `docs/` for manuals, audits, release notes, validation records, and technical references
- `CHANGELOG.md` for release history

The `wiki/` directory is different. It is a maintainer-oriented knowledge base that captures:

- protocol understanding
- controller and firmware-specific behavior
- real-hardware validation synthesis
- historical decisions and tradeoffs
- cross-file and cross-source conclusions
- unresolved questions and investigation trails

The wiki is LLM-maintained. Humans review and use it, but should not have to do routine cross-referencing and bookkeeping manually.

## Three Layers

There are three distinct layers in this repository:

1. Raw sources
   These are the source-of-truth materials the wiki is built from. Examples:
   - files in `docs/validation/`
   - files in `docs/audit/`
   - files in `docs/compat/`
   - official reference material such as PDFs in `docs/`
   - issue writeups, deep-dive notes, test results, and historical analyses

2. Wiki
   This is the synthesized layer in `wiki/`. The wiki may summarize, compare, connect, and flag conflicts across sources. It is allowed to be opinionated about confidence and evidence quality, but it must remain traceable to sources.

3. Schema
   This file is the schema. It tells the agent how to ingest sources, update pages, keep the wiki coherent, and avoid polluting user-facing docs.

## Non-Goals

Do not use the wiki as:

- a replacement for `README.md`
- a dumping ground for raw notes
- a duplicate copy of `docs/`
- a place for speculative claims without evidence markers
- a substitute for tests or validation artifacts

When a change affects users directly, update the authoritative user-facing docs first. The wiki may then record the synthesis behind those docs.

## Directory Layout

The wiki starts small and should grow only as needed.

- `wiki/README.md`
  Short explanation of what the wiki is and how to use it.
- `wiki/index.md`
  Catalog of pages with one-line summaries.
- `wiki/log.md`
  Append-only chronological activity log.

Agents may add subdirectories under `wiki/` when useful, for example:

- `wiki/controllers/`
- `wiki/protocol/`
- `wiki/wrapper-parity/`
- `wiki/limitations/`
- `wiki/investigations/`

Use clear names. Prefer a few stable categories over many shallow directories.

## Source Hierarchy

When sources disagree, prefer the following order unless there is explicit evidence to the contrary:

1. Current code and tests
2. Current release gates, audits, and validation records
3. Official vendor/protocol references
4. Historical project analyses
5. Chat discussion or implicit assumptions

The wiki must explicitly flag conflicts instead of silently flattening them.

## Page Conventions

Each wiki page should be concise and skimmable. Prefer this structure when it fits:

```md
# Title

## Summary

## Current Understanding

## Evidence

## Open Questions

## Related Pages
```

Rules:

- Use Markdown only.
- Prefer short sections and bullets over long prose.
- Link related wiki pages using relative Markdown links.
- Link authoritative source documents whenever making a non-trivial claim.
- Mark uncertainty explicitly with words like `confirmed`, `likely`, `unclear`, or `superseded`.
- Include dates when referring to validation results or release-specific behavior.

## Required Behaviors

When ingesting a new source, the agent should:

1. Read the source and identify which existing wiki pages are affected.
2. Update `wiki/index.md` if pages are added, renamed, or materially reframed.
3. Update or create the relevant wiki pages.
4. Record the operation in `wiki/log.md`.
5. Preserve traceability back to the source material.

When answering a complex question, the agent should:

1. Read `wiki/index.md` first.
2. Read the relevant wiki pages.
3. Read the underlying source files when the wiki does not provide enough confidence.
4. Answer with citations to authoritative files.
5. If the answer produces durable synthesis, file it back into the wiki.

When linting the wiki, the agent should look for:

- contradictions between pages
- stale claims superseded by newer validation
- missing links or orphan pages
- missing pages for frequently referenced concepts
- claims with weak evidence
- places where user-facing docs should be updated instead of the wiki

## Logging Format

`wiki/log.md` should remain append-only. Each entry starts with a stable heading:

```md
## [YYYY-MM-DD] operation | short title
```

Operation types:

- `ingest`
- `query`
- `lint`
- `reframe`

Each entry should list:

- pages added or updated
- key outcome
- source files used

This format should remain grep-friendly.

## Index Format

`wiki/index.md` is content-oriented, not chronological.

Each page entry should include:

- page link
- one-line summary
- optional status such as `confirmed`, `active`, `historical`, or `needs-review`

Keep the index short enough that an agent can read it first on most wiki tasks.

## Editing Discipline

When writing to the wiki:

- do not rewrite large sections without need
- preserve stable page names when possible
- favor updating existing synthesis over creating duplicate pages
- avoid copying source documents verbatim
- keep conclusions aligned with the repository's current released or mainline state

When a wiki finding should change product docs, tests, or code, call that out explicitly instead of silently leaving the insight only in the wiki.

## Suggested First Use Cases

High-value initial pages for this repository include:

- controller and firmware behavior differences
- route-path behavior and ControlLogix backplane notes
- UDT and STRING write limitation synthesis
- Rust vs C# wrapper parity status
- validation evidence by release line

## Practical Rule

If a note is primarily for users, put it in `README.md` or `docs/`.

If a note is primarily for accumulated engineering understanding across many sources, put it in `wiki/`.
