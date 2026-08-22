import os

from rust_ethernet_ip import Client


def main() -> None:
    address = os.getenv("RUST_ETHERNET_IP_PLC_ADDRESS", "192.168.0.10:44818")
    with Client(address) as plc:
        plc.read_tag("ProductionCount")
        snapshot = plc.get_diagnostics_snapshot(detailed=True)

        print(f"healthy: {plc.check_health()}")
        print(f"reads: {snapshot.operations.total_reads}")
        print(f"failed reads: {snapshot.operations.failed_reads}")
        print(f"average read latency: {snapshot.performance.avg_read_latency_ms:.2f} ms")
        print(f"last error: {snapshot.errors.last_error_message}")


if __name__ == "__main__":
    main()
