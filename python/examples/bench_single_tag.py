"""Single-tag read/write microbench for the Python wrapper."""
from __future__ import annotations

import os
import statistics
import sys
import time

sys.path.insert(0, os.path.abspath(os.path.join(os.path.dirname(__file__), "..")))

from rust_ethernet_ip import Client, RoutePath  # noqa: E402


def report(name: str, samples_us: list[float], total_ms: float) -> None:
    samples_us.sort()
    n = len(samples_us)
    p50 = samples_us[n // 2] / 1000.0
    p95 = samples_us[(n * 95) // 100] / 1000.0
    p99 = samples_us[(n * 99) // 100] / 1000.0
    avg = total_ms / n
    ops = n / (total_ms / 1000.0)
    print(
        f"{name:<6} n={n}  total={total_ms:.1f}ms  avg={avg:.3f}ms  "
        f"p50={p50:.3f}ms  p95={p95:.3f}ms  p99={p99:.3f}ms  ops/sec={ops:.1f}"
    )


def main() -> int:
    address = os.environ.get("TEST_PLC_ADDRESS", "192.168.0.1:44818")
    slot = int(os.environ.get("TEST_PLC_SLOT", "0"))
    iters = int(os.environ.get("BENCH_ITERATIONS", "500"))
    tag = "gTestArray_DINT[0]"

    print("Python wrapper single-tag bench")
    print(f"PLC: {address} (slot {slot})  tag: {tag}  iterations: {iters}")

    with Client(address, route_path=RoutePath(slots=[slot])) as client:
        for _ in range(10):
            client.read_tag(tag)

        read_samples: list[float] = []
        read_start = time.perf_counter()
        for _ in range(iters):
            t = time.perf_counter()
            client.read_tag(tag)
            read_samples.append((time.perf_counter() - t) * 1_000_000.0)
        read_total_ms = (time.perf_counter() - read_start) * 1000.0

        write_samples: list[float] = []
        write_start = time.perf_counter()
        for i in range(iters):
            t = time.perf_counter()
            client.write_tag(tag, 100_000 + i, value_type="DINT")
            write_samples.append((time.perf_counter() - t) * 1_000_000.0)
        write_total_ms = (time.perf_counter() - write_start) * 1000.0

        client.write_tag(tag, 999_999, value_type="DINT")

    report("read", read_samples, read_total_ms)
    report("write", write_samples, write_total_ms)
    _ = statistics.median  # silence unused-import warning
    return 0


if __name__ == "__main__":
    sys.exit(main())
