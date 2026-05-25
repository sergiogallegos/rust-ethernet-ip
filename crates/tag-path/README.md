# rust-ethernet-ip-tag-path

Allen-Bradley Logix tag-path parser for the [`rust-ethernet-ip`](https://crates.io/crates/rust-ethernet-ip) ecosystem.

Parses the full Logix tag-name syntax into a structured `TagPath` and renders it back into CIP path-segment bytes:

- Controller-scoped tags — `"MyTag"`
- Program-scoped tags — `"Program:MainProgram.MyTag"`
- Array elements — `"MyArray[5]"`, `"MyArray[1,2,3]"`
- BOOL array elements — `"gBoolArray[5]"` (DWORD bit-extraction handled at a higher layer)
- Bit access — `"StatusWord.15"`
- UDT members — `"MotorData.Speed"`
- Nested paths — `"Cell_NestData[90].PartData.Member"`

## Who should use this crate

Most consumers want the top-level [`rust-ethernet-ip`](https://crates.io/crates/rust-ethernet-ip) crate, which uses this parser internally and re-exports the user-facing items.

Depend on `rust-ethernet-ip-tag-path` directly only if you are building an independent tool that needs Logix tag-path parsing without the network client (schema validators, code-generators, lints over tag references, etc.).

## Usage

```rust
use rust_ethernet_ip_tag_path::TagPath;

let path = TagPath::parse("Program:Main.MotorData.Speed").unwrap();
let cip_bytes = path.to_cip_path().unwrap();
```

## License

MIT. Part of the [`rust-ethernet-ip`](https://github.com/sergiogallegos/rust-ethernet-ip) workspace.
