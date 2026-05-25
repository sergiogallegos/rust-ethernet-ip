use crate::tag_manager::{
    TagMetadata, TagPermissions as MetadataPermissions, TagScope as MetadataScope,
};
use crate::udt::{TagAttributes, TagPermissions, TagScope, UdtDefinition, UdtMember};
use crate::{RouteHop, RoutePath};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaExport {
    pub schema_version: String,
    pub generated_at_utc: String,
    pub library: SchemaLibraryInfo,
    pub target: SchemaTargetInfo,
    pub capabilities: SchemaCapabilities,
    pub tags: Vec<SchemaTag>,
    pub udts: Vec<SchemaUdt>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaLibraryInfo {
    pub name: String,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaTargetInfo {
    pub address: Option<String>,
    pub route_path: Option<SchemaRoutePath>,
    pub controller_family: Option<String>,
    pub firmware_revision: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaRoutePath {
    pub slots: Vec<u8>,
    pub ports: Vec<u8>,
    pub addresses: Vec<String>,
    pub hops: Vec<SchemaRouteHop>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SchemaRouteHop {
    Backplane { port: u8, slot: u8 },
    Ethernet { port: u8, address: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaCapabilities {
    pub tag_discovery: bool,
    pub tag_attributes: bool,
    pub udt_definitions: bool,
    pub program_tags: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaTag {
    pub name: String,
    pub scope: SchemaScope,
    pub data_type: SchemaDataType,
    pub dimensions: Vec<u32>,
    pub size_bytes: u32,
    pub permissions: String,
    pub template_instance_id: Option<u32>,
    pub udt_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaUdt {
    pub name: String,
    pub template_instance_id: Option<u32>,
    pub size_bytes: u32,
    pub members: Vec<SchemaUdtMember>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaUdtMember {
    pub name: String,
    pub offset_bytes: u32,
    pub size_bytes: u32,
    pub data_type: SchemaDataType,
    pub dimensions: Vec<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaScope {
    pub kind: String,
    pub program: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaDataType {
    pub cip_code: u16,
    pub name: String,
    pub kind: String,
}

impl SchemaExport {
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
