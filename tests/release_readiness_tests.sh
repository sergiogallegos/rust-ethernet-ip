#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SCRIPT="$ROOT/scripts/check-release-readiness"
VERSION="$(awk -F '"' '/^version = / { print $2; exit }' "$ROOT/Cargo.toml")"

make_fixture() {
  local destination="$1"
  mkdir -p \
    "$destination/src" \
    "$destination/crates/"{types,tag-path,protocol,udt} \
    "$destination/csharp/RustEtherNetIp.Tests" \
    "$destination/csharp/RustEtherNetIp" \
    "$destination/python/tests" \
    "$destination/examples/"{AspNetExample,CSharpWebHmi,WpfExample,WinFormsExample} \
    "$destination/scripts" \
    "$destination/docs/agents" \
    "$destination/wiki/controllers" \
    "$destination/wiki/wrapper-parity"
  cp "$ROOT/"{Cargo.toml,VERSION,README.md,CHANGELOG.md} "$destination/"
  cp "$ROOT/src/"{version.rs,lib.rs} "$destination/src/"
  cp "$ROOT/crates/types/Cargo.toml" "$destination/crates/types/"
  cp "$ROOT/crates/tag-path/Cargo.toml" "$destination/crates/tag-path/"
  cp "$ROOT/crates/protocol/Cargo.toml" "$destination/crates/protocol/"
  cp "$ROOT/crates/udt/Cargo.toml" "$destination/crates/udt/"
  cp "$ROOT/csharp/RustEtherNetIp/"{RustEtherNetIp.csproj,README.md} "$destination/csharp/RustEtherNetIp/"
  cp "$ROOT/csharp/RustEtherNetIp.Tests/AbiContractTests.cs" "$destination/csharp/RustEtherNetIp.Tests/"
  cp "$ROOT/python/"{pyproject.toml,README.md} "$destination/python/"
  cp "$ROOT/python/tests/test_abi_contract.py" "$destination/python/tests/"
  cp "$ROOT/examples/AspNetExample/AspNetExample.csproj" "$destination/examples/AspNetExample/"
  cp "$ROOT/examples/CSharpWebHmi/CSharpWebHmi.csproj" "$destination/examples/CSharpWebHmi/"
  cp "$ROOT/examples/WpfExample/WpfExample.csproj" "$destination/examples/WpfExample/"
  cp "$ROOT/examples/WinFormsExample/WinFormsExample.csproj" "$destination/examples/WinFormsExample/"
  cp "$ROOT/scripts/check-release-readiness.txt" "$destination/scripts/"
  cp "$ROOT/docs/"{VERSION_MANAGEMENT.md,OFFICIAL_SOURCES.md,programmer_manual.md,SOFTWARE_ARCHITECTURE.md,CODEX_PYTHON_PLATFORM_EXPANSION_PROMPT.md,LIBRARY_COMPARISON_AND_IMPROVEMENTS.md} "$destination/docs/"
  cp "$ROOT/docs/agents/board.md" "$destination/docs/agents/"
  cp "$ROOT/wiki/controllers/firmware-behavior.md" "$destination/wiki/controllers/"
  cp "$ROOT/wiki/wrapper-parity/rust-vs-csharp.md" "$destination/wiki/wrapper-parity/"
}

# --development mode must tolerate registry-facing docs that still name the
# last-published version while main prepares $VERSION, as long as the
# Unreleased/next-patch markers are present, and plain mode must reject that
# same lag. This uses a synthetic fixture (not live repo content) because
# whether main is currently mid-development-line-prep or just shipped
# $VERSION is a point-in-time fact about the real repo, not something this
# test should depend on.
dev_fixture="$(mktemp -d)"
make_fixture "$dev_fixture"

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
make_fixture "$tmp"
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

cp "$ROOT/VERSION" "$tmp/VERSION"
python3 - "$tmp/docs/VERSION_MANAGEMENT.md" <<'PYEOF'
import re
import sys
from pathlib import Path

path = Path(sys.argv[1])
text = path.read_text(encoding="utf-8")
text = re.sub(
    r"Current stable published version:\*\* `[^`]+`",
    "Current stable published version:** `1.0.0`",
    text,
    count=1,
)
path.write_text(text, encoding="utf-8")
PYEOF
set +e
"$SCRIPT" "$VERSION" --root "$tmp" --skip-package >"$tmp/stale-doc" 2>"$tmp/stale-doc.err"
status=$?
set -e
if [[ "$status" -eq 0 ]] || ! grep -q "docs/VERSION_MANAGEMENT.md" "$tmp/stale-doc"; then
  echo "expected active release-document drift to fail and name the source file" >&2
  cat "$tmp/stale-doc" "$tmp/stale-doc.err" >&2
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

# Built-package inspection must accept current release markers across Cargo,
# NuGet, and Python archive formats, and reject stale lifecycle/install text.
artifact_fixture="$(mktemp -d)"
python3 - "$artifact_fixture" "$VERSION" <<'PYEOF'
import io
import sys
import tarfile
import zipfile
from pathlib import Path

root, version = Path(sys.argv[1]), sys.argv[2]
good = f"""# Package

- Current stable release: `{version}`

```toml
rust-ethernet-ip = "{version}"
```
"""
stale = """# Package

- Latest published PyPI package: `1.0.0`
- `1.2.1` is not published yet.

```bash
python -m pip install rust-ethernet-ip==1.0.0
```
"""

with tarfile.open(root / "good.crate", "w:gz") as archive:
    payload = good.encode()
    info = tarfile.TarInfo("package/README.md")
    info.size = len(payload)
    archive.addfile(info, io.BytesIO(payload))

with zipfile.ZipFile(root / "good.nupkg", "w") as archive:
    archive.writestr("README.md", good)

with zipfile.ZipFile(root / "good.whl", "w") as archive:
    archive.writestr("package-1.0.0.dist-info/METADATA", good)

with zipfile.ZipFile(root / "stale.whl", "w") as archive:
    archive.writestr("package-1.0.0.dist-info/METADATA", stale)
PYEOF

"$ROOT/scripts/check-packaged-release-docs" "$VERSION" \
  --require-version-marker \
  "$artifact_fixture/good.crate" \
  "$artifact_fixture/good.nupkg" \
  "$artifact_fixture/good.whl" >"$artifact_fixture/good.out"

set +e
"$ROOT/scripts/check-packaged-release-docs" "$VERSION" \
  "$artifact_fixture/stale.whl" >"$artifact_fixture/stale.out" 2>"$artifact_fixture/stale.err"
status=$?
set -e
if [[ "$status" -eq 0 ]] || ! grep -q "stale release-lifecycle claim" "$artifact_fixture/stale.err"; then
  echo "expected packaged-document inspection to reject stale release text" >&2
  cat "$artifact_fixture/stale.out" "$artifact_fixture/stale.err" >&2
  exit 1
fi

rm -rf "$artifact_fixture"
echo "release_readiness_tests: ok"
