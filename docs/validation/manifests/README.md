# Hardware Validation Manifests

These JSON files are machine-readable companions to the dated Markdown records
in `docs/validation/`. Markdown remains authoritative for commands, anomalies,
interpretation, and safety context; manifests expose stable fields for indexing
and comparison.

Rules:

- Never include controller addresses, credentials, or other secrets.
- Identify the source Markdown record and repository commit.
- Use exact controller and firmware evidence; do not generalize.
- Record every attempted binding as `pass`, `fail`, `blocked`, or `not-run`.
- Include restore or settle status whenever writes occurred.
- Validate with `scripts/validate-hardware-manifests`.

`hardware-result.schema.json` documents schema version 1. The repository
validator enforces the required subset without adding a JSON Schema dependency.
