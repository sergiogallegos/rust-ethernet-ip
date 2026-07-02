# 2026-07-02 STRING write strategy probe — CompactLogix 5069-L330ERM fw38

Date: 2026-07-02
Tester: Claude + Sergio Gallegos (maintainer-owned hardware)
Library version: `1.1.0` (`main` at `b2fcf33`)
Trigger: maintainer question — since a Logix STRING is internally a `DINT .LEN` + `SINT .DATA[82]` structure, can the library write it through those members instead of the (documented-as-firmware-blocked) direct STRING write?

## Headline

**The "firmware blocks direct STRING writes" claim in [`docs/agents/notes/ab-firmware-quirks.md`](../agents/notes/ab-firmware-quirks.md) is a misdiagnosis.** A direct, single-request Write Tag to a controller-scoped STRING tag succeeds on this controller when the request uses the documented Logix structure encoding. The CIP extended error `0x2107` that drove the original conclusion is the Logix Data Access error for *"tag type used in request does not match the target tag's data type"* — i.e. the controller was rejecting the library's malformed encoding, not the operation. Component-level writes through `.LEN` / `.DATA` also work.

## Setup

- Controller: CompactLogix **5069-L330ERM**, firmware **38**, at `192.168.0.101:44818`, slot 0, direct connect.
- Tag: controller-scoped `gTest_STRING` (standard `STRING` type, from the full-coverage test program).
- Method: a temporary probe example built on the public API (`EipClient::connect`, `read_tag`, `write_tag`, `send_cip_request`), one strategy at a time, each verified by an independent `read_tag` read-back. The tag's original value (`"STRING FROM CONTROLLER TAG"`, LEN 26) was restored at the end.

## Results

| # | Strategy | Request shape | Result |
|---|---|---|---|
| A | Current wired path: `write_tag(PlcValue::String)` | `0x4D`, path, type `CE 00` (atomic 0x00CE), count `01 00`, `LEN u32` + data (no padding) | ❌ general status `0xFF`, extended `0x2107` (tag type mismatch) |
| B1 | Raw structure write, 86-byte payload | `0x4D`, path, type `A0 02 CE 0F`, count `01 00`, `LEN u32` + `DATA[82]` | ❌ general status `0x13` (not enough data) |
| B2 | **Raw structure write, 88-byte payload** | `0x4D`, path, type `A0 02 CE 0F`, count `01 00`, `LEN u32` + `DATA[82]` + 2 pad bytes | ✅ general status `0x00`; read-back confirms value |
| C | Component writes via public `write_tag`: one `PlcValue::Sint` per `.DATA[i]`, then `.LEN` as `PlcValue::Dint` | library-generated requests | ✅ read-back confirms value |
| D | Raw component writes, 2 round trips: `.DATA` as one 82-element SINT write (`0x4D`, path `gTest_STRING`+`DATA`, type `C2 00`, count `52 00`, 82 bytes), then `.LEN` as DINT | hand-built requests | ✅ both general status `0x00`; read-back confirms value |

Supporting observations:

- **Structure instance size is 88 bytes** (`LEN` DINT 4 + `DATA` 82 + 2 alignment pad). B1 vs B2 isolates this: the same request with an 86-byte payload gets `0x13` "not enough data"; with 88 bytes it succeeds. This matches the `0x00CE => 88` size already recorded in `crates/udt/src/lib.rs`.
- **`0x0FCE` is the structure handle for the standard STRING template**, carried in the type field as `A0 02 CE 0F` (structure marker `0x02A0` + handle). This is the same encoding pycomm3 and libplctag use for Logix STRING writes.
- **Reads return the raw structure**: `read_tag("gTest_STRING")` on this hardware returns `PlcValue::Udt` whose data is `[CE 0F][LEN u32 LE][DATA 82][pad 2]` (90 bytes), not `PlcValue::String`. The in-process simulator returns atomic type `0x00CE` for STRING reads, so sim tests decode to `PlcValue::String` — a sim/hardware divergence.
- Strategy C succeeded through today's public `write_tag`, but the mechanism was not traced (the `.DATA[i]` writes are likely routed through the array-element read-modify-write workaround rather than a direct element-segment path). This does **not** contradict CODEX-AM's finding that `TagPath`'s `StringData` segment encoding is malformed.

## Interpretation

1. Direct STRING writes are achievable in a single request with the correct structure encoding. The library's wired path fails because it emits atomic type `0x00CE` with an unpadded payload.
2. The extended error `0x2107` returned for the malformed writes is documented in the Logix 5000 Data Access reference as a Read/Write Tag Service data-type mismatch, not a vendor-specific/composite error (as the library's error text currently claims) and not a firmware prohibition.
3. The maintainer-proposed `.LEN`/`.DATA` component write is real and works today, but is superseded as a primary mechanism by the fixed direct write; its remaining value is as evidence and possibly a fallback (relevant to CODEX-AP item 5).

## Limitations

- Only a **controller-scoped, standard (82-char) STRING** was probed. Not yet hardware-tested: program-scoped STRING tags, STRING members inside UDTs and UDT array elements, custom-length `STRINGnn` types (different template handles and instance sizes), and batch STRING writes.
- Single controller / single firmware (5069-L330ERM fw38). The encoding matches the format used broadly by the ecosystem (pycomm3, libplctag), so cross-firmware risk is low, but the full-coverage manifest relabel must be confirmed by a full hardware run.

## Follow-up

Remediation is briefed as **CODEX-AT** (`docs/agents/tasks/CODEX-AT-string-write-wire-format.md`): fix the write encoding, decode standard STRING reads to `PlcValue::String`, correct the quirks note and manifest labels, and align the simulator with the hardware-observed format.

## Post-fix review smoke — same day, same controller

After the CODEX-AT implementation, a second hardware session ran the *fixed library's public API* against the same 5069-L330ERM fw38 (values restored afterwards):

| Check | Result |
|---|---|
| `read_tag("gTest_STRING")` decodes to `PlcValue::String` (not `Udt`) | ✅ |
| `write_tag(PlcValue::String)` round-trip, controller-scoped | ✅ |
| `write_tag(PlcValue::String)` round-trip, **program-scoped** (`Program:TestProgram.gTest_STRING` — not covered by the original probe; proves the manifest relabel) | ✅ |
| Shorter-over-longer write leaves no residue | ✅ |
| **Batch (Multiple Service Packet) STRING-only write** | ✅ |
| Batch STRING + DINT-array-element mixed read | ✅ |

One instructive false alarm: a first batch attempt paired the STRING write with a nonexistent tag name. The controller answered MSP general status `0x1E` (embedded service error) for the *bad tag's* service — while still applying the valid STRING write — but the library's `parse_multiple_service_response` attributed the MSP-level failure to the batch wholesale, reporting the STRING write as failed even though it had landed. Two lessons: (1) the historical "batch STRING writes fail with 0x1E" observation may trace to exactly this attribution behavior; (2) per-service reply attribution when the MSP-level status is nonzero is a real gap — relevant to CODEX-AN's response-parsing scope.
