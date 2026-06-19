FROM rust:1.96-bookworm AS builder

WORKDIR /workspace

COPY Cargo.toml Cargo.lock build.rs VERSION ./
COPY crates ./crates
COPY src ./src
COPY examples ./examples
COPY python ./python

# The Python wrapper loads the FFI exports, which are gated behind the `ffi`
# feature.
RUN cargo build --release --features ffi

FROM python:3.12-slim-bookworm

WORKDIR /app

ENV PYTHONUNBUFFERED=1
ENV RUST_ETHERNET_IP_NATIVE_LIB=/app/lib/librust_ethernet_ip.so

COPY --from=builder /workspace/target/release/librust_ethernet_ip.so /app/lib/librust_ethernet_ip.so
COPY python /app/python

RUN pip install --no-cache-dir /app/python[api,mqtt]

CMD ["python", "python/examples/fastapi_service_example.py"]
