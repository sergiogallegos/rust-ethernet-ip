# rust-ethernet-ip-udt

Logix UDT parsing and serialization helpers for the [`rust-ethernet-ip`](https://crates.io/crates/rust-ethernet-ip) ecosystem.

Operates on the `UdtData` envelope (raw `{ symbol_id, data }` bytes returned by the PLC) and a `UdtDefinition` (member layout) to:

- Parse `UdtData` bytes into a `HashMap<String, PlcValue>` keyed by member name
- Serialize a `HashMap<String, PlcValue>` back into `UdtData` bytes for write-back
- Read or write a single named member with correct byte-offset and DWORD-bit handling
- Walk nested arrays inside UDTs (e.g. `Array_DINT[5]` inside a UDT element)

Built on `rust-ethernet-ip-types` for the shared `PlcValue` / `UdtData` model.

## Who should use this crate

Most consumers want the top-level [`rust-ethernet-ip`](https://crates.io/crates/rust-ethernet-ip) crate, which uses these helpers internally and exposes `EipClient::read_udt_chunked` / `read_udt_member` / `write_udt_member` directly.

Depend on `rust-ethernet-ip-udt` directly only if you are building an out-of-band UDT-mapping layer (schema export, code generation against L5X exports, byte-level UDT auditing) without the network client.

## Allen-Bradley firmware notes

UDT member-write quirks that are firmware-side, not library bugs — direct writes to STRING members and to UDT-array-element members reject with CIP `0x2107`. The top-level `rust-ethernet-ip` crate exposes service-layer helpers (`write_udt_member`, `write_string_tag`, `write_udt_array_member`) that package the read-modify-write workaround.

## License

MIT. Part of the [`rust-ethernet-ip`](https://github.com/sergiogallegos/rust-ethernet-ip) workspace.
