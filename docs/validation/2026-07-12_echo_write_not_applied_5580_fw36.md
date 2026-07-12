# 2026-07-12 write acknowledged-but-not-applied — FactoryTalk Logix Echo 5580 fw36

Date: 2026-07-12
Library version: `1.2.0` (`main`)
Trigger: independent evaluation of the crate as a candidate dependency for a third-party
project, prototyping tag reads/writes against a live FactoryTalk Logix Echo instance.

## Headline

A plain controller-scope DINT tag on a FactoryTalk Logix Echo-emulated ControlLogix 5580
(firmware 36) accepted a `write_tag` call with a clean CIP success reply (Write Tag Service,
service `0xCD`, status `0x00`) while the underlying value never changed. The request bytes are
correctly formed (confirmed via `RUST_LOG=rust_ethernet_ip=trace`) and the controller's own
reply claims success, so `write_tag` cannot detect this at the protocol level — the reply is
indistinguishable on the wire from a real success. This is reported as a caution and mitigated
with a new opt-in verified-write API (`EipClient::write_tag_verified`), not as a wire-encoding
fix, because no incorrect encoding was found.

## Setup

- Controller: FactoryTalk Logix Echo-emulated **ControlLogix 5580** ("Emulate 5580 Controller"
  per CIP List-Identity), firmware **36.11**, at `<echo-host>:44818`, backplane slot **1**
  (`RoutePath::new().add_backplane(1, 1)`), reachable directly over a real LAN (Echo was
  configured with a routable IP, not the default `127.0.0.1`/loopback binding).
- Loaded project: a production controller export (~1100 tags), not a minimal test program.
- Tag: controller-scope `Spare_DInt` (DINT), a generic scratch tag.
- Cross-validation client: [pylogix](https://github.com/dmroeder/pylogix) 1.1.5, same
  controller, same tag, same slot.

## Reproduction

```rust
let route = RoutePath::new().add_backplane(1, 1);
let mut client = EipClient::with_route_path("<echo-host>:44818", route).await?;

let before = client.read_tag("Spare_DInt").await?;         // Dint(37)
client.write_tag("Spare_DInt", PlcValue::Dint(4242)).await?; // Ok(())
let after = client.read_tag("Spare_DInt").await?;            // Dint(37) — unchanged
```

Reproduced 3/3 attempts across both the same connection and a fresh connection per attempt.
Five other DINT/STRING scratch tags on the same controller, through the same client and
connection, wrote and read back correctly in the same session (`_testAdd`,
`Program:Optimization_MG._rsETest`, `Program:TableInfeedChains._thTestCopyData1`,
`_th_VFDTEST`, `_TH_VFDTEST`, `StringGeneral`) — this is not a systemic type-encoding
regression, it is narrower and at minimum tag-dependent.

## Wire evidence

`RUST_LOG=rust_ethernet_ip=trace` on the write to `Spare_DInt`:

```
Built CIP write request (22 bytes): 4D 06 91 0A 53 70 61 72 65 5F 44 49 6E 74 C4 00 01 00 92 10 00 00
...
Received response (20 bytes): ... B2 00 04 00 CD 00 00 00
Write response - Service: 0xCD, Status: 0x00
```

Decoding the request: service `0x4D` (Write Tag Service), symbolic path `Spare_DInt`, type
`0x00C4` (DINT), count `1`, value `92 10 00 00` LE = `0x00001092` = `4242` — a correctly formed
request for the intended value. The reply's embedded service is `0xCD` (Write Tag Service
Reply) with general status `0x00` (success) — not the `0xD2` Unconnected Send failure shape
covered by [`docs/agents/notes/unconnected-send.md`](../agents/notes/unconnected-send.md). The
subsequent read, framed identically (same route path, same symbolic path resolution), returns
`25 00 00 00` LE = `37` — the pre-write value.

A packet capture of pylogix's successful write to the same tag shows its SendRRData items use
encapsulation item type `0x00A1` (Connected Address Item), i.e. pylogix holds a real CIP
connection (Forward Open) for its explicit messaging, whereas this library always wraps CIP
requests in Unconnected Send (item type `0x00B2`) — see
[`docs/agents/notes/unconnected-send.md`](../agents/notes/unconnected-send.md), "Unconnected
Send (service `0x52`) is the primary path, always". The write payload bytes themselves are
otherwise the same shape (service `0x4D`, type `0x00C4`, count `1`, little-endian value) between
the two clients. Whether the connected-vs-unconnected distinction is the actual mechanism behind
the discrepancy is not confirmed here — it is offered as the leading, but unproven, explanation.

## Interpretation

1. The request encoding is correct; this is not a repeat of the STRING/UDT firmware-quirk class
   documented in [`docs/agents/notes/ab-firmware-quirks.md`](../agents/notes/ab-firmware-quirks.md)
   (those surface as an explicit non-zero CIP status). Here the controller's own reply claims
   success.
2. This has not been observed on physical hardware. The existing 1.2.0 hardware validation
   (CompactLogix 5069-L330ERM fw38: 2304 reads, 2285 writes, 2285 verify, 0 anomalies — see
   `docs/validation/2026-07-08_cross-binding_full-coverage_5069-L330ERM_fw38.md`) found no
   equivalent case. Treat this as an Echo-specific caution, not a general claim about the
   library or about Allen-Bradley firmware, until it is reproduced (or ruled out) against
   physical hardware.
3. Because the false-success reply is wire-indistinguishable from a true one, no client-side
   parsing fix is possible. The mitigation implemented alongside this report is an explicit,
   opt-in verified-write primitive (`EipClient::write_tag_verified`) that reads back after
   writing and returns `EtherNetIpError::WriteNotApplied` on mismatch, plus a
   `SimBehavior::ghost_write_tags` simulator hook so the failure mode has deterministic
   regression coverage without requiring the Echo instance that surfaced it.

## Open follow-ups

- Reproduce (or rule out) against physical ControlLogix/CompactLogix hardware.
- Determine whether the Unconnected-Send-only design (vs. a Forward-Open/connected mode, which
  pylogix uses) is actually the mechanism, or coincidental.
- Consider whether `write_tag_verified` should become the default `write_tag` behavior behind a
  config flag, given the cost is one extra read per write.
