import json
import sqlite3
from datetime import datetime, timezone

from rust_ethernet_ip import BatchReadError, Client


def main() -> None:
    tags = ["Tag1", "Tag2", "Program:Main.Tag3"]
    with Client("192.168.0.10:44818") as plc:
        try:
            values = plc.read_tags(tags)
        except BatchReadError as exc:
            print("Read failed:", exc.errors)
            return

    with sqlite3.connect("plc_data.sqlite") as conn:
        conn.execute(
            """
            CREATE TABLE IF NOT EXISTS plc_samples (
                timestamp_utc TEXT NOT NULL,
                tag_name TEXT NOT NULL,
                value_json TEXT NOT NULL
            )
            """
        )
        timestamp = datetime.now(timezone.utc).isoformat()
        conn.executemany(
            "INSERT INTO plc_samples (timestamp_utc, tag_name, value_json) VALUES (?, ?, ?)",
            [(timestamp, tag_name, json.dumps(value)) for tag_name, value in values.items()],
        )
        conn.commit()


if __name__ == "__main__":
    main()
