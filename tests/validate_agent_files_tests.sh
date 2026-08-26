#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SCRIPT="$ROOT/scripts/validate-agent-files"
FIXTURES="$ROOT/tests/agent_files_fixtures"

run_case() {
  local name="$1" expected="$2" needle="$3"
  local tmp
  tmp="$(mktemp -d)"
  mkdir -p "$tmp/tasks"
  cp "$FIXTURES/$name.md" "$tmp/tasks/CODEX-TEST-$name.md"
  if [[ "$name" == "wrong_filename" ]]; then
    mv "$tmp/tasks/CODEX-TEST-$name.md" "$tmp/tasks/CODEX-WRONG-$name.md"
  fi
  cat >"$tmp/board.md" <<'BOARD'
# Board

## Open

| Id | Title | Owner | Status | Last update | File |
|---|---|---|---|---|---|
| CODEX-TEST | Fixture | codex | open | 2026-05-25 codex [gpt-5] | [`tasks/CODEX-TEST-valid_task.md`](tasks/CODEX-TEST-valid_task.md) |

## Done

| Id | Title | Owner | Merge commit |
|---|---|---|---|
BOARD
  printf '2026-05-25  codex  [gpt-5]  CODEX-TEST  fixture\n' >"$tmp/log.md"
  if [[ "$name" == "bad_log_line" ]]; then
    cp "$FIXTURES/bad_log_line.md" "$tmp/log.md"
    cp "$FIXTURES/valid_task.md" "$tmp/tasks/CODEX-TEST-valid_task.md"
    rm -f "$tmp/tasks/CODEX-TEST-bad_log_line.md"
  fi

  set +e
  "$SCRIPT" --root "$tmp" --tasks-dir tasks --board board.md --log log.md >"$tmp/out" 2>"$tmp/err"
  local status=$?
  set -e
  if [[ "$status" -ne "$expected" ]]; then
    echo "$name: expected exit $expected, got $status" >&2
    cat "$tmp/out" "$tmp/err" >&2
    exit 1
  fi
  if [[ -n "$needle" ]] && ! grep -q "$needle" "$tmp/err"; then
    echo "$name: missing expected error fragment: $needle" >&2
    cat "$tmp/err" >&2
    exit 1
  fi
  rm -rf "$tmp"
}

run_case valid_task 0 ""
run_case missing_status 1 "missing required frontmatter key: status"
run_case wrong_filename 1 "does not match filename"
run_case out_of_order_sections 1 "required sections are out of order"
run_case bad_log_line 1 "must include \\[model\\] tag"

tmp="$(mktemp -d)"
mkdir -p "$tmp/tasks"
cp "$FIXTURES/valid_role_task.md" "$tmp/tasks/TASK-001-valid-role-task.md"
printf '%s\n' \
  '# Board' '' '## Open' '' \
  '| Id | Title | Owner | Status | Last update | File |' \
  '|---|---|---|---|---|---|' \
  '| TASK-001 | Fixture | primary | open | 2026-08-25 codex [gpt-5.6] | task |' \
  '' '## Done' '' \
  '| Id | Title | Owner | Merge commit |' \
  '|---|---|---|---|' >"$tmp/board.md"
printf '%s\n' '2026-08-25 codex [gpt-5.6] TASK-001 opened' >"$tmp/log.md"
"$SCRIPT" --root "$tmp" --tasks-dir tasks --board board.md --log log.md
rm -rf "$tmp"

echo "validate_agent_files_tests: ok"
