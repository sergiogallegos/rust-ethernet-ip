# Writing and Reading Logix STRINGs (built-in and custom types)

How this library handles Allen-Bradley Logix string tags — the built-in `STRING` type, custom
user-defined string types (your own name and length), the size limits, and how to use the API
for each case. Validated on CompactLogix 5069-L330ERM firmware 38.

## TL;DR

- **Built-in `STRING`** (82-char) works everywhere, standalone and as a UDT member.
- **Custom string types** (e.g. `Str82`, `Str400` — your own name/length) also work, because the
  library discovers the tag's real *structure handle* at write time. Use the same
  `write_tag` / `WriteString` / `write_tag` API as for built-in strings.
- **Size limit:** one string write/read must fit in a single CIP packet. The practical maximum
  is about **420–456 bytes of `DATA`** depending on tag-path length (see the table). Larger
  custom types (e.g. `Str500`) are **not supported yet** — they need CIP fragmentation.

## Background: a Logix STRING is a structure, not an atomic type

A Logix `STRING` is a predefined structure:

```
STRING := { LEN : DINT ; DATA : SINT[82] }
```

On the wire, a structure write/read carries a 2-byte **structure handle** in the CIP type field
(`0x02A0` marker + handle). The controller compares the handle in the request against the target
tag's own handle and rejects a mismatch with extended error **`0x2107`** ("tag type used in
request does not match the target tag's data type").

The handle is a fingerprint of the **type definition** (its name and members), **not** the byte
layout. The built-in `STRING` has handle **`0x0FCE`**.

## Default STRING vs. custom string types

In Studio 5000 you can define your own string type with a custom name and length, for example:

```
Str82  := { LEN : DINT ; DATA : SINT[82]  }   // same layout as STRING, different name
Str400 := { LEN : DINT ; DATA : SINT[400] }
```

Even though `Str82` is byte-identical to the built-in `STRING`, **it is a different type with a
different structure handle** (on the validated controller, `Str82` = `0x9621`,
`Str400` = `0x213F`, built-in `STRING` = `0x0FCE`).

Earlier versions of this library hardcoded the built-in handle `0x0FCE` for every string write,
so writes to a custom string type failed with `0x2107`. **The library now discovers the target's
real handle** (by reading the tag first) and writes with it, so custom string types work.

| Case | Structure handle | Write result |
|---|---|---|
| Built-in `STRING` (standalone or UDT member) | `0x0FCE` | works |
| Custom string type (`Str82`, `Str400`, …) | type-specific | works (handle discovered at write time) |
| Custom string **larger than one CIP packet** (`Str500`+) | type-specific | not yet supported — see Limits |

## How to use the library

The public API is the same for built-in and custom string types — you don't specify the handle.

### Rust

```rust
use rust_ethernet_ip::{EipClient, PlcValue, RoutePath};

let mut client = EipClient::with_route_path("192.168.0.1:44818", RoutePath::new().add_slot(0)).await?;

// Write — works for built-in STRING and custom string types alike.
client.write_tag("gTest_STRING", PlcValue::String("hello".into())).await?;
client.write_tag("gTestUDT.Member5_String", PlcValue::String("custom Str82".into())).await?;      // Str82
client.write_tag("gTestUDT.Member7_String", PlcValue::String("custom Str400".into())).await?;      // Str400

// Read back as text (built-in OR custom string type):
let s: String = client.read_string_tag("gTestUDT.Member7_String").await?;
```

Note on reads: `read_tag` decodes the **built-in** `STRING` (handle `0x0FCE`) to
`PlcValue::String`, but returns a **custom** string type as `PlcValue::Udt` (the library can't
tell a custom string from any other structure by handle alone). Use **`read_string_tag`** when
you know a tag is a string of any type — it decodes both.

### C#

```csharp
client.WriteString("gTestUDT.Member5_String", "custom Str82");   // custom type — works
string v = client.ReadString("gTestUDT.Member7_String");         // decodes built-in or custom
```

### Python

```python
client.write_tag("gTestUDT.Member5_String", "custom Str82")      # -> eip_write_string
# read via the string FFI export for custom types:
```
`write_tag(tag, "text")` writes any string type. For reading a **custom** string type, use the
string read path (the generic `read_tag` returns a structure for custom types).

### C/C++

```cpp
client.write_string("gTestUDT.Member5_String", "custom Str82");  // works
auto v = client.read_string("gTestUDT.Member7_String");          // decodes built-in or custom
```

## Limits: maximum string size

A single string read/write must fit in one CIP packet (this library does **not** implement CIP
fragmentation — services `0x52`/`0x53`). Measured on 5069-L330ERM fw38:

- **Write request ceiling: ~494 bytes total** (494 accepted, 498 rejected with encapsulation
  status `0x03`).
- **Read reply ceiling: ~500 bytes of value** (beyond that the controller returns CIP `0x06`
  "Partial Transfer", which needs a fragmented read).

Because the request size includes the tag path, the maximum `DATA` size of a custom string type
depends on where the tag lives:

| Tag location | Max `DATA` (bytes) | Safe custom type |
|---|---|---|
| Controller-scoped standalone / UDT member | ~456 | `Str440` |
| Controller-scoped UDT array element member | ~448 | `Str440` |
| **Program-scoped** UDT member | ~432 | `Str420` |
| **Program-scoped** UDT array element member | ~424 | `Str400` |

**Recommendation:** a custom string type with `DATA ≤ 400 bytes` (`Str400`) works in **every**
scope. If a string only lives in controller-scoped tags, up to `Str440` is fine. `Str500` and
larger do not work in one packet and are unsupported until fragmentation lands.

The library returns a clear error if a write would exceed the single-packet limit (rather than a
raw `0x03`), telling you to use a shorter type or a shorter path.

## Error reference

| Symptom | Cause | Fix |
|---|---|---|
| `0x2107` on a string write | request handle didn't match the target's type (old library, or a bug) | ensure the tag is a string type; the current library discovers the handle automatically |
| `0x06` Partial Transfer on read | structure larger than one CIP reply (`Str500`+) | reduce the custom string size, or wait for fragmentation support |
| `0x03` bad length / "over the single-packet limit" on write | write request exceeds ~494 bytes | use a shorter custom string type or shorter tag path |
| Read returns a `Udt` instead of a string | it's a custom string type read via `read_tag` | use `read_string_tag` / `ReadString` / `read_string` |

## What's not supported yet

- **Custom strings larger than one CIP packet** (`Str500`+, or long strings in deep
  program-scoped paths). Requires CIP Read/Write Tag **Fragmented** (`0x52`/`0x53`). Tracked as a
  library task.
- **`read_tag` auto-decoding a custom string to text.** By design `read_tag` returns custom
  string types as `Udt` (a custom handle is indistinguishable from a UDT); use `read_string_tag`.

See also [`docs/agents/notes/ab-firmware-quirks.md`](agents/notes/ab-firmware-quirks.md) (STRING
Members) and the validation record
[`docs/validation/2026-07-08_cross-binding_full-coverage_5069-L330ERM_fw38.md`](validation/2026-07-08_cross-binding_full-coverage_5069-L330ERM_fw38.md).
