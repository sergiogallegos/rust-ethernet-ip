# Unconnected Send and Route Path

Use this page when reviewing or modifying `EipClient::send_cip_request`, `EipClient::build_unconnected_send`, `EipClient::unwrap_unconnected_send_reply`, or anything that touches CIP request wrapping or `RoutePath`. The wrapping behavior is not what it looks like at a glance.

Verified against `src/client.rs` `send_cip_request` and `build_unconnected_send` as of the current main.

## Unconnected Send (service `0x52`) is the primary path, always

- Every CIP request from `EipClient` is wrapped in Unconnected Send by default — not just requests with a route path configured. See `EipClient::send_cip_request` calling `build_unconnected_send` unconditionally.
- The wrapping is not gated on `route_path.is_some()`. CLAUDE.md's wording suggesting otherwise is stale; trust the code.
- The `0x52` envelope carries: service code, request path size, request path (Connection Manager class `0x06` instance `0x01`), priority/tick fields, embedded message length, embedded CIP message, optional pad, and the route path bytes at the end when present.

## Route path goes at the **end** of the Unconnected Send envelope

- Per CIP spec, route path bytes are appended after the embedded CIP message, not embedded into the inner CIP request.
- `build_unconnected_send` handles this — `route_path_bytes` are pushed after the inner message.
- Do not insert route path bytes into the CIP request path. The controller will reject the request, and the failure mode (wrong path size word) is not obvious.

## Direct-CIP fallback exists for controllers that reject `0x52`

`send_cip_request` retries with a direct CIP `SendRRData` (no Unconnected Send wrapping) when **all** of these hold:

1. The Unconnected Send response service byte is `0xD2` (Unconnected Send reply marker).
2. The CIP general status byte is non-zero.
3. No route path is configured (`self.route_path.is_none()`).

This exists for controllers that reject the `0x52` pattern for specific services. The third condition matters: direct CIP cannot carry a route path, so the fallback is unsafe when routing is required.

Do not remove this fallback. Do not relax its conditions. If a future controller needs different fallback behavior, add a new branch — do not generalize the existing one.

## When `RoutePath` is configured

- `RoutePath` is set via `EipClient::with_route_path`. It encodes ControlLogix backplane and ethernet link hops (see `route.rs`).
- The route hops are added to the Unconnected Send envelope at request time. They do not change the inner CIP request bytes.
- For a CompactLogix (slot 0, no backplane routing), `RoutePath` is `None` and no route bytes are appended. The `0x52` wrapping still happens.

## Common mistakes this page exists to prevent

- "Skip the `0x52` wrapping when there's no route path, it's simpler." No — many controllers expect the wrapping for service routing through the Connection Manager even when there are zero route hops.
- "Embed the route path inside the CIP request." No — it goes at the end of the `0x52` envelope.
- "Remove the direct-CIP fallback, the test suite is green." No — the fallback exists for a class of controller behavior the unit tests don't cover. Removing it will surface as a `0xD2 / 0x0X` failure in the field.
