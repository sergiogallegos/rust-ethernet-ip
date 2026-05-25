#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
MANIFEST="$ROOT/examples/full_coverage_tags.json"

python3 - "$MANIFEST" <<'PY'
import json
import sys

path = sys.argv[1]
manifest = json.load(open(path, encoding="utf-8"))
allowed = {
    "writeable",
    "read_only",
    "firmware_blocked_string",
    "firmware_blocked_udt_string_member",
    "firmware_blocked_udt_array_element_member",
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
                blocked += mode.startswith("firmware_blocked_")
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
                blocked += n if mode.startswith("firmware_blocked_") else 0
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
    blocked += n if mode.startswith("firmware_blocked_") else 0
    readonly += n if mode == "read_only" else 0

expected = (2299, 2206, 74, 19)
actual = (total, writeable, blocked, readonly)
if actual != expected:
    raise SystemExit(f"count mismatch: expected {expected}, got {actual}")
print("full_coverage_manifest_tests: ok")
PY
