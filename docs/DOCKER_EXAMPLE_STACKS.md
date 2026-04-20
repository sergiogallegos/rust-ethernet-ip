# Docker Example Stacks

Date: 2026-04-19

## Summary

This document describes the first local Docker stack for the Python service examples.

The goal is:

- make the Python API and collector examples easier to run
- keep Docker concerns outside the Rust core crate
- provide a realistic local integration shape for future PLC validation

## Current Stack

The first stack lives under:

- `docker/python-stack/`

It includes:

- `api`: FastAPI wrapper service
- `collector`: batch-first collector writing to SQLite
- `mqtt`: optional Eclipse Mosquitto broker
- `mqtt-publisher`: optional snapshot publisher service

## Files

- `docker/python-stack/Dockerfile`
- `docker/python-stack/docker-compose.yml`
- `docker/python-stack/config/collector_config.json`
- `docker/python-stack/config/mqtt_publisher_config.json`
- `docker/python-stack/mosquitto.conf`

## Usage

Set the PLC address in your environment:

```bash
export RUST_ETHERNET_IP_PLC_ADDRESS=192.168.1.10:44818
```

Start the API and collector:

```bash
docker compose -f docker/python-stack/docker-compose.yml up --build
```

Start the MQTT profile too:

```bash
docker compose -f docker/python-stack/docker-compose.yml --profile mqtt up --build
```

## Notes

- The stack does not bundle a PLC simulator.
- It is intended as a local deployment pattern for a reachable PLC.
- The Python services use environment overrides so the container config files can stay stable.

## Design Guardrail

- Rust remains the protocol core.
- Docker only packages wrapper and service examples.
- MQTT and HTTP stay above the wrapper boundary.
