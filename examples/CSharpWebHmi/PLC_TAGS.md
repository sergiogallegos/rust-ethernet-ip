# PLC Tags for the Web HMI Demo

This is the minimal Studio 5000 tag set read by the dashboard. Names and types
are case-sensitive from the application's perspective.

## Controller-Scoped Tags

Open **Controller Tags** and create:

| Name | Logix data type | Minimum dimensions | Dashboard use |
|---|---|---:|---|
| `gTestArray_DINT` | `DINT` array | `[8]` | Eight counter channels |
| `gTestArray_REAL` | `REAL` array | `[8]` | Eight analog profile channels |
| `gTestArray_BOOL` | `BOOL` array | `[12]` | Twelve digital indicators and optional pulse tag |
| `gTestArray_INT` | `INT` array | `[1]` | One 16-bit integer channel |
| `gTest_STRING` | built-in `STRING` | scalar | Controller-scope message |
| `gTestUDT` | `TEST_UDT` | scalar | Typed UDT member panel |

Suggested starting values:

```text
gTestArray_DINT[0..7] = 100, 240, 360, 515, 680, 820, 940, 1080
gTestArray_REAL[0..7] = 18.2, 24.8, 31.4, 42.0, 55.7, 61.3, 58.9, 67.2
gTestArray_BOOL[0..11] = TRUE, TRUE, FALSE, TRUE, FALSE, FALSE,
                         TRUE, FALSE, FALSE, TRUE, FALSE, FALSE
gTestArray_INT[0] = 128
gTest_STRING = "CELL_READY"
```

The full repository validation PLC uses larger dimensions. Larger arrays work
without changes because the dashboard reads only the indices listed above.

## User-Defined Type

Open **Data Types → User-Defined**, create `TEST_UDT`, and add these members in
this order:

| Member | Logix data type | Example value |
|---|---|---|
| `Member1_DINT` | `DINT` | `1842` |
| `Member2_REAL` | `REAL` | `73.4` |
| `Member3_BOOL` | `BOOL` | `TRUE` |
| `Member4_INT` | `INT` | `42` |
| `Member5_String` | built-in `STRING` | `"STRUCTURE_OK"` |

Create controller tag `gTestUDT` with data type `TEST_UDT`, then enter the
example member values. Extra UDT members are allowed; the comprehensive
hardware test layout also contains nested arrays.

## Program-Scoped Tags

Create or use a program named exactly `TestProgram`. Open **Parameters and
Local Tags** for that program and create:

| Local name | Logix data type | Minimum dimensions | Dashboard path |
|---|---|---:|---|
| `gTestArray_DINT` | `DINT` array | `[1]` | `Program:TestProgram.gTestArray_DINT[0]` |
| `gTestArray_REAL` | `REAL` array | `[1]` | `Program:TestProgram.gTestArray_REAL[0]` |
| `gTestArray_BOOL` | `BOOL` array | `[1]` | `Program:TestProgram.gTestArray_BOOL[0]` |
| `gTest_STRING` | built-in `STRING` | scalar | `Program:TestProgram.gTest_STRING` |

Suggested values:

```text
gTestArray_DINT[0] = 9204
gTestArray_REAL[0] = 64.8
gTestArray_BOOL[0] = TRUE
gTest_STRING = "PROGRAM_READY"
```

The `Program:TestProgram.` prefix is supplied by the application. Do not put
that prefix into the Studio 5000 local tag name.

## External Access

For every tag read by the demo:

1. leave **External Access** set to `Read/Write` for the simplest evaluation;
2. download the project to the controller;
3. confirm the PC can reach the controller or Ethernet module;
4. confirm TCP port `44818` is permitted;
5. identify whether the application should connect directly or route to a CPU
   slot through a ControlLogix communication module.

The demo can monitor read-only tags, but `gTestArray_BOOL` must permit writes
if the optional pulse demonstration is enabled.

## Validation Checklist

Before starting live mode, confirm:

- [ ] all six controller-scoped tags exist;
- [ ] `TEST_UDT` contains the five members with matching types;
- [ ] `gTestUDT` is an instance of `TEST_UDT`;
- [ ] `TestProgram` exists and contains all four local tags;
- [ ] the PC can reach the correct IP address;
- [ ] the CPU slot is known for routed ControlLogix access;
- [ ] no test tag commands real equipment;
- [ ] writes remain disabled unless `gTestArray_BOOL[0]` is safe to pulse.

For the repository's complete 2,304-tag validation layout, use
[`docs/PLC_TEST_TAG_DEFINITIONS.md`](../../docs/PLC_TEST_TAG_DEFINITIONS.md) and
[`examples/full_coverage_tags.json`](../full_coverage_tags.json).
