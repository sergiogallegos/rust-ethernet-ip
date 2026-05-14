# Route-Path Behavior

## Summary

The current implementation treats route-path support as stable for the validated `0.7.0` scenarios. For ControlLogix, the route path must be carried through Unconnected Send handling rather than prepended to the symbolic service request path.

## Current Understanding

- `confirmed`: The current library behavior for routed Logix access is aligned with the documented implementation and with real-hardware validation on `1756-L81ES` via `1756-EN3TR` slot `0`.
- `confirmed`: The route path is appended as part of Unconnected Send handling, which resolved the prior path-segment error behavior.
- `confirmed`: Route-path tests passed on both the CompactLogix validation pass and the routed ControlLogix validation pass.
- `confirmed`: The common backplane route path format in this repo is `RoutePath::new().add_slot(slot)`, encoding backplane port `1` plus the CPU slot.
- `confirmed`: The pre-`0.8.0` route model grouped `slots`, `ports`, and `addresses` separately, so builder call order could not express mixed topologies such as backplane -> Ethernet -> backplane.
- `confirmed`: The current `0.8.0` draft Rust route model now stores ordered `RouteHop` entries while preserving the legacy grouped fields for compatibility.
- `confirmed`: Ethernet hops in the current draft are encoded as extended link-address segments using an ASCII/NUL link address, rather than the earlier unvalidated raw IPv4-octet shape.
- `confirmed`: Empty ordered-hop lists fall back to legacy grouped-field encoding so direct public-field construction from older code does not silently become an empty route.
- `likely`: Ordered route modeling is the correct API direction for multi-hop CIP routing, but only the single-hop ControlLogix backplane route has real-hardware validation in this repo so far.

## Practical Guidance

- Use direct connect for integrated-Ethernet CompactLogix unless a route path is specifically required by the environment.
- Use `RoutePath::new().add_slot(slot)` for ControlLogix chassis access when the CPU is reached through an Ethernet module.
- For new Rust code that needs mixed route ordering, prefer the ordered hop builders such as `add_ethernet(...)` and `add_ethernet_with_port(...)` instead of relying on separate slot/port/address collections.
- Treat complex multi-hop routing as supported by the Rust API design, but still requiring real-hardware validation before being documented as confirmed behavior.

## Evidence

- [docs/CONTROLLOGIX_ROUTING_IMPLEMENTATION.md](../../docs/CONTROLLOGIX_ROUTING_IMPLEMENTATION.md)
- [docs/EtherNetIP_Connection_Paths_and_Routing.md](../../docs/EtherNetIP_Connection_Paths_and_Routing.md)
- [docs/validation/2026-04-07_real_plc_1756-L81ES_via_1756-EN3TR_slot0.md](../../docs/validation/2026-04-07_real_plc_1756-L81ES_via_1756-EN3TR_slot0.md)
- [docs/validation/2026-04-07_real_plc_5069-L320ERMS3_fw35.md](../../docs/validation/2026-04-07_real_plc_5069-L320ERMS3_fw35.md)
- [src/route.rs](../../src/route.rs)
- [tests/udt_discovery_tests.rs](../../tests/udt_discovery_tests.rs)

## Open Questions

- Whether the wiki should split route-path behavior into separate pages for connection-path theory, implementation mechanics, and validation evidence once more hardware topologies are added.
- Whether multi-hop Ethernet routing should get its own validation synthesis page after dedicated field testing.
- Whether wrapper route APIs should grow an ordered-hop FFI shape instead of passing legacy grouped slot/port/address arrays.

## Related Pages

- [../releases/0.7.0-validation-synthesis.md](../releases/0.7.0-validation-synthesis.md)
