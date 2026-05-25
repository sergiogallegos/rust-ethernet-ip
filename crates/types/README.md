# rust-ethernet-ip-types

Shared PLC value types for the [`rust-ethernet-ip`](https://crates.io/crates/rust-ethernet-ip) ecosystem.

This crate exposes the wire-neutral data model that the protocol, UDT, and client crates all agree on:

- `PlcValue` — tagged enum covering every Allen-Bradley CIP scalar type (`Bool`, `Sint`, `Int`, `Dint`, `Lint`, `Usint`, `Uint`, `Udint`, `Ulint`, `Real`, `Lreal`, `String`, `Udt`)
- `UdtData` — `{ symbol_id, data }` envelope for Logix User-Defined Types
- `UdtCodec` — trait used by the `rust-ethernet-ip-udt` crate to parse/serialize UDT byte buffers
- `TypeError` — value-model error type

## Who should use this crate

Most consumers want the top-level [`rust-ethernet-ip`](https://crates.io/crates/rust-ethernet-ip) crate, which re-exports the user-facing items from this crate alongside the network client, FFI surface, and protocol codecs.

Depend on `rust-ethernet-ip-types` directly only if you are building an independent tooling layer (codec, transport, schema export, etc.) that needs the wire-neutral model without pulling in the full client.

## Usage

```rust
use rust_ethernet_ip_types::PlcValue;

let v = PlcValue::Dint(1500);
assert_eq!(v.to_string(), "Dint(1500)");
```

## License

MIT. Part of the [`rust-ethernet-ip`](https://github.com/sergiogallegos/rust-ethernet-ip) workspace.
