#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SCRIPT="$ROOT/scripts/check-release-readiness"
VERSION="$(awk -F '"' '/^version = / { print $2; exit }' "$ROOT/Cargo.toml")"

# --development mode must tolerate registry-facing docs that still name the
# last-published version while main prepares $VERSION, as long as the
# Unreleased/next-patch markers are present, and plain mode must reject that
# same lag. This uses a synthetic fixture (not live repo content) because
# whether main is currently mid-development-line-prep or just shipped
# $VERSION is a point-in-time fact about the real repo, not something this
# test should depend on.
dev_fixture="$(mktemp -d)"
cp -R "$ROOT"/{Cargo.toml,VERSION,README.md,CHANGELOG.md,src,crates,csharp,python,examples,scripts} "$dev_fixture"/

python3 - "$dev_fixture" "$VERSION" <<'PYEOF'
import re
import sys
from pathlib import Path

root, version = Path(sys.argv[1]), sys.argv[2]

changelog = root / "CHANGELOG.md"
text = changelog.read_text(encoding="utf-8")
text = re.sub(
    r"(?m)^## \[.*$",
    f"## [Unreleased]\n\nTarget release: `{version}`.\n\n## [1.0.0] - 2020-01-01",
    text,
    count=1,
)
changelog.write_text(text, encoding="utf-8")

readme = root / "README.md"
text = readme.read_text(encoding="utf-8")
text = re.sub(r'(?m)^rust-ethernet-ip = "[^"]+"', 'rust-ethernet-ip = "1.0.0"', text, count=1)
text = text.replace(
    "## Version Status",
    f"## Version Status\n\nNext patch in preparation: `{version}`.",
    1,
)
readme.write_text(text, encoding="utf-8")

csharp_readme = root / "csharp/RustEtherNetIp/README.md"
text = csharp_readme.read_text(encoding="utf-8")
text = re.sub(r"current published package: `[^`]+`", "current published package: `1.0.0`", text, count=1)
csharp_readme.write_text(text, encoding="utf-8")
PYEOF

"$SCRIPT" "$VERSION" --root "$dev_fixture" --development --skip-package >/tmp/release-readiness-ok.out
grep -q '~' /tmp/release-readiness-ok.out || {
  echo "expected development mode to tolerate exempted sites with a '~' marker" >&2
  cat /tmp/release-readiness-ok.out >&2
  exit 1
}

set +e
"$SCRIPT" "$VERSION" --root "$dev_fixture" --skip-package >"/tmp/release-readiness-publish.out" 2>"/tmp/release-readiness-publish.err"
status=$?
set -e
if [[ "$status" -eq 0 ]] || ! grep -q "README.md" "/tmp/release-readiness-publish.out"; then
  echo "expected publish mode to reject development-only release markers" >&2
  cat "/tmp/release-readiness-publish.out" "/tmp/release-readiness-publish.err" >&2
  exit 1
fi

rm -rf "$dev_fixture"

tmp="$(mktemp -d)"
mkdir -p "$tmp"
cp -R "$ROOT"/{Cargo.toml,VERSION,README.md,CHANGELOG.md,src,crates,csharp,python,examples,scripts} "$tmp"/
printf '1.0.1\n' >"$tmp/VERSION"
set +e
"$SCRIPT" "$VERSION" --root "$tmp" --development --skip-package >"$tmp/out" 2>"$tmp/err"
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
