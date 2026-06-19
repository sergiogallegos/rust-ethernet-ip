# Migrating from 0.7.x to 1.0.0

`1.0.0` is a SemVer-major release that bundles every deferred breaking change.
This guide covers the API changes most likely to require source edits. See the
`[1.0.0]` entry in [CHANGELOG.md](../CHANGELOG.md) for the full list.

## RoutePath: ordered hop builders replace grouped public fields

`RoutePath` previously exposed grouped public fields (`slots`, `ports`,
`addresses`) that callers could construct directly. These public fields have
been removed in favor of ordered hop builders that preserve the exact sequence
of backplane and Ethernet hops on the wire.

Replace direct field construction with the builder methods:

- `add_backplane(slot)` — append a backplane hop to the given slot.
- `add_ethernet(address)` — append an Ethernet hop (default port).
- `add_ethernet_with_port(port, address)` — append an Ethernet hop on a
  specific port.

The grouped accessors `slots()`, `ports()`, and `addresses()` remain available
as **read-only views** over the ordered hops, so existing read paths that only
inspected route data continue to work. Only construction via the old public
fields needs to change.

The `RouteHop::Ethernet` variant now emits the spec-correct ASCII extended
link-address encoding instead of raw IPv4 octets; routed ControlLogix
connections that previously relied on the old encoding should be re-validated.

## Public enums are now `#[non_exhaustive]`

Public enums (including the error and value enums) are marked
`#[non_exhaustive]` so future minor releases can add variants without a major
bump. Any `match` over these enums must now include a wildcard arm:

```rust
match err {
    EtherNetIpError::Timeout { .. } => { /* ... */ }
    // ...
    _ => { /* required: handles future variants */ }
}
```

A non-wildcard match that compiled against 0.7.x will fail to compile until a
`_ =>` arm is added.

## Workspace now publishes five crates

The repository is now a Cargo workspace. The main `rust-ethernet-ip` crate
re-exports four publishable sibling crates:

- `rust-ethernet-ip-types` — `PlcValue`, `UdtData`, session/connection types.
- `rust-ethernet-ip-tag-path` — the `TagPath` parser.
- `rust-ethernet-ip-protocol` — the `Encode`/`Decode` wire codec.
- `rust-ethernet-ip-udt` — UDT discovery and serialization.

Most consumers do not need to change anything: the main crate continues to
re-export these types, so existing `use rust_ethernet_ip::...` paths still
resolve. Depend on a sibling crate directly only if you need its surface
without the full client.
