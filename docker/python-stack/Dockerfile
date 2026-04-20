FROM rust:1.95-bookworm AS builder

WORKDIR /workspace

COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY examples ./examples
COPY python ./python

RUN cargo build --release

FROM python:3.12-slim-bookworm

WORKDIR /app

ENV PYTHONUNBUFFERED=1
ENV RUST_ETHERNET_IP_NATIVE_LIB=/app/lib/librust_ethernet_ip.so

COPY --from=builder /workspace/target/release/librust_ethernet_ip.so /app/lib/librust_ethernet_ip.so
COPY python /app/python

RUN pip install --no-cache-dir /app/python[api,mqtt]

CMD ["python", "python/examples/fastapi_service_example.py"]
