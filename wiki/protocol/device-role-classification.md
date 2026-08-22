# EtherNet/IP Device-Role Classification

## Summary

The current library is best described as a **CIP explicit-messaging
client/originator for Logix data access**. This classification is confirmed by
the active transport and request path as of 2026-08-22.

It is not currently an EtherNet/IP I/O Scanner or I/O Adapter implementation.
Do not shorten the description to just "originator" without saying
"explicit-messaging": an I/O Scanner is also an originator, but it originates
implicit I/O connections that this library does not implement.

## Current Understanding

- The client opens TCP port 44818 and registers an EtherNet/IP encapsulation
  session.
- Operational CIP requests use `SendRRData`, primarily carrying Connection
  Manager Unconnected Send requests. They provide Logix tag reads, writes,
  batches, discovery, routing, and related explicit services.
- Registering an encapsulation session is not the same as opening a connected
  CIP Class 3 explicit-message connection.
- The current code does not provide cyclic Class 1 implicit I/O over UDP,
  requested packet intervals, assembly-data production/consumption, or I/O
  Forward Open behavior. It therefore is not an I/O Scanner.
- The library does not present itself to a Scanner as a cyclic I/O target. It
  therefore is not an I/O Adapter.
- A retired experimental connected STRING method remains only as an
  unsupported, deprecated compatibility surface. It does not make the current
  library a connected Class 3 implementation.
- The PLC or communication bridge/CPU route is the explicit-message target;
  the library is the request originator.

ODVA's class labels describe product capability sets. The project should avoid
claiming an ODVA conformance class or certification unless it has completed the
corresponding conformance process. Functional wording such as "explicit-
messaging client/originator" is more precise for project and website copy.

## Evidence

- [`src/client.rs`](../../src/client.rs) opens `TcpStream`, registers the
  encapsulation session, and sends live CIP traffic through `SendRRData`.
- [`docs/agents/notes/unconnected-send.md`](../../docs/agents/notes/unconnected-send.md)
  records the primary Unconnected Send path and direct-CIP fallback.
- [`src/client/string.rs`](../../src/client/string.rs) marks the former Class 3
  STRING path unsupported and deprecated.
- [`README.md`](../../README.md) and [`website/index.html`](../../website/index.html)
  use the explicit-messaging client-driver classification in public project
  copy as of 2026-08-22.
- [ODVA EtherNet/IP Technology Overview](https://www.odva.org/publication_download/ethernet-ip-technology-overview/)
  distinguishes explicit Messaging, I/O Adapter, and I/O Scanner capabilities.
- [ODVA Common Industrial Protocol and the Family of CIP Networks](https://www.odva.org/wp-content/uploads/2020/06/PUB00123R1_Common-Industrial_Protocol_and_Family_of_CIP_Networks.pdf)
  distinguishes Explicit Message Client, Explicit Message Server, I/O Adapter,
  and I/O Scanner roles.

## Open Questions

- Whether valid connected Class 3 explicit messaging should ever be added as a
  performance option is a separate roadmap decision.
- I/O Scanner or Adapter support would be a major new protocol/product scope,
  not an extension of the existing tag-read/write API.

## Related Pages

- [Route-path behavior](route-path-behavior.md)
- [ABI contract](abi-contract.md)
- [Software architecture map](../investigations/software-architecture-map.md)
