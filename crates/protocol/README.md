# rust-ethernet-ip-protocol

EtherNet/IP wire-protocol codecs for the [`rust-ethernet-ip`](https://crates.io/crates/rust-ethernet-ip) ecosystem.

Pure encode/decode boundary for the EtherNet/IP encapsulation header, CIP framing, CPF items, and PLC value serialization. Operates only on `bytes::BytesMut` / `&[u8]` — no I/O, no async, no client state.

Submodules:

- `encap` — EtherNet/IP encapsulation header (`SendRRData`, `RegisterSession`, `ListIdentity`, etc.)
- `cip` — CIP Connection Manager framing (`CipRequest`, `CipResponse`, `SendDataRequest`, `CpfItem`)
- `values` — `PlcValue` encode/decode and BOOL-array DWORD packing helpers

Public traits:

- `Encode` — infallible `fn encode(&self, &mut BytesMut)`
- `Decode` — `fn decode(buf: &mut impl Buf) -> Result<Self>`

## Who should use this crate

Most consumers want the top-level [`rust-ethernet-ip`](https://crates.io/crates/rust-ethernet-ip) crate.

Depend on `rust-ethernet-ip-protocol` directly only if you are building an alternative network layer (UDP discovery, custom connection management, packet replay tooling, etc.) and want the wire codecs without the rest of the client.

## Hardened against silent failures

- `CipRequest::encode` returns `Result` and validates empty paths, odd-length paths, and paths beyond the 510-byte CIP limit before writing any bytes
- Pinned-byte tests cover AB STRING padding, UDT type-tag synthesis, CIP error responses, and route-path encoding

## License

MIT. Part of the [`rust-ethernet-ip`](https://github.com/sergiogallegos/rust-ethernet-ip) workspace.
