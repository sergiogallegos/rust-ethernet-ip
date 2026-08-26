# Agent-Ready Repository Practices (2026-08-25)

## Summary

Current 2026 guidance converges on a repository as an agent harness, not merely
a body of code with a long prompt attached. The highest-value pattern is:

- a small, human-authored agent entry point that acts as a map
- deeper, versioned knowledge loaded only when relevant
- narrow repo-local workflows backed by deterministic scripts
- explicit acceptance criteria and outcome-based verification
- bounded permissions, isolated workspaces, and auditable execution
- recurring maintenance that prevents documentation and architecture drift

This repository already has unusually strong validation evidence, CI gates,
task handoffs, and maintainer synthesis. Its largest current opportunity is
progressive disclosure: the root `AGENTS.md` is wiki-specific while the broad
engineering map is in a separate, large `CLAUDE.md`, and there are no repo-local
skills that route agents into the scripts and evidence already present.

## Current Understanding

### Cross-source consensus

- `confirmed`: Keep the always-loaded instruction file small. OpenAI describes
  `AGENTS.md` as a map rather than an encyclopedia; Simon Willison's `llm`
  repository uses an 18-line file containing only setup, tests, and docs build
  commands. A 2026 empirical study also recommends minimal, human-written
  requirements rather than generated repository summaries.
- `confirmed`: Put durable domain knowledge in version-controlled files and
  make it discoverable from a short index. Knowledge outside the repository is
  effectively unavailable to an unattended coding agent.
- `confirmed`: Convert repeated procedures into narrow skills and scripts.
  OpenAI's Agents SDK repositories use `AGENTS.md` for conditional triggers,
  repo-local skills for workflow knowledge, deterministic scripts for
  mechanics, and GitHub Actions for enforcement.
- `confirmed`: Verify outcomes, not just plausible-looking code. Simon
  Willison emphasizes confident verification; Anthropic recommends
  deterministic graders where possible and inspecting the final environment
  state rather than trusting an agent's statement that work is complete.
- `confirmed`: Tests are necessary but not sufficient for autonomous work.
  Anthropic's 2026 compiler experiment passed extensive suites while retaining
  important quality and completeness limitations.
- `confirmed`: Long-running work benefits from explicit task contracts,
  file-backed handoffs, independent evaluation, and isolated workspaces.
- `confirmed`: Agent permissions should be bounded by filesystem and network
  policy. Autonomous loops belong in containers or equivalent sandboxes, and
  external content must be treated as untrusted.
- `likely`: Multi-agent orchestration should be added only when work divides
  cleanly or independent review provides measurable lift. Anthropic's newer
  harness work stresses that scaffolding assumptions become stale as models
  improve and should be removed when they no longer earn their cost.

### Important evidence tension

Two January/February 2026 studies report different operational effects for
`AGENTS.md` files:

- one study of 124 pull requests reports lower median runtime and output-token
  use with `AGENTS.md`
- another study across SWE-bench and CTXbench reports no significant accuracy
  gain and more than 20% higher inference cost, especially for generated or
  redundant context

The safe conclusion is not "more context is better." Human-authored,
repo-specific constraints should be small, non-redundant, and evaluated against
representative tasks in this repository.

## Repository Assessment

### Existing strengths

- `confirmed`: [`docs/agents/`](../../docs/agents/README.md) provides durable
  task state, review roles, and structured handoff records.
- `confirmed`: `scripts/validate-agent-files` and CI mechanically validate the
  task protocol instead of relying only on prose.
- `confirmed`: Release, ABI, wrapper, simulator, and hardware gates provide
  strong outcome evidence that most repositories cannot reproduce.
- `confirmed`: [`wiki/index.md`](../index.md) is a compact map into deeper
  synthesis, and wiki claims preserve links to authoritative sources.
- `confirmed`: Existing scripts such as `check-release-readiness`,
  `schema-change-gate`, `run-cross-binding-feature-gate.sh`, and
  `agent-commit` already encode valuable deterministic workflow mechanics.

### Implementation status

- `confirmed` (2026-08-25): Root [`AGENTS.md`](../../AGENTS.md) is now a compact,
  tool-neutral repository map. The wiki schema moved to scoped
  [`../AGENTS.md`](../AGENTS.md), and [`CLAUDE.md`](../../CLAUDE.md) is a small
  adapter to the shared contract.
- `confirmed` (2026-08-25): [`docs/agents/README.md`](../../docs/agents/README.md)
  now defaults to one primary agent, permits an optional reviewer, and assigns
  roles per task rather than by product. Historical task files remain valid.
- `confirmed` (2026-08-25): Three repo-local skills route code verification,
  wiki maintenance, and hardware handoff into existing project knowledge and
  scripts.
- `confirmed` (2026-08-25): `scripts/validate-wiki` checks index coverage,
  local links, titles, and log headings locally and in CI.
- `pilot` (2026-08-25): `docs/agents/evals/cases.toml` defines five historical
  workflow-evaluation cases. It validates metadata but does not yet run model
  trials automatically.
- `pilot` (2026-08-25): One recent four-binding hardware result now has a
  validated JSON companion under `docs/validation/manifests/`; historical
  evidence has not been bulk-converted.
- `partial` (2026-08-25): The existing weekly scheduled CI now runs the new
  deterministic wiki, skill, eval-manifest, and hardware-manifest checks.
  Autonomous docs-sync or refactoring remains deferred until those checks
  demonstrate low-noise behavior.

## Recommended Improvement Order

1. **Create a compact, tool-neutral root map.** Keep root `AGENTS.md` focused on
   entry points, non-obvious invariants, conditional workflow triggers, and the
   shortest correct verification commands. Move the current wiki schema to
   `wiki/AGENTS.md`, using nested instruction scope. Keep shared truth outside
   vendor-specific wrappers.
2. **Add three narrow repo-local skills first.** Recommended starting set:
   `code-change-verification`, `wiki-ingest-and-lint`, and
   `hardware-validation-handoff`. Each should wrap existing scripts and docs,
   declare a clear trigger, and produce a concrete result.
3. **Add a wiki/docs integrity check.** Verify index coverage, local links,
   duplicate page titles, log heading format, and required evidence sections.
   Start report-only, then promote stable checks to CI.
4. **Turn completed tasks into a small agent regression set.** Select roughly
   20 representative historical tasks spanning Rust protocol code, FFI,
   wrappers, docs, and release work. Grade final repository state primarily
   with deterministic tests and artifact checks; retain a small human-reviewed
   rubric for architecture and maintainability.
5. **Make validation evidence more machine-readable.** Preserve the Markdown
   reports, but add a compact manifest for controller, firmware, route, binding,
   command, result, date, and commit so agents can compare evidence without
   re-parsing prose.
6. **Schedule entropy control.** Run periodic docs-sync, wiki lint, stale-version
   checks, and architecture-rule checks. Prefer small targeted findings over
   broad automatic rewrites.
7. **Measure before adding orchestration.** Compare completion rate, wall time,
   steps, and review findings for the compact-map and skill changes. Add
   planner/evaluator or parallel-agent workflows only for task classes where
   the regression set shows a benefit.

Items 1-3 are implemented. Items 4-5 have bounded pilots. Item 6 has scheduled
deterministic checks but no autonomous rewriting, and item 7 remains the
decision rule for any future orchestration.

## Evidence

- [OpenAI, "Harness engineering: leveraging Codex in an agent-first world"
  (2026-02-11)](https://openai.com/index/harness-engineering/)
- [OpenAI Developers, "Using skills to accelerate OSS maintenance"
  (2026-03-09)](https://developers.openai.com/blog/skills-agents-sdk)
- [Official OpenAI `AGENTS.md`
  guidance](https://learn.chatgpt.com/docs/agent-configuration/agents-md)
- [Simon Willison, "Agentic Engineering Patterns"
  (2026)](https://simonwillison.net/guides/agentic-engineering-patterns/)
- [Simon Willison's `llm/AGENTS.md`](https://github.com/simonw/llm/blob/main/AGENTS.md)
- [Simon Willison, "More than just code review"
  (2026-08-22)](https://simonwillison.net/2026/Aug/22/more-than-just-code-review/)
- [Hamel Husain, "Evals Skills for Coding Agents"
  (2026-03-03)](https://hamelhusain.substack.com/p/evals-skills-for-coding-agents)
- [Hamel Husain and Shreya Shankar, current eval-skills
  repository](https://github.com/ai-evals-course/evals-skills)
- [Anthropic, "Demystifying evals for AI agents"
  (2026-01-09)](https://www.anthropic.com/engineering/demystifying-evals-for-ai-agents)
- [Anthropic, "Building a C compiler with a team of parallel Claudes"
  (2026-02-05)](https://www.anthropic.com/engineering/building-c-compiler)
- [Anthropic, "Harness design for long-running application development"
  (2026-03-24)](https://www.anthropic.com/engineering/harness-design-long-running-apps)
- [Gloaguen et al., "Evaluating AGENTS.md" (2026)](https://arxiv.org/abs/2602.11988)
- [Lulla et al., "On the Impact of AGENTS.md Files" (2026)](https://arxiv.org/abs/2601.20404)

## Open Questions

- Which 20 historical tasks best represent this repository's actual agent work?
- Should the tool-neutral shared map be a separate file referenced by both
  `AGENTS.md` and `CLAUDE.md`, or should one wrapper be generated from it?
- Which hardware-validation fields can be normalized without losing the nuance
  currently preserved in Markdown reports?
- What threshold of measured review improvement justifies an independent
  evaluator agent for a task class?

## Related Pages

- [llm-knowledge-base-pattern.md](llm-knowledge-base-pattern.md)
- [test-coverage-strength-2026-05-18.md](test-coverage-strength-2026-05-18.md)
- [software-architecture-map.md](software-architecture-map.md)
- [../controllers/hardware-validation-program.md](../controllers/hardware-validation-program.md)
