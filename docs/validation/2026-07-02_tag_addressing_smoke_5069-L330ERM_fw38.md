# 2026-07-02 Tag-addressing review smoke — CompactLogix 5069-L330ERM fw38

Date: 2026-07-02
Tester: Claude + Sergio Gallegos (maintainer-owned hardware)
Library version: `1.1.0` line, CODEX-AM working tree (parent `7676137`)
Trigger: CODEX-AM review — the task rewrites emitted request bytes on five addressing paths, and the bench controller was available. Public API only; all written values restored.

## Results by fix

| Fix | Check | Result |
|---|---|---|
| 5 | `discover_program_tags("Program:TestProgram")` | ✅ 7 tags returned (parent code addressed the wrong object and returned nothing usable) |
| 4 | `read_tag("gTest_STRING.DATA[0]")` | ✅ `Sint(83)` (`'S'`) — corrected `.DATA[i]` Element ID segment accepted |
| 1 | `write_tag("gTestUDT_Array[0].Member1_DINT", Dint(777))` | ⚠️ **Succeeded** — see below. Sibling member `Member2_REAL` untouched; no element clobber. Value restored to 0. |
| 2 | `read_tag("gTestUDT.Member1_DINT.15")` + `write_tag(".15", Bool)` | ✅ bit read correct against host value 999999; RMW flip produced exactly `0x000F423F → 0x000FC23F` (neighbor bits preserved); restored |
| 3 | Batch write `gTestArray_BOOL[35]` flip | ✅ `[35]` took the write, `[3]` (same-index-mod-32 alias victim in the parent code) unchanged; restored |

## The fix-1 surprise: UDT array element member writes are NOT firmware-blocked

The expectation was an honest CIP `0x2107` rejection (per [`docs/agents/notes/ab-firmware-quirks.md`](../agents/notes/ab-firmware-quirks.md)) with the element left intact — the fix's purpose being to stop the silent whole-element clobber. Instead the write **succeeded**: `Member1_DINT` became 777, `Member2_REAL` stayed untouched, read-back confirmed.

This is the same pattern as the 2026-07-02 STRING finding ([`2026-07-02_string_write_probe_5069-L330ERM_fw38.md`](2026-07-02_string_write_probe_5069-L330ERM_fw38.md)): the historical `0x2107` evidence was gathered through request paths the library built incorrectly (member suffixes dropped, malformed element segments). With well-formed paths, at least a DINT member of a UDT array element writes directly on this controller/firmware.

Consequences:

1. The quirks note's "UDT array element member writes" section is now suspect — same misdiagnosis class as its former STRING section.
2. `examples/full_coverage_tags.json` carries dozens of `firmware_blocked_udt_array_element_member` / `firmware_blocked_udt_string_member` labels whose writes may now **succeed** — the pre-1.2.0 full-coverage run would report them as unexpected anomalies and fail its zero-anomalies gate unless the labels are re-validated first.
3. Scope of this observation: one DINT member, one element, one controller/firmware. STRING members, REAL/BOOL members, program-scoped variants, and other firmwares are unverified. Systematic re-validation is briefed as **CODEX-AV**.

## Method note

Temporary example driving only public API (`discover_program_tags`, `read_tag`, `write_tag`, `write_tags_batch`, `read_tags_batch`); each mutation verified by read-back and restored (`Member1_DINT` → 0, `gTestUDT.Member1_DINT` → 999999, `gTestArray_BOOL[35]` → original).
