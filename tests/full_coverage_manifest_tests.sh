#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
MANIFEST="$ROOT/examples/full_coverage_tags.json"
EXPECTED_RUST="would-test binding=rust tags=2304 writeable=2268 blocked=17 read_only=19"
EXPECTED_CSHARP="would-test binding=csharp tags=2304 writeable=2268 blocked=17 read_only=19"
EXPECTED_PYTHON="would-test binding=python tags=2304 writeable=2268 blocked=17 read_only=19"
EXPECTED_BLOCKED_PROBE="would-probe binding=rust blocked_targets=4 all_blocked=false"

python3 - "$MANIFEST" <<'PY'
import json
import sys

path = sys.argv[1]
manifest = json.load(open(path, encoding="utf-8"))
allowed = {
    "writeable",
    "read_only",
    "firmware_blocked_string",
    "encoding_blocked_udt_string_member",
    "service_layer_writeable",
}
required_category = {"name", "scope", "pattern"}
total = writeable = blocked = readonly = 0

def rng(spec):
    if spec is None:
        return range(1)
    start, end = spec["range"]
    return range(start, end)

for category in manifest["categories"]:
    missing = required_category - category.keys()
    if missing:
        raise SystemExit(f"{category.get('name', '<unknown>')}: missing {sorted(missing)}")
    if "members" in category:
        for i in rng(category.get("indices")):
            _ = i
            for member, spec in category["members"].items():
                if "kind" not in spec:
                    raise SystemExit(f"{category['name']}.{member}: missing kind")
                mode = spec["writeability"]
                if mode not in allowed:
                    raise SystemExit(f"{category['name']}.{member}: bad writeability {mode}")
                total += 1
                writeable += mode == "writeable" or mode == "service_layer_writeable"
                blocked += mode.startswith("firmware_blocked_") or mode.startswith("encoding_blocked_")
                readonly += mode == "read_only"
        continue
    if "inner" in category:
        for i in rng(category.get("outer_indices")):
            _ = i
            for field, spec in category["inner"].items():
                if "kind" not in spec or "range" not in spec:
                    raise SystemExit(f"{category['name']}.{field}: missing kind/range")
                mode = spec["writeability"]
                if mode not in allowed:
                    raise SystemExit(f"{category['name']}.{field}: bad writeability {mode}")
                n = len(rng(spec))
                total += n
                writeable += n if mode in {"writeable", "service_layer_writeable"} else 0
                blocked += n if mode.startswith("firmware_blocked_") or mode.startswith("encoding_blocked_") else 0
                readonly += n if mode == "read_only" else 0
        continue
    if "kind" not in category:
        raise SystemExit(f"{category['name']}: missing kind")
    mode = category["writeability"]
    if mode not in allowed:
        raise SystemExit(f"{category['name']}: bad writeability {mode}")
    n = len(rng(category.get("indices")))
    total += n
    writeable += n if mode in {"writeable", "service_layer_writeable"} else 0
    blocked += n if mode.startswith("firmware_blocked_") or mode.startswith("encoding_blocked_") else 0
    readonly += n if mode == "read_only" else 0

expected = (2304, 2268, 17, 19)
actual = (total, writeable, blocked, readonly)
if actual != expected:
    raise SystemExit(f"count mismatch: expected {expected}, got {actual}")
print("full_coverage_manifest_tests: ok")
PY

assert_contains() {
    local output="$1"
    local expected="$2"
    local label="$3"
    if [[ "$output" != *"$expected"* ]]; then
        printf '%s: expected output to contain %q\n%s\n' "$label" "$expected" "$output" >&2
        exit 1
    fi
}

assert_clean_manifest_error() {
    local output="$1"
    local label="$2"
    assert_contains "$output" "manifest-error:" "$label"
    if [[ "$output" == *"Unhandled exception"* || "$output" == *"Traceback"* ]]; then
        printf '%s: manifest error included a stack trace\n%s\n' "$label" "$output" >&2
        exit 1
    fi
}

run_from_tmp() {
    (cd /tmp && "$@")
}

capture_or_die() {
    local label="$1"
    shift
    local output
    if ! output="$("$@" 2>&1)"; then
        local status=$?
        printf '%s: command failed (exit %d)\n----- captured output -----\n%s\n---------------------------\n' \
            "$label" "$status" "$output" >&2
        exit 1
    fi
    printf '%s' "$output"
}

rust_default="$(capture_or_die "rust default manifest from /tmp" run_from_tmp cargo run --manifest-path "$ROOT/Cargo.toml" --example test_plc_full_coverage --locked -- --dry-run)"
assert_contains "$rust_default" "$EXPECTED_RUST" "rust default manifest from /tmp"

csharp_default="$(capture_or_die "csharp default manifest from /tmp" run_from_tmp dotnet run --project "$ROOT/examples/CSharpFullCoverage/CSharpFullCoverage.csproj" -c Release -- --dry-run)"
assert_contains "$csharp_default" "$EXPECTED_CSHARP" "csharp default manifest from /tmp"

python_default="$(capture_or_die "python default manifest from /tmp" run_from_tmp env PYTHONPATH="$ROOT/python" python3 "$ROOT/python/examples/test_plc_full_coverage.py" --dry-run)"
assert_contains "$python_default" "$EXPECTED_PYTHON" "python default manifest from /tmp"

rust_override="$(capture_or_die "rust manifest override from /tmp" run_from_tmp cargo run --manifest-path "$ROOT/Cargo.toml" --example test_plc_full_coverage --locked -- --manifest "$MANIFEST" --dry-run)"
assert_contains "$rust_override" "$EXPECTED_RUST" "rust manifest override from /tmp"

csharp_override="$(capture_or_die "csharp manifest override from /tmp" run_from_tmp dotnet run --project "$ROOT/examples/CSharpFullCoverage/CSharpFullCoverage.csproj" -c Release -- --manifest "$MANIFEST" --dry-run)"
assert_contains "$csharp_override" "$EXPECTED_CSHARP" "csharp manifest override from /tmp"

python_override="$(capture_or_die "python manifest override from /tmp" run_from_tmp env PYTHONPATH="$ROOT/python" python3 "$ROOT/python/examples/test_plc_full_coverage.py" --manifest "$MANIFEST" --dry-run)"
assert_contains "$python_override" "$EXPECTED_PYTHON" "python manifest override from /tmp"

blocked_probe="$(capture_or_die "blocked-label probe dry-run from /tmp" run_from_tmp cargo run --manifest-path "$ROOT/Cargo.toml" --example probe_blocked_write_labels --locked -- --manifest "$MANIFEST" --dry-run)"
assert_contains "$blocked_probe" "$EXPECTED_BLOCKED_PROBE" "blocked-label probe dry-run from /tmp"

if rust_bad="$(run_from_tmp cargo run --manifest-path "$ROOT/Cargo.toml" --example test_plc_full_coverage --locked -- --manifest /tmp/nonexistent_full_coverage_tags.json --dry-run 2>&1)"; then
    printf 'rust bad manifest unexpectedly succeeded\n%s\n' "$rust_bad" >&2
    exit 1
fi
assert_clean_manifest_error "$rust_bad" "rust bad manifest"

if csharp_bad="$(run_from_tmp dotnet run --project "$ROOT/examples/CSharpFullCoverage/CSharpFullCoverage.csproj" -c Release -- --manifest /tmp/nonexistent_full_coverage_tags.json --dry-run 2>&1)"; then
    printf 'csharp bad manifest unexpectedly succeeded\n%s\n' "$csharp_bad" >&2
    exit 1
fi
assert_clean_manifest_error "$csharp_bad" "csharp bad manifest"

if python_bad="$(run_from_tmp env PYTHONPATH="$ROOT/python" python3 "$ROOT/python/examples/test_plc_full_coverage.py" --manifest /tmp/nonexistent_full_coverage_tags.json --dry-run 2>&1)"; then
    printf 'python bad manifest unexpectedly succeeded\n%s\n' "$python_bad" >&2
    exit 1
fi
assert_clean_manifest_error "$python_bad" "python bad manifest"

echo "full_coverage_manifest_cwd_tests: ok"
