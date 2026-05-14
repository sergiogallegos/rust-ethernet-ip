---
id: CODEX-F
title: RoutePath ordered hops + ASCII-encoded ethernet link addresses
owner: codex
status: merged
created: 2026-05-14
last-update: 2026-05-14 claude
note: brief authored retroactively after implementation; see Verdict for the process explanation
---

## Brief

### Goal

Address a public GitHub issue reporting that the pre-0.8.0 `RoutePath` model could not represent ordered multi-hop CIP routes such as `backplane → ethernet → backplane`. The issue's structural diagnosis was correct, and review of the existing implementation surfaced a second latent bug: ethernet hops were encoded as raw IPv4 octets rather than the standard Allen-Bradley extended-link-address shape (ASCII IP string + NUL terminator + even-byte padding under a port byte with bit 4 set).

This is a contained fix targeted at the **0.8.0 draft**, deliberately keeping the change non-breaking. The cleaner private-storage `RoutePath { hops: Vec<RouteHop> }` shape with the legacy `pub` fields removed is deferred to the SemVer-major release-window brief and a 1.0.0 cut.

### Context to read first

- The GitHub issue surfacing the routing limitation. Quoting the user's framing: *"My understanding of CIP routing is that it consists of a series of (port segment, link/address segment) pairs… The current implementation looks to support a limited/strange subset of this routing: N backplane hops followed by M ethernet hops."* The proposed `enum RouteHop` shape is exactly the right model.
- `src/route.rs` pre-CODEX-F — three parallel `Vec` fields (`slots`, `ports`, `addresses`) with an encoder that iterates `slots` first then `addresses`; builder order has no effect on wire output.
- `docs/EtherNetIP_Connection_Paths_and_Routing.md` — the in-repo wire-format reference for port segments and extended link addresses.
- `wiki/protocol/route-path-behavior.md` — confirmed/likely synthesis page for route-path behavior; updated alongside this brief.
- `CLAUDE.md` "Important Invariants" and "PLC Firmware Limitations" — the validated targets (CompactLogix `5069-L320ERMS3`, ControlLogix `1756-L81ES`) are direct-connect or single-hop backplane only, which is why the latent ethernet-hop bug was never surfaced by the validation matrix.

### Behavior

Five contained changes, landing as one submission.

**1. Add an ordered `RouteHop` enum and an ordered `hops` field on `RoutePath`.**

```rust
pub enum RouteHop {
    Backplane { port: u8, slot: u8 },
    Ethernet { port: u8, address: String },
}

pub struct RoutePath {
    pub slots: Vec<u8>,        // legacy, retained
    pub ports: Vec<u8>,        // legacy, retained
    pub addresses: Vec<String>, // legacy, retained
    pub hops: Vec<RouteHop>,   // new — primary source of truth
}
```

The legacy parallel-`Vec` fields are kept `pub` for compatibility with existing Rust callers and FFI/wrapper code that may construct `RoutePath` literally. The builder methods (`add_slot`, `add_port`, `add_address`) update both the legacy fields and `hops` in sync. `add_port` finds the most-recent ethernet hop and patches its port number so late `.add_port(...)` calls continue to work the way they did pre-CODEX-F.

**2. Add explicit hop builders for the non-default cases.**

```rust
pub fn add_backplane(self, port: u8, slot: u8) -> Self;
pub fn add_ethernet(self, address: impl Into<String>) -> Self;
pub fn add_ethernet_with_port(self, port: u8, address: impl Into<String>) -> Self;
```

`add_ethernet` defaults to port 2 (the conventional Allen-Bradley ethernet port). `add_ethernet_with_port` is the escape hatch for devices where the convention differs. Constants `DEFAULT_BACKPLANE_PORT = 1` and `DEFAULT_ETHERNET_PORT = 2` are private to the module.

**3. Fix the ethernet wire encoding.**

The pre-CODEX-F encoder wrote `[port_byte, ip_octet_0, ip_octet_1, ip_octet_2, ip_octet_3]`. That is not a valid CIP path segment for any standard ethernet port. The correct extended-link-address form is:

- Port byte with bit 4 set: `0x10 | (port & 0x0F)`
- Length byte: ASCII string length + 1 for the NUL terminator
- ASCII IP string bytes
- NUL terminator byte
- Optional pad byte if total length-plus-NUL is odd (so the next segment lands on an even boundary)

For `add_ethernet("192.168.1.5")` with default port 2, the wire bytes are `[0x12, 0x0C, b'1', b'9', b'2', b'.', b'1', b'6', b'8', b'.', b'1', b'.', b'5', 0x00]` — 14 bytes, no pad (12 is even).

**4. `to_cip_bytes` falls back to legacy grouped fields when `hops` is empty.**

The dual-state hazard from the initial implementation pass — direct construction with `slots: vec![0], hops: vec![]` would silently produce an empty route — is mitigated by reading from `hops` when non-empty and falling back to the legacy grouped-field encoder otherwise. Both paths use the same shared `append_hop` helper internally so they produce identical wire bytes for equivalent inputs.

**5. Mirror the structure on the schema export.**

`SchemaRoutePath` (in `src/schema.rs`) gains a `hops: Vec<SchemaRouteHop>` field with Serde-derived shape `{ "kind": "backplane" | "ethernet", ... }` (snake_case tag, named fields). Legacy `slots`/`ports`/`addresses` fields are preserved for backward-compatible JSON consumers.

### Test requirements

- `cargo fmt -- --check` — must pass.
- `cargo clippy --all-features -- -D warnings` — must pass.
- `cargo clippy --no-default-features --lib -- -D warnings` — must pass.
- `SKIP_PLC_TESTS=1 cargo test --workspace --locked` — must pass (including doctests).
- `cargo test --test plc_sim_tests` — must pass.
- `cargo build --release --features ffi` — produces a cdylib with exactly 56 `_eip_` exports.
- C# wrapper `dotnet build && dotnet test` — must pass.

Pinned-byte test fixtures must cover at minimum:
1. The existing `test_route_path_cip_bytes` updated to assert exact bytes for `add_slot(0).add_address("192.168.1.100")`.
2. `test_route_path_preserves_mixed_hop_order` for `add_slot(0).add_ethernet("192.168.1.5").add_slot(3)`.
3. `test_route_path_uses_explicit_ethernet_port` for `add_slot(1).add_ethernet_with_port(3, "10.20.30.40")`.
4. `test_legacy_port_address_pairing_still_works_when_port_is_added_late` for `add_slot(0).add_address("192.168.1.100").add_port(3)`.
5. `test_legacy_public_field_construction_still_encodes_route` exercising the fallback path with `RoutePath { slots: vec![0], ports: vec![3], addresses: vec!["10.20.30.40".to_string()], hops: Vec::new() }`.

### Acceptance criteria

- [x] `RouteHop` enum exists in `src/route.rs` and is re-exported from the crate root.
- [x] `RoutePath::hops` is a `pub` field alongside the legacy parallel-`Vec` fields; builder methods keep both representations in sync.
- [x] Explicit-port builders `add_backplane`, `add_ethernet`, `add_ethernet_with_port` are present.
- [x] Ethernet hops encode as `[0x10 | port, ascii_len + 1, ascii…, 0x00, optional_pad]`. Pinned-byte tests lock the encoding for at least three port/address combinations.
- [x] `to_cip_bytes` falls back to legacy grouped-field encoding when `hops.is_empty()`; the fallback uses the same per-hop encoder so wire output is identical.
- [x] `SchemaRoutePath` exports both legacy fields and the new `hops` array.
- [x] FFI symbol parity preserved at 56 `_eip_` exports.
- [x] Full test matrix is green: fmt, both clippy variants, workspace tests, sim tests, doctests, C# wrapper.
- [x] CHANGELOG entry under "Fixed" or "Internal" describing the routing fix; honest about the multi-hop case still requiring real-hardware validation.
- [x] Wiki `route-path-behavior.md` updated with the new behavior, clearly labelled `confirmed` for the ordered structure and `likely` for the ethernet-encoding correctness pending hardware validation.

### Out of scope

- Removing the legacy `pub slots`, `pub ports`, `pub addresses` fields from `RoutePath`. That is a SemVer-major change and belongs in the release-window brief targeting 1.0.0.
- Making `RouteHop` `#[non_exhaustive]`. Also SemVer-major; deferred.
- Adding `RouteHop::ControlNet`, `RouteHop::DeviceNet`, or other transport variants. Out of scope until a user surfaces a concrete need and supplies test fixtures.
- Changing the FFI/wrapper API for `eip_connect_with_route`. The C# wrapper continues to pass flat slot/port/address arrays through the legacy path. A new FFI shape for ordered hops is a separate brief.
- Hardware validation against a real multi-hop topology. The maintainer's validation matrix today is direct-connect and single-hop backplane only.

### Risks and gotchas

- **Wire encoding is not yet hardware-verified.** The ASCII/NUL/pad shape matches the spec and matches every reference implementation surveyed, but no real Allen-Bradley multi-hop topology has been exercised with this code path. Document this honestly in the CHANGELOG and the wiki. Hardware validation is a follow-up; do not claim ethernet routing as "confirmed" until a real PLC accepts the new bytes.
- **Dual-state hazard.** Direct literal construction with both `slots`/`ports`/`addresses` non-empty AND `hops` non-empty is now ambiguous — the encoder reads from `hops` only. This is unlikely in practice (the builder methods keep both in sync) but worth a documented note. The 1.0.0 cleanup removes the hazard entirely by making the legacy fields go away.
- **Public-field mutation bypasses the new validation invariants.** Anyone calling `route.slots.push(0)` directly today still works (the fallback handles it), but anyone calling `route.hops.push(...)` directly is responsible for keeping the legacy fields in sync (or accepting that `to_cip_bytes` ignores them). The right fix is private storage; deferred.
- **`add_port(p)` after `add_address(a)` is supported via the `update_ethernet_hop_port` helper** which walks the existing ethernet hops and patches the corresponding port. This preserves the legacy "late port" pairing semantics. If a future refactor removes the legacy fields, this method's behavior should be re-examined.

## Codex log

### 2026-05-14 (synthesized)  codex

Initial implementation submitted directly in response to the GitHub issue, without a Claude-authored brief. The submission added ordered `RouteHop` storage, builder methods, and a fix for ethernet hop encoding using raw IPv4 octets.

Claude review identified three real defects:

1. `cargo clippy --all-features -- -D warnings` failed with `clippy::manual-is-multiple-of` on the new code; the submission report claimed verification passed, but this clippy variant had not been run.
2. The previously-passing `connect_with_stream` doctest at `src/client.rs:670` failed to compile after the changes (TcpStream trait bounds error; root cause: doctest needed an import adjustment unrelated to the routing change, but was masked by Codex's local environment skipping doctest compilation in `cargo check`).
3. The ethernet hop encoding used raw 4-byte IPv4 octets, which is not the standard Allen-Bradley extended-link-address shape. Hardware would reject this encoding even though the structural model was now correct.

### 2026-05-14 (synthesized)  codex — follow-up fixes

Addressed all three blockers:

- Replaced `link_address.len() % 2 != 0` with `!len.is_multiple_of(2)` to satisfy the lint.
- Rewrote the failing doctest to inject an `std::io::Cursor` stream instead of `tokio::net::TcpStream`, demonstrating the generic-stream API without requiring tokio feature resolution in the doctest harness.
- Re-encoded ethernet hops as `[0x10 | port, ascii_len + 1, ascii…, 0x00, optional_pad]` matching the spec. Added pinned-byte tests covering five route shapes including the legacy-public-field fallback path.

Verification on the corrected submission:

- `cargo clippy --all-features -- -D warnings`
- `cargo test -p rust-ethernet-ip --doc --locked`
- `cargo test --test udt_discovery_tests route --locked`
- `SKIP_PLC_TESTS=1 cargo test --workspace --locked`
- `git diff --check`

## Claude review

### 2026-05-14 12:00  claude — review pass 1

Independent verification on the corrected submission:

- `cargo fmt -- --check` — clean.
- `cargo clippy --all-features -- -D warnings` — clean.
- `cargo clippy --no-default-features --lib -- -D warnings` — clean.
- `SKIP_PLC_TESTS=1 cargo test --workspace --locked` — all suites green; `udt_discovery_tests` went 11 → 18 (+7 route tests, of which 5 are the new pinned-byte fixtures); doctests back to 25/0 (the temporarily failing `connect_with_stream` doctest is green).
- `cargo test --test plc_sim_tests` — 10/10.
- `cargo build --release --features ffi` — cdylib symbol parity at 56.

**Pinned-byte verification walkthrough.** I worked through each fixture by hand to confirm the spec interpretation:

- `add_slot(0).add_ethernet("192.168.1.5").add_slot(3)` → `[0x01, 0x00, 0x12, 0x0C, "192.168.1.5", 0x00, 0x01, 0x03]`: backplane port 1 + slot 0, then extended port segment `0x10 | 2 = 0x12`, length `11 + 1 = 12 = 0x0C`, 11 ASCII bytes + NUL (total 12 bytes, even, no pad), then backplane port 1 + slot 3. ✓
- `add_slot(1).add_ethernet_with_port(3, "10.20.30.40")` → `[0x01, 0x01, 0x13, 0x0C, "10.20.30.40", 0x00]`: port byte `0x10 | 3 = 0x13`, length 12, 11 ASCII + NUL. ✓
- `RoutePath { slots: vec![0], ports: vec![3], addresses: vec!["10.20.30.40"], hops: Vec::new() }` → identical encoding to the equivalent builder chain via the legacy-grouped fallback path. ✓

**Strong points:**

- ✅ **Wire encoding now matches the spec.** Extended link-address segment with `0x10 | port`, ASCII IP + NUL + even padding. The pinned-byte tests lock this down; any future refactor that drifts will fail loudly.
- ✅ **Dual-state hazard meaningfully mitigated.** The `if self.hops.is_empty()` fallback at `route.rs:107` catches the direct-literal-construction case I flagged in the prior review. The `test_legacy_public_field_construction_still_encodes_route` test directly exercises this path and asserts the same wire output as the builder-chain equivalent.
- ✅ **Shared per-hop encoder.** Both the modern (`hops`) and legacy (`slots/ports/addresses`) paths use `append_hop` internally, so there is exactly one place where the wire shape lives. Less drift risk.
- ✅ **Explicit-port builders are good ergonomics.** `add_ethernet_with_port` is the right escape hatch for non-standard port assignments without inviting users to mutate `ports` directly.
- ✅ **Schema export mirrors the new structure** with proper Serde representation; both legacy and ordered consumers continue to work.
- ✅ **Doctest fix is the cleaner shape.** Using `std::io::Cursor` for the `connect_with_stream` doctest demonstrates the generic-stream API without binding the doctest to tokio's feature surface.
- ✅ **Wiki page is honest.** Confirmed/likely classification is correct: structure is confirmed, encoding correctness against real hardware is "likely" pending validation.

**Polish (🟡 — non-blocking):**

- 🟡 **`pub` field `hops` is still mutable.** A caller can `route.hops.push(RouteHop::Backplane { port: 1, slot: 0 })` directly, which leaves `slots`/`ports`/`addresses` out of sync. The encoder will use `hops` (correct), so wire output is right, but downstream code reading the legacy fields will see stale data. Not load-bearing today; cleaned up by the 1.0.0 brief.
- 🟡 **No `RouteHop` `Default` impl, and no `#[non_exhaustive]`.** Both intentional — `#[non_exhaustive]` would be SemVer-major if added later, so deferring the decision to 1.0.0 is correct. Worth a one-line doc note that adding variants is a breaking change today.
- 🟡 **The legacy `add_port` semantics are subtle.** `update_ethernet_hop_port` walks ethernet hops by index, which is the correct behavior for the legacy "add slot, add address, add port" sequence. But if a caller interleaves `add_slot` and `add_address` and then `add_port`, the "port index" semantics may surprise them. The pinned-byte test for late-port pairing covers the common case; document that interleaving with multiple ethernet hops before any `add_port` calls behaves as expected, others may not.

**No 🟠 concerns.**

**Acceptance criteria tally:** all eight checked.

**Brief errors owned by Claude:** none in this brief (it was authored retroactively after the work). The prior context where Claude recommended Option B (full SemVer-major reshape) is acknowledged: Codex picked Option A (non-breaking with fallback), and the maintainer agreed. Both are defensible; the dual-state hazard is the explicit price of Option A and is documented.

## Verdict

**Merged** at `9a3d192` — `route: ordered RouteHop and ASCII ethernet link-address encoding`.

The implementation is faithful to what the GitHub issue asked for, fixes a latent wire-encoding bug on top of the structural model, and ships in the 0.8.0 draft window without breaking existing callers. The non-breaking shape (legacy fields preserved, fallback on empty `hops`) has a documented dual-state hazard that the SemVer-major release-window brief will resolve.

**Process note (important):** This brief was authored *retroactively* after Codex implemented the work in direct response to the GitHub issue, without a prior Claude-authored brief. Per `docs/agents/README.md`, "Briefs are always authored by claude" and Codex normally acts only on a brief. The bypass happened because the issue had a clearly correct technical direction and an architectural framing already discussed in maintainer-Claude chat; Codex chose to act rather than wait for the brief authoring step.

The bypass is not a sustainable pattern, but the resolution here is acceptable for two reasons:
1. Claude's review caught two real defects (clippy lint failure, broken doctest) plus an architectural concern (raw-octet ethernet encoding) that Codex's self-reported verification missed. The cross-agent review loop worked even though the brief loop didn't.
2. Codex acknowledged the bypass in their follow-up message and agreed that retroactive brief authoring was the right cleanup.

For future similar situations, the right pattern is: Codex flags the GitHub issue to the maintainer, maintainer asks Claude to author a brief, Codex implements against the brief. The brief authoring step is what gives both agents a durable contract to verify against. Without it, "did the implementation match the intent" is unanswerable.

Hardware validation against a real multi-hop ethernet topology remains a follow-up. The wiki and CHANGELOG correctly flag this. The validated CompactLogix and ControlLogix targets remain direct-connect or single-hop backplane; nothing in this change affects those paths.

Future brief candidates surfaced by CODEX-F:
1. **Release-window brief (1.0.0)**: remove the legacy `pub slots`/`ports`/`addresses` fields from `RoutePath`; make storage private; add `#[non_exhaustive]` to `RouteHop`. Bundle with the other deferred SemVer-major items.
2. **FFI ordered-hop shape**: extend `eip_connect_with_route` to accept an ordered hop array instead of flat slot/port/address arrays. Required for C# / Python wrappers to expose the new routing surface.
3. **Hardware validation pass**: real multi-hop test on a ControlLogix rack with EN2T or similar. Validate the ASCII-encoded ethernet hop output against a real PLC. Promote the wiki's `likely` markers to `confirmed`.
