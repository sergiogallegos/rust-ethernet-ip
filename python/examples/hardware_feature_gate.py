"""Restore-safe companion gate for batch, whole-UDT, and discovery coverage."""

from __future__ import annotations

import argparse
import os
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

from rust_ethernet_ip import BatchWriteItem, Client, RoutePath  # noqa: E402


BATCH_READ_TAGS = [
    "gTestArray_DINT[0]",
    "gTestArray_INT[0]",
    "gTestArray_REAL[0]",
    "gTestArray_BOOL[0]",
    "gTest_STRING",
    "gTestUDT.Member1_DINT",
    "Program:TestProgram.gTestArray_DINT[0]",
    "Program:TestProgram.gTestUDT.Member1_DINT",
    "Program:TestProgram.gTestArray_REAL[0]",
    "Program:TestProgram.gTestArray_BOOL[0]",
]

BATCH_WRITE_TAGS = [
    "gTestArray_DINT[50]",
    "gTestArray_DINT[51]",
    "Program:TestProgram.gTestArray_DINT[50]",
    "Program:TestProgram.gTestArray_DINT[51]",
]

WHOLE_UDT_TAGS = [
    "gTestUDT",
    "gTestUDT_Array[0]",
    "Program:TestProgram.gTestUDT",
    "Program:TestProgram.gTestUDT_Array[0]",
]


def with_program(tag: str, program: str) -> str:
    return tag.replace("TestProgram", program)


def exercise_value(original: int) -> int:
    return 123_456_788 if original == 123_456_789 else 123_456_789


def require_grouped_writes(client: Client, values: dict[str, int]) -> None:
    results = client.write_tags(
        [BatchWriteItem(tag, value, value_type="DINT") for tag, value in values.items()]
    )
    failures = [
        f"{tag}: {result.error}"
        for tag, result in results.items()
        if not result.success
    ]
    if failures:
        raise RuntimeError("grouped write failures: " + "; ".join(failures))


def verify_dints(client: Client, expected: dict[str, int]) -> None:
    for tag, value in expected.items():
        actual = client.read_tag(tag)
        if actual != value:
            raise RuntimeError(f"{tag}: expected {value}, received {actual}")


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--plc-address",
        default=os.environ.get("TEST_PLC_ADDRESS", "192.168.0.1:44818"),
    )
    parser.add_argument(
        "--plc-slot",
        type=int,
        default=int(os.environ.get("TEST_PLC_SLOT", "0")),
    )
    parser.add_argument(
        "--program",
        default=os.environ.get("TEST_PLC_PROGRAM", "TestProgram"),
    )
    parser.add_argument("--dry-run", action="store_true")
    parser.add_argument("--allow-writes", action="store_true")
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    print("Python companion feature gate")
    print(f"target={args.plc_address} slot={args.plc_slot} program={args.program}")
    print("writes are limited to four DINT test elements and are restored")

    if args.dry_run:
        print(
            "would-test binding=python "
            f"batch_read={len(BATCH_READ_TAGS)} "
            f"grouped_write={len(BATCH_WRITE_TAGS)} "
            f"whole_udt={len(WHOLE_UDT_TAGS)} "
            "controller_discovery=N/A program_discovery=N/A restore=yes"
        )
        return 0
    if not args.allow_writes:
        print(
            "FAIL: live mode requires --allow-writes; four dedicated DINT test "
            "elements will be changed and restored",
            file=sys.stderr,
        )
        return 1

    try:
        route = RoutePath(slots=[args.plc_slot])
        with Client(args.plc_address, route_path=route) as plc:
            print("Phase 1 — controller discovery: N/A (not exposed by Python 1.2.x)")
            print("Phase 2 — program discovery: N/A (not exposed by Python 1.2.x)")

            print("Phase 3 — whole UDT reads")
            for template in WHOLE_UDT_TAGS:
                tag = with_program(template, args.program)
                value = plc.read_tag(tag)
                if not isinstance(value, dict) or not value.get("data"):
                    raise RuntimeError(f"{tag}: empty or non-UDT result: {value!r}")
                print(f"  PASS: {tag} ({len(value['data'])} payload bytes)")

            print("Phase 4 — mixed-type native batch read")
            read_tags = [with_program(tag, args.program) for tag in BATCH_READ_TAGS]
            reads = plc.read_tags(read_tags)
            if set(reads) != set(read_tags):
                missing = sorted(set(read_tags) - set(reads))
                raise RuntimeError(f"batch read omitted tags: {missing}")
            print(f"  PASS: {len(reads)}/{len(read_tags)}")

            print("Phase 5 — restore-safe grouped write")
            print("  NOTE: Python 1.2.x preserves per-tag accuracy using sequential writes")
            write_tags = [with_program(tag, args.program) for tag in BATCH_WRITE_TAGS]
            originals = {tag: plc.read_tag(tag) for tag in write_tags}
            if any(type(value) is not int for value in originals.values()):
                raise RuntimeError(f"expected DINT originals, received {originals}")
            exercise = {
                tag: exercise_value(value) for tag, value in originals.items()
            }

            exercise_error: Exception | None = None
            try:
                require_grouped_writes(plc, exercise)
                verify_dints(plc, exercise)
            except Exception as error:  # restoration must still run
                exercise_error = error

            try:
                require_grouped_writes(plc, originals)
                verify_dints(plc, originals)
                print(f"  RESTORED: {len(originals)}/{len(originals)}")
            except Exception as restore_error:
                raise RuntimeError(f"RESTORE FAILED: {restore_error}") from restore_error

            if exercise_error is not None:
                raise exercise_error
            print("  PASS: grouped write and read-back")

        print("PASS: Python companion gate completed with original write values restored")
        return 0
    except Exception as error:
        print(f"FAIL: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
