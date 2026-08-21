//! Serializable, language-neutral controller schema export models.

use crate::tag_manager::{
    TagMetadata, TagPermissions as MetadataPermissions, TagScope as MetadataScope,
};
use crate::udt::{TagAttributes, TagPermissions, TagScope, UdtDefinition, UdtMember};
use crate::{RouteHop, RoutePath};
use serde::{Deserialize, Serialize};

/// Complete schema document exported from a connected client.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaExport {
    /// Version of the JSON schema contract, independent of the library version.
    pub schema_version: String,
    /// UTC generation time in RFC 3339 format.
    pub generated_at_utc: String,
    /// Library identity that produced the export.
    pub library: SchemaLibraryInfo,
    /// Controller address, route, and identity information when known.
    pub target: SchemaTargetInfo,
    /// Discovery surfaces available in this export.
    pub capabilities: SchemaCapabilities,
    /// Discovered controller- and program-scoped tags.
    pub tags: Vec<SchemaTag>,
    /// Discovered user-defined types.
    pub udts: Vec<SchemaUdt>,
    /// Omissions or uncertainty consumers should surface.
    pub warnings: Vec<String>,
}

/// Name and version of the exporting library.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaLibraryInfo {
    /// Cargo package name.
    pub name: String,
    /// Semantic library version.
    pub version: String,
}

/// Best-effort identity of the schema source controller.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaTargetInfo {
    /// Socket address, when retained by the client.
    pub address: Option<String>,
    /// Ordered route to the controller, when configured.
    pub route_path: Option<SchemaRoutePath>,
    /// Controller product family, when discovered.
    pub controller_family: Option<String>,
    /// Controller firmware revision, when discovered.
    pub firmware_revision: Option<String>,
}

/// Serializable view of an ordered CIP route.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaRoutePath {
    /// Backplane slots, retained for compatibility with older consumers.
    pub slots: Vec<u8>,
    /// Route ports, retained for compatibility with older consumers.
    pub ports: Vec<u8>,
    /// Network addresses, retained for compatibility with older consumers.
    pub addresses: Vec<String>,
    /// Authoritative ordered route hops.
    pub hops: Vec<SchemaRouteHop>,
}

/// One ordered hop in an exported CIP route.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SchemaRouteHop {
    /// Backplane port followed by a chassis slot.
    Backplane {
        /// CIP port number.
        port: u8,
        /// Target chassis slot.
        slot: u8,
    },
    /// Network port followed by a link address.
    Ethernet {
        /// CIP port number.
        port: u8,
        /// Link address, normally an IP address.
        address: String,
    },
}

/// Indicates which discovery features contributed to an export.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaCapabilities {
    /// Controller tag discovery was available.
    pub tag_discovery: bool,
    /// Detailed tag attributes were available.
    pub tag_attributes: bool,
    /// UDT template definitions were available.
    pub udt_definitions: bool,
    /// Program-scoped tag enumeration was available.
    pub program_tags: bool,
}

/// Portable description of one Logix tag.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaTag {
    /// Fully qualified symbolic tag name.
    pub name: String,
    /// Controller or program scope.
    pub scope: SchemaScope,
    /// CIP data type description.
    pub data_type: SchemaDataType,
    /// Array dimensions, empty for a scalar.
    pub dimensions: Vec<u32>,
    /// Encoded value size in bytes.
    pub size_bytes: u32,
    /// Stable permission label such as `read_write`.
    pub permissions: String,
    /// UDT template instance id, when known.
    pub template_instance_id: Option<u32>,
    /// UDT name, when known.
    pub udt_name: Option<String>,
}

/// Portable description of a Logix user-defined type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaUdt {
    /// Logix data type name.
    pub name: String,
    /// Controller template instance id, when known.
    pub template_instance_id: Option<u32>,
    /// Encoded structure size in bytes.
    pub size_bytes: u32,
    /// Members in template order.
    pub members: Vec<SchemaUdtMember>,
}

/// Portable description of one UDT member.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaUdtMember {
    /// Member name.
    pub name: String,
    /// Byte offset from the start of the structure.
    pub offset_bytes: u32,
    /// Encoded member size in bytes.
    pub size_bytes: u32,
    /// Member CIP data type.
    pub data_type: SchemaDataType,
    /// Array dimensions, empty for a scalar member.
    pub dimensions: Vec<u32>,
}

/// Portable controller/program scope representation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaScope {
    /// Stable scope label: `controller`, `program`, `global`, `local`, or `unknown`.
    pub kind: String,
    /// Program name when `kind` is `program`.
    pub program: Option<String>,
}

/// Portable CIP data type representation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaDataType {
    /// Numeric CIP type code.
    pub cip_code: u16,
    /// Human-readable type name.
    pub name: String,
    /// Stable broad kind such as `integer`, `float`, or `structure`.
    pub kind: String,
}

impl SchemaExport {
    /// Creates an empty export populated with library and route metadata.
    pub fn new(route_path: Option<&RoutePath>) -> Self {
        let warnings = vec![
            "Target address is not currently retained on EipClient and is omitted from schema export."
                .to_string(),
        ];

        Self {
            schema_version: "0.1".to_string(),
            generated_at_utc: current_utc_timestamp_rfc3339(),
            library: SchemaLibraryInfo {
                name: env!("CARGO_PKG_NAME").to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
            },
            target: SchemaTargetInfo {
                address: None,
                route_path: route_path.map(Into::into),
                controller_family: None,
                firmware_revision: None,
            },
            capabilities: SchemaCapabilities {
                tag_discovery: true,
                tag_attributes: true,
                udt_definitions: true,
                program_tags: false,
            },
            tags: Vec::new(),
            udts: Vec::new(),
            warnings,
        }
    }
}

impl From<&RoutePath> for SchemaRoutePath {
    fn from(value: &RoutePath) -> Self {
        Self {
            slots: value.slots(),
            ports: value.ports(),
            addresses: value.addresses(),
            hops: value.hops().iter().map(Into::into).collect(),
        }
    }
}

impl From<&RouteHop> for SchemaRouteHop {
    fn from(value: &RouteHop) -> Self {
        match value {
            RouteHop::Backplane { port, slot } => Self::Backplane {
                port: *port,
                slot: *slot,
            },
            RouteHop::Ethernet { port, address } => Self::Ethernet {
                port: *port,
                address: address.clone(),
            },
        }
    }
}

impl From<&TagAttributes> for SchemaTag {
    fn from(value: &TagAttributes) -> Self {
        Self {
            name: value.name.clone(),
            scope: schema_scope_from_tag_attributes(&value.scope),
            data_type: SchemaDataType::from_cip(value.data_type, &value.data_type_name),
            dimensions: value.dimensions.clone(),
            size_bytes: value.size,
            permissions: schema_permissions_from_tag_attributes(&value.permissions),
            template_instance_id: value.template_instance_id,
            udt_name: (value.data_type == 0x00A0).then(|| value.name.clone()),
        }
    }
}

impl From<&TagMetadata> for SchemaTag {
    fn from(value: &TagMetadata) -> Self {
        Self {
            name: String::new(),
            scope: schema_scope_from_metadata(&value.scope),
            data_type: SchemaDataType::from_cip(value.data_type, data_type_name(value.data_type)),
            dimensions: value.dimensions.clone(),
            size_bytes: value.size,
            permissions: schema_permissions_from_metadata(&value.permissions),
            template_instance_id: None,
            udt_name: value.is_structure().then(|| "structure".to_string()),
        }
    }
}

impl SchemaUdt {
    /// Converts an internal UDT definition into the portable schema form.
    pub fn from_definition(
        definition: &UdtDefinition,
        template_instance_id: Option<u32>,
        source_tag_size: u32,
    ) -> Self {
        Self {
            name: definition.name.clone(),
            template_instance_id,
            size_bytes: source_tag_size,
            members: definition
                .members
                .iter()
                .map(SchemaUdtMember::from)
                .collect(),
        }
    }
}

impl From<&UdtMember> for SchemaUdtMember {
    fn from(value: &UdtMember) -> Self {
        Self {
            name: value.name.clone(),
            offset_bytes: value.offset,
            size_bytes: value.size,
            data_type: SchemaDataType::from_cip(value.data_type, data_type_name(value.data_type)),
            dimensions: Vec::new(),
        }
    }
}

impl SchemaDataType {
    /// Creates a portable data type from a CIP code and display name.
    pub fn from_cip(cip_code: u16, name: &str) -> Self {
        Self {
            cip_code,
            name: name.to_string(),
            kind: data_type_kind(cip_code).to_string(),
        }
    }
}

fn schema_scope_from_tag_attributes(scope: &TagScope) -> SchemaScope {
    match scope {
        TagScope::Controller => SchemaScope {
            kind: "controller".to_string(),
            program: None,
        },
        TagScope::Program(name) => SchemaScope {
            kind: "program".to_string(),
            program: Some(name.clone()),
        },
        TagScope::Unknown => SchemaScope {
            kind: "unknown".to_string(),
            program: None,
        },
    }
}

fn schema_scope_from_metadata(scope: &MetadataScope) -> SchemaScope {
    match scope {
        MetadataScope::Controller => SchemaScope {
            kind: "controller".to_string(),
            program: None,
        },
        MetadataScope::Program(name) => SchemaScope {
            kind: "program".to_string(),
            program: Some(name.clone()),
        },
        MetadataScope::Global => SchemaScope {
            kind: "global".to_string(),
            program: None,
        },
        MetadataScope::Local => SchemaScope {
            kind: "local".to_string(),
            program: None,
        },
    }
}

fn schema_permissions_from_tag_attributes(permissions: &TagPermissions) -> String {
    match permissions {
        TagPermissions::ReadOnly => "read_only".to_string(),
        TagPermissions::ReadWrite => "read_write".to_string(),
        TagPermissions::WriteOnly => "write_only".to_string(),
        TagPermissions::Unknown => "unknown".to_string(),
    }
}

fn schema_permissions_from_metadata(permissions: &MetadataPermissions) -> String {
    match (permissions.readable, permissions.writable) {
        (true, true) => "read_write",
        (true, false) => "read_only",
        (false, true) => "write_only",
        (false, false) => "unknown",
    }
    .to_string()
}

fn data_type_kind(cip_code: u16) -> &'static str {
    match cip_code {
        0x00A0 | 0x02A0 => "udt",
        0x00CE | 0x00DA => "string",
        0x00C1..=0x00CB | 0x00D3 => "primitive",
        _ => "unknown",
    }
}

fn data_type_name(cip_code: u16) -> &'static str {
    match cip_code {
        0x00A0 => "UDT",
        0x02A0 => "STRUCTURE",
        0x00C1 => "BOOL",
        0x00C2 => "SINT",
        0x00C3 => "INT",
        0x00C4 => "DINT",
        0x00C5 => "LINT",
        0x00C6 => "USINT",
        0x00C7 => "UINT",
        0x00C8 => "UDINT",
        0x00C9 => "ULINT",
        0x00CA => "REAL",
        0x00CB => "LREAL",
        0x00CE => "STRING",
        0x00DA => "STRING",
        0x00D3 => "UDINT",
        _ => "UNKNOWN",
    }
}

fn current_utc_timestamp_rfc3339() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    format_unix_seconds_as_rfc3339(secs)
}

// Howard Hinnant's civil-from-days algorithm; valid for any i64 Unix second.
// Avoids platform libc divergence (gmtime_r/gmtime_s/strftime).
fn format_unix_seconds_as_rfc3339(secs: i64) -> String {
    let days = secs.div_euclid(86_400);
    let tod = secs.rem_euclid(86_400);
    let hour = (tod / 3600) as u32;
    let minute = ((tod % 3600) / 60) as u32;
    let second = (tod % 60) as u32;

    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    let year = if m <= 2 { y + 1 } else { y };

    format!("{year:04}-{m:02}-{d:02}T{hour:02}:{minute:02}:{second:02}Z")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::udt::{TagAttributes, TagPermissions, TagScope, UdtDefinition, UdtMember};

    #[test]
    fn schema_data_type_classifies_core_types() {
        assert_eq!(SchemaDataType::from_cip(0x00C4, "DINT").kind, "primitive");
        assert_eq!(SchemaDataType::from_cip(0x00CE, "STRING").kind, "string");
        assert_eq!(SchemaDataType::from_cip(0x00A0, "UDT").kind, "udt");
    }

    #[test]
    fn timestamp_helper_returns_rfc3339_utc_shape() {
        let timestamp = current_utc_timestamp_rfc3339();
        assert_eq!(timestamp.len(), 20);
        assert!(timestamp.ends_with('Z'));
        assert_eq!(&timestamp[4..5], "-");
        assert_eq!(&timestamp[7..8], "-");
        assert_eq!(&timestamp[10..11], "T");
    }

    #[test]
    fn rfc3339_format_matches_known_unix_seconds() {
        assert_eq!(format_unix_seconds_as_rfc3339(0), "1970-01-01T00:00:00Z");
        assert_eq!(
            format_unix_seconds_as_rfc3339(1_700_000_000),
            "2023-11-14T22:13:20Z"
        );
        // 2024-02-29T12:34:56Z — leap year boundary.
        assert_eq!(
            format_unix_seconds_as_rfc3339(1_709_210_096),
            "2024-02-29T12:34:56Z"
        );
    }

    #[test]
    fn schema_tag_maps_program_scope_and_template_id() {
        let attrs = TagAttributes {
            name: "Program:Main.MotorData".to_string(),
            data_type: 0x00A0,
            data_type_name: "UDT".to_string(),
            dimensions: vec![4],
            permissions: TagPermissions::ReadWrite,
            scope: TagScope::Program("Main".to_string()),
            template_instance_id: Some(123),
            size: 64,
        };

        let tag = SchemaTag::from(&attrs);
        assert_eq!(tag.name, "Program:Main.MotorData");
        assert_eq!(tag.scope.kind, "program");
        assert_eq!(tag.scope.program.as_deref(), Some("Main"));
        assert_eq!(tag.data_type.kind, "udt");
        assert_eq!(tag.template_instance_id, Some(123));
        assert_eq!(tag.dimensions, vec![4]);
        assert_eq!(tag.permissions, "read_write");
        assert_eq!(tag.udt_name.as_deref(), Some("Program:Main.MotorData"));
    }

    #[test]
    fn schema_udt_maps_members_and_size() {
        let definition = UdtDefinition {
            name: "MotorData".to_string(),
            members: vec![
                UdtMember {
                    name: "Speed".to_string(),
                    data_type: 0x00CA,
                    offset: 0,
                    size: 4,
                },
                UdtMember {
                    name: "Enabled".to_string(),
                    data_type: 0x00C1,
                    offset: 4,
                    size: 1,
                },
            ],
        };

        let udt = SchemaUdt::from_definition(&definition, Some(77), 64);
        assert_eq!(udt.name, "MotorData");
        assert_eq!(udt.template_instance_id, Some(77));
        assert_eq!(udt.size_bytes, 64);
        assert_eq!(udt.members.len(), 2);
        assert_eq!(udt.members[0].name, "Speed");
        assert_eq!(udt.members[0].data_type.name, "REAL");
        assert_eq!(udt.members[1].name, "Enabled");
        assert_eq!(udt.members[1].data_type.name, "BOOL");
    }

    #[test]
    fn schema_export_serializes_stable_top_level_fields() {
        let mut export = SchemaExport::new(None);
        export.tags.push(SchemaTag {
            name: "ProductionCount".to_string(),
            scope: SchemaScope {
                kind: "controller".to_string(),
                program: None,
            },
            data_type: SchemaDataType::from_cip(0x00C4, "DINT"),
            dimensions: Vec::new(),
            size_bytes: 4,
            permissions: "read_write".to_string(),
            template_instance_id: None,
            udt_name: None,
        });

        let json = serde_json::to_value(&export).expect("serialize schema export");
        assert_eq!(json["schema_version"], "0.1");
        assert_eq!(json["library"]["name"], env!("CARGO_PKG_NAME"));
        assert!(json["generated_at_utc"].as_str().is_some());
        assert!(json["tags"].is_array());
        assert!(json["warnings"].is_array());
    }
}
