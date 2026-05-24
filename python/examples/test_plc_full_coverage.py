"""Full-coverage random/verify/nines exerciser for the Python wrapper."""
from __future__ import annotations

import math
import os
import random
import sys
import time
from dataclasses import dataclass
from enum import Enum

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
    BLOCKED = "firmware_blocked"
    READ_ONLY = "read_only"


@dataclass(frozen=True, slots=True)
class Tag:
    name: str
    category: str
    kind: Kind
    mode: Mode


def build_tags() -> list[Tag]:
    t: list[Tag] = []

    for i in range(100): t.append(Tag(f"gTestArray_DINT[{i}]",  "ctrl.DINT_array",  Kind.DINT, Mode.WRITEABLE))
    for i in range(50):  t.append(Tag(f"gTestArray_REAL[{i}]",  "ctrl.REAL_array",  Kind.REAL, Mode.WRITEABLE))
    for i in range(128): t.append(Tag(f"gTestArray_BOOL[{i}]",  "ctrl.BOOL_array",  Kind.BOOL, Mode.WRITEABLE))
    for i in range(200): t.append(Tag(f"gTestArray_INT[{i}]",   "ctrl.INT_array",   Kind.INT,  Mode.WRITEABLE))
    for i in range(1000):t.append(Tag(f"gTestArray_Large[{i}]", "ctrl.Large_DINT",  Kind.DINT, Mode.WRITEABLE))

    t.append(Tag("gTest_STRING", "ctrl.STRING", Kind.STRING, Mode.BLOCKED))

    t.append(Tag("gTestUDT", "ctrl.UDT_whole", Kind.UDT, Mode.READ_ONLY))
    t.append(Tag("gTestUDT.Member1_DINT",   "ctrl.UDT_members", Kind.DINT,   Mode.WRITEABLE))
    t.append(Tag("gTestUDT.Member2_REAL",   "ctrl.UDT_members", Kind.REAL,   Mode.WRITEABLE))
    t.append(Tag("gTestUDT.Member3_BOOL",   "ctrl.UDT_members", Kind.BOOL,   Mode.WRITEABLE))
    t.append(Tag("gTestUDT.Member4_INT",    "ctrl.UDT_members", Kind.INT,    Mode.WRITEABLE))
    t.append(Tag("gTestUDT.Member5_String", "ctrl.UDT_members", Kind.STRING, Mode.BLOCKED))
    for i in range(10): t.append(Tag(f"gTestUDT.Array_DINT[{i}]", "ctrl.UDT_nested", Kind.DINT, Mode.WRITEABLE))
    for i in range(5):  t.append(Tag(f"gTestUDT.Array_REAL[{i}]", "ctrl.UDT_nested", Kind.REAL, Mode.WRITEABLE))
    for i in range(20): t.append(Tag(f"gTestUDT.Array_BOOL[{i}]", "ctrl.UDT_nested", Kind.BOOL, Mode.WRITEABLE))

    t.append(Tag("gTestUDT_Array", "ctrl.UDTarr_whole", Kind.UDT, Mode.READ_ONLY))
    for i in range(10):
        t.append(Tag(f"gTestUDT_Array[{i}]", "ctrl.UDTarr_element", Kind.UDT, Mode.READ_ONLY))
        t.append(Tag(f"gTestUDT_Array[{i}].Member1_DINT",   "ctrl.UDTarr_elem_members", Kind.DINT,   Mode.BLOCKED))
        t.append(Tag(f"gTestUDT_Array[{i}].Member2_REAL",   "ctrl.UDTarr_elem_members", Kind.REAL,   Mode.BLOCKED))
        t.append(Tag(f"gTestUDT_Array[{i}].Member3_BOOL",   "ctrl.UDTarr_elem_members", Kind.BOOL,   Mode.BLOCKED))
        t.append(Tag(f"gTestUDT_Array[{i}].Member4_INT",    "ctrl.UDTarr_elem_members", Kind.INT,    Mode.BLOCKED))
        t.append(Tag(f"gTestUDT_Array[{i}].Member5_String", "ctrl.UDTarr_elem_members", Kind.STRING, Mode.BLOCKED))
        for j in range(10): t.append(Tag(f"gTestUDT_Array[{i}].Array_DINT[{j}]", "ctrl.UDTarr_elem_nested", Kind.DINT, Mode.WRITEABLE))
        for j in range(5):  t.append(Tag(f"gTestUDT_Array[{i}].Array_REAL[{j}]", "ctrl.UDTarr_elem_nested", Kind.REAL, Mode.WRITEABLE))
        for j in range(20): t.append(Tag(f"gTestUDT_Array[{i}].Array_BOOL[{j}]", "ctrl.UDTarr_elem_nested", Kind.BOOL, Mode.WRITEABLE))

    for i in range(100): t.append(Tag(f"Program:TestProgram.gTestArray_DINT[{i}]", "prog.DINT_array", Kind.DINT, Mode.WRITEABLE))
    for i in range(50):  t.append(Tag(f"Program:TestProgram.gTestArray_REAL[{i}]", "prog.REAL_array", Kind.REAL, Mode.WRITEABLE))
    for i in range(100): t.append(Tag(f"Program:TestProgram.gTestArray_BOOL[{i}]", "prog.BOOL_array", Kind.BOOL, Mode.WRITEABLE))
    t.append(Tag("Program:TestProgram.gTest_STRING", "prog.STRING", Kind.STRING, Mode.BLOCKED))

    t.append(Tag("Program:TestProgram.gTestUDT", "prog.UDT_whole", Kind.UDT, Mode.READ_ONLY))
    t.append(Tag("Program:TestProgram.gTestUDT.Member1_DINT",   "prog.UDT_members", Kind.DINT,   Mode.WRITEABLE))
    t.append(Tag("Program:TestProgram.gTestUDT.Member2_REAL",   "prog.UDT_members", Kind.REAL,   Mode.WRITEABLE))
    t.append(Tag("Program:TestProgram.gTestUDT.Member3_BOOL",   "prog.UDT_members", Kind.BOOL,   Mode.WRITEABLE))
    t.append(Tag("Program:TestProgram.gTestUDT.Member4_INT",    "prog.UDT_members", Kind.INT,    Mode.WRITEABLE))
    t.append(Tag("Program:TestProgram.gTestUDT.Member5_String", "prog.UDT_members", Kind.STRING, Mode.BLOCKED))
    for i in range(10): t.append(Tag(f"Program:TestProgram.gTestUDT.Array_DINT[{i}]", "prog.UDT_nested", Kind.DINT, Mode.WRITEABLE))
    for i in range(5):  t.append(Tag(f"Program:TestProgram.gTestUDT.Array_REAL[{i}]", "prog.UDT_nested", Kind.REAL, Mode.WRITEABLE))
    for i in range(20): t.append(Tag(f"Program:TestProgram.gTestUDT.Array_BOOL[{i}]", "prog.UDT_nested", Kind.BOOL, Mode.WRITEABLE))

    t.append(Tag("Program:TestProgram.gTestUDT_Array", "prog.UDTarr_whole", Kind.UDT, Mode.READ_ONLY))
    for i in range(5):
        t.append(Tag(f"Program:TestProgram.gTestUDT_Array[{i}]", "prog.UDTarr_element", Kind.UDT, Mode.READ_ONLY))
        t.append(Tag(f"Program:TestProgram.gTestUDT_Array[{i}].Member1_DINT", "prog.UDTarr_elem_members", Kind.DINT, Mode.BLOCKED))
        t.append(Tag(f"Program:TestProgram.gTestUDT_Array[{i}].Member2_REAL", "prog.UDTarr_elem_members", Kind.REAL, Mode.BLOCKED))
        t.append(Tag(f"Program:TestProgram.gTestUDT_Array[{i}].Member3_BOOL", "prog.UDTarr_elem_members", Kind.BOOL, Mode.BLOCKED))
        t.append(Tag(f"Program:TestProgram.gTestUDT_Array[{i}].Member4_INT",  "prog.UDTarr_elem_members", Kind.INT,  Mode.BLOCKED))
        for j in range(10): t.append(Tag(f"Program:TestProgram.gTestUDT_Array[{i}].Array_DINT[{j}]", "prog.UDTarr_elem_nested", Kind.DINT, Mode.WRITEABLE))

    return t


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


class CatStats:
    __slots__ = ("read_ok","read_fail","write_ok","write_fail","verify_ok","verify_fail","blocked_ok","blocked_unexpected")
    def __init__(self): self.read_ok=self.read_fail=self.write_ok=self.write_fail=self.verify_ok=self.verify_fail=self.blocked_ok=self.blocked_unexpected=0


def main() -> int:
    address = os.environ.get("TEST_PLC_ADDRESS", "192.168.0.1:44818")
    slot = int(os.environ.get("TEST_PLC_SLOT", "0"))
    rng = random.Random()
    tags = build_tags()
    writeable = sum(1 for t in tags if t.mode == Mode.WRITEABLE)
    blocked = sum(1 for t in tags if t.mode == Mode.BLOCKED)
    readonly = sum(1 for t in tags if t.mode == Mode.READ_ONLY)

    print("Python wrapper — full-coverage exerciser")
    print(f"PLC: {address} (slot {slot})  total tags: {len(tags)}")
    print(f"  writeable: {writeable}   firmware-blocked: {blocked}   read-only: {readonly}")
    print()

    stats: dict[str, CatStats] = {}
    def S(c: str) -> CatStats:
        if c not in stats: stats[c] = CatStats()
        return stats[c]

    written: list[tuple[Tag, object]] = []

    with Client(address, route_path=RoutePath(slots=[slot])) as client:
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
            if tag.mode != Mode.WRITEABLE: continue
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
            if tag.mode != Mode.BLOCKED: continue
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
            if tag.mode != Mode.WRITEABLE: continue
            v = nines(tag.kind)
            if v is None: continue
            try:
                client.write_tag(tag.name, v, value_type=tag.kind.value)
                settle_ok += 1
            except Exception:
                settle_fail += 1
        print(f"  done in {time.perf_counter()-t4:.1f}s  settle_ok={settle_ok} settle_fail={settle_fail}")
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

    unexpected = T.read_fail + T.write_fail + T.verify_fail + T.blocked_unexpected + settle_fail
    print(
        f"Summary: reads={T.read_ok}/{T.read_ok+T.read_fail}  "
        f"writes={T.write_ok}/{T.write_ok+T.write_fail}  "
        f"verify={T.verify_ok}/{T.verify_ok+T.verify_fail}  "
        f"blocked_as_expected={T.blocked_ok}  unexpected_anomalies={unexpected}"
    )
    print("RESULT: PASS" if unexpected == 0 else f"RESULT: FAIL ({unexpected} anomalies)")
    return 0 if unexpected == 0 else 1


if __name__ == "__main__":
    sys.exit(main())
