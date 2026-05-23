# Allen-Bradley Firmware Quirks

Use this page when reviewing or modifying anything that writes tag data — STRINGs, UDTs, or UDT arrays. These are CIP-side firmware behaviors, not library bugs. The workarounds in the library are intentional and must not be "simplified" away.

Verified against Allen-Bradley CompactLogix and ControlLogix CIP behavior, observed consistently across the firmware revisions this project has been tested on.

## STRING writes

- Do not attempt a direct CIP write to a STRING tag. The PLC returns CIP general status `0x2107` (object state conflict). This is a firmware restriction on Logix STRING handling, not a library limitation.
- Workaround in code: write the entire containing UDT instead. If the STRING is at the top level, wrap it in its parent UDT path and write the whole struct.
- Do not add a "retry the STRING write" branch. The PLC will reject every attempt with the same error.

## UDT array element member writes

- Do not write an individual member of a UDT array element (e.g. `Cell_Data[5].Speed = 100`). Returns CIP `0x2107`.
- Workaround: read the entire array element, modify the member in memory, write the whole element back. The library already does this for `write_tag` paths that resolve to a UDT array element member.
- Same as STRING: do not add a retry. Same firmware restriction.

## UDT writes always need a symbol_id

- `UdtData` carries `{ symbol_id: i32, data: Vec<u8> }`. The `symbol_id` is assigned by the PLC and must match the controller's current UDT instance for the write to succeed.
- Always read a UDT before writing it. The read captures the current `symbol_id`. A stale or zero `symbol_id` produces a CIP error that *looks* like a path or access error but is actually a symbol mismatch.
- Don't fabricate a `symbol_id` from a definition file or cached value across sessions. The PLC may reassign it.

## Reading this against a failing 0x2107

When a user reports CIP error `0x2107`, the layer is almost always one of:
1. Direct STRING write — fix by writing parent UDT.
2. UDT array element member write — fix by read-modify-write of the whole element.
3. Stale `symbol_id` — fix by reading the UDT first.

If the failure is none of these, escalate before "fixing" — `0x2107` from a different cause is a real bug and the wrong patch will mask it.
