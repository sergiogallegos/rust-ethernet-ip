"""Live companion runner for docs/validation/SCHEMA_CHANGE_GATE.md (Python leg).

Automates the repeatable, non-editing steps of the schema-change validation
procedure against a real controller: baseline capture, post-edit
read/recovery observation, explicit refresh_schema(), and restore-safe write
verification. Every Studio 5000 action stays manual and maintainer-controlled
- this tool only pauses on stdin between phases and never issues a schema
edit itself. Mirrors examples/schema_change_gate_live.rs (the Rust
companion) and examples/CSharpSchemaGateLive (the C# companion) phase for
phase; tag/attribute discovery is not exposed by the Python 1.2.x wrapper,
so that phase is reported N/A here, matching hardware_feature_gate.py.
"""

from __future__ import annotations

import argparse
import os
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

from rust_ethernet_ip import Client, RoutePath  # noqa: E402
from rust_ethernet_ip.types import DiagnosticsSchemaCacheMetrics  # noqa: E402

INDICES = [5, 40]


def describe(value: object) -> str:
    if isinstance(value, bool):
        return f"Bool({value})"
    if isinstance(value, int):
        return f"Dint({value})"
    if isinstance(value, float):
        return f"Real({value})"
    return f"{type(value).__name__}({value!r})"


def values_equal(a: object, b: object) -> bool:
    return type(a) is type(b) and a == b


def exercise(value: object) -> object:
    """Produces a distinguishable probe value of the same type, for a
    restore-safe write/read-back check. Only the two shapes this gate swaps
    between (DINT[] and packed BOOL[]) are supported."""
    if isinstance(value, bool):
        return not value
    if isinstance(value, int):
        return 123_456_788 if value == 123_456_789 else 123_456_789
    raise RuntimeError(
        f"unsupported schema-swap element type for a write probe: {type(value).__name__}"
    )


def write_and_verify(plc: Client, path: str, value: object) -> None:
    plc.write_tag(path, value)
    read_back = plc.read_tag(path)
    if not values_equal(read_back, value):
        raise RuntimeError(f"{path}: wrote {describe(value)}, read back {describe(read_back)}")


def print_metrics_delta(
    label: str,
    before: DiagnosticsSchemaCacheMetrics,
    after: DiagnosticsSchemaCacheMetrics,
) -> None:
    print(f"  {label}:")
    print(
        f"    generation: {before.generation} -> {after.generation} "
        f"({after.generation - before.generation:+d})"
    )
    print(
        f"    refreshes: {before.refreshes} -> {after.refreshes} "
        f"({after.refreshes - before.refreshes:+d})"
    )
    print(
        "    array classification hits/misses/evictions: "
        f"{before.array_classification_hits}/{before.array_classification_misses}/"
        f"{before.array_classification_evictions} -> "
        f"{after.array_classification_hits}/{after.array_classification_misses}/"
        f"{after.array_classification_evictions}"
    )
    print(
        f"    datatype contradictions: {before.datatype_contradictions} -> "
        f"{after.datatype_contradictions} "
        f"({after.datatype_contradictions - before.datatype_contradictions:+d})"
    )
    print(
        "    read recoveries succeeded/failed: "
        f"{before.successful_read_recoveries}/{before.failed_read_recoveries} -> "
        f"{after.successful_read_recoveries}/{after.failed_read_recoveries}"
    )


def pause_for_studio5000(message: str) -> None:
    print()
    print("=== MAINTAINER ACTION REQUIRED ===")
    print(message)
    print("This tool never edits controller schema. Perform the Studio 5000 action now.")
    input("Press Enter once the change is downloaded and online: ")


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
    parser.add_argument("--tag", default="gSchemaSwap")
    parser.add_argument("--dry-run", action="store_true")
    parser.add_argument("--allow-writes", action="store_true")
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    print("Schema-change live gate companion (Python)")
    print(
        f"target={args.plc_address} slot={args.plc_slot} program={args.program} "
        f"tag={args.tag} allow_writes={args.allow_writes}"
    )
    print("This tool never edits controller schema; every Studio 5000 action stays manual.")

    if args.dry_run:
        print(
            f"would-test scopes=controller,program indices={INDICES} "
            f"allow_writes={args.allow_writes}"
        )
        return 0
    if not args.allow_writes:
        print(
            "FAIL: live mode requires --allow-writes; dedicated gSchemaSwap "
            "elements will be changed and restored",
            file=sys.stderr,
        )
        return 1

    try:
        route = RoutePath(slots=[args.plc_slot])
        with Client(args.plc_address, route_path=route) as plc:
            print(f"Phase 0 — connected; healthy={plc.check_health()}")

            scopes = [
                ("controller", args.tag),
                ("program", f"Program:{args.program}.{args.tag}"),
            ]

            baseline_metrics = plc.get_diagnostics_snapshot().schema_cache
            print(
                "Phase 1 — baseline schema_cache_metrics: "
                f"generation={baseline_metrics.generation} refreshes={baseline_metrics.refreshes}"
            )

            print("Phase 2 — pre-edit reads (twice, to warm classification cache)")
            pre_edit_values: list[tuple[str, object]] = []
            for scope_name, base in scopes:
                for index in INDICES:
                    path = f"{base}[{index}]"
                    first = plc.read_tag(path)
                    second = plc.read_tag(path)
                    if not values_equal(first, second):
                        raise RuntimeError(
                            f"{path}: unstable read before any edit: "
                            f"{describe(first)} then {describe(second)}"
                        )
                    print(f"  {scope_name} {path} = {describe(second)}")
                    pre_edit_values.append((path, second))

            print("Phase 3 — restore-safe pre-edit write smoke check")
            for path, original in pre_edit_values:
                probe = exercise(original)
                write_and_verify(plc, path, probe)
                write_and_verify(plc, path, original)
                print(f"  {path}: exercised and restored to {describe(original)}")

            pause_for_studio5000(
                f"Move any test-only references off '{args.tag}', delete the unused "
                f"original, and rename the replacement to '{args.tag}' — for both "
                "controller and program scope."
            )

            print("Phase 4 — post-edit reads without calling refresh_schema() first")
            pre_refresh_metrics = plc.get_diagnostics_snapshot().schema_cache
            post_edit_values: list[tuple[str, object]] = []
            for scope_name, base in scopes:
                for index in INDICES:
                    path = f"{base}[{index}]"
                    try:
                        value = plc.read_tag(path)
                        print(
                            f"  {scope_name} {path} = {describe(value)} "
                            "(automatic recovery applies if the type changed)"
                        )
                        post_edit_values.append((path, value))
                    except Exception as error:  # noqa: BLE001 - observation phase, not fatal
                        print(f"  {scope_name} {path}: read error before refresh: {error}")
            post_read_metrics = plc.get_diagnostics_snapshot().schema_cache
            print_metrics_delta(
                "automatic recovery (no explicit refresh yet)",
                pre_refresh_metrics,
                post_read_metrics,
            )

            print("Phase 5 — explicit refresh_schema()")
            plc.refresh_schema()
            post_refresh_metrics = plc.get_diagnostics_snapshot().schema_cache
            if (
                post_refresh_metrics.generation != pre_refresh_metrics.generation + 1
                or post_refresh_metrics.refreshes != pre_refresh_metrics.refreshes + 1
            ):
                raise RuntimeError(
                    "refresh_schema() did not advance generation/refresh count by exactly "
                    f"one: before=(gen={pre_refresh_metrics.generation}, "
                    f"refreshes={pre_refresh_metrics.refreshes}) "
                    f"after=(gen={post_refresh_metrics.generation}, "
                    f"refreshes={post_refresh_metrics.refreshes})"
                )
            print(f"  generation now {post_refresh_metrics.generation}")

            print("Phase 6 — rediscovery: N/A (not exposed by the Python 1.2.x wrapper)")

            print("Phase 7 — post-refresh reads")
            post_refresh_values: list[tuple[str, object]] = []
            for scope_name, base in scopes:
                for index in INDICES:
                    path = f"{base}[{index}]"
                    value = plc.read_tag(path)
                    print(f"  {scope_name} {path} = {describe(value)}")
                    post_refresh_values.append((path, value))

            print("Phase 8 — restore-safe post-refresh write/verify")
            for path, current in post_refresh_values:
                probe = exercise(current)
                write_and_verify(plc, path, probe)
                write_and_verify(plc, path, current)
                print(
                    f"  {path}: exercised the new addressing shape and restored to "
                    f"{describe(current)}"
                )

            final_metrics = plc.get_diagnostics_snapshot().schema_cache
            print()
            print("=== Paste into the dated validation record ===")
            print(
                "session survived: yes (single connection held for the entire run; "
                f"healthy={plc.check_health()})"
            )
            print_metrics_delta("baseline -> final", baseline_metrics, final_metrics)
            print("Python: PASS")

        return 0
    except Exception as error:  # noqa: BLE001 - top-level result reporting
        print(f"FAIL: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
