# Writing and Reading Logix STRINGs (built-in and custom types)

How this library handles Allen-Bradley Logix string tags — the built-in `STRING` type, custom
user-defined string types (your own name and length), the size limits, and how to use the API
for each case. Validated on CompactLogix 5069-L330ERM firmware 38.

## TL;DR

- **Built-in `STRING`** (82-char) works everywhere, standalone and as a UDT member.
- **Custom string types** (e.g. `Str82`, `Str400` — your own name/length) also work, because the
  library discovers the tag's real *structure handle* at write time. Use the same
  `write_tag` / `WriteString` / `write_tag` API as for built-in strings.
- **Large strings:** strings that exceed one CIP packet use CIP Read Tag Fragmented (`0x52`) and
  Write Tag Fragmented (`0x53`). Simulator coverage proves `Str500+`-sized payloads round-trip;
  maintainer hardware re-validation should confirm the same on the next PLC session.

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
| Custom string **larger than one CIP packet** (`Str500`+) | type-specific | fragmented read/write path |

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
value = client.read_string("gTestUDT.Member5_String")            # custom type -> text
```
`write_tag(tag, "text")` writes any string type. For reading a **custom** string type, use the
string read path (the generic `read_tag` returns a structure for custom types).

### C/C++

```cpp
client.write_string("gTestUDT.Member5_String", "custom Str82");  // works
auto v = client.read_string("gTestUDT.Member7_String");          // decodes built-in or custom
```

## Large strings and fragmentation

Small strings keep the single-packet fast path. When a custom string/structure is too large for
one packet, the client now uses:

- **Read Tag Fragmented (`0x52`)** after a normal read reports CIP `0x06` Partial Transfer.
- **Write Tag Fragmented (`0x53`)** when the handle-aware structure write would exceed the
  measured single-packet write ceiling.

The fragmented path is simulator-covered with a 600-byte custom string payload. The earlier
5069-L330ERM fw38 measurements still explain why fragmentation is needed:

- Single write request ceiling: about **494 bytes total**.
- Single read reply ceiling: about **500 bytes of value** before CIP `0x06`.

Hardware re-validation for `Str500+` controller/program-scope tags remains a release-gate
confidence item.

## Error reference

| Symptom | Cause | Fix |
|---|---|---|
| `0x2107` on a string write | request handle didn't match the target's type (old library, or a bug) | ensure the tag is a string type; the current library discovers the handle automatically |
| `0x06` Partial Transfer on read | structure larger than one CIP reply (`Str500`+) | current client should continue with fragmented reads; report if surfaced to callers |
| `0x03` bad length / "over the single-packet limit" on write | write request exceeds one packet before fragmentation applies | report with tag path and string type size |
| Read returns a `Udt` instead of a string | it's a custom string type read via `read_tag` | use `read_string_tag` / `ReadString` / `read_string` |

## What's not supported yet

- **`read_tag` auto-decoding a custom string to text.** By design `read_tag` returns custom
  string types as `Udt` (a custom handle is indistinguishable from a UDT); use `read_string_tag`.

See also [`docs/agents/notes/ab-firmware-quirks.md`](agents/notes/ab-firmware-quirks.md) (STRING
Members) and the validation record
[`docs/validation/2026-07-08_cross-binding_full-coverage_5069-L330ERM_fw38.md`](validation/2026-07-08_cross-binding_full-coverage_5069-L330ERM_fw38.md).
