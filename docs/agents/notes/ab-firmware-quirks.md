# Allen-Bradley Firmware Quirks

Use this page when reviewing or modifying anything that writes tag data — STRINGs, UDTs, or UDT arrays. These are CIP-side firmware behaviors or protocol wire-format requirements, not library bugs. The workarounds in the library are intentional and must not be "simplified" away.

Verified against Allen-Bradley CompactLogix and ControlLogix CIP behavior. Record controller and firmware scope for each finding; several older `0x2107` conclusions were later traced to malformed request encoding.

## STRING writes

- Confirmed 2026-07-02 on 5069-L330ERM fw38: direct writes to standalone standard Logix `STRING` tags work when the request is encoded as structure type `0x02A0`, standard STRING handle `0x0FCE`, element count `1`, and an 88-byte payload (`LEN` u32 LE + `DATA[82]` zero-padded + 2 pad bytes).
- The old library path failed because it emitted atomic type `0x00CE` with an unpadded payload. The resulting `0xFF/0x2107` status is the Logix Data Access Read/Write Tag data-type mismatch, not proof that firmware blocks the operation.
- Read replies for standard STRING can arrive as structure type `0x02A0` with payload beginning `CE 0F`; decode that handle to `PlcValue::String`. Other structure handles remain UDT data until CODEX-AO handles general structure decoding.
- Direct writes to `STRING` members inside UDTs remain restricted under the library's current encoding. The 2026-07-03 CODEX-AV matrix observed consistent `0xFF/0x2107` rejections for controller/program UDT members and UDT-array-element `Member5_String` targets on 5069-L330ERM fw38. Whether a member-tailored encoding exists remains CODEX-AO wire-format territory.
- Evidence: [2026-07-02 STRING write probe](../../validation/2026-07-02_string_write_probe_5069-L330ERM_fw38.md).

## UDT array element member writes

Historical validation classified direct writes to individual members of UDT array elements (e.g. `Cell_Data[5].Speed = 100`) as firmware-blocked `0x2107` cases. CODEX-AM and CODEX-AV disproved that blanket claim on a 5069-L330ERM fw38 once the library preserved the full member path.

### Scalar Members

- Confirmed 2026-07-03 on 5069-L330ERM fw38: all 60 scalar UDT-array-element-member targets in the full blocked-label sweep wrote successfully, verified read-back, preserved sibling members, and restored cleanly.
- Covered scalar types: DINT, REAL, BOOL, and INT in controller and program scopes.
- The full-coverage manifest now treats these scalar member paths as `writeable`.
- Service-layer helpers attempt direct scalar member writes first and fall back to whole-UDT read-modify-write only if the controller returns the `0x2107` data-type mismatch shape.

### STRING Members — root cause is a hardcoded structure handle, not a firmware/member block

- **Confirmed 2026-07-08 on 5069-L330ERM fw38: the `0x2107` on STRING-member writes is a structure-handle mismatch, and the member is fully writeable once the request carries the target's real handle.** The rejection is not firmware, not a member-path problem, and not a missing "member-specific encoding".
- The library hardcodes the built-in STRING handle `0x0FCE` for *every* `PlcValue::String` write (`crates/protocol/src/values.rs::write_data_type_bytes` emits `A0 02 CE 0F` unconditionally). This matches a built-in `STRING` tag, so `gTest_STRING` writes succeed.
- Studio 5000 lets users define their own string types with a custom name and length (e.g. `Str82` — DINT `LEN` + SINT `DATA[82]`, same layout as STRING but a different type). **A custom string type has its own structure handle.** On this controller the UDT member `gTestUDT.Member5_String` is type `Str82` with handle **`0x9621`** (the read reply's structure prefix is `21 96`), while the built-in STRING is `0x0FCE`. The controller compares the request handle against the target's real handle and returns `0xFF/0x2107` ("tag type used in request does not match the target's data type") on mismatch.
- Proven end-to-end (2026-07-08 probe, values restored): (A) `write_tag(String)` → `0x2107`, unchanged; (B) raw write with wrong handle `0x0FCE` → unchanged; (C) raw write with correct handle `0x9621` → value changes, read-back confirms. The member is writeable in a single direct request with the correct handle.
- **Side-by-side control (2026-07-08, same UDT):** a built-in `STRING` member `Member6_String` (handle `0x0FCE`) writes successfully through the normal `write_tag` in **both** `gTestUDT.Member6_String` and `gTestUDT_Array[0].Member6_String`, while the `Str82` `Member5_String` (handle `0x9621`) fails the same call and needs the real handle. Identical member path, opposite result → the failure is purely the structure handle, not the member/array-element path.
- **Fix direction (CODEX-AO):** discover the target's real structure handle instead of assuming `0x0FCE`. A read-before-write captures it (the read reply already carries `A0 02 <handle>`), mirroring the `symbol_id` read-before-write pattern below. This applies to standalone custom-string tags too, not only UDT members. Keep the RMW service-layer path as a fallback until the handle-aware write lands.
- Manifest entries still use `encoding_blocked_udt_string_member` as a conservative label; once handle-aware writes land they become `writeable` (they read as writeable on hardware today).

Evidence: [2026-07-08 cross-binding validation](../../validation/2026-07-08_cross-binding_full-coverage_5069-L330ERM_fw38.md), [2026-07-02 tag-addressing smoke](../../validation/2026-07-02_tag_addressing_smoke_5069-L330ERM_fw38.md), [2026-07-03 blocked write-label probe](../../validation/2026-07-03_blocked_write_label_probe_plan.md).

## UDT writes always need a symbol_id

- `UdtData` carries `{ symbol_id: i32, data: Vec<u8> }`. The `symbol_id` is assigned by the PLC and must match the controller's current UDT instance for the write to succeed.
- Always read a UDT before writing it. The read captures the current `symbol_id`. A stale or zero `symbol_id` produces a CIP error that *looks* like a path or access error but is actually a symbol mismatch.
- Don't fabricate a `symbol_id` from a definition file or cached value across sessions. The PLC may reassign it.

## Reading this against a failing 0x2107

When a user reports CIP error `0x2107`, the layer is almost always one of:
1. Request data type does not match the target tag, including malformed STRING writes.
2. Direct UDT array element member write through a malformed path. Scalar members are confirmed writeable on 5069-L330ERM fw38 when encoded correctly.
3. Stale `symbol_id` — fix by reading the UDT first.
4. **STRING write to a custom string type** (custom name/length, e.g. `Str82`) — the library sends the hardcoded built-in STRING handle `0x0FCE`, which mismatches the custom type's real handle. Proven writeable with the correct handle (see STRING Members above). Not a firmware block.

If the failure is none of these, escalate before "fixing" — `0x2107` from a different cause is a real bug and the wrong patch will mask it.
