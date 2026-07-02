# Allen-Bradley Firmware Quirks

Use this page when reviewing or modifying anything that writes tag data — STRINGs, UDTs, or UDT arrays. These are CIP-side firmware behaviors or protocol wire-format requirements, not library bugs. The workarounds in the library are intentional and must not be "simplified" away.

Verified against Allen-Bradley CompactLogix and ControlLogix CIP behavior, observed consistently across the firmware revisions this project has been tested on.

## STRING writes

- Confirmed 2026-07-02 on 5069-L330ERM fw38: direct writes to standalone standard Logix `STRING` tags work when the request is encoded as structure type `0x02A0`, standard STRING handle `0x0FCE`, element count `1`, and an 88-byte payload (`LEN` u32 LE + `DATA[82]` zero-padded + 2 pad bytes).
- The old library path failed because it emitted atomic type `0x00CE` with an unpadded payload. The resulting `0xFF/0x2107` status is the Logix Data Access Read/Write Tag data-type mismatch, not proof that firmware blocks the operation.
- Read replies for standard STRING can arrive as structure type `0x02A0` with payload beginning `CE 0F`; decode that handle to `PlcValue::String`. Other structure handles remain UDT data until CODEX-AO handles general structure decoding.
- Direct writes to `STRING` members inside UDTs were not validated by this probe. Keep those paths in the restricted/read-modify-write bucket until hardware evidence says otherwise.
- Evidence: [2026-07-02 STRING write probe](../../validation/2026-07-02_string_write_probe_5069-L330ERM_fw38.md).

## UDT array element member writes

- Historical validation classified direct writes to individual members of UDT
  array elements (e.g. `Cell_Data[5].Speed = 100`) as firmware-blocked
  `0x2107` cases. CODEX-AM disproved that blanket claim for one concrete path:
  `write_tag("gTestUDT_Array[0].Member1_DINT", Dint(777))` succeeded on a
  5069-L330ERM fw38 once the library preserved the full member path, and the
  sibling member remained untouched.
- Current status: **under revalidation**. Keep the manifest's
  `firmware_blocked_udt_array_element_member` labels until CODEX-AV records the
  full matrix, but do not treat `0x2107` as proof of an inherent firmware ban;
  malformed paths can produce the same data-type mismatch.
- Existing read-modify-write service-layer helpers remain the conservative
  fallback until CODEX-AV proves which direct member writes are reliable across
  scalar types and scopes.
- Evidence: [2026-07-02 tag-addressing smoke](../../validation/2026-07-02_tag_addressing_smoke_5069-L330ERM_fw38.md).

## UDT writes always need a symbol_id

- `UdtData` carries `{ symbol_id: i32, data: Vec<u8> }`. The `symbol_id` is assigned by the PLC and must match the controller's current UDT instance for the write to succeed.
- Always read a UDT before writing it. The read captures the current `symbol_id`. A stale or zero `symbol_id` produces a CIP error that *looks* like a path or access error but is actually a symbol mismatch.
- Don't fabricate a `symbol_id` from a definition file or cached value across sessions. The PLC may reassign it.

## Reading this against a failing 0x2107

When a user reports CIP error `0x2107`, the layer is almost always one of:
1. Request data type does not match the target tag, including malformed STRING writes.
2. Direct UDT array element member write through a malformed or unvalidated path
   — check CODEX-AV evidence before classifying it as a firmware limitation.
3. Stale `symbol_id` — fix by reading the UDT first.
4. Direct STRING member write inside a UDT — keep restricted until validated.

If the failure is none of these, escalate before "fixing" — `0x2107` from a different cause is a real bug and the wrong patch will mask it.
