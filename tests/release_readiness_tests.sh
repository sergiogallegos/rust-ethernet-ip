#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SCRIPT="$ROOT/scripts/check-release-readiness"

"$SCRIPT" 1.0.0 --skip-package >/tmp/release-readiness-ok.out

tmp="$(mktemp -d)"
mkdir -p "$tmp"
cp -R "$ROOT"/{Cargo.toml,VERSION,CHANGELOG.md,src,crates,csharp,python,examples,scripts} "$tmp"/
printf '1.0.1\n' >"$tmp/VERSION"
set +e
"$SCRIPT" 1.0.0 --root "$tmp" --skip-package >"$tmp/out" 2>"$tmp/err"
status=$?
set -e
if [[ "$status" -eq 0 ]] || ! grep -q "VERSION" "$tmp/out"; then
  echo "expected VERSION drift to fail and name VERSION" >&2
  cat "$tmp/out" "$tmp/err" >&2
  exit 1
fi

set +e
"$SCRIPT" 0.9.9 --skip-package >"$tmp/mismatch" 2>"$tmp/mismatch.err"
status=$?
set -e
if [[ "$status" -eq 0 ]] || ! grep -q "Cargo.toml" "$tmp/mismatch"; then
  echo "expected mismatched arg to fail and name mismatching sites" >&2
  cat "$tmp/mismatch" "$tmp/mismatch.err" >&2
  exit 1
fi

rm -rf "$tmp"
echo "release_readiness_tests: ok"
