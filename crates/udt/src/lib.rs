//! UDT template parsing, cached metadata, and member value conversion.

use rust_ethernet_ip_types::{PlcValue, TypeError, UdtCodec, UdtData};

/// Result type returned by UDT operations.
pub type Result<T> = std::result::Result<T, UdtError>;

/// Error returned while parsing or converting a UDT.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UdtError {
    /// The PLC response or UDT payload is malformed.
    Protocol(String),
    /// The requested tag was not found.
    TagNotFound(String),
    /// A member value has a different type than its definition.
    DataTypeMismatch {
        /// Data type required by the UDT definition.
        expected: String,
        /// Data type supplied by the caller.
        actual: String,
    },
}

impl UdtError {
    /// Creates a protocol-format error.
    pub fn protocol(message: impl Into<String>) -> Self {
        Self::Protocol(message.into())
    }
}

impl std::fmt::Display for UdtError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UdtError::Protocol(message) => write!(f, "Protocol error: {message}"),
            UdtError::TagNotFound(tag) => write!(f, "Tag not found: {tag}"),
            UdtError::DataTypeMismatch { expected, actual } => {
                write!(f, "Data type mismatch: expected {expected}, got {actual}")
            }
        }
    }
}

impl std::error::Error for UdtError {}
use std::collections::HashMap;

/// Definition of a User Defined Type
#[derive(Debug, Clone)]
pub struct UdtDefinition {
    /// Logix data type name.
    pub name: String,
    /// Members in template order.
    pub members: Vec<UdtMember>,
}

/// Member of a UDT
#[derive(Debug, Clone)]
pub struct UdtMember {
    /// Member name.
    pub name: String,
    /// CIP data type code.
    pub data_type: u16,
    /// Byte offset from the start of the structure.
    pub offset: u32,
    /// Encoded member size in bytes.
    pub size: u32,
}

/// UDT Template information from PLC
#[derive(Debug, Clone)]
pub struct UdtTemplate {
    /// Template object instance id.
    pub template_id: u32,
    /// Logix data type name.
    pub name: String,
    /// Encoded structure size in bytes.
    pub size: u32,
    /// Member count reported by the controller.
    pub member_count: u16,
    /// Parsed, named members.
    pub members: Vec<UdtMember>,
}

/// Tag attributes from PLC
#[derive(Debug, Clone)]
pub struct TagAttributes {
    /// Fully qualified symbolic tag name.
    pub name: String,
    /// CIP data type code.
    pub data_type: u16,
    /// Human-readable data type name.
    pub data_type_name: String,
    /// Declared array dimensions, or an empty vector for a scalar.
    pub dimensions: Vec<u32>,
    /// Reported access permissions.
    pub permissions: TagPermissions,
    /// Controller or program scope.
    pub scope: TagScope,
    /// Template instance id for a structured value, when known.
    pub template_instance_id: Option<u32>,
    /// Encoded tag size in bytes.
    pub size: u32,
}

#[derive(Clone, Copy)]
struct RawMember {
    info: u16,
    raw_type: u16,
    offset: u32,
}

/// Tag permissions
#[derive(Debug, Clone, PartialEq)]
pub enum TagPermissions {
    /// Reads are allowed and writes are not.
    ReadOnly,
    /// Reads and writes are allowed.
    ReadWrite,
    /// Writes are allowed and reads are not.
    WriteOnly,
    /// Permissions were not reported or could not be inferred.
    Unknown,
}

/// Tag scope
#[derive(Debug, Clone, PartialEq)]
pub enum TagScope {
    /// Controller-scoped tag.
    Controller,
    /// Program-scoped tag carrying the program name.
    Program(String),
    /// Scope was not reported or could not be inferred.
    Unknown,
}

/// Manager for UDT operations
#[derive(Debug)]
pub struct UdtManager {
    definitions: HashMap<String, UdtDefinition>,
    templates: HashMap<u32, UdtTemplate>,
    tag_attributes: HashMap<String, TagAttributes>,
}

impl UdtManager {
    /// Creates an empty UDT metadata cache.
    pub fn new() -> Self {
        Self {
            definitions: HashMap::new(),
            templates: HashMap::new(),
            tag_attributes: HashMap::new(),
        }
    }

    /// Adds a UDT definition to the cache
    pub fn add_definition(&mut self, definition: UdtDefinition) {
        self.definitions.insert(definition.name.clone(), definition);
    }

    /// Gets a cached UDT definition
    pub fn get_definition(&self, name: &str) -> Option<&UdtDefinition> {
        self.definitions.get(name)
    }

    /// Adds a UDT template to the cache
    pub fn add_template(&mut self, template: UdtTemplate) {
        self.templates.insert(template.template_id, template);
    }

    /// Gets a cached UDT template
    pub fn get_template(&self, template_id: u32) -> Option<&UdtTemplate> {
        self.templates.get(&template_id)
    }

    /// Adds tag attributes to the cache
    pub fn add_tag_attributes(&mut self, attributes: TagAttributes) {
        self.tag_attributes
            .insert(attributes.name.clone(), attributes);
    }

    /// Gets cached tag attributes
    pub fn get_tag_attributes(&self, name: &str) -> Option<&TagAttributes> {
        self.tag_attributes.get(name)
    }

    /// Lists all cached UDT definitions
    pub fn list_definitions(&self) -> Vec<String> {
        self.definitions.keys().cloned().collect()
    }

    /// Lists all cached templates
    pub fn list_templates(&self) -> Vec<u32> {
        self.templates.keys().cloned().collect()
    }

    /// Lists all cached tag attributes
    pub fn list_tag_attributes(&self) -> Vec<String> {
        self.tag_attributes.keys().cloned().collect()
    }

    /// Clears all caches
    pub fn clear_cache(&mut self) {
        self.definitions.clear();
        self.templates.clear();
        self.tag_attributes.clear();
    }

    /// Parses UDT template data from the Template Read service payload.
    pub fn parse_udt_template(
        &self,
        template_id: u32,
        member_count: u16,
        structure_size: u32,
        data: &[u8],
    ) -> Result<UdtTemplate> {
        let member_section_len = member_count as usize * 8;
        if data.len() < member_section_len {
            return Err(UdtError::protocol(
                "UDT template data too short".to_string(),
            ));
        }

        let mut raw_members = Vec::with_capacity(member_count as usize);
        for i in 0..member_count as usize {
            let base = i * 8;
            raw_members.push(RawMember {
                info: u16::from_le_bytes([data[base], data[base + 1]]),
                raw_type: u16::from_le_bytes([data[base + 2], data[base + 3]]),
                offset: u32::from_le_bytes([
                    data[base + 4],
                    data[base + 5],
                    data[base + 6],
                    data[base + 7],
                ]),
            });
        }

        let strings = self.parse_null_terminated_strings(&data[member_section_len..]);
        let template_name = strings
            .first()
            .and_then(|value| value.split(';').next())
            .filter(|value| !value.is_empty())
            .map(|value| value.to_string())
            .unwrap_or_else(|| format!("Template_{}", template_id));

        let member_names = strings.into_iter().skip(1);
        let mut members = Vec::new();

        for (raw_member, member_name) in raw_members.into_iter().zip(member_names) {
            if member_name.is_empty() || member_name.starts_with("ZZZZZZZZZZ") {
                continue;
            }

            let normalized_type = self.normalize_member_data_type(raw_member.raw_type);
            let member_size = self.estimate_member_size(
                raw_member,
                normalized_type,
                &data[..member_section_len],
                structure_size,
            );

            members.push(UdtMember {
                name: member_name,
                data_type: normalized_type,
                offset: raw_member.offset,
                size: member_size,
            });
        }

        Ok(UdtTemplate {
            template_id,
            name: template_name,
            size: structure_size,
            member_count,
            members,
        })
    }

    fn parse_null_terminated_strings(&self, data: &[u8]) -> Vec<String> {
        data.split(|byte| *byte == 0)
            .map(|chunk| String::from_utf8_lossy(chunk).to_string())
            .collect()
    }

    fn normalize_member_data_type(&self, raw_type: u16) -> u16 {
        let base_type = raw_type & 0x0FFF;
        let is_structure = (raw_type & 0x8000) != 0;

        if is_structure {
            0x00A0
        } else if base_type != 0 {
            base_type
        } else {
            raw_type
        }
    }

    fn estimate_member_size(
        &self,
        raw_member: RawMember,
        normalized_type: u16,
        member_section: &[u8],
        structure_size: u32,
    ) -> u32 {
        let is_array = (raw_member.raw_type & 0x2000) != 0;
        let is_structure = (raw_member.raw_type & 0x8000) != 0;

        if normalized_type == 0x00C1 {
            return 1;
        }

        if is_array {
            let element_size = self.get_data_type_size(normalized_type);
            return if normalized_type == 0x00D3 {
                element_size
            } else {
                element_size.saturating_mul(raw_member.info as u32)
            };
        }

        if is_structure {
            let next_offset = member_section
                .as_chunks::<8>()
                .0
                .iter()
                .filter_map(|chunk| {
                    let offset = u32::from_le_bytes([chunk[4], chunk[5], chunk[6], chunk[7]]);
                    (offset > raw_member.offset).then_some(offset)
                })
                .min()
                .unwrap_or(structure_size);

            return next_offset.saturating_sub(raw_member.offset).max(1);
        }

        self.get_data_type_size(normalized_type)
    }

    /// Gets the size of a data type in bytes
    fn get_data_type_size(&self, data_type: u16) -> u32 {
        match data_type {
            0x00C1 => 1,  // BOOL
            0x00C2 => 1,  // SINT (8-bit signed)
            0x00C3 => 2,  // INT (16-bit signed)
            0x00C4 => 4,  // DINT (32-bit signed)
            0x00C5 => 8,  // LINT (64-bit signed)
            0x00C6 => 1,  // USINT (8-bit unsigned)
            0x00C7 => 2,  // UINT (16-bit unsigned)
            0x00C8 => 4,  // UDINT (32-bit unsigned)
            0x00C9 => 8,  // ULINT (64-bit unsigned)
            0x00CA => 4,  // REAL (32-bit float)
            0x00CB => 8,  // LREAL (64-bit float)
            0x00CE => 88, // STRING (4-byte DINT length + 82 chars + 2 padding)
            _ => 4,       // Default to 4 bytes for unknown types
        }
    }

    /// Parses tag attributes from CIP response
    pub fn parse_tag_attributes(&self, tag_name: &str, data: &[u8]) -> Result<TagAttributes> {
        if data.len() < 8 {
            return Err(UdtError::protocol(
                "Tag attributes data too short".to_string(),
            ));
        }

        let mut offset = 0;

        // Parse data type
        let data_type = u16::from_le_bytes([data[offset], data[offset + 1]]);
        offset += 2;

        // Parse size
        let size = u32::from_le_bytes([
            data[offset],
            data[offset + 1],
            data[offset + 2],
            data[offset + 3],
        ]);
        offset += 4;

        // Parse dimensions (if present)
        let mut dimensions = Vec::new();
        if data.len() > offset {
            let dimension_count = data[offset] as usize;
            offset += 1;

            for _ in 0..dimension_count {
                if offset + 4 <= data.len() {
                    let dim = u32::from_le_bytes([
                        data[offset],
                        data[offset + 1],
                        data[offset + 2],
                        data[offset + 3],
                    ]);
                    dimensions.push(dim);
                    offset += 4;
                }
            }
        }

        // Parse permissions (simplified - would need more CIP data)
        let permissions = TagPermissions::ReadWrite; // Default assumption

        // Parse scope (simplified - would need more CIP data)
        let scope = if tag_name.contains(':') {
            let parts: Vec<&str> = tag_name.split(':').collect();
            if parts.len() >= 2 {
                TagScope::Program(parts[0].to_string())
            } else {
                TagScope::Controller
            }
        } else {
            TagScope::Controller
        };

        // Get data type name
        let data_type_name = self.get_data_type_name(data_type);

        // Check if this is a UDT (has template instance ID)
        let template_instance_id = if data_type == 0x00A0 {
            // UDT type
            Some(0) // Would need to extract from additional CIP data
        } else {
            None
        };

        Ok(TagAttributes {
            name: tag_name.to_string(),
            data_type,
            data_type_name,
            dimensions,
            permissions,
            scope,
            template_instance_id,
            size,
        })
    }

    /// Gets the human-readable name of a data type
    fn get_data_type_name(&self, data_type: u16) -> String {
        match data_type {
            0x00C1 => "BOOL".to_string(),
            0x00C2 => "SINT".to_string(),
            0x00C3 => "INT".to_string(),
            0x00C4 => "DINT".to_string(),
            0x00C5 => "LINT".to_string(),
            0x00C6 => "USINT".to_string(),
            0x00C7 => "UINT".to_string(),
            0x00C8 => "UDINT".to_string(),
            0x00C9 => "ULINT".to_string(),
            0x00CA => "REAL".to_string(),
            0x00CB => "LREAL".to_string(),
            0x00CE => "STRING".to_string(),
            0x00A0 => "UDT".to_string(),
            _ => format!("UNKNOWN(0x{:04X})", data_type),
        }
    }

    /// Parse a UDT instance from raw bytes
    ///
    /// Returns raw UDT data in generic format. Note: symbol_id will be 0
    /// since it's not available in this context. For proper UDT handling with
    /// symbol_id, use read_tag() which gets tag attributes first.
    pub fn parse_udt_instance(&self, _udt_name: &str, data: &[u8]) -> Result<PlcValue> {
        // Return raw UDT data in generic format
        // symbol_id is 0 since it's not available in this context
        Ok(PlcValue::Udt(UdtData {
            symbol_id: 0, // Not available in this context
            data: data.to_vec(),
        }))
    }

    /// Serialize a UDT instance to bytes
    pub fn serialize_udt_instance(
        &self,
        _udt_value: &HashMap<String, PlcValue>,
    ) -> Result<Vec<u8>> {
        Err(UdtError::protocol(
            "UDT instance serialization is not implemented yet".to_string(),
        ))
    }
}

impl Default for UdtManager {
    fn default() -> Self {
        Self::new()
    }
}

// Note: Types are already defined above, no need to re-export

/// Represents a User Defined Type (UDT)
#[derive(Debug, Clone)]
pub struct UserDefinedType {
    /// Name of the UDT
    pub name: String,
    /// Total size of the UDT in bytes
    pub size: u32,
    /// Members of the UDT
    pub members: Vec<UdtMember>,
    /// Cache of member offsets for quick lookup
    member_offsets: HashMap<String, u32>,
    /// Optional Logix BOOL bit index by member name for packed BOOL members.
    member_bit_indices: HashMap<String, u8>,
}

impl UserDefinedType {
    /// Creates a new UDT
    pub fn new(name: String) -> Self {
        Self {
            name,
            size: 0,
            members: Vec::new(),
            member_offsets: HashMap::new(),
            member_bit_indices: HashMap::new(),
        }
    }

    /// Adds a member to the UDT
    pub fn add_member(&mut self, member: UdtMember) {
        self.add_member_with_bit_index(member, None);
    }

    /// Adds a member with optional packed-BOOL bit metadata.
    pub fn add_member_with_bit_index(&mut self, member: UdtMember, bit_index: Option<u8>) {
        self.member_offsets
            .insert(member.name.clone(), member.offset);
        if let Some(bit_index) = bit_index {
            self.member_bit_indices
                .insert(member.name.clone(), bit_index.min(7));
        }
        self.members.push(member);
        // Calculate total size including padding
        self.size = self
            .members
            .iter()
            .map(|m| m.offset + m.size)
            .max()
            .unwrap_or(0);
    }

    /// Gets the offset of a member by name
    pub fn get_member_offset(&self, name: &str) -> Option<u32> {
        self.member_offsets.get(name).copied()
    }

    /// Parses a UDT from CIP data
    pub fn from_cip_data(_data: &[u8]) -> Result<Self> {
        Err(UdtError::protocol(
            "UDT CIP definition parsing is not implemented yet".to_string(),
        ))
    }

    /// Converts a UDT instance to a `HashMap` of member values
    pub fn to_hash_map(&self, data: &[u8]) -> Result<HashMap<String, PlcValue>> {
        if data.is_empty() {
            return Err(UdtError::protocol("UDT data is empty".to_string()));
        }

        let mut result = HashMap::new();

        for member in &self.members {
            let offset = member.offset as usize;
            let size = member.size as usize;
            if offset.checked_add(size).is_none_or(|end| end > data.len()) {
                return Err(UdtError::protocol(format!(
                    "Member {} data incomplete: offset {} size {} exceeds UDT data length {}",
                    member.name,
                    member.offset,
                    member.size,
                    data.len()
                )));
            }

            let member_data = &data[offset..offset + size];
            let value = self.parse_member_value(member, member_data)?;
            result.insert(member.name.clone(), value);
        }

        Ok(result)
    }

    /// Converts a `HashMap` of member values to raw UDT bytes
    pub fn from_hash_map(&self, values: &HashMap<String, PlcValue>) -> Result<Vec<u8>> {
        let mut data = vec![0u8; self.size as usize];

        for member in &self.members {
            let Some(value) = values.get(&member.name) else {
                return Err(UdtError::protocol(format!(
                    "Missing UDT member value: {}",
                    member.name
                )));
            };

            if member.data_type == 0x00C1
                && let Some(bit_index) = self.member_bit_indices.get(&member.name)
            {
                let PlcValue::Bool(value) = value else {
                    return Err(UdtError::DataTypeMismatch {
                        expected: "BOOL".to_string(),
                        actual: format!("{:?}", value),
                    });
                };
                let offset = member.offset as usize;
                let Some(byte) = data.get_mut(offset) else {
                    return Err(UdtError::protocol(format!(
                        "Member {} data exceeds UDT size",
                        member.name
                    )));
                };
                let mask = 1_u8 << (*bit_index).min(7);
                if *value {
                    *byte |= mask;
                } else {
                    *byte &= !mask;
                }
                continue;
            }

            let member_data = self.serialize_member_value(member, value)?;
            let offset = member.offset as usize;
            let end_offset = offset + member_data.len();

            if end_offset <= data.len() {
                data[offset..end_offset].copy_from_slice(&member_data);
            } else {
                return Err(UdtError::protocol(format!(
                    "Member {} data exceeds UDT size",
                    member.name
                )));
            }
        }

        Ok(data)
    }

    /// Reads a specific UDT member by name
    pub fn read_member(&self, data: &[u8], member_name: &str) -> Result<PlcValue> {
        if let Some(member) = self.members.iter().find(|m| m.name == member_name) {
            let offset = member.offset as usize;
            if offset + member.size as usize <= data.len() {
                let member_data = &data[offset..offset + member.size as usize];
                self.parse_member_value(member, member_data)
            } else {
                Err(UdtError::protocol(format!(
                    "Member {} data incomplete",
                    member_name
                )))
            }
        } else {
            Err(UdtError::TagNotFound(format!(
                "UDT member '{}' not found",
                member_name
            )))
        }
    }

    /// Writes a specific UDT member by name
    pub fn write_member(&self, data: &mut [u8], member_name: &str, value: &PlcValue) -> Result<()> {
        if let Some(member) = self.members.iter().find(|m| m.name == member_name) {
            let member_data = self.serialize_member_value(member, value)?;
            let offset = member.offset as usize;
            let end_offset = offset + member_data.len();

            if end_offset <= data.len() {
                data[offset..end_offset].copy_from_slice(&member_data);
                Ok(())
            } else {
                Err(UdtError::protocol(format!(
                    "Member {} data exceeds UDT size",
                    member_name
                )))
            }
        } else {
            Err(UdtError::TagNotFound(format!(
                "UDT member '{}' not found",
                member_name
            )))
        }
    }

    /// Gets the size of a specific member
    pub fn get_member_size(&self, member_name: &str) -> Option<u32> {
        self.members
            .iter()
            .find(|m| m.name == member_name)
            .map(|m| m.size)
    }

    /// Gets the data type of a specific member
    pub fn get_member_data_type(&self, member_name: &str) -> Option<u16> {
        self.members
            .iter()
            .find(|m| m.name == member_name)
            .map(|m| m.data_type)
    }

    /// Parses a member value from raw data
    pub fn parse_member_value(&self, member: &UdtMember, data: &[u8]) -> Result<PlcValue> {
        match member.data_type {
            0x00C1 => {
                if data.is_empty() {
                    return Err(UdtError::protocol("BOOL data too short".to_string()));
                }
                let value = if let Some(bit_index) = self.member_bit_indices.get(&member.name) {
                    let mask = 1_u8 << (*bit_index).min(7);
                    data[0] & mask != 0
                } else {
                    data[0] != 0
                };
                Ok(PlcValue::Bool(value))
            }
            0x00C2 => {
                // SINT (8-bit signed integer)
                if data.is_empty() {
                    return Err(UdtError::protocol("SINT data too short".to_string()));
                }
                Ok(PlcValue::Sint(data[0] as i8))
            }
            0x00C3 => {
                // INT (16-bit signed integer)
                if data.len() < 2 {
                    return Err(UdtError::protocol("INT data too short".to_string()));
                }
                let mut bytes = [0u8; 2];
                bytes.copy_from_slice(&data[..2]);
                Ok(PlcValue::Int(i16::from_le_bytes(bytes)))
            }
            0x00C4 => {
                // DINT (32-bit signed integer)
                if data.len() < 4 {
                    return Err(UdtError::protocol("DINT data too short".to_string()));
                }
                let mut bytes = [0u8; 4];
                bytes.copy_from_slice(&data[..4]);
                Ok(PlcValue::Dint(i32::from_le_bytes(bytes)))
            }
            0x00C5 => {
                // LINT (64-bit signed integer)
                if data.len() < 8 {
                    return Err(UdtError::protocol("LINT data too short".to_string()));
                }
                let mut bytes = [0u8; 8];
                bytes.copy_from_slice(&data[..8]);
                Ok(PlcValue::Lint(i64::from_le_bytes(bytes)))
            }
            0x00C6 => {
                // USINT (8-bit unsigned integer)
                if data.is_empty() {
                    return Err(UdtError::protocol("USINT data too short".to_string()));
                }
                Ok(PlcValue::Usint(data[0]))
            }
            0x00C7 => {
                // UINT (16-bit unsigned integer)
                if data.len() < 2 {
                    return Err(UdtError::protocol("UINT data too short".to_string()));
                }
                let mut bytes = [0u8; 2];
                bytes.copy_from_slice(&data[..2]);
                Ok(PlcValue::Uint(u16::from_le_bytes(bytes)))
            }
            0x00C8 => {
                // UDINT (32-bit unsigned integer)
                if data.len() < 4 {
                    return Err(UdtError::protocol("UDINT data too short".to_string()));
                }
                let mut bytes = [0u8; 4];
                bytes.copy_from_slice(&data[..4]);
                Ok(PlcValue::Udint(u32::from_le_bytes(bytes)))
            }
            0x00C9 => {
                // ULINT (64-bit unsigned integer)
                if data.len() < 8 {
                    return Err(UdtError::protocol("ULINT data too short".to_string()));
                }
                let mut bytes = [0u8; 8];
                bytes.copy_from_slice(&data[..8]);
                Ok(PlcValue::Ulint(u64::from_le_bytes(bytes)))
            }
            0x00CA => {
                // REAL (32-bit float)
                if data.len() < 4 {
                    return Err(UdtError::protocol("REAL data too short".to_string()));
                }
                let mut bytes = [0u8; 4];
                bytes.copy_from_slice(&data[..4]);
                Ok(PlcValue::Real(f32::from_le_bytes(bytes)))
            }
            0x00CB => {
                // LREAL (64-bit float)
                if data.len() < 8 {
                    return Err(UdtError::protocol("LREAL data too short".to_string()));
                }
                let mut bytes = [0u8; 8];
                bytes.copy_from_slice(&data[..8]);
                Ok(PlcValue::Lreal(f64::from_le_bytes(bytes)))
            }
            0x00CE => {
                // STRING type - first 4 bytes are length (DINT), followed by data (up to 82 bytes)
                if data.len() < 4 {
                    return Err(UdtError::protocol("STRING data too short".to_string()));
                }
                let length = u32::from_le_bytes([data[0], data[1], data[2], data[3]]) as usize;
                if data.len() - 4 < length {
                    return Err(UdtError::protocol("STRING data incomplete".to_string()));
                }
                let string_data = &data[4..4 + length];
                let string_value = String::from_utf8_lossy(string_data).to_string();
                Ok(PlcValue::String(string_value))
            }
            _ => Err(UdtError::protocol(format!(
                "Unsupported UDT data type: 0x{:04X}",
                member.data_type
            ))),
        }
    }

    /// Serializes a member value to raw data
    pub fn serialize_member_value(&self, member: &UdtMember, value: &PlcValue) -> Result<Vec<u8>> {
        match member.data_type {
            0x00C1 => match value {
                PlcValue::Bool(b) => Ok(vec![if *b { 0xFF } else { 0x00 }]),
                _ => Err(UdtError::DataTypeMismatch {
                    expected: "BOOL".to_string(),
                    actual: format!("{:?}", value),
                }),
            },
            0x00C2 => match value {
                PlcValue::Sint(s) => Ok(vec![*s as u8]),
                _ => Err(UdtError::DataTypeMismatch {
                    expected: "SINT".to_string(),
                    actual: format!("{:?}", value),
                }),
            },
            0x00C3 => match value {
                PlcValue::Int(i) => Ok(i.to_le_bytes().to_vec()),
                _ => Err(UdtError::DataTypeMismatch {
                    expected: "INT".to_string(),
                    actual: format!("{:?}", value),
                }),
            },
            0x00C4 => match value {
                PlcValue::Dint(d) => Ok(d.to_le_bytes().to_vec()),
                _ => Err(UdtError::DataTypeMismatch {
                    expected: "DINT".to_string(),
                    actual: format!("{:?}", value),
                }),
            },
            0x00C5 => match value {
                PlcValue::Lint(l) => Ok(l.to_le_bytes().to_vec()),
                _ => Err(UdtError::DataTypeMismatch {
                    expected: "LINT".to_string(),
                    actual: format!("{:?}", value),
                }),
            },
            0x00C6 => match value {
                PlcValue::Usint(u) => Ok(vec![*u]),
                _ => Err(UdtError::DataTypeMismatch {
                    expected: "USINT".to_string(),
                    actual: format!("{:?}", value),
                }),
            },
            0x00C7 => match value {
                PlcValue::Uint(u) => Ok(u.to_le_bytes().to_vec()),
                _ => Err(UdtError::DataTypeMismatch {
                    expected: "UINT".to_string(),
                    actual: format!("{:?}", value),
                }),
            },
            0x00C8 => match value {
                PlcValue::Udint(u) => Ok(u.to_le_bytes().to_vec()),
                _ => Err(UdtError::DataTypeMismatch {
                    expected: "UDINT".to_string(),
                    actual: format!("{:?}", value),
                }),
            },
            0x00C9 => match value {
                PlcValue::Ulint(u) => Ok(u.to_le_bytes().to_vec()),
                _ => Err(UdtError::DataTypeMismatch {
                    expected: "ULINT".to_string(),
                    actual: format!("{:?}", value),
                }),
            },
            0x00CA => match value {
                PlcValue::Real(r) => Ok(r.to_le_bytes().to_vec()),
                _ => Err(UdtError::DataTypeMismatch {
                    expected: "REAL".to_string(),
                    actual: format!("{:?}", value),
                }),
            },
            0x00CB => match value {
                PlcValue::Lreal(l) => Ok(l.to_le_bytes().to_vec()),
                _ => Err(UdtError::DataTypeMismatch {
                    expected: "LREAL".to_string(),
                    actual: format!("{:?}", value),
                }),
            },
            0x00CE => {
                match value {
                    PlcValue::String(s) => {
                        let mut result = Vec::new();
                        let max_data_len = member.size.saturating_sub(4); // Subtract 4 for DINT length field
                        let max_chars = (max_data_len as usize).min(82); // Max STRING length is 82
                        let length = (s.len() as u32).min(max_chars as u32);
                        // Length field is 4 bytes (DINT)
                        result.extend_from_slice(&length.to_le_bytes());
                        result.extend_from_slice(&s.as_bytes()[..length as usize]);
                        // Pad to even byte boundary, but don't exceed member size
                        while result.len() < member.size as usize && result.len() % 2 != 0 {
                            result.push(0);
                        }
                        // Ensure we don't exceed member size
                        if result.len() > member.size as usize {
                            result.truncate(member.size as usize);
                        }
                        Ok(result)
                    }
                    _ => Err(UdtError::DataTypeMismatch {
                        expected: "STRING".to_string(),
                        actual: format!("{:?}", value),
                    }),
                }
            }
            _ => Err(UdtError::protocol(format!(
                "Unsupported UDT data type for serialization: 0x{:04X}",
                member.data_type
            ))),
        }
    }
}

impl UdtCodec for UserDefinedType {
    fn to_hash_map(
        &self,
        data: &[u8],
    ) -> rust_ethernet_ip_types::Result<HashMap<String, PlcValue>> {
        UserDefinedType::to_hash_map(self, data).map_err(|error| TypeError::new(error.to_string()))
    }

    fn encode_hash_map(
        &self,
        values: &HashMap<String, PlcValue>,
    ) -> rust_ethernet_ip_types::Result<Vec<u8>> {
        UserDefinedType::from_hash_map(self, values)
            .map_err(|error| TypeError::new(error.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_udt_member_offsets() {
        let mut udt = UserDefinedType::new("TestUDT".to_string());

        udt.add_member(UdtMember {
            name: "Bool1".to_string(),
            data_type: 0x00C1,
            offset: 0,
            size: 1,
        });

        udt.add_member(UdtMember {
            name: "Dint1".to_string(),
            data_type: 0x00C4,
            offset: 4,
            size: 4,
        });

        assert_eq!(udt.get_member_offset("Bool1"), Some(0));
        assert_eq!(udt.get_member_offset("Dint1"), Some(4));
        assert_eq!(udt.size, 8);
    }

    #[test]
    fn test_udt_parsing() {
        let mut udt = UserDefinedType::new("TestUDT".to_string());

        udt.add_member(UdtMember {
            name: "Bool1".to_string(),
            data_type: 0x00C1,
            offset: 0,
            size: 1,
        });

        udt.add_member(UdtMember {
            name: "Dint1".to_string(),
            data_type: 0x00C4,
            offset: 4,
            size: 4,
        });

        let data = vec![0xFF, 0x00, 0x00, 0x00, 0x2A, 0x00, 0x00, 0x00];
        let result = udt.to_hash_map(&data).unwrap();

        assert_eq!(result.get("Bool1"), Some(&PlcValue::Bool(true)));
        assert_eq!(result.get("Dint1"), Some(&PlcValue::Dint(42)));
    }

    #[test]
    fn to_hash_map_errors_on_truncated_member_data() {
        let mut udt = UserDefinedType::new("TestUDT".to_string());
        udt.add_member(UdtMember {
            name: "Dint1".to_string(),
            data_type: 0x00C4,
            offset: 0,
            size: 4,
        });
        udt.add_member(UdtMember {
            name: "Dint2".to_string(),
            data_type: 0x00C4,
            offset: 4,
            size: 4,
        });

        let error = udt.to_hash_map(&[1, 0, 0, 0, 2, 0]).unwrap_err();
        assert!(error.to_string().contains("Dint2 data incomplete"));
    }

    #[test]
    fn from_hash_map_errors_on_missing_member() {
        let mut udt = UserDefinedType::new("TestUDT".to_string());
        udt.add_member(UdtMember {
            name: "Dint1".to_string(),
            data_type: 0x00C4,
            offset: 0,
            size: 4,
        });
        udt.add_member(UdtMember {
            name: "Dint2".to_string(),
            data_type: 0x00C4,
            offset: 4,
            size: 4,
        });

        let values = HashMap::from([("Dint1".to_string(), PlcValue::Dint(1))]);
        let error = udt.from_hash_map(&values).unwrap_err();
        assert!(
            error
                .to_string()
                .contains("Missing UDT member value: Dint2")
        );
    }

    #[test]
    fn packed_bool_members_use_template_info_bit_index() {
        let mut udt = UserDefinedType::new("PackedBoolUDT".to_string());
        udt.add_member_with_bit_index(
            UdtMember {
                name: "Bit0".to_string(),
                data_type: 0x00C1,
                offset: 0,
                size: 1,
            },
            Some(0),
        );
        udt.add_member_with_bit_index(
            UdtMember {
                name: "Bit3".to_string(),
                data_type: 0x00C1,
                offset: 0,
                size: 1,
            },
            Some(3),
        );

        let parsed = udt.to_hash_map(&[0b0000_1000]).unwrap();
        assert_eq!(parsed.get("Bit0"), Some(&PlcValue::Bool(false)));
        assert_eq!(parsed.get("Bit3"), Some(&PlcValue::Bool(true)));

        let encoded = udt
            .from_hash_map(&HashMap::from([
                ("Bit0".to_string(), PlcValue::Bool(true)),
                ("Bit3".to_string(), PlcValue::Bool(true)),
            ]))
            .unwrap();
        assert_eq!(encoded, vec![0b0000_1001]);
    }

    #[test]
    fn test_from_cip_data_returns_explicit_error_until_implemented() {
        let result = UserDefinedType::from_cip_data(&[0x01, 0x02, 0x03]);
        assert!(result.is_err());
        let error_text = result.err().unwrap().to_string();
        assert!(error_text.contains("not implemented"));
    }

    #[test]
    fn test_serialize_udt_instance_returns_explicit_error_until_implemented() {
        let manager = UdtManager::new();
        let values = HashMap::new();
        let result = manager.serialize_udt_instance(&values);
        assert!(result.is_err());
        let error_text = result.err().unwrap().to_string();
        assert!(error_text.contains("not implemented"));
    }

    #[test]
    fn test_parse_udt_template_reads_live_style_member_records() {
        let manager = UdtManager::new();
        let data = vec![
            0x00, 0x00, 0xC4, 0x00, 0x00, 0x00, 0x00, 0x00, // DINT @ 0
            0x04, 0x00, 0xCA, 0x00, 0x04, 0x00, 0x00, 0x00, // REAL @ 4
            0x08, 0x00, 0xC2, 0x00, 0x08, 0x00, 0x00, 0x00, // hidden SINT host @ 8
            0x00, 0x00, 0xC1, 0x00, 0x08, 0x00, 0x00, 0x00, // BOOL @ 8
            b'T', b'E', b'S', b'T', b'_', b'U', b'D', b'T', b';', b'n', 0x00, b'M', b'e', b'm',
            b'b', b'e', b'r', b'1', 0x00, b'M', b'e', b'm', b'b', b'e', b'r', b'2', 0x00, b'Z',
            b'Z', b'Z', b'Z', b'Z', b'Z', b'Z', b'Z', b'Z', b'Z', 0x00, b'F', b'l', b'a', b'g',
            0x00,
        ];

        let template = manager
            .parse_udt_template(123, 4, 12, &data)
            .expect("template should parse");

        assert_eq!(template.name, "TEST_UDT");
        assert_eq!(template.members.len(), 3);
        assert_eq!(template.members[0].name, "Member1");
        assert_eq!(template.members[0].data_type, 0x00C4);
        assert_eq!(template.members[0].offset, 0);
        assert_eq!(template.members[1].name, "Member2");
        assert_eq!(template.members[1].data_type, 0x00CA);
        assert_eq!(template.members[1].offset, 4);
        assert_eq!(template.members[2].name, "Flag");
        assert_eq!(template.members[2].data_type, 0x00C1);
        assert_eq!(template.members[2].offset, 8);
        assert_eq!(template.members[2].size, 1);
    }

    #[test]
    fn test_parse_udt_template_preserves_empty_member_names_positionally() {
        let manager = UdtManager::new();
        let data = vec![
            0x00, 0x00, 0xC4, 0x00, 0x00, 0x00, 0x00, 0x00, // DINT @ 0
            0x00, 0x00, 0xC2, 0x00, 0x04, 0x00, 0x00, 0x00, // hidden host @ 4
            0x01, 0x00, 0xC1, 0x00, 0x04, 0x00, 0x00, 0x00, // BOOL bit 1 @ 4
            // Template strings are a NUL-separated template name followed by
            // one member name per member record. The empty middle name models
            // hidden/host members and must not shift following names left.
            b'T', b'E', b'S', b'T', b'_', b'U', b'D', b'T', 0x00, b'C', b'o', b'u', b'n', b't',
            0x00, 0x00, b'F', b'l', b'a', b'g', 0x00,
        ];

        let template = manager
            .parse_udt_template(321, 3, 5, &data)
            .expect("template should parse");

        assert_eq!(template.members.len(), 2);
        assert_eq!(template.members[0].name, "Count");
        assert_eq!(template.members[1].name, "Flag");
        assert_eq!(template.members[1].offset, 4);
    }
}
