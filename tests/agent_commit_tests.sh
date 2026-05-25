#!/usr/bin/env bash
set -euo pipefail

ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
SCRIPT="$ROOT/scripts/agent-commit"

tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT

setup_repo() {
  local dir=$1
  mkdir -p "$dir"
  git -C "$dir" init -q
  git -C "$dir" config user.email agent@example.invalid
  git -C "$dir" config user.name "Agent Test"
}

expect_fail_in() {
  local dir=$1
  shift
  local needle=$1
  shift
  local out
  set +e
  out=$(cd "$dir" && "$@" 2>&1)
  local status=$?
  set -e
  if [[ $status -eq 0 ]]; then
    echo "expected failure: $*" >&2
    exit 1
  fi
  if [[ "$out" != *"$needle"* ]]; then
    echo "missing expected output '$needle'" >&2
    echo "$out" >&2
    exit 1
  fi
}

repo="$tmp/happy"
setup_repo "$repo"
printf 'one\n' >"$repo/a.txt"
printf 'two\n' >"$repo/b.txt"
(cd "$repo" && "$SCRIPT" "commit two files" a.txt b.txt) >/dev/null
test "$(git -C "$repo" rev-list --count HEAD)" = "1"
test "$(git -C "$repo" diff-tree --root --no-commit-id --name-only -r HEAD | wc -l | tr -d ' ')" = "2"

repo="$tmp/rejects"
setup_repo "$repo"
printf 'x\n' >"$repo/a.txt"
expect_fail_in "$repo" "forbids wildcards" "$SCRIPT" "bad" .
expect_fail_in "$repo" "must not be empty" "$SCRIPT" "   " a.txt
expect_fail_in "$repo" "file does not exist" "$SCRIPT" "missing" missing.txt
printf 'secret\n' >"$repo/.env"
expect_fail_in "$repo" "possible secret" "$SCRIPT" "secret" .env

repo="$tmp/deletion"
setup_repo "$repo"
printf 'tracked\n' >"$repo/a.txt"
git -C "$repo" add a.txt
git -C "$repo" commit -m initial >/dev/null
rm "$repo/a.txt"
expect_fail_in "$repo" "pass --allow-deletion" "$SCRIPT" "delete a" a.txt
(cd "$repo" && "$SCRIPT" --allow-deletion "delete a" a.txt) >/dev/null
test "$(git -C "$repo" rev-list --count HEAD)" = "2"

repo="$tmp/hygiene"
setup_repo "$repo"
printf 'base\n' >"$repo/a.txt"
printf 'base\n' >"$repo/b.txt"
git -C "$repo" add a.txt b.txt
git -C "$repo" commit -m initial >/dev/null
printf 'changed\n' >"$repo/a.txt"
printf 'changed\n' >"$repo/b.txt"
git -C "$repo" add a.txt
(cd "$repo" && "$SCRIPT" "commit only b" b.txt) >/dev/null
git -C "$repo" diff --quiet HEAD -- a.txt || true
if git -C "$repo" diff --cached --quiet -- a.txt; then
  :
else
  echo "pre-stage hygiene left unrelated file staged" >&2
  exit 1
fi
test "$(git -C "$repo" diff-tree --no-commit-id --name-only -r HEAD)" = "b.txt"

repo="$tmp/amend"
setup_repo "$repo"
printf 'one\n' >"$repo/a.txt"
(cd "$repo" && "$SCRIPT" "initial" a.txt) >/dev/null
printf 'two\n' >"$repo/a.txt"
expect_fail_in "$repo" "--amend is blocked" "$SCRIPT" --amend "blocked amend" a.txt
(cd "$repo" && "$SCRIPT" --amend-anyway "amended" a.txt) >/dev/null
test "$(git -C "$repo" rev-list --count HEAD)" = "1"

echo "agent_commit_tests: ok"
