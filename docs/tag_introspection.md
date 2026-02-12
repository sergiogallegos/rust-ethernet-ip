# Tag introspection

Use **tag introspection** to discover a tag’s type, size, dimensions, and scope before reading or writing. This is useful for:

- Validating tag names and types at startup
- Building generic tools that work with any tag
- Handling arrays and UDTs when you don’t have a fixed schema

## API

```rust
let attrs = client.get_tag_attributes("MyTag").await?;
```

`TagAttributes` includes:

| Field | Description |
|-------|-------------|
| `name` | Tag name as returned by the PLC |
| `data_type` | CIP type code (e.g. 0xC4 for DINT) |
| `data_type_name` | Human-readable type (e.g. `"DINT"`) |
| `dimensions` | Array dimensions; empty for scalars |
| `permissions` | `ReadOnly`, `ReadWrite`, or `WriteOnly` |
| `scope` | `Controller` or `Program(name)` |
| `size` | Size in bytes |
| `template_instance_id` | Set for UDTs (for template discovery) |

## Example: discover then read

```rust
use rust_ethernet_ip::{EipClient, PlcValue};

let attrs = client.get_tag_attributes("ProductionCount").await?;
println!("Type: {}, size: {} bytes", attrs.data_type_name, attrs.size);

let value = client.read_tag("ProductionCount").await?;
match value {
    PlcValue::Dint(n) => println!("Count: {}", n),
    _ => println!("Unexpected type"),
}
```

## Example: check if tag is an array

```rust
let attrs = client.get_tag_attributes("LineData").await?;
if attrs.dimensions.is_empty() {
    println!("Scalar tag");
} else {
    println!("Array dimensions: {:?}", attrs.dimensions);
}
```

## Caching

Attributes are cached per tag for the lifetime of the `EipClient`. The first call for a given tag name performs a CIP Get Attribute List request; subsequent calls for the same tag return the cached result.

## Related

- [`get_udt_definition`](https://docs.rs/rust_ethernet_ip/latest/rust_ethernet_ip/struct.EipClient.html#method.get_udt_definition) for full UDT member layout
- [`discover_tags`](https://docs.rs/rust_ethernet_ip/latest/rust_ethernet_ip/struct.EipClient.html#method.discover_tags) to list controller tags
