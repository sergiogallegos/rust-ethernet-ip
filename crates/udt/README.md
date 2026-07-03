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

UDT member-write behavior depends on the target member type and exact Logix wire encoding. Scalar UDT-array-element members are confirmed writeable on 5069-L330ERM fw38 when the full member path is preserved. STRING members inside UDTs and UDT-array elements reject with CIP `0x2107` under the current member encoding, so the top-level `rust-ethernet-ip` crate keeps a read-modify-write path for those cases.

## License

MIT. Part of the [`rust-ethernet-ip`](https://github.com/sergiogallegos/rust-ethernet-ip) workspace.
