# Cross-Agent Collaboration Protocol

This directory is the **durable communication channel** between two LLM agents working on rust-ethernet-ip:

- **Claude** — design, architecture, code review.
- **Codex** — development, debugging, refactoring.

The repository maintainer routes messages between them. Neither agent has access to the other's conversation; this directory is the only shared context that persists across turns.

> If you are an LLM reading this for the first time in a session, identify yourself (`claude` or `codex`) before making any changes. Read this whole file before writing to any other file in this directory.

## Why this exists

A single conversation context belongs to one agent. When two agents collaborate on the same codebase, they need a shared, append-mostly artifact to:

- Hand off task briefs without re-explaining context every turn.
- Ask each other clarifying questions that survive across sessions.
- Record decisions and review verdicts that bind both agents.
- Let either agent reconstruct project state by reading the directory cold.

## File layout

```
docs/agents/
├── README.md                       # this file — the protocol
├── board.md                        # status of every task at a glance
├── log.md                          # append-only chronological transcript
└── tasks/
    ├── CODEX-A-<slug>.md
    ├── CODEX-B-<slug>.md
    └── CODEX-C-<slug>.md           # one file per task, full lifecycle
```

- **`board.md`** — single table summarizing every task: id, title, owner, status, last update. Kanban-style snapshot.
- **`log.md`** — append-only one-liners. Format: `YYYY-MM-DD HH:MM <author> <task-id> <event>`. Newest at bottom. Never edit prior entries.
- **`tasks/<id>.md`** — full lifecycle for one task: the brief, Codex's working notes, Claude's review, the verdict. Each task gets one file. Don't split a task across files.

## Task lifecycle

Status flow:

```
open ──▶ in-progress ──▶ submitted ──▶ under-review ──┬──▶ merged
                ▲                                     │
                └─────────── rejected ◀───────────────┘
```

| Status | Meaning | Set by |
|---|---|---|
| `open` | Brief written, no work started | claude (when authoring brief) |
| `in-progress` | Codex acknowledged and started | codex (when starting work) |
| `submitted` | Codex finished, awaiting review | codex (with commit ref or diff) |
| `under-review` | Claude has begun reviewing | claude (when starting review) |
| `merged` | Approved and integrated | claude (with merge commit ref) |
| `rejected` | Changes requested; back to `in-progress` after Codex addresses | claude (with punch list) |

Status changes are reflected in **three** places:
1. The task file's frontmatter `status:` field.
2. The corresponding row in `board.md`.
3. A new line in `log.md`.

All three are part of the same edit.

## Authoring conventions

### Sections in a task file

Every task file has these sections, in this order:

1. **Frontmatter** (yaml). Fields: `id`, `title`, `owner`, `status`, `created`, `last-update`.
2. **Brief** — written once by claude. Codex must not edit this. If the brief is wrong, codex appends a question in `## Codex log` rather than editing.
3. **Codex log** — append-only by codex. Each entry is timestamped and signed.
4. **Claude review** — append-only by claude after submission.
5. **Verdict** — final disposition by claude.

### Entry format

Within `## Codex log` and `## Claude review`, each entry starts with a header line:

```markdown
### 2026-05-01 14:30  codex
<content>

### 2026-05-01 15:10  claude — review pass 1
<content>
```

Entries are appended, never edited. If something written earlier was wrong, write a new entry that supersedes it; don't edit the old one.

### Asking questions

Either agent can ask the other a question by appending an entry that begins with `### YYYY-MM-DD HH:MM  <author> — question`. The other agent responds in their own log section (codex in `## Codex log`, claude in `## Claude review`) with a header starting `— answer to <date>`.

Open questions block the task — while a question is open and unanswered, the task does not advance.

**Ambiguity threshold — when to stop vs proceed:**

- **Stop and ask** when the ambiguity affects acceptance criteria, contradicts a brief assumption, or requires source-of-truth context the brief doesn't provide. Examples: brief pins a crate version that doesn't exist on crates.io; brief contradicts an architectural decision in CLAUDE.md; acceptance test description is internally inconsistent; brief specifies an API that conflicts with upstream's actual surface.
- **Document and proceed** when the ambiguity is a normal implementation choice with no contract impact. Examples: variable naming, internal helper structure, log message wording, choice between two equivalent stdlib calls. Add a one-line entry to `## Codex log` recording the assumption ("Assumed X because Y; revisit if review disagrees.") so review can cheaply override.

The cost of stalling on small details is higher than the cost of a v1.1 polish item.

### Decisions

If a task surfaces a decision that affects more than just this task, claude records it in `docs/` (an architecture page or `CLAUDE.md` update) and links to it from the task file. Don't bury cross-cutting decisions in a task file alone.

### Out of scope for this protocol

- Code style nits → use review entries with explicit `file:line` references.
- Long-running side discussions → spawn a new task file or keep them in PR comments after merge.
- Routine status updates → one line in `log.md` is enough.

### Voice

Use neutral framing in everything written into this directory and into project docs (`CLAUDE.md`, `README.md`, `docs/`, commit messages, PR descriptions):

- **No first-person.** Write "Codex implemented X" / "Claude-authored brief" / "the original brief" / "brief error owned by Claude". Not "I added X" / "my brief".
- **No maintainer profiling.** Write "the maintainer requested" / "per maintainer direction". Not "the user wants X" / direct quotes of maintainer chat.
- **No agent attribution in commit messages or PR descriptions.** Public artifacts read as the project's own voice.
- **End-user references are fine when domain-relevant.** "the user's PLC tag", "the integrator's calling code" are correct when they refer to actual library users.
- **Paraphrase, don't quote.** If a maintainer message defines a project convention, restate it neutrally.

This repo is published as a public Rust crate and NuGet package. Personal phrasing leaks behavioral signals (work patterns, incidents, preferences) that belong in private agent memory, not in project history. Both agents should self-edit before committing; reviewers flag voice drift in the same pass as technical findings.

## Who edits what

| File | Claude edits | Codex edits |
|---|:-:|:-:|
| `README.md` (this file) | rarely (protocol changes) | only if asked by claude |
| `board.md` | yes, on status changes | yes, on status changes |
| `log.md` | append on events | append on events |
| `tasks/<id>.md` Brief | yes, when authoring | never |
| `tasks/<id>.md` Codex log | never | append-only |
| `tasks/<id>.md` Claude review | append-only | never |
| `tasks/<id>.md` Verdict | yes (sole author) | never |
| `tasks/<id>.md` frontmatter | yes (when status flips that claude owns) | yes (when status flips that codex owns) |

## Commit and push expectations

Both agents may stage and commit edits to task files, `board.md`, and `log.md` as part of normal task work. The lifecycle three-place update (frontmatter + board + log) should commit together.

**Pushing to the remote is not automatic:**

- Push only when the maintainer explicitly asks ("commit and push", "ship it"), or when an unambiguous task convention requires it (e.g. backfilling a merge ref in a follow-up commit).
- Push only when the local environment permits it. If push is blocked (network, auth, safe-directory), surface the blocker — don't retry silently or work around it.
- A successful local commit is not a successful push. Always confirm the push step ran before claiming a task moved to `merged` or `submitted`.

This prevents the case where one agent's session pushes to the remote while the other agent's session has unpushed local commits, leaving the two views diverged.

## How to add a new task

1. Pick the next id (`CODEX-A`, `CODEX-B`, …).
2. Create `tasks/<id>-<short-name>.md` with frontmatter, Brief, empty Codex log, empty Claude review, empty Verdict.
3. Add a row to `board.md`.
4. Append an `open` event to `log.md`.

## How to consume this protocol if you are…

**…the maintainer routing messages.** Tell each agent which file to read. Examples:
- "Codex, read `docs/agents/tasks/CODEX-A-<slug>.md` and start the task."
- "Claude, codex submitted CODEX-B; review it."

**…claude, reading at the start of a turn.** Read `board.md` first to see overall state. Then read the specific task file the maintainer pointed you at. Update status + log + board in the same turn as your work.

**…codex, reading at the start of a turn.** Same — board first, then task. Append your work to the Codex log section. Don't touch Brief or Claude review. Don't edit prior entries.

## Source-of-truth precedence inside this directory

If `board.md` and a task file's frontmatter disagree, the **task file frontmatter wins** and `board.md` should be corrected. `log.md` is historical; it does not override current state.

## Relationship to other docs

- **`CLAUDE.md`** at repo root — short version of this protocol plus rust-ethernet-ip-specific project context (build commands, architecture, tag-path syntax, PLC firmware limits). Both apply simultaneously: read CLAUDE.md for project knowledge, this directory for cross-agent state.
- **`README.md`** at repo root — for human consumers of the library. Humans don't need this protocol; it's purely an LLM-to-LLM channel.
