# Agent Workflow Evaluation Pilot

This directory holds a small, repository-specific regression set for evaluating
changes to agent instructions, skills, or orchestration. It does not benchmark
foundation models in general.

The initial cases in `cases.toml` are sampled from completed repository tasks
across protocol code, FFI, Python, C++, and agent infrastructure. They are
metadata for controlled replays, not commands that CI should ask an agent to
execute automatically.

## Evaluation Method

1. Create an isolated worktree at the parent of the historical implementation.
2. Give the candidate agent the original `## Brief` and normal repository
   instructions, without the historical work log, review, verdict, or patch.
3. Allow only the permissions appropriate to that case. Hardware remains
   unavailable unless a separate maintainer-approved evaluation explicitly
   provides it.
4. Grade final repository state with the listed deterministic commands and
   artifact checks.
5. Record completion, wall time, tool steps, input/output tokens when available,
   review findings, and any human intervention.
6. Repeat enough trials to distinguish a workflow effect from one lucky run.

Do not optimize instructions against these five cases alone. Add cases from
real failures and new work over time, keeping a separate holdout set once the
collection is large enough.

Run `scripts/validate-agent-evals` to validate the manifest and referenced
historical artifacts.
