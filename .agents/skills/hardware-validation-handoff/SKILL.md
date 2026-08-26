---
name: hardware-validation-handoff
description: Prepare, review, or record real-PLC validation for protocol, wrapper, routing, schema, performance, or release work. Use whenever live hardware evidence is required; never infer authorization to connect or write tags.
---

# Hardware Validation Handoff

Produce a safe, reproducible handoff or evidence record. The maintainer owns
controller access and authorization for every live run.

## Before Any Live Command

- Read `docs/agents/notes/release-hardware-validation.md`,
  `docs/validation/REAL_PLC_TESTING.md`, and the relevant gate document.
- Resolve the exact controller family, firmware, topology, route, bindings, and
  acceptance criteria.
- Require explicit authorization for the target and for writes. An address in
  a document or environment variable is not authorization.
- For writes, identify dedicated non-production tags, capture starting values,
  and define restore or settle behavior plus interruption risk.
- Run dry-run, simulator, build, and schema checks first.

## Handoff Modes

- **Plan only:** create a checklist from
  `docs/validation/REAL_HARDWARE_RESULT_TEMPLATE.md` with commands containing
  placeholders for sensitive addresses. Stop before live execution.
- **Maintainer-executed:** provide exact reviewed commands, expected artifacts,
  pass criteria, and restore checks; label the result pending until evidence is
  returned.
- **Authorized agent-executed:** run only the approved target and scope, monitor
  restoration, stop on the first unsafe ambiguity, and retain no credentials or
  sensitive addresses in committed files.
- **Record evidence:** write a dated Markdown result and its machine-readable
  manifest under `docs/validation/`; distinguish pass, fail, blocked, and not
  run for every binding.

Never generalize one processor, firmware, route, or binding result to an
untested combination. A skipped binding is `not-run`, not `pass`.
