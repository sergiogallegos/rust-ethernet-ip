# 2026-07-03 Blocked write-label probe plan

Date prepared: 2026-07-03
Prepared by: Codex
Trigger: CODEX-AV

## Purpose

CODEX-AM proved on the 5069-L330ERM fw38 that
`gTestUDT_Array[0].Member1_DINT` writes successfully when the request path is
well formed. That invalidates the old blanket assumption behind
`firmware_blocked_udt_array_element_member` labels. Before the 1.2.0
full-coverage gate, the currently blocked labels need hardware evidence.

## Maintainer command

Default matrix, one representative per blocked class:

```powershell
$env:TEST_PLC_ADDRESS='192.168.0.101:44818'
$env:TEST_PLC_SLOT='0'
cargo run --example probe_blocked_write_labels --locked
```

Full sweep over every currently blocked manifest tag:

```powershell
$env:TEST_PLC_ADDRESS='192.168.0.101:44818'
$env:TEST_PLC_SLOT='0'
cargo run --example probe_blocked_write_labels --locked -- --all-blocked
```

The probe writes JSON evidence to `examples/full_coverage_results/`.

## Evidence to record after the run

- Controller model and firmware.
- Probe command and output summary.
- JSON result artifact path.
- For each class: write succeeded or failed, observed CIP error if failed,
  read-back verification if succeeded, sibling-integrity result, and restore
  result.
- Manifest relabel decision: only classes with successful write, read-back,
  sibling-integrity, and restore evidence should move from `firmware_blocked_*`
  to writeable/service-layer writeable labels.

## Restore discipline

The probe reads the original value before writing, writes a type-specific test
value, verifies read-back, checks a sibling member where the path has one, then
writes the original value back and verifies the restore. Any setup, verify,
sibling, or restore failure makes the probe exit non-zero.

## Evidence — 2026-07-03 bench run (maintainer-authorized, executed during the probe-stage review)

- Controller: CompactLogix **5069-L330ERM firmware 38** at `192.168.0.101:44818`, slot 0.
- Commands: representative matrix and `--all-blocked` sweep, both via
  `--plc-address 192.168.0.101:44818`.
- Artifacts (gitignored): `examples/full_coverage_results/blocked_write_label_probe_rust_1783035652.json`
  (11 targets) and `blocked_write_label_probe_rust_1783035835.json` (72 targets).
- Both runs: `setup_failed=0 verify_failed=0 sibling_changed=0 restore_failed=0 unexpected=0 RESULT=PASS` —
  every mutated value verified and restored, every checked sibling untouched.

### Results by class (72-target sweep)

| Class | Kind | Targets | Outcome |
|---|---|---|---|
| `firmware_blocked_udt_array_element_member` | DINT | 15 | ✅ all write+verify+restore |
| `firmware_blocked_udt_array_element_member` | REAL | 15 | ✅ all write+verify+restore |
| `firmware_blocked_udt_array_element_member` | BOOL | 15 | ✅ all write+verify+restore |
| `firmware_blocked_udt_array_element_member` | INT | 15 | ✅ all write+verify+restore |
| `firmware_blocked_udt_array_element_member` | STRING (`Member5_String`, 10 array elements) | 10 | ❌ all CIP `0xFF`/`0x2107` |
| `firmware_blocked_udt_string_member` | STRING (ctrl + program UDT) | 2 | ❌ both CIP `0xFF`/`0x2107` |

### Interpretation

- **Every scalar (DINT/REAL/BOOL/INT) UDT member and UDT-array-element-member
  write succeeds** with well-formed paths, controller- and program-scoped. The
  old blanket block was request-encoding lore (see the CODEX-AM validation doc).
- **STRING members inside UDTs consistently draw `0x2107`** with the current
  write encoding (`PlcValue::String` emits the standalone STRING structure
  handle `0x0FCE`). Whether a member-specific encoding could succeed is a
  wire-format question (CODEX-AO territory); for labeling purposes the class is
  *blocked under the library's current encoding* and the service-layer RMW
  fallback remains required for STRING members.
- Manifest asymmetry found during review: `prog.UDTarr_elem_members` carries no
  blocked `Member5_String` entry while the controller-scope twin does — the
  relabel stage must reconcile it (probe evidence predicts `0x2107` for it too).

### Relabel directive for the follow-up stage

- The 60 scalar `firmware_blocked_udt_array_element_member` entries →
  writeable, per this evidence.
- All STRING-member entries stay blocked; use a label that says what the
  evidence says (STRING member writes rejected `0x2107` on L330ERM fw38 with
  the current encoding), and fix the prog-array asymmetry.
- Firmware scope: single controller. Record the scope in the manifest docs;
  older-firmware controllers re-validate whenever they are next benched.

## Follow-up

Update `examples/full_coverage_tags.json`,
`tests/full_coverage_manifest_tests.sh`, quirks documentation, and service-layer
routing decisions from the recorded results above.

## Relabel Stage Result

- Scalar UDT-array-element DINT/REAL/BOOL/INT entries moved from
  `firmware_blocked_udt_array_element_member` to `writeable`.
- UDT STRING-member entries use `encoding_blocked_udt_string_member`: rejected
  with `0x2107` under the current member encoding, not labeled as a firmware
  ban.
- The missing program-scope UDT-array-element `Member5_String` entry was added
  for parity with the controller-scope manifest shape. These five entries did
  not exist at probe time and were not individually probed; their blocked label
  is class-inferred from the controller-scope twins, which all rejected
  `0x2107`. The pre-1.2.0 full-coverage gate run exercises them directly.
- The manifest now expands to 2304 targets: 2268 writeable, 17
  current-encoding blocked, and 19 read-only.
