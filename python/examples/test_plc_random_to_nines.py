"""Random -> verify -> nines exerciser for the Python wrapper."""
from __future__ import annotations

import math
import os
import random
import sys
from dataclasses import dataclass

sys.path.insert(0, os.path.abspath(os.path.join(os.path.dirname(__file__), "..")))

from rust_ethernet_ip import Client, RoutePath  # noqa: E402


@dataclass(frozen=True, slots=True)
class Spec:
    tag: str
    kind: str  # "DINT" | "INT" | "REAL" | "BOOL"


TAGS: list[Spec] = [
    Spec("gTestArray_DINT[0]", "DINT"),
    Spec("gTestArray_DINT[5]", "DINT"),
    Spec("gTestArray_DINT[9]", "DINT"),
    Spec("gTestArray_REAL[0]", "REAL"),
    Spec("gTestArray_REAL[4]", "REAL"),
    Spec("gTestArray_INT[0]", "INT"),
    Spec("gTestArray_INT[9]", "INT"),
    Spec("gTestArray_BOOL[0]", "BOOL"),
    Spec("gTestArray_BOOL[5]", "BOOL"),
    Spec("gTestArray_Large[300]", "DINT"),
    Spec("gTestArray_Large[999]", "DINT"),
    Spec("gTestUDT.Member1_DINT", "DINT"),
    Spec("gTestUDT.Member2_REAL", "REAL"),
    Spec("gTestUDT.Member3_BOOL", "BOOL"),
    Spec("gTestUDT.Member4_INT", "INT"),
    Spec("gTestUDT.Array_DINT[5]", "DINT"),
    Spec("gTestUDT.Array_REAL[2]", "REAL"),
    Spec("gTestUDT.Array_BOOL[10]", "BOOL"),
    Spec("Program:TestProgram.gTestArray_DINT[5]", "DINT"),
    Spec("Program:TestProgram.gTestArray_REAL[0]", "REAL"),
    Spec("Program:TestProgram.gTestArray_BOOL[0]", "BOOL"),
    Spec("Program:TestProgram.gTestUDT.Member1_DINT", "DINT"),
    Spec("Program:TestProgram.gTestUDT.Member2_REAL", "REAL"),
    Spec("Program:TestProgram.gTestUDT.Member3_BOOL", "BOOL"),
    Spec("Program:TestProgram.gTestUDT.Member4_INT", "INT"),
    Spec("Program:TestProgram.gTestUDT.Array_DINT[5]", "DINT"),
    Spec("Program:TestProgram.gTestUDT.Array_REAL[2]", "REAL"),
]


def rand_value(kind: str, rng: random.Random) -> object:
    if kind == "DINT":
        return rng.randint(1_000, 900_000)
    if kind == "INT":
        return rng.randint(100, 20_000)
    if kind == "REAL":
        return round(rng.uniform(1.0, 9000.0), 3)
    if kind == "BOOL":
        return bool(rng.randint(0, 1))
    raise ValueError(kind)


def nines_value(kind: str) -> object:
    if kind == "DINT":
        return 999_999
    if kind == "INT":
        return 9_999
    if kind == "REAL":
        return 99.99
    if kind == "BOOL":
        return True
    raise ValueError(kind)


def values_match(a: object, b: object, kind: str) -> bool:
    if kind == "REAL":
        return isinstance(a, (int, float)) and isinstance(b, (int, float)) and math.isclose(a, b, abs_tol=1e-3)
    return a == b


def main() -> int:
    address = os.environ.get("TEST_PLC_ADDRESS", "192.168.0.1:44818")
    slot = int(os.environ.get("TEST_PLC_SLOT", "0"))
    rng = random.Random()

    print("Python wrapper random->verify->nines cycle")
    print(f"PLC: {address} (slot {slot})")
    print(f"Tags: {len(TAGS)}")
    print()

    with Client(address, route_path=RoutePath(slots=[slot])) as client:
        written: list[tuple[Spec, object]] = []

        print("Phase 1 - write random values")
        write_ok = write_fail = 0
        for spec in TAGS:
            v = rand_value(spec.kind, rng)
            try:
                client.write_tag(spec.tag, v, value_type=spec.kind)
                print(f"  WR  {spec.tag:<55} {v}")
                written.append((spec, v))
                write_ok += 1
            except Exception as ex:
                print(f"  ERR {spec.tag:<55} {ex}")
                write_fail += 1
        print(f"  -> {write_ok} ok, {write_fail} failed")
        print()

        print("Phase 2 - read back and verify")
        verify_ok = verify_fail = 0
        for spec, expected in written:
            try:
                actual = client.read_tag(spec.tag)
                ok = values_match(actual, expected, spec.kind)
                print(f"  {'OK ' if ok else 'MIS'}  {spec.tag:<55} expected={expected} actual={actual}")
                if ok:
                    verify_ok += 1
                else:
                    verify_fail += 1
            except Exception as ex:
                print(f"  ERR {spec.tag:<55} {ex}")
                verify_fail += 1
        print(f"  -> {verify_ok} matched, {verify_fail} mismatched/failed")
        print()

        print("Phase 3 - settle to terminal state (DINT=999999, INT=9999, REAL=99.99, BOOL=true)")
        final_ok = final_fail = 0
        for spec in TAGS:
            try:
                client.write_tag(spec.tag, nines_value(spec.kind), value_type=spec.kind)
                final_ok += 1
            except Exception as ex:
                print(f"  ERR {spec.tag:<55} {ex}")
                final_fail += 1
        print(f"  -> {final_ok} settled to nines/true, {final_fail} failed")
        print()

    print(
        f"Summary: random_writes={write_ok}/{len(TAGS)}, "
        f"verify={verify_ok}/{write_ok}, terminal_writes={final_ok}/{len(TAGS)}"
    )
    if write_fail == 0 and verify_fail == 0 and final_fail == 0:
        print("RESULT: PASS")
        return 0
    print("RESULT: FAIL")
    return 1


if __name__ == "__main__":
    sys.exit(main())
