from __future__ import annotations

import argparse
import csv
import json
import os
import sqlite3
import sys
import time
from dataclasses import dataclass
from datetime import datetime, timezone
from pathlib import Path

from rust_ethernet_ip import BatchReadError, Client, PlcConnectionError, PlcOperationError


@dataclass(slots=True)
class PlcConfig:
    address: str


@dataclass(slots=True)
class PollingConfig:
    interval_ms: int
    max_retry_count: int
    retry_delay_ms: int


@dataclass(slots=True)
class SinkConfig:
    kind: str
    path: str


@dataclass(slots=True)
class CollectorConfig:
    plc: PlcConfig
    polling: PollingConfig
    tags: list[str]
    sink: SinkConfig


@dataclass(slots=True)
class SampleRow:
    timestamp_utc: str
    tag_name: str
    value_json: str
    value_type: str


class CsvSink:
    def __init__(self, path: Path):
        self._path = path
        self._path.parent.mkdir(parents=True, exist_ok=True)
        self._fieldnames = ["timestamp_utc", "tag_name", "value_json", "value_type"]

        if not self._path.exists() or self._path.stat().st_size == 0:
            with self._path.open("w", newline="", encoding="utf-8") as handle:
                writer = csv.DictWriter(handle, fieldnames=self._fieldnames)
                writer.writeheader()

    def write_rows(self, rows: list[SampleRow]) -> None:
        if not rows:
            return

        with self._path.open("a", newline="", encoding="utf-8") as handle:
            writer = csv.DictWriter(handle, fieldnames=self._fieldnames)
            for row in rows:
                writer.writerow(
                    {
                        "timestamp_utc": row.timestamp_utc,
                        "tag_name": row.tag_name,
                        "value_json": row.value_json,
                        "value_type": row.value_type,
                    }
                )


class SqliteSink:
    def __init__(self, path: Path):
        self._path = path
        self._path.parent.mkdir(parents=True, exist_ok=True)
        self._conn = sqlite3.connect(self._path)
        self._conn.execute(
            """
            CREATE TABLE IF NOT EXISTS plc_samples (
                timestamp_utc TEXT NOT NULL,
                tag_name TEXT NOT NULL,
                value_json TEXT NOT NULL,
                value_type TEXT NOT NULL
            )
            """
        )
        self._conn.commit()

    def write_rows(self, rows: list[SampleRow]) -> None:
        if not rows:
            return

        self._conn.executemany(
            """
            INSERT INTO plc_samples (timestamp_utc, tag_name, value_json, value_type)
            VALUES (?, ?, ?, ?)
            """,
            [
                (row.timestamp_utc, row.tag_name, row.value_json, row.value_type)
                for row in rows
            ],
        )
        self._conn.commit()

    def close(self) -> None:
        self._conn.close()


def load_config(path: Path) -> CollectorConfig:
    payload = json.loads(path.read_text(encoding="utf-8"))

    tags = payload.get("tags")
    if not isinstance(tags, list) or not tags or not all(isinstance(tag, str) for tag in tags):
        raise ValueError("config.tags must be a non-empty list of strings")

    plc_address = payload.get("plc", {}).get("address")
    plc_address = os.environ.get("RUST_ETHERNET_IP_PLC_ADDRESS", plc_address)
    if not isinstance(plc_address, str) or not plc_address.strip():
        raise ValueError("config.plc.address must be a non-empty string")

    sink_kind = payload.get("sink", {}).get("kind")
    sink_path = payload.get("sink", {}).get("path")
    sink_path = os.environ.get("RUST_ETHERNET_IP_SINK_PATH", sink_path)
    if sink_kind not in {"csv", "sqlite"}:
        raise ValueError("config.sink.kind must be 'csv' or 'sqlite'")
    if not isinstance(sink_path, str) or not sink_path.strip():
        raise ValueError("config.sink.path must be a non-empty string")

    interval_ms = int(payload.get("polling", {}).get("interval_ms", 1000))
    max_retry_count = int(payload.get("polling", {}).get("max_retry_count", 3))
    retry_delay_ms = int(payload.get("polling", {}).get("retry_delay_ms", 1000))

    if interval_ms <= 0:
        raise ValueError("config.polling.interval_ms must be > 0")
    if max_retry_count < 0:
        raise ValueError("config.polling.max_retry_count must be >= 0")
    if retry_delay_ms < 0:
        raise ValueError("config.polling.retry_delay_ms must be >= 0")

    return CollectorConfig(
        plc=PlcConfig(address=plc_address),
        polling=PollingConfig(
            interval_ms=interval_ms,
            max_retry_count=max_retry_count,
            retry_delay_ms=retry_delay_ms,
        ),
        tags=tags,
        sink=SinkConfig(kind=sink_kind, path=sink_path),
    )


def create_sink(config: SinkConfig) -> CsvSink | SqliteSink:
    path = Path(config.path)
    if config.kind == "csv":
        return CsvSink(path)
    return SqliteSink(path)


def normalize_values(timestamp_utc: str, values: dict[str, object]) -> list[SampleRow]:
    return [
        SampleRow(
            timestamp_utc=timestamp_utc,
            tag_name=tag_name,
            value_json=json.dumps(value),
            value_type=type(value).__name__,
        )
        for tag_name, value in values.items()
    ]


def collect_once(client: Client, tags: list[str]) -> tuple[list[SampleRow], dict[str, str]]:
    timestamp_utc = datetime.now(timezone.utc).isoformat()
    try:
        values = client.read_tags(tags)
        return normalize_values(timestamp_utc, values), {}
    except BatchReadError as exc:
        return normalize_values(timestamp_utc, exc.partial_values), exc.errors


def run_collector(config: CollectorConfig, *, once: bool, cycle_limit: int | None) -> int:
    sink = create_sink(config.sink)
    cycles_completed = 0
    retries = 0
    client: Client | None = None

    try:
        while True:
            try:
                if client is None or not client.is_connected:
                    client = Client(config.plc.address)
                    retries = 0
                    print(f"[collector] connected to {config.plc.address}")

                rows, errors = collect_once(client, config.tags)
                sink.write_rows(rows)
                cycles_completed += 1

                if rows:
                    print(
                        f"[collector] wrote {len(rows)} rows to {config.sink.kind} sink at {config.sink.path}"
                    )

                if errors:
                    print(f"[collector] partial batch errors: {errors}", file=sys.stderr)

                if once or (cycle_limit is not None and cycles_completed >= cycle_limit):
                    return 0

                time.sleep(config.polling.interval_ms / 1000.0)
            except (PlcConnectionError, PlcOperationError) as exc:
                if client is not None:
                    try:
                        client.close()
                    except PlcConnectionError:
                        pass
                client = None
                retries += 1
                print(f"[collector] collection error: {exc}", file=sys.stderr)

                if retries > config.polling.max_retry_count:
                    print("[collector] max retry count exceeded", file=sys.stderr)
                    return 1

                time.sleep(config.polling.retry_delay_ms / 1000.0)
    except KeyboardInterrupt:
        print("[collector] interrupted, shutting down")
        return 0
    finally:
        if client is not None:
            try:
                client.close()
            except PlcConnectionError:
                pass

        if isinstance(sink, SqliteSink):
            sink.close()


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Batch-first PLC collector service example")
    parser.add_argument(
        "--config",
        type=Path,
        required=True,
        help="Path to collector JSON config",
    )
    parser.add_argument(
        "--once",
        action="store_true",
        help="Collect exactly one batch snapshot and exit",
    )
    parser.add_argument(
        "--cycles",
        type=int,
        default=None,
        help="Collect a fixed number of cycles before exiting",
    )
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    try:
        config = load_config(args.config)
    except (OSError, ValueError, json.JSONDecodeError) as exc:
        print(f"[collector] failed to load config: {exc}", file=sys.stderr)
        return 1

    if args.cycles is not None and args.cycles <= 0:
        print("[collector] --cycles must be > 0", file=sys.stderr)
        return 1

    return run_collector(config, once=args.once, cycle_limit=args.cycles)


if __name__ == "__main__":
    raise SystemExit(main())
