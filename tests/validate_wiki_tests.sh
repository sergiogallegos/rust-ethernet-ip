#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SCRIPT="$ROOT/scripts/validate-wiki"

make_fixture() {
  local target="$1"
  mkdir -p "$target/wiki/topic"
  printf '%s\n' '# Wiki' '' '[Page](topic/page.md)' >"$target/wiki/index.md"
  printf '%s\n' '# README' >"$target/wiki/README.md"
  printf '%s\n' '# Rules' >"$target/wiki/AGENTS.md"
  printf '%s\n' '# Log' '' '## [2026-08-25] lint | fixture' >"$target/wiki/log.md"
  printf '%s\n' '# Page' '' '## Summary' '' 'Fixture.' '' '## Evidence' '' '- Source.' >"$target/wiki/topic/page.md"
}

run_failure() {
  local name="$1" needle="$2"
  local tmp
  tmp="$(mktemp -d)"
  make_fixture "$tmp"
  case "$name" in
    orphan)
      printf '%s\n' '# Orphan' '' '## Summary' '' 'Fixture.' '' '## Evidence' '' '- Source.' >"$tmp/wiki/topic/orphan.md"
      ;;
    broken-link)
      printf '%s\n' '' '[Missing](missing.md)' >>"$tmp/wiki/topic/page.md"
      ;;
    bad-log)
      printf '%s\n' '## [2026-08-25] update | invalid operation' >>"$tmp/wiki/log.md"
      ;;
    duplicate-title)
      printf '%s\n' '# Page' '' '## Summary' '' 'Fixture.' '' '## Evidence' '' '- Source.' >"$tmp/wiki/topic/second.md"
      printf '%s\n' '[Second](topic/second.md)' >>"$tmp/wiki/index.md"
      ;;
  esac
  set +e
  "$SCRIPT" --root "$tmp" >"$tmp/out" 2>"$tmp/err"
  local status=$?
  set -e
  if [[ "$status" -ne 1 ]] || ! grep -q "$needle" "$tmp/err"; then
    echo "$name: expected failure containing: $needle" >&2
    cat "$tmp/out" "$tmp/err" >&2
    exit 1
  fi
  rm -rf "$tmp"
}

tmp="$(mktemp -d)"
make_fixture "$tmp"
"$SCRIPT" --root "$tmp"
rm -rf "$tmp"

run_failure orphan "orphan page"
run_failure broken-link "broken local link"
run_failure bad-log "invalid wiki log heading"
run_failure duplicate-title "duplicate title"

echo "validate_wiki_tests: ok"
