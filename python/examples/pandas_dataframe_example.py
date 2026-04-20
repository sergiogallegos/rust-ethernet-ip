from __future__ import annotations

from datetime import datetime, timezone

try:
    import pandas as pd
except ImportError as exc:
    raise SystemExit(
        "This example requires pandas. Install it with "
        "`pip install '.[analytics]'` from the python/ directory."
    ) from exc

from rust_ethernet_ip import BatchReadError, Client


def main() -> None:
    tags = ["DINT_TAG", "REAL_TAG", "BOOL_TAG", "STRING_TAG"]

    with Client("192.168.0.10:44818") as plc:
        try:
            values = plc.read_tags(tags)
        except BatchReadError as exc:
            print("Batch read failed")
            print("Partial values:", exc.partial_values)
            print("Errors:", exc.errors)
            return

    timestamp = datetime.now(timezone.utc).isoformat()
    rows = [
        {
            "timestamp_utc": timestamp,
            "tag_name": tag_name,
            "value": value,
            "value_type": type(value).__name__,
        }
        for tag_name, value in values.items()
    ]

    frame = pd.DataFrame(rows)
    print(frame)


if __name__ == "__main__":
    main()
