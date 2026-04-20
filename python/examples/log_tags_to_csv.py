import csv
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

    with open("plc_snapshot.csv", "w", newline="", encoding="utf-8") as handle:
        writer = csv.writer(handle)
        writer.writerow(["timestamp_utc", "tag_name", "value"])
        timestamp = datetime.now(timezone.utc).isoformat()
        for tag_name, value in values.items():
            writer.writerow([timestamp, tag_name, value])


if __name__ == "__main__":
    main()
