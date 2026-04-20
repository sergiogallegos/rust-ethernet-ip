from __future__ import annotations

import argparse
import json
import os
import sys
import time
from dataclasses import dataclass
from datetime import datetime, timezone
from pathlib import Path

try:
    import paho.mqtt.client as mqtt
except ImportError as exc:
    raise SystemExit(
        "This example requires paho-mqtt. Install it with "
        "`pip install '.[mqtt]'` from the python/ directory."
    ) from exc

from rust_ethernet_ip import BatchReadError, Client, PlcConnectionError, PlcOperationError


@dataclass(slots=True)
class PlcConfig:
    address: str
    name: str
    site: str


@dataclass(slots=True)
class PollingConfig:
    interval_ms: int
    max_retry_count: int
    retry_delay_ms: int


@dataclass(slots=True)
class MqttConfig:
    host: str
    port: int
    topic_root: str
    client_id: str
    qos: int
    retain: bool


@dataclass(slots=True)
class PublisherConfig:
    plc: PlcConfig
    polling: PollingConfig
    tags: list[str]
    mqtt: MqttConfig


def load_config(path: Path) -> PublisherConfig:
    payload = json.loads(path.read_text(encoding="utf-8"))

    tags = payload.get("tags")
    if not isinstance(tags, list) or not tags or not all(isinstance(tag, str) for tag in tags):
        raise ValueError("config.tags must be a non-empty list of strings")

    plc_payload = payload.get("plc", {})
    plc_address = plc_payload.get("address")
    plc_address = os.environ.get("RUST_ETHERNET_IP_PLC_ADDRESS", plc_address)
    plc_name = plc_payload.get("name")
    plc_site = plc_payload.get("site")
    if not isinstance(plc_address, str) or not plc_address.strip():
        raise ValueError("config.plc.address must be a non-empty string")
    if not isinstance(plc_name, str) or not plc_name.strip():
        raise ValueError("config.plc.name must be a non-empty string")
    if not isinstance(plc_site, str) or not plc_site.strip():
        raise ValueError("config.plc.site must be a non-empty string")

    polling_payload = payload.get("polling", {})
    interval_ms = int(polling_payload.get("interval_ms", 1000))
    max_retry_count = int(polling_payload.get("max_retry_count", 3))
    retry_delay_ms = int(polling_payload.get("retry_delay_ms", 1000))
    if interval_ms <= 0:
        raise ValueError("config.polling.interval_ms must be > 0")
    if max_retry_count < 0:
        raise ValueError("config.polling.max_retry_count must be >= 0")
    if retry_delay_ms < 0:
        raise ValueError("config.polling.retry_delay_ms must be >= 0")

    mqtt_payload = payload.get("mqtt", {})
    host = mqtt_payload.get("host")
    host = os.environ.get("RUST_ETHERNET_IP_MQTT_HOST", host)
    topic_root = mqtt_payload.get("topic_root", "factory")
    client_id = mqtt_payload.get("client_id", "rust-ethernet-ip-publisher")
    port = int(mqtt_payload.get("port", 1883))
    qos = int(mqtt_payload.get("qos", 0))
    retain = bool(mqtt_payload.get("retain", False))
    if not isinstance(host, str) or not host.strip():
        raise ValueError("config.mqtt.host must be a non-empty string")
    if not isinstance(topic_root, str) or not topic_root.strip():
        raise ValueError("config.mqtt.topic_root must be a non-empty string")
    if not isinstance(client_id, str) or not client_id.strip():
        raise ValueError("config.mqtt.client_id must be a non-empty string")
    if not 0 <= qos <= 2:
        raise ValueError("config.mqtt.qos must be between 0 and 2")

    return PublisherConfig(
        plc=PlcConfig(address=plc_address, name=plc_name, site=plc_site),
        polling=PollingConfig(
            interval_ms=interval_ms,
            max_retry_count=max_retry_count,
            retry_delay_ms=retry_delay_ms,
        ),
        tags=tags,
        mqtt=MqttConfig(
            host=host,
            port=port,
            topic_root=topic_root,
            client_id=client_id,
            qos=qos,
            retain=retain,
        ),
    )


def build_snapshot_payload(
    plc_name: str,
    values: dict[str, object],
    errors: dict[str, str],
) -> dict[str, object]:
    return {
        "timestamp_utc": datetime.now(timezone.utc).isoformat(),
        "plc_name": plc_name,
        "values": values,
        "errors": errors,
    }


def build_snapshot_topic(config: PublisherConfig) -> str:
    topic_root = config.mqtt.topic_root.rstrip("/")
    return f"{topic_root}/{config.plc.site}/plc/{config.plc.name}/snapshot"


def collect_snapshot(client: Client, tags: list[str]) -> tuple[dict[str, object], dict[str, str]]:
    try:
        return client.read_tags(tags), {}
    except BatchReadError as exc:
        return exc.partial_values, exc.errors


def publish_loop(config: PublisherConfig, *, once: bool, cycle_limit: int | None) -> int:
    publisher = mqtt.Client(mqtt.CallbackAPIVersion.VERSION2, client_id=config.mqtt.client_id)
    publisher.connect(config.mqtt.host, config.mqtt.port, keepalive=60)
    publisher.loop_start()

    topic = build_snapshot_topic(config)
    client: Client | None = None
    cycles_completed = 0
    retries = 0

    try:
        while True:
            try:
                if client is None or not client.is_connected:
                    client = Client(config.plc.address)
                    retries = 0
                    print(f"[mqtt] connected to PLC {config.plc.address}")

                values, errors = collect_snapshot(client, config.tags)
                payload = build_snapshot_payload(config.plc.name, values, errors)
                publisher.publish(
                    topic,
                    json.dumps(payload, separators=(",", ":")),
                    qos=config.mqtt.qos,
                    retain=config.mqtt.retain,
                ).wait_for_publish()

                cycles_completed += 1
                print(f"[mqtt] published snapshot to {topic}")
                if errors:
                    print(f"[mqtt] partial batch errors: {errors}", file=sys.stderr)

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
                print(f"[mqtt] publish cycle error: {exc}", file=sys.stderr)

                if retries > config.polling.max_retry_count:
                    print("[mqtt] max retry count exceeded", file=sys.stderr)
                    return 1

                time.sleep(config.polling.retry_delay_ms / 1000.0)
    except KeyboardInterrupt:
        print("[mqtt] interrupted, shutting down")
        return 0
    finally:
        if client is not None:
            try:
                client.close()
            except PlcConnectionError:
                pass
        publisher.loop_stop()
        publisher.disconnect()


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="MQTT snapshot publisher example")
    parser.add_argument(
        "--config",
        type=Path,
        required=True,
        help="Path to MQTT publisher JSON config",
    )
    parser.add_argument(
        "--once",
        action="store_true",
        help="Publish exactly one snapshot and exit",
    )
    parser.add_argument(
        "--cycles",
        type=int,
        default=None,
        help="Publish a fixed number of cycles before exiting",
    )
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    config = load_config(args.config)
    return publish_loop(config, once=args.once, cycle_limit=args.cycles)


if __name__ == "__main__":
    raise SystemExit(main())
