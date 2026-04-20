# Wiki README

This directory is an LLM-maintained engineering knowledge base for `rust-ethernet-ip`.

It is not the main documentation set. The authoritative user-facing docs remain:

- `README.md`
- `docs/`
- `CHANGELOG.md`

Use the wiki for synthesized understanding that spans multiple sources, for example:

- controller and firmware quirks
- protocol behavior summarized across Rockwell and project references
- release validation takeaways
- wrapper parity gaps
- ongoing investigations and open questions

Start with:

- [index.md](index.md)
- [log.md](log.md)

Maintenance rules are defined in `../AGENTS.md`.

## Prompt Recipes

Use short operational prompts. The most useful action words are:

- `ingest`
- `query`
- `lint`

Preferred prompt patterns:

- `Ingest this new validation note into the wiki and update any affected pages: <path>`
- `Read these changed files and update the wiki if the synthesis changed: <paths>`
- `Lint the wiki against the latest docs and tell me what is stale, contradictory, or missing.`
- `Answer this from the wiki first, then update the wiki if the answer creates durable synthesis: <question>`

Examples:

- `Ingest docs/validation/2026-04-15_real_plc_xxx.md into the wiki. Update index and log too.`
- `We changed batch error handling. Check src/ffi.rs, csharp/RustEtherNetIp/EthernetNetIpClient.cs, and docs/README.md, then update wrapper parity if needed.`
- `Lint the wiki after the 0.8.0 docs changes. Focus on release validation, limitations, and wrapper parity.`
- `Answer this from the wiki first: what is the current supported pattern for writing STRING-related values? If the wiki is missing something, update it.`

Reusable default prompt:

```text
Ingest this change into the repo wiki. Read the relevant sources, update any affected wiki pages, update wiki/index.md and wiki/log.md if needed, and tell me if any public docs should also change.
```
