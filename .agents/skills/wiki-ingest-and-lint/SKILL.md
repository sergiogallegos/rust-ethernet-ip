---
name: wiki-ingest-and-lint
description: Ingest durable engineering findings into the repository wiki or lint wiki structure, links, index coverage, evidence, and freshness. Use for wiki changes and complex repository synthesis; not for ordinary user documentation alone.
---

# Wiki Ingest And Lint

Maintain the wiki as concise, source-traceable synthesis.

## Ingest or Query

1. Read `wiki/index.md` first.
2. Read affected wiki pages, then authoritative underlying files when the wiki
   is insufficient or stale.
3. Follow the source hierarchy and page contract in `wiki/AGENTS.md`.
4. Update existing synthesis when possible; add a page only for a durable,
   distinct reference point.
5. Update `wiki/index.md` for added, renamed, or materially reframed pages.
6. Append a grep-friendly entry to `wiki/log.md` for `ingest`, `query`, `lint`,
   or `reframe` work.

## Validate

Run:

```bash
scripts/validate-wiki
tests/validate_wiki_tests.sh
```

The deterministic validator covers structure. Also use judgment to look for
contradictions, superseded conclusions, weak evidence, duplicate concepts, and
findings that belong in code, tests, or user-facing docs instead.

Do not rewrite broad sections merely to normalize style, and do not turn raw
research notes into authoritative claims without confidence markers and source
links.
