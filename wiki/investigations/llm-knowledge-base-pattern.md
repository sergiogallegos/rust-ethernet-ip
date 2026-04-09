# LLM Knowledge-Base Pattern

## Summary

This repo's wiki is a narrow, maintainable version of the broader "LLM knowledge base" pattern: raw sources remain in `docs/` and code, while the wiki acts as an LLM-maintained compiled synthesis layer. The useful part of the pattern is explicit file-backed memory with traceability; the risky part is letting the synthesized layer drift away from source authority.

## Current Understanding

- `confirmed`: The repository already follows the core pattern in lightweight form:
  - raw sources live in code, tests, and `docs/`
  - compiled synthesis lives in `wiki/`
  - update discipline is defined in [`AGENTS.md`](../../AGENTS.md)
- `confirmed`: The strongest properties of the pattern are explicitness, local ownership, file interoperability, and model portability.
- `confirmed`: For this repo, the wiki should remain a maintainer-oriented synthesis layer, not a general-purpose second documentation set and not a dumping ground for research notes.
- `likely`: At the current repo scale, disciplined index pages, concise summaries, and direct source links are more important than adding a full retrieval stack.
- `likely`: If a repo-doc assistant is added later, the wiki should be one source in that system, not the only source. The retrieval layer should still prefer code/tests and current validation records over older synthesis pages.

## Why The Pattern Helps Here

- It makes accumulated engineering understanding explicit and inspectable.
- It keeps repo knowledge in normal files that can be diffed, reviewed, searched, and moved across tools.
- It lets multiple agents or models operate over the same knowledge layer without vendor lock-in.
- It creates a place to preserve cross-source conclusions that do not belong in user-facing docs.

## Failure Modes To Guard Against

- `needs-care`: duplicate pages that summarize the same topic differently
- `needs-care`: stale wiki conclusions after code, validation, or release docs change
- `needs-care`: weakly sourced claims that sound authoritative because they are written clearly
- `needs-care`: letting conceptual notes about AI workflows dilute the repo's EtherNet/IP engineering focus

## Repo-Specific Guidance

- Keep the source hierarchy from [`AGENTS.md`](../../AGENTS.md) intact: code/tests first, then current validation and audits, then vendor references, then historical analysis.
- Prefer updating an existing wiki page over creating a new conceptual page unless the concept changes maintenance behavior.
- When a query produces durable synthesis, file back only the distilled result, not the full investigation transcript.
- If retrieval tooling is added later, rank by source authority and recency before semantic similarity.

## Evidence

- [`AGENTS.md`](../../AGENTS.md)
- [`README.md`](../../README.md)
- [`../README.md`](../README.md)
- User-provided excerpts on 2026-04-09 describing:
  - explicit file-backed "wiki LLM" memory
  - raw-source plus compiled-wiki workflow
  - Obsidian-based viewing and artifact generation
  - linting and incremental wiki maintenance

## Open Questions

- Whether this repo eventually needs a dedicated doc-retrieval tool for wiki maintenance once `docs/` and `wiki/` grow materially larger.
- Whether a future investigation page should specify repo-local retrieval heuristics such as metadata fields, chunk boundaries, and reranking rules.

## Related Pages

- [../index.md](../index.md)
- [../releases/0.7.0-validation-synthesis.md](../releases/0.7.0-validation-synthesis.md)
- [../controllers/firmware-behavior.md](../controllers/firmware-behavior.md)
- [../limitations/string-and-udt-write-behavior.md](../limitations/string-and-udt-write-behavior.md)
