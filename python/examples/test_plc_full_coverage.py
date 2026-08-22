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

import rust_ethernet_ip as eip_package  # noqa: E402
from rust_ethernet_ip import BatchWriteItem, Client, RoutePath  # noqa: E402


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
    ENCODING_BLOCKED_UDT_STRING_MEMBER = "encoding_blocked_udt_string_member"
    SERVICE_LAYER_WRITEABLE = "service_layer_writeable"

    @classmethod
    def from_manifest(cls, value: str) -> "Mode":
        try:
            return cls(value)
        except ValueError as exc:
            raise ValueError(f"unknown writeability: {value}") from exc

    def is_writeable(self) -> bool:
        return self in {Mode.WRITEABLE, Mode.SERVICE_LAYER_WRITEABLE}

    def is_expected_blocked(self) -> bool:
        return self == Mode.ENCODING_BLOCKED_UDT_STRING_MEMBER


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
    if k == Kind.STRING: return f"FC{rng.randrange(0x100000000):08X}"
    return None


def nines(k: Kind) -> object | None:
    if k == Kind.DINT: return 999_999
    if k == Kind.INT:  return 9_999
    if k == Kind.REAL: return 99.99
    if k == Kind.BOOL: return True
    if k == Kind.STRING: return "SETTLED"
    return None


def values_match(a, b, k: Kind) -> bool:
    if k == Kind.REAL: return math.isclose(a, b, abs_tol=1e-3)
    return a == b


def read_value(client: Client, tag: Tag) -> object:
    if tag.kind == Kind.STRING:
        return client.read_string(tag.name)
    return client.read_tag(tag.name)


def latency_summary(samples_ms: list[float], failures: int) -> dict[str, float | int]:
    ordered = sorted(samples_ms)
    total_ms = sum(ordered)

    def percentile(fraction: float) -> float:
        if not ordered:
            return 0.0
        return ordered[round((len(ordered) - 1) * fraction)]

    q1 = percentile(0.25)
    q3 = percentile(0.75)
    iqr = q3 - q1
    lower_fence = q1 - 1.5 * iqr
    upper_fence = q3 + 1.5 * iqr
    filtered = [sample for sample in ordered if lower_fence <= sample <= upper_fence]
    return {
        "samples": len(ordered),
        "failures": failures,
        "total_ms": total_ms,
        "avg_ms": total_ms / len(ordered) if ordered else 0.0,
        "min_ms": ordered[0] if ordered else 0.0,
        "p50_ms": percentile(0.50),
        "p95_ms": percentile(0.95),
        "p99_ms": percentile(0.99),
        "max_ms": ordered[-1] if ordered else 0.0,
        "ops_per_sec": len(ordered) * 1000.0 / total_ms if total_ms else 0.0,
        "outlier_method": "Tukey 1.5*IQR",
        "outlier_count": len(ordered) - len(filtered),
        "outlier_filtered_avg_ms": sum(filtered) / len(filtered) if filtered else 0.0,
    }


def run_batch_benchmark(client: Client, tags: list[Tag], args: argparse.Namespace) -> int:
    if not args.allow_writes:
        print("batch benchmark writes terminal DINT values; rerun with --allow-writes", file=sys.stderr)
        return 2
    sizes = [1, 5, 10, 20, 50, 100]
    pool = [
        tag for tag in tags
        if tag.category == "ctrl.DINT_array" and tag.kind == Kind.DINT and tag.mode.is_writeable()
    ][:100]
    if len(pool) != 100:
        raise RuntimeError(f"batch benchmark requires 100 controller DINT array tags, found {len(pool)}")
    rows = []
    print(
        f"Batch benchmark — min {args.batch_min_tag_operations} tag operations and "
        f"{args.batch_min_seconds:.1f}s per size/direction"
    )
    for size in sizes:
        required_batches = (args.batch_min_tag_operations + size - 1) // size
        selected = pool[:size]
        names = [tag.name for tag in selected]
        writes = [BatchWriteItem(tag.name, 999_999, value_type="DINT") for tag in selected]
        for _ in range(10):
            if len(client.read_tags(names)) != size:
                raise RuntimeError(f"batch read warm-up failed at size {size}")
            warm_writes = client.write_tags(writes)
            if any(not result.success for result in warm_writes.values()):
                raise RuntimeError(f"grouped write warm-up failed at size {size}")

        read_samples: list[float] = []
        read_failures = 0
        window = time.perf_counter()
        while len(read_samples) + read_failures < required_batches or time.perf_counter() - window < args.batch_min_seconds:
            started = time.perf_counter_ns()
            try:
                result = client.read_tags(names)
                elapsed = (time.perf_counter_ns() - started) / 1_000_000
                if len(result) == size:
                    read_samples.append(elapsed)
                else:
                    read_failures += 1
            except Exception:
                read_failures += 1

        write_samples: list[float] = []
        write_failures = 0
        window = time.perf_counter()
        while len(write_samples) + write_failures < required_batches or time.perf_counter() - window < args.batch_min_seconds:
            started = time.perf_counter_ns()
            try:
                result = client.write_tags(writes)
                elapsed = (time.perf_counter_ns() - started) / 1_000_000
                if len(result) == size and all(item.success for item in result.values()):
                    write_samples.append(elapsed)
                else:
                    write_failures += 1
            except Exception:
                write_failures += 1

        reads = latency_summary(read_samples, read_failures)
        write_metrics = latency_summary(write_samples, write_failures)
        reads["tags_per_sec"] = reads["ops_per_sec"] * size
        write_metrics["tags_per_sec"] = write_metrics["ops_per_sec"] * size
        print(
            f"  size {size:>3}: read avg={reads['avg_ms']:>7.3f}ms "
            f"filtered={reads['outlier_filtered_avg_ms']:>7.3f}ms; "
            f"write avg={write_metrics['avg_ms']:>7.3f}ms "
            f"filtered={write_metrics['outlier_filtered_avg_ms']:>7.3f}ms"
        )
        rows.append({"batch_size": size, "reads": reads, "writes": write_metrics})

    failures = sum(row[direction]["failures"] for row in rows for direction in ("reads", "writes"))
    terminal_verify_failures = sum(1 for tag in pool if client.read_tag(tag.name) != 999_999)
    result = {
        "schema_version": 1,
        "workload": "controller_dint_logical_batch_sizes",
        "binding": "python",
        "binding_version": eip_package.LIBRARY_VERSION or "unknown",
        "plc_address": args.plc_address,
        "plc_slot": args.plc_slot,
        "batch_sizes": sizes,
        "min_tag_operations_per_size_direction": args.batch_min_tag_operations,
        "min_seconds_per_size_direction": args.batch_min_seconds,
        "read_api": "native CIP multiple-service batch",
        "write_api": "public Python grouped-write API (sequential native operations in 1.2.x)",
        "packet_policy": "default: max 20 operations and 504 bytes per CIP packet",
        "rows": rows,
        "terminal_verify": {"ok": len(pool) - terminal_verify_failures, "fail": terminal_verify_failures},
        "result": "PASS" if failures == 0 and terminal_verify_failures == 0 else "FAIL",
    }
    os.makedirs(args.out_dir, exist_ok=True)
    stamp = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ")
    path = os.path.join(args.out_dir, f"python_batch_benchmark_{stamp}.json")
    with open(path, "w", encoding="utf-8") as fh:
        json.dump(result, fh, indent=2)
    print(f"wrote {path}")
    return 0 if failures == 0 and terminal_verify_failures == 0 else 1


def run_benchmark(client: Client, tags: list[Tag], args: argparse.Namespace) -> int:
    if not args.allow_writes:
        print("benchmark mode writes terminal values; rerun with --allow-writes", file=sys.stderr)
        return 2
    writeable = [tag for tag in tags if tag.mode.is_writeable()]
    read_samples: list[float] = []
    write_samples: list[float] = []
    read_failures = write_failures = 0
    print(f"Benchmark — {args.benchmark_passes} passes, {len(tags)} reads/pass, {len(writeable)} writes/pass")
    for pass_index in range(args.benchmark_passes):
        pass_start = time.perf_counter()
        for tag in tags:
            started = time.perf_counter_ns()
            try:
                read_value(client, tag)
                read_samples.append((time.perf_counter_ns() - started) / 1_000_000)
            except Exception:
                read_failures += 1
        print(f"  read pass {pass_index + 1}/{args.benchmark_passes}: {time.perf_counter() - pass_start:.1f}s")
    for pass_index in range(args.benchmark_passes):
        pass_start = time.perf_counter()
        for tag in writeable:
            value = nines(tag.kind)
            if value is None:
                continue
            started = time.perf_counter_ns()
            try:
                client.write_tag(tag.name, value, value_type=tag.kind.value)
                write_samples.append((time.perf_counter_ns() - started) / 1_000_000)
            except Exception:
                write_failures += 1
        print(f"  write pass {pass_index + 1}/{args.benchmark_passes}: {time.perf_counter() - pass_start:.1f}s")

    result = {
        "schema_version": 1,
        "workload": "full_coverage_manifest_sequential",
        "binding": "python",
        "binding_version": eip_package.LIBRARY_VERSION or "unknown",
        "plc_address": args.plc_address,
        "plc_slot": args.plc_slot,
        "passes": args.benchmark_passes,
        "tag_count": len(tags),
        "writeable_tag_count": len(writeable),
        "warmup": "one full read-only preflight pass",
        "reads": latency_summary(read_samples, read_failures),
        "writes": latency_summary(write_samples, write_failures),
        "result": "PASS" if read_failures == 0 and write_failures == 0 else "FAIL",
    }
    os.makedirs(args.out_dir, exist_ok=True)
    stamp = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ")
    path = os.path.join(args.out_dir, f"python_benchmark_{stamp}.json")
    with open(path, "w", encoding="utf-8") as fh:
        json.dump(result, fh, indent=2)
    print(json.dumps(result, indent=2))
    print(f"wrote {path}")
    return 0 if result["result"] == "PASS" else 1


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
        ("ctrl.STRING", Tag("gTest_STRING", "ctrl.STRING", Kind.STRING, Mode.WRITEABLE), "SETTLED"),
        ("ctrl.UDT_members", Tag("gTestUDT.Member5_String", "ctrl.UDT_members", Kind.STRING, Mode.WRITEABLE), "SETTLED"),
        ("prog.BOOL_array", Tag("Program:TestProgram.gTestArray_BOOL[5]", "prog.BOOL_array", Kind.BOOL, Mode.WRITEABLE), True),
        ("prog.DINT_array", Tag("Program:TestProgram.gTestArray_DINT[42]", "prog.DINT_array", Kind.DINT, Mode.WRITEABLE), 999_999),
        ("prog.REAL_array", Tag("Program:TestProgram.gTestArray_REAL[10]", "prog.REAL_array", Kind.REAL, Mode.WRITEABLE), 99.99),
        ("prog.UDT_members", Tag("Program:TestProgram.gTestUDT.Member1_DINT", "prog.UDT_members", Kind.DINT, Mode.WRITEABLE), 999_999),
        ("prog.UDT_nested", Tag("Program:TestProgram.gTestUDT.Array_DINT[5]", "prog.UDT_nested", Kind.DINT, Mode.WRITEABLE), 999_999),
        ("prog.UDTarr_elem_nested", Tag("Program:TestProgram.gTestUDT_Array[2].Array_DINT[3]", "prog.UDTarr_elem_nested", Kind.DINT, Mode.WRITEABLE), 999_999),
        ("prog.STRING", Tag("Program:TestProgram.gTest_STRING", "prog.STRING", Kind.STRING, Mode.WRITEABLE), "SETTLED"),
        ("prog.UDT_members", Tag("Program:TestProgram.gTestUDT.Member5_String", "prog.UDT_members", Kind.STRING, Mode.WRITEABLE), "SETTLED"),
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
    parser.add_argument("--benchmark-passes", type=int, default=0)
    parser.add_argument("--batch-benchmark", action="store_true")
    parser.add_argument("--batch-min-tag-operations", type=int, default=1_000)
    parser.add_argument("--batch-min-seconds", type=float, default=30.0)
    parser.add_argument("--allow-writes", action="store_true")
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
    blocked = sum(1 for t in tags if t.mode.is_expected_blocked())
    readonly = sum(1 for t in tags if t.mode == Mode.READ_ONLY)

    print("Python wrapper — full-coverage exerciser")
    print(f"PLC: {address} (slot {slot})  total tags: {len(tags)}")
    print(f"  writeable: {writeable}   expected-blocked: {blocked}   read-only: {readonly}")
    print()

    if args.dry_run:
        print(f"would-test binding=python tags={len(tags)} writeable={writeable} blocked={blocked} read_only={readonly}")
        return 0
    if args.benchmark_passes < 0:
        parser.error("--benchmark-passes must be zero or greater")
    if args.benchmark_passes and args.skip_preflight:
        parser.error("benchmark mode requires the warm-up/preflight pass")
    if args.batch_benchmark and args.skip_preflight:
        parser.error("batch benchmark requires the warm-up/preflight pass")
    if args.batch_benchmark and args.benchmark_passes:
        parser.error("choose either sequential or batch benchmark mode")
    if args.batch_min_tag_operations <= 0 or args.batch_min_seconds < 0:
        parser.error("batch minimum samples must be positive and seconds non-negative")

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
                    read_value(client, tag)
                    preflight_ok += 1
                except Exception as exc:
                    preflight_fail += 1
                    print(f"setup-error: tag {tag.name} failed preflight ({exc}) — verify the PLC project against docs/PLC_TEST_TAG_DEFINITIONS.md", file=sys.stderr)
            print(f"  done in {time.perf_counter()-tp:.1f}s  preflight={preflight_ok}/{preflight_ok + preflight_fail}")
            if preflight_fail:
                return 2

        if args.benchmark_passes:
            return run_benchmark(client, tags, args)
        if args.batch_benchmark:
            return run_batch_benchmark(client, tags, args)

        print("Phase 1 — read every tag")
        t0 = time.perf_counter()
        for tag in tags:
            try:
                read_value(client, tag); S(tag.category).read_ok += 1
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
                actual = read_value(client, tag)
                if values_match(actual, expected, tag.kind):
                    S(tag.category).verify_ok += 1
                else:
                    S(tag.category).verify_fail += 1
            except Exception:
                S(tag.category).verify_fail += 1
        print(f"  done in {time.perf_counter()-t2:.1f}s")

        print("Phase 4 — confirm expected-blocked writes are still rejected")
        t3 = time.perf_counter()
        for tag in tags:
            if not tag.mode.is_expected_blocked(): continue
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
                actual = read_value(client, tag)
                if values_match(actual, expected, tag.kind):
                    settle_verify_ok += 1
                    print(f"  verify-settle  {category:<28} {tag.name:<48} OK")
                else:
                    settle_verify_fail += 1
                    print(f"  verify-settle  {category:<28} {tag.name:<48} FAIL MISMATCH: expected {expected!r}, got {actual!r}")
            except Exception as exc:
                settle_verify_fail += 1
                print(f"  verify-settle  {category:<28} {tag.name:<48} FAIL READ ERROR: {exc}")
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
                "phase4_blocked": {"ok": T.blocked_ok, "fail": T.blocked_unexpected, "note": "expected current-encoding rejections"},
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
