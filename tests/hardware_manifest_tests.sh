#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SCRIPT="$ROOT/scripts/validate-hardware-manifests"
SOURCE_MANIFEST="$ROOT/docs/validation/manifests/2026-08-22_1756-L75_fw33_post-BF-BI-cross-binding-full-coverage.json"

tmp="$(mktemp -d)"
mkdir -p "$tmp/docs/validation/manifests" "$tmp/examples"
cp "$SOURCE_MANIFEST" "$tmp/docs/validation/manifests/result.json"
touch "$tmp/docs/validation/2026-08-22_1756-L75_fw33_post-BF-BI-cross-binding-full-coverage.md"
touch "$tmp/examples/full_coverage_tags.json"

"$SCRIPT" --root "$tmp"

sed 's/"total_paths": 2304/"total_paths": 2305/' \
  "$tmp/docs/validation/manifests/result.json" >"$tmp/docs/validation/manifests/invalid.json"
rm "$tmp/docs/validation/manifests/result.json"

set +e
"$SCRIPT" --root "$tmp" >"$tmp/out" 2>"$tmp/err"
status=$?
set -e
if [[ "$status" -ne 1 ]] || ! grep -q "must equal total_paths" "$tmp/err"; then
  echo "invalid-counts: expected count-consistency failure" >&2
  cat "$tmp/out" "$tmp/err" >&2
  exit 1
fi

rm -rf "$tmp"
echo "hardware_manifest_tests: ok"
