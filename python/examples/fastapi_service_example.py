from __future__ import annotations

import os

try:
    from fastapi import FastAPI, HTTPException
except ImportError as exc:
    raise SystemExit(
        "This example requires FastAPI. Install it with "
        "`pip install '.[api]'` from the python/ directory."
    ) from exc

try:
    import uvicorn
except ImportError:
    uvicorn = None

from rust_ethernet_ip import Client, PlcOperationError


PLC_ADDRESS = os.environ.get("RUST_ETHERNET_IP_PLC_ADDRESS", "192.168.0.10:44818")
API_HOST = os.environ.get("RUST_ETHERNET_IP_API_HOST", "0.0.0.0")
API_PORT = int(os.environ.get("RUST_ETHERNET_IP_API_PORT", "8000"))
app = FastAPI(title="rust-ethernet-ip example service")


@app.get("/health")
def health() -> dict[str, str]:
    return {"status": "ok"}


@app.get("/diagnostics")
def diagnostics(detailed: bool = False) -> dict[str, object]:
    try:
        with Client(PLC_ADDRESS) as plc:
            snapshot = plc.get_diagnostics_snapshot(detailed=detailed)
    except PlcOperationError as exc:
        raise HTTPException(status_code=502, detail=str(exc)) from exc

    return {
        "captured_at_unix_ms": snapshot.captured_at_unix_ms,
        "system_metrics_are_placeholders": snapshot.system_metrics_are_placeholders,
        "health": snapshot.health.__dict__,
        "errors": snapshot.errors.__dict__,
    }


@app.get("/tags/{tag_name}")
def read_tag(tag_name: str) -> dict[str, object]:
    try:
        with Client(PLC_ADDRESS) as plc:
            value = plc.read_tag(tag_name)
    except PlcOperationError as exc:
        raise HTTPException(status_code=502, detail=str(exc)) from exc

    return {"tag_name": tag_name, "value": value}


if __name__ == "__main__":
    if uvicorn is None:
        raise SystemExit(
            "This example requires uvicorn. Install it with "
            "`pip install '.[api]'` from the python/ directory."
        )
    uvicorn.run(app, host=API_HOST, port=API_PORT)
