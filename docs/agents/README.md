# Agent Task and Handoff Protocol

This directory is the optional durable coordination layer for one or two coding
agents working on `rust-ethernet-ip`. It records task contracts, progress,
review, and project decisions across sessions without assigning permanent jobs
to a particular product or model.

The normal case is one primary agent. Add an independent reviewer when the
change is high risk or when a second perspective is likely to provide useful
evidence.

## Roles

- **Primary** — may research, design, implement, test, and perform a fresh
  self-review.
- **Reviewer** — independently checks the brief, diff, tests, compatibility,
  and residual risk. The reviewer must not rely only on the primary's summary.
- **Maintainer** — selects roles, resolves strategic decisions, authorizes
  remote publication and PLC access, and performs or supervises hardware work.

Codex, Claude, or another capable agent may fill either agent role. Record the
actual agent and model in entries for auditability; never infer the role from
the product name.

Two agents must not edit the same working tree concurrently. Use a sequential
handoff or separate worktrees with explicit ownership of files and integration.

## Layout

```text
docs/agents/
├── README.md
├── board.md
├── log.md
├── review-template.md
├── evals/
├── notes/
└── tasks/
```

- `board.md` is the current status map. Task-file frontmatter wins if they
  disagree.
- `log.md` is an append-only chronological event stream.
- `notes/` holds load-bearing decisions by technical surface.
- `tasks/` holds one full lifecycle per durable task.
- `evals/` holds the small historical-task regression set used to assess agent
  workflow changes.

## Task Identity and Compatibility

New tasks use neutral identifiers such as `TASK-001` and filenames such as
`TASK-001-short-name.md`. Historical `CODEX-*` identifiers remain valid and
must not be renamed merely for consistency.

New task files use:

```yaml
---
id: TASK-001
title: Short title
owner: primary
status: open
created: YYYY-MM-DD
last-update: YYYY-MM-DD <agent> [<model>]
---
```

`owner` names the current role or explicitly assigned agent. Model attribution
belongs in `last-update` and signed entries, not in public commit messages.

## Task Sections

New task files contain these sections in order:

1. `## Brief` — problem, evidence to read, constraints, acceptance criteria,
   and verification expectations.
2. `## Work log` — append-only progress, assumptions, questions, commands, and
   results from the primary.
3. `## Independent review` — reviewer findings, or a clearly labeled fresh
   self-review when one agent owns the task end to end.
4. `## Verdict` — final disposition and residual risk.

Historical `## Codex log` and `## Claude review` sections are a supported legacy
schema. Do not bulk-rewrite them. The validator accepts both schemas but does
not allow mixing their section names within one task.

Entries are signed:

```markdown
### 2026-08-25 14:30  codex [gpt-5.6] — primary

### 2026-08-25 16:10  claude [Opus 4.7] — independent review
```

If the model is unknown, write `[unknown]`.

## Lifecycle

```text
open → in-progress → submitted → under-review → merged
                    ▲               │
                    └── rejected ───┘
```

- `open`: contract exists; no work started.
- `in-progress`: primary is working.
- `submitted`: implementation and primary verification are ready.
- `under-review`: an independent or fresh self-review is in progress.
- `rejected`: findings require another primary pass.
- `merged`: approved and integrated.

Every status change updates task frontmatter, the board row, and the append-only
log in the same change.

Independent review is expected for protocol wire behavior, `unsafe` or FFI,
public API compatibility, security boundaries, releases, and claims based on
live hardware. Low-risk documentation, focused tests, and mechanical
maintenance may use a single agent and a fresh self-review.

## Brief and Review Quality

A brief specifies observable outcomes without dictating an implementation that
has not yet been justified. Everything a deterministic grader checks must be
discoverable from the brief.

The primary stops for ambiguity that changes acceptance criteria, conflicts
with an authoritative source, expands permissions, or requires unavailable
hardware evidence. It records and proceeds through ordinary internal choices
that do not change the contract.

Use [`review-template.md`](review-template.md) for new reviews. Verify the final
repository state and user-visible behavior, not merely the path the primary
took. A passing test suite is evidence, not proof that every quality claim is
true.

## Log and Voice

Log lines use:

```text
YYYY-MM-DD <agent> [<model>] <task-id-or--> <event>
```

Keep this directory neutral and suitable for a public repository:

- Attribute decisions to roles or recorded agents without first-person prose.
- Do not profile or quote the maintainer.
- Keep agent/model tags inside `docs/agents/`, not commit messages or product
  documentation.
- Correct prior append-only entries with a later superseding entry.

## Commits, Pushes, and Hardware

Use `scripts/agent-commit` when a task requires a local commit; it stages only
explicit paths and rejects broad or suspicious targets. Push only on explicit
maintainer direction or an unambiguous task contract. Never treat a local
commit as a successful push.

Live PLC execution requires explicit maintainer authorization. Use
`$hardware-validation-handoff`; do not infer access or write permission from a
stored address, environment variable, or historical command.

## Validation

Run:

```bash
scripts/validate-agent-files
tests/validate_agent_files_tests.sh
```

Install the optional pre-commit hooks with `scripts/install-hooks`. CI runs the
same deterministic checks.

## Resume and Handoff

To resume durable work:

1. Read `board.md`.
2. Read the specific active task, including all prior work and review entries.
3. Read the last relevant lines of `log.md`.
4. Load only matching pages from `notes/`, the wiki, and authoritative docs.

At handoff, record files changed, commands actually run, results, unverified
claims, hardware gaps, and residual risk. Do not re-derive the entire repository
when the board and task record already provide the necessary state.
