"""Full-coverage random/verify/nines exerciser for the Python wrapper."""
from __future__ import annotations

import math
import argparse
import json
import os
import random
import sys
import time
from dataclasses import dataclass
from datetime import datetime, timezone
from enum import Enum
from pathlib import Path

sys.path.insert(0, os.path.abspath(os.path.join(os.path.dirname(__file__), "..")))

from rust_ethernet_ip import Client, RoutePath  # noqa: E402


class Kind(Enum):
    DINT = "DINT"
    INT = "INT"
    REAL = "REAL"
    BOOL = "BOOL"
    STRING = "STRING"
    UDT = "UDT"


class Mode(Enum):
    WRITEABLE = "writeable"
    READ_ONLY = "read_only"
    FIRMWARE_BLOCKED_UDT_STRING_MEMBER = "firmware_blocked_udt_string_member"
    FIRMWARE_BLOCKED_UDT_ARRAY_ELEMENT_MEMBER = "firmware_blocked_udt_array_element_member"
    SERVICE_LAYER_WRITEABLE = "service_layer_writeable"

    @classmethod
    def from_manifest(cls, value: str) -> "Mode":
        try:
            return cls(value)
        except ValueError as exc:
            raise ValueError(f"unknown writeability: {value}") from exc

    def is_writeable(self) -> bool:
        return self in {Mode.WRITEABLE, Mode.SERVICE_LAYER_WRITEABLE}

    def is_firmware_blocked(self) -> bool:
        return self in {
            Mode.FIRMWARE_BLOCKED_UDT_STRING_MEMBER,
            Mode.FIRMWARE_BLOCKED_UDT_ARRAY_ELEMENT_MEMBER,
        }


@dataclass(frozen=True, slots=True)
class Tag:
    name: str
    category: str
    kind: Kind
    mode: Mode


def build_tags(manifest_path: str) -> list[Tag]:
    with open(manifest_path, "r", encoding="utf-8") as fh:
        manifest = json.load(fh)
    tags: list[Tag] = []
    for category in manifest["categories"]:
        tags.extend(expand_category(category))
    return tags


def expand_category(category: dict) -> list[Tag]:
    tags: list[Tag] = []
    pattern = category["pattern"]
    name = category["name"]
    if "members" in category:
        for i in range_or_once(category.get("indices")):
            for member, spec in category["members"].items():
                tags.append(Tag(render_pattern(pattern, i=i, member=member), name, Kind(spec["kind"].upper()), Mode.from_manifest(spec["writeability"])))
        return tags
    if "inner" in category:
        for i in range_or_once(category.get("outer_indices")):
            for field, spec in category["inner"].items():
                for j in range(*spec["range"]):
                    tags.append(Tag(render_pattern(pattern, i=i, field=field, j=j), name, Kind(spec["kind"].upper()), Mode.from_manifest(spec["writeability"])))
        return tags
    for i in range_or_once(category.get("indices")):
        tags.append(Tag(render_pattern(pattern, i=i), name, Kind(category["kind"].upper()), Mode.from_manifest(category["writeability"])))
    return tags


def range_or_once(indices: dict | None) -> range:
    if indices is None:
        return range(1)
    return range(*indices["range"])


def render_pattern(pattern: str, i: int | None = None, member: str | None = None, field: str | None = None, j: int | None = None) -> str:
    output = pattern
    if i is not None: output = output.replace("{i}", str(i))
    if member is not None: output = output.replace("{member}", member)
    if field is not None: output = output.replace("{field}", field)
    if j is not None: output = output.replace("{j}", str(j))
    return output


def rand_value(k: Kind, rng: random.Random) -> object | None:
    if k == Kind.DINT: return rng.randint(1_000, 900_000)
    if k == Kind.INT:  return rng.randint(100, 20_000)
    if k == Kind.REAL: return round(rng.uniform(1.0, 9000.0), 3)
    if k == Kind.BOOL: return bool(rng.randint(0, 1))
    return None


def nines(k: Kind) -> object | None:
    if k == Kind.DINT: return 999_999
    if k == Kind.INT:  return 9_999
    if k == Kind.REAL: return 99.99
    if k == Kind.BOOL: return True
    return None


def values_match(a, b, k: Kind) -> bool:
    if k == Kind.REAL: return math.isclose(a, b, abs_tol=1e-3)
    return a == b


def settle_samples() -> list[tuple[str, Tag, object]]:
    return [
        ("ctrl.BOOL_array", Tag("gTestArray_BOOL[5]", "ctrl.BOOL_array", Kind.BOOL, Mode.WRITEABLE), True),
        ("ctrl.DINT_array", Tag("gTestArray_DINT[42]", "ctrl.DINT_array", Kind.DINT, Mode.WRITEABLE), 999_999),
        ("ctrl.INT_array", Tag("gTestArray_INT[100]", "ctrl.INT_array", Kind.INT, Mode.WRITEABLE), 9_999),
        ("ctrl.Large_DINT", Tag("gTestArray_Large[500]", "ctrl.Large_DINT", Kind.DINT, Mode.WRITEABLE), 999_999),
        ("ctrl.REAL_array", Tag("gTestArray_REAL[10]", "ctrl.REAL_array", Kind.REAL, Mode.WRITEABLE), 99.99),
        ("ctrl.UDT_members", Tag("gTestUDT.Member1_DINT", "ctrl.UDT_members", Kind.DINT, Mode.WRITEABLE), 999_999),
        ("ctrl.UDT_nested", Tag("gTestUDT.Array_DINT[5]", "ctrl.UDT_nested", Kind.DINT, Mode.WRITEABLE), 999_999),
        ("ctrl.UDTarr_elem_nested", Tag("gTestUDT_Array[2].Array_DINT[3]", "ctrl.UDTarr_elem_nested", Kind.DINT, Mode.WRITEABLE), 999_999),
        ("prog.BOOL_array", Tag("Program:TestProgram.gTestArray_BOOL[5]", "prog.BOOL_array", Kind.BOOL, Mode.WRITEABLE), True),
        ("prog.DINT_array", Tag("Program:TestProgram.gTestArray_DINT[42]", "prog.DINT_array", Kind.DINT, Mode.WRITEABLE), 999_999),
        ("prog.REAL_array", Tag("Program:TestProgram.gTestArray_REAL[10]", "prog.REAL_array", Kind.REAL, Mode.WRITEABLE), 99.99),
        ("prog.UDT_members", Tag("Program:TestProgram.gTestUDT.Member1_DINT", "prog.UDT_members", Kind.DINT, Mode.WRITEABLE), 999_999),
        ("prog.UDT_nested", Tag("Program:TestProgram.gTestUDT.Array_DINT[5]", "prog.UDT_nested", Kind.DINT, Mode.WRITEABLE), 999_999),
        ("prog.UDTarr_elem_nested", Tag("Program:TestProgram.gTestUDT_Array[2].Array_DINT[3]", "prog.UDTarr_elem_nested", Kind.DINT, Mode.WRITEABLE), 999_999),
    ]


class CatStats:
    __slots__ = ("read_ok","read_fail","write_ok","write_fail","verify_ok","verify_fail","blocked_ok","blocked_unexpected")
    def __init__(self): self.read_ok=self.read_fail=self.write_ok=self.write_fail=self.verify_ok=self.verify_fail=self.blocked_ok=self.blocked_unexpected=0


def main() -> int:
    # Script lives at python/examples/<name>.py; walk up two levels to the repo root.
    default_manifest = Path(__file__).resolve().parents[2] / "examples" / "full_coverage_tags.json"
    parser = argparse.ArgumentParser()
    parser.add_argument("--plc-address", default=os.environ.get("TEST_PLC_ADDRESS", "192.168.0.1:44818"))
    parser.add_argument("--plc-slot", type=int, default=int(os.environ.get("TEST_PLC_SLOT", "0")))
    parser.add_argument("--manifest", default=str(default_manifest))
    parser.add_argument("--out-dir", default="examples/full_coverage_results")
    parser.add_argument("--dry-run", action="store_true")
    parser.add_argument("--skip-preflight", action="store_true")
    args = parser.parse_args()
    address = args.plc_address
    slot = args.plc_slot
    rng = random.Random()
    try:
        tags = build_tags(args.manifest)
    except (OSError, json.JSONDecodeError, KeyError, ValueError) as exc:
        print(f"manifest-error: failed to load {args.manifest}: {exc}", file=sys.stderr)
        return 2
    writeable = sum(1 for t in tags if t.mode.is_writeable())
    blocked = sum(1 for t in tags if t.mode.is_firmware_blocked())
    readonly = sum(1 for t in tags if t.mode == Mode.READ_ONLY)

    print("Python wrapper — full-coverage exerciser")
    print(f"PLC: {address} (slot {slot})  total tags: {len(tags)}")
    print(f"  writeable: {writeable}   firmware-blocked: {blocked}   read-only: {readonly}")
    print()

    if args.dry_run:
        print(f"would-test binding=python tags={len(tags)} writeable={writeable} blocked={blocked} read_only={readonly}")
        return 0

    stats: dict[str, CatStats] = {}
    def S(c: str) -> CatStats:
        if c not in stats: stats[c] = CatStats()
        return stats[c]

    written: list[tuple[Tag, object]] = []

    with Client(address, route_path=RoutePath(slots=[slot])) as client:
        preflight_ok = preflight_fail = 0
        if not args.skip_preflight:
            print("Phase 0 — preflight tag inventory")
            tp = time.perf_counter()
            for tag in tags:
                try:
                    client.read_tag(tag.name)
                    preflight_ok += 1
                except Exception as exc:
                    preflight_fail += 1
                    print(f"setup-error: tag {tag.name} failed preflight ({exc}) — verify the PLC project against docs/PLC_TEST_TAG_DEFINITIONS.md", file=sys.stderr)
            print(f"  done in {time.perf_counter()-tp:.1f}s  preflight={preflight_ok}/{preflight_ok + preflight_fail}")
            if preflight_fail:
                return 2

        print("Phase 1 — read every tag")
        t0 = time.perf_counter()
        for tag in tags:
            try:
                client.read_tag(tag.name); S(tag.category).read_ok += 1
            except Exception:
                S(tag.category).read_fail += 1
        print(f"  done in {time.perf_counter()-t0:.1f}s")

        print("Phase 2 — write random values to writeable tags")
        t1 = time.perf_counter()
        for tag in tags:
            if not tag.mode.is_writeable(): continue
            v = rand_value(tag.kind, rng)
            if v is None: continue
            try:
                client.write_tag(tag.name, v, value_type=tag.kind.value)
                S(tag.category).write_ok += 1
                written.append((tag, v))
            except Exception:
                S(tag.category).write_fail += 1
        print(f"  done in {time.perf_counter()-t1:.1f}s")

        print("Phase 3 — verify writes via read-back")
        t2 = time.perf_counter()
        for tag, expected in written:
            try:
                actual = client.read_tag(tag.name)
                if values_match(actual, expected, tag.kind):
                    S(tag.category).verify_ok += 1
                else:
                    S(tag.category).verify_fail += 1
            except Exception:
                S(tag.category).verify_fail += 1
        print(f"  done in {time.perf_counter()-t2:.1f}s")

        print("Phase 4 — confirm firmware-blocked writes are still blocked")
        t3 = time.perf_counter()
        for tag in tags:
            if not tag.mode.is_firmware_blocked(): continue
            v = rand_value(tag.kind, rng)
            if v is None: continue
            try:
                client.write_tag(tag.name, v, value_type=tag.kind.value)
                S(tag.category).blocked_unexpected += 1
            except Exception:
                S(tag.category).blocked_ok += 1
        print(f"  done in {time.perf_counter()-t3:.1f}s")

        print("Phase 5 — settle writeable tags to terminal state")
        t4 = time.perf_counter()
        settle_ok = settle_fail = 0
        for tag in tags:
            if not tag.mode.is_writeable(): continue
            v = nines(tag.kind)
            if v is None: continue
            try:
                client.write_tag(tag.name, v, value_type=tag.kind.value)
                settle_ok += 1
            except Exception:
                settle_fail += 1
        print(f"  done in {time.perf_counter()-t4:.1f}s  settle_ok={settle_ok} settle_fail={settle_fail}")
        print()

        print("Phase 6 — verify settle (sample read-back)")
        t5 = time.perf_counter()
        settle_verify_ok = settle_verify_fail = 0
        for category, tag, expected in settle_samples():
            try:
                actual = client.read_tag(tag.name)
                if values_match(actual, expected, tag.kind):
                    settle_verify_ok += 1
                    print(f"  verify-settle  {category:<28} {tag.name:<48} ✓")
                else:
                    settle_verify_fail += 1
                    print(f"  verify-settle  {category:<28} {tag.name:<48} ✗ MISMATCH: expected {expected!r}, got {actual!r}")
            except Exception as exc:
                settle_verify_fail += 1
                print(f"  verify-settle  {category:<28} {tag.name:<48} ✗ READ ERROR: {exc}")
        print(f"  done in {time.perf_counter()-t5:.1f}s  settle_verify={settle_verify_ok}/{settle_verify_ok + settle_verify_fail}")
        print()

    print("Per-category results:")
    print(f"  {'category':<32} {'read+':>9} {'read-':>9} {'write+':>9} {'write-':>9} {'verify+':>9} {'blocked+':>9}")
    T = CatStats()
    for cat in sorted(stats):
        s = stats[cat]
        print(f"  {cat:<32} {s.read_ok:>9} {s.read_fail:>9} {s.write_ok:>9} {s.write_fail:>9} {s.verify_ok:>9} {s.blocked_ok:>9}")
        T.read_ok += s.read_ok; T.read_fail += s.read_fail
        T.write_ok += s.write_ok; T.write_fail += s.write_fail
        T.verify_ok += s.verify_ok; T.verify_fail += s.verify_fail
        T.blocked_ok += s.blocked_ok; T.blocked_unexpected += s.blocked_unexpected
    print(f"  {'TOTAL':<32} {T.read_ok:>9} {T.read_fail:>9} {T.write_ok:>9} {T.write_fail:>9} {T.verify_ok:>9} {T.blocked_ok:>9}")
    print()

    unexpected = T.read_fail + T.write_fail + T.verify_fail + T.blocked_unexpected + settle_fail + settle_verify_fail
    print(
        f"Summary: reads={T.read_ok}/{T.read_ok+T.read_fail}  "
        f"writes={T.write_ok}/{T.write_ok+T.write_fail}  "
        f"verify={T.verify_ok}/{T.verify_ok+T.verify_fail}  "
        f"blocked_as_expected={T.blocked_ok}  unexpected_anomalies={unexpected}"
    )
    print(
        f"binding=python tags={len(tags)} reads={T.read_ok}/{T.read_ok+T.read_fail} "
        f"writes={T.write_ok}/{T.write_ok+T.write_fail} verify={T.verify_ok}/{T.verify_ok+T.verify_fail} "
        f"blocked={T.blocked_ok} anomalies={unexpected} RESULT={'PASS' if unexpected == 0 else 'FAIL'}"
    )
    os.makedirs(args.out_dir, exist_ok=True)
    stamp = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ")
    with open(os.path.join(args.out_dir, f"python_{stamp}.json"), "w", encoding="utf-8") as fh:
        json.dump({
            "schema_version": 1,
            "binding": "python",
            "binding_version": "1.0.0",
            "plc_address": address,
            "plc_slot": slot,
            "manifest_version": 1,
            "tag_count": len(tags),
            "result": "PASS" if unexpected == 0 else "FAIL",
            "anomalies": unexpected,
            "phases": {
                "preflight": {"ok": preflight_ok, "fail": preflight_fail},
                "phase1_read": {"ok": T.read_ok, "fail": T.read_fail},
                "phase2_write": {"ok": T.write_ok, "fail": T.write_fail},
                "phase3_verify": {"ok": T.verify_ok, "fail": T.verify_fail},
                "phase4_blocked": {"ok": T.blocked_ok, "fail": T.blocked_unexpected, "note": "expected firmware rejections"},
                "phase5_settle": {"ok": settle_ok, "fail": settle_fail},
                "phase6_verify_settle": {"ok": settle_verify_ok, "fail": settle_verify_fail},
            },
            "categories": {
                category: {
                    "read_ok": s.read_ok,
                    "read_fail": s.read_fail,
                    "write_ok": s.write_ok,
                    "write_fail": s.write_fail,
                    "verify_ok": s.verify_ok,
                    "verify_fail": s.verify_fail,
                    "blocked_as_expected": s.blocked_ok,
                    "blocked_unexpected_pass": s.blocked_unexpected,
                }
                for category, s in sorted(stats.items())
            },
        }, fh, indent=2)
    print("RESULT: PASS" if unexpected == 0 else f"RESULT: FAIL ({unexpected} anomalies)")
    return 0 if unexpected == 0 else 1


if __name__ == "__main__":
    sys.exit(main())
