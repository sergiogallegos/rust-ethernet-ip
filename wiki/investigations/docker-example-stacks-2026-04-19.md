# Docker Example Stacks

## Summary

The repo now has a first local Docker stack for the Python service examples.

The key conclusion is:

- Docker belongs at the example/service layer
- it should package the Python wrapper and example services
- it should not push HTTP, MQTT, or deployment concerns into the Rust core

## Current Understanding

- The first stack packages the FastAPI example, the collector example, and an optional MQTT publisher path.
- The stack uses environment overrides for PLC and broker addresses so the example configs can remain stable in-repo.
- The stack is designed for a reachable PLC and does not bundle a simulator.

## Evidence

- [docs/DOCKER_EXAMPLE_STACKS.md](../../docs/DOCKER_EXAMPLE_STACKS.md)
- [docker/python-stack/Dockerfile](../../docker/python-stack/Dockerfile)
- [docker/python-stack/docker-compose.yml](../../docker/python-stack/docker-compose.yml)
- [python/examples/fastapi_service_example.py](../../python/examples/fastapi_service_example.py)
- [python/examples/collector_service.py](../../python/examples/collector_service.py)
- [python/examples/mqtt_publisher_example.py](../../python/examples/mqtt_publisher_example.py)

## Open Questions

- Whether a future stack should include a simulator profile for local-only validation.
- Whether a later production-oriented stack should split the API and collector into separate runtime images.
- How much real-PLC validation should be gathered before treating the Docker stack as a recommended deployment starter.

## Related Pages

- [collector-service-mvp-design-2026-04-19.md](collector-service-mvp-design-2026-04-19.md)
- [rest-mqtt-adapter-boundaries-2026-04-19.md](rest-mqtt-adapter-boundaries-2026-04-19.md)
- [monitoring-diagnostics-plan-2026-04-19.md](monitoring-diagnostics-plan-2026-04-19.md)
