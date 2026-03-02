use crate::error::{EtherNetIpError, Result};
use crate::PlcValue;
use std::collections::HashMap;

/// Definition of a User Defined Type
#[derive(Debug, Clone)]
pub struct UdtDefinition {
    pub name: String,
    pub members: Vec<UdtMember>,
}

/// Member of a UDT
#[derive(Debug, Clone)]
pub struct UdtMember {
    pub name: String,
    pub data_type: u16,
    pub offset: u32,
    pub size: u32,
}

/// UDT Template information from PLC
#[derive(Debug, Clone)]
pub struct UdtTemplate {
    pub template_id: u32,
    pub name: String,
    pub size: u32,
    pub member_count: u16,
    pub members: Vec<UdtMember>,
}

/// Tag attributes from PLC
#[derive(Debug, Clone)]
pub struct TagAttributes {
    pub name: String,
    pub data_type: u16,
    pub data_type_name: String,
    pub dimensions: Vec<u32>,
    pub permissions: TagPermissions,
    pub scope: TagScope,
    pub template_instance_id: Option<u32>,
    pub size: u32,
}

/// Tag permissions
#[derive(Debug, Clone, PartialEq)]
pub enum TagPermissions {
    ReadOnly,
    ReadWrite,
    WriteOnly,
    Unknown,
}

/// Tag scope
#[derive(Debug, Clone, PartialEq)]
pub enum TagScope {
    Controller,
    Program(String),
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

    /// Parses UDT template data from CIP response
    pub fn parse_udt_template(&self, template_id: u32, data: &[u8]) -> Result<UdtTemplate> {
        if data.len() < 8 {
            return Err(EtherNetIpError::Protocol(
                "UDT template data too short".to_string(),
            ));
        }

        let mut offset = 0;

        // Parse template header
        let structure_size = u32::from_le_bytes([
            data[offset],
            data[offset + 1],
            data[offset + 2],
            data[offset + 3],
        ]);
        offset += 4;

        let member_count = u16::from_le_bytes([data[offset], data[offset + 1]]);
        offset += 2;

        // Skip reserved bytes
        offset += 2;

        let mut members = Vec::new();
        let mut current_offset = 0u32;

        // Parse each member
        for i in 0..member_count {
            if offset + 8 > data.len() {
                return Err(EtherNetIpError::Protocol(format!(
                    "UDT template member {} data incomplete",
                    i
                )));
            }

            // Parse member info
            let member_info = u32::from_le_bytes([
                data[offset],
                data[offset + 1],
                data[offset + 2],
                data[offset + 3],
            ]);
            offset += 4;

            let member_name_length = u16::from_le_bytes([data[offset], data[offset + 1]]);
            offset += 2;

            // Skip reserved bytes
            offset += 2;

            // Extract member properties from member_info
            let data_type = (member_info & 0xFFFF) as u16;
            let _dimensions = ((member_info >> 16) & 0xFF) as u8;

            // Read member name
            if offset + member_name_length as usize > data.len() {
                return Err(EtherNetIpError::Protocol(format!(
                    "UDT template member {} name data incomplete",
                    i
                )));
            }

            let name_bytes = &data[offset..offset + member_name_length as usize];
            let member_name = String::from_utf8_lossy(name_bytes).to_string();
            offset += member_name_length as usize;

            // Align to 4-byte boundary
            offset = (offset + 3) & !3;

            // Calculate member size based on data type
            let member_size = self.get_data_type_size(data_type);

            // Create member
            let member = UdtMember {
                name: member_name,
                data_type,
                offset: current_offset,
                size: member_size,
            };

            members.push(member);
            current_offset += member_size;
        }

        Ok(UdtTemplate {
            template_id,
            name: format!("Template_{}", template_id),
            size: structure_size,
            member_count,
            members,
        })
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
            return Err(EtherNetIpError::Protocol(
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
        Ok(PlcValue::Udt(crate::UdtData {
            symbol_id: 0, // Not available in this context
            data: data.to_vec(),
        }))
    }

    /// Serialize a UDT instance to bytes
    pub fn serialize_udt_instance(
        &self,
        _udt_value: &HashMap<String, PlcValue>,
    ) -> Result<Vec<u8>> {
        // For now, return empty bytes
        // Full UDT serialization can be implemented later
        Ok(Vec::new())
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
}

impl UserDefinedType {
    /// Creates a new UDT
    pub fn new(name: String) -> Self {
        Self {
            name,
            size: 0,
            members: Vec::new(),
            member_offsets: HashMap::new(),
        }
    }

    /// Adds a member to the UDT
    pub fn add_member(&mut self, member: UdtMember) {
        self.member_offsets
            .insert(member.name.clone(), member.offset);
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
    pub fn from_cip_data(_data: &[u8]) -> crate::error::Result<Self> {
        // TODO: Implement CIP data parsing
        Ok(Self {
            name: String::new(),
            members: Vec::new(),
            size: 0,
            member_offsets: HashMap::new(),
        })
    }

    /// Converts a UDT instance to a `HashMap` of member values
    pub fn to_hash_map(&self, data: &[u8]) -> crate::error::Result<HashMap<String, PlcValue>> {
        if data.is_empty() {
            return Err(crate::error::EtherNetIpError::Protocol(
                "UDT data is empty".to_string(),
            ));
        }

        let mut result = HashMap::new();

        for member in &self.members {
            let offset = member.offset as usize;
            if offset + member.size as usize <= data.len() {
                let member_data = &data[offset..offset + member.size as usize];
                let value = self.parse_member_value(member, member_data)?;
                result.insert(member.name.clone(), value);
            }
        }

        Ok(result)
    }

    /// Converts a `HashMap` of member values to raw UDT bytes
    pub fn from_hash_map(
        &self,
        values: &HashMap<String, PlcValue>,
    ) -> crate::error::Result<Vec<u8>> {
        let mut data = vec![0u8; self.size as usize];

        for member in &self.members {
            if let Some(value) = values.get(&member.name) {
                let member_data = self.serialize_member_value(member, value)?;
                let offset = member.offset as usize;
                let end_offset = offset + member_data.len();

                if end_offset <= data.len() {
                    data[offset..end_offset].copy_from_slice(&member_data);
                } else {
                    return Err(crate::error::EtherNetIpError::Protocol(format!(
                        "Member {} data exceeds UDT size",
                        member.name
                    )));
                }
            }
        }

        Ok(data)
    }

    /// Reads a specific UDT member by name
    pub fn read_member(&self, data: &[u8], member_name: &str) -> crate::error::Result<PlcValue> {
        if let Some(member) = self.members.iter().find(|m| m.name == member_name) {
            let offset = member.offset as usize;
            if offset + member.size as usize <= data.len() {
                let member_data = &data[offset..offset + member.size as usize];
                self.parse_member_value(member, member_data)
            } else {
                Err(crate::error::EtherNetIpError::Protocol(format!(
                    "Member {} data incomplete",
                    member_name
                )))
            }
        } else {
            Err(crate::error::EtherNetIpError::TagNotFound(format!(
                "UDT member '{}' not found",
                member_name
            )))
        }
    }

    /// Writes a specific UDT member by name
    pub fn write_member(
        &self,
        data: &mut [u8],
        member_name: &str,
        value: &PlcValue,
    ) -> crate::error::Result<()> {
        if let Some(member) = self.members.iter().find(|m| m.name == member_name) {
            let member_data = self.serialize_member_value(member, value)?;
            let offset = member.offset as usize;
            let end_offset = offset + member_data.len();

            if end_offset <= data.len() {
                data[offset..end_offset].copy_from_slice(&member_data);
                Ok(())
            } else {
                Err(crate::error::EtherNetIpError::Protocol(format!(
                    "Member {} data exceeds UDT size",
                    member_name
                )))
            }
        } else {
            Err(crate::error::EtherNetIpError::TagNotFound(format!(
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
    pub fn parse_member_value(
        &self,
        member: &UdtMember,
        data: &[u8],
    ) -> crate::error::Result<PlcValue> {
        match member.data_type {
            0x00C1 => Ok(PlcValue::Bool(data[0] != 0)),
            0x00C2 => {
                // SINT (8-bit signed integer)
                Ok(PlcValue::Sint(data[0] as i8))
            }
            0x00C3 => {
                // INT (16-bit signed integer)
                if data.len() < 2 {
                    return Err(crate::error::EtherNetIpError::Protocol(
                        "INT data too short".to_string(),
                    ));
                }
                let mut bytes = [0u8; 2];
                bytes.copy_from_slice(&data[..2]);
                Ok(PlcValue::Int(i16::from_le_bytes(bytes)))
            }
            0x00C4 => {
                // DINT (32-bit signed integer)
                if data.len() < 4 {
                    return Err(crate::error::EtherNetIpError::Protocol(
                        "DINT data too short".to_string(),
                    ));
                }
                let mut bytes = [0u8; 4];
                bytes.copy_from_slice(&data[..4]);
                Ok(PlcValue::Dint(i32::from_le_bytes(bytes)))
            }
            0x00C5 => {
                // LINT (64-bit signed integer)
                if data.len() < 8 {
                    return Err(crate::error::EtherNetIpError::Protocol(
                        "LINT data too short".to_string(),
                    ));
                }
                let mut bytes = [0u8; 8];
                bytes.copy_from_slice(&data[..8]);
                Ok(PlcValue::Lint(i64::from_le_bytes(bytes)))
            }
            0x00C6 => {
                // USINT (8-bit unsigned integer)
                Ok(PlcValue::Usint(data[0]))
            }
            0x00C7 => {
                // UINT (16-bit unsigned integer)
                if data.len() < 2 {
                    return Err(crate::error::EtherNetIpError::Protocol(
                        "UINT data too short".to_string(),
                    ));
                }
                let mut bytes = [0u8; 2];
                bytes.copy_from_slice(&data[..2]);
                Ok(PlcValue::Uint(u16::from_le_bytes(bytes)))
            }
            0x00C8 => {
                // UDINT (32-bit unsigned integer)
                if data.len() < 4 {
                    return Err(crate::error::EtherNetIpError::Protocol(
                        "UDINT data too short".to_string(),
                    ));
                }
                let mut bytes = [0u8; 4];
                bytes.copy_from_slice(&data[..4]);
                Ok(PlcValue::Udint(u32::from_le_bytes(bytes)))
            }
            0x00C9 => {
                // ULINT (64-bit unsigned integer)
                if data.len() < 8 {
                    return Err(crate::error::EtherNetIpError::Protocol(
                        "ULINT data too short".to_string(),
                    ));
                }
                let mut bytes = [0u8; 8];
                bytes.copy_from_slice(&data[..8]);
                Ok(PlcValue::Ulint(u64::from_le_bytes(bytes)))
            }
            0x00CA => {
                // REAL (32-bit float)
                if data.len() < 4 {
                    return Err(crate::error::EtherNetIpError::Protocol(
                        "REAL data too short".to_string(),
                    ));
                }
                let mut bytes = [0u8; 4];
                bytes.copy_from_slice(&data[..4]);
                Ok(PlcValue::Real(f32::from_le_bytes(bytes)))
            }
            0x00CB => {
                // LREAL (64-bit float)
                if data.len() < 8 {
                    return Err(crate::error::EtherNetIpError::Protocol(
                        "LREAL data too short".to_string(),
                    ));
                }
                let mut bytes = [0u8; 8];
                bytes.copy_from_slice(&data[..8]);
                Ok(PlcValue::Lreal(f64::from_le_bytes(bytes)))
            }
            0x00CE => {
                // STRING type - first 4 bytes are length (DINT), followed by data (up to 82 bytes)
                if data.len() < 4 {
                    return Err(crate::error::EtherNetIpError::Protocol(
                        "STRING data too short".to_string(),
                    ));
                }
                let length = u32::from_le_bytes([data[0], data[1], data[2], data[3]]) as usize;
                if data.len() - 4 < length {
                    return Err(crate::error::EtherNetIpError::Protocol(
                        "STRING data incomplete".to_string(),
                    ));
                }
                let string_data = &data[4..4 + length];
                let string_value = String::from_utf8_lossy(string_data).to_string();
                Ok(PlcValue::String(string_value))
            }
            _ => Err(crate::error::EtherNetIpError::Protocol(format!(
                "Unsupported UDT data type: 0x{:04X}",
                member.data_type
            ))),
        }
    }

    /// Serializes a member value to raw data
    pub fn serialize_member_value(
        &self,
        member: &UdtMember,
        value: &PlcValue,
    ) -> crate::error::Result<Vec<u8>> {
        match member.data_type {
            0x00C1 => match value {
                PlcValue::Bool(b) => Ok(vec![if *b { 0xFF } else { 0x00 }]),
                _ => Err(crate::error::EtherNetIpError::DataTypeMismatch {
                    expected: "BOOL".to_string(),
                    actual: format!("{:?}", value),
                }),
            },
            0x00C2 => match value {
                PlcValue::Sint(s) => Ok(vec![*s as u8]),
                _ => Err(crate::error::EtherNetIpError::DataTypeMismatch {
                    expected: "SINT".to_string(),
                    actual: format!("{:?}", value),
                }),
            },
            0x00C3 => match value {
                PlcValue::Int(i) => Ok(i.to_le_bytes().to_vec()),
                _ => Err(crate::error::EtherNetIpError::DataTypeMismatch {
                    expected: "INT".to_string(),
                    actual: format!("{:?}", value),
                }),
            },
            0x00C4 => match value {
                PlcValue::Dint(d) => Ok(d.to_le_bytes().to_vec()),
                _ => Err(crate::error::EtherNetIpError::DataTypeMismatch {
                    expected: "DINT".to_string(),
                    actual: format!("{:?}", value),
                }),
            },
            0x00C5 => match value {
                PlcValue::Lint(l) => Ok(l.to_le_bytes().to_vec()),
                _ => Err(crate::error::EtherNetIpError::DataTypeMismatch {
                    expected: "LINT".to_string(),
                    actual: format!("{:?}", value),
                }),
            },
            0x00C6 => match value {
                PlcValue::Usint(u) => Ok(vec![*u]),
                _ => Err(crate::error::EtherNetIpError::DataTypeMismatch {
                    expected: "USINT".to_string(),
                    actual: format!("{:?}", value),
                }),
            },
            0x00C7 => match value {
                PlcValue::Uint(u) => Ok(u.to_le_bytes().to_vec()),
                _ => Err(crate::error::EtherNetIpError::DataTypeMismatch {
                    expected: "UINT".to_string(),
                    actual: format!("{:?}", value),
                }),
            },
            0x00C8 => match value {
                PlcValue::Udint(u) => Ok(u.to_le_bytes().to_vec()),
                _ => Err(crate::error::EtherNetIpError::DataTypeMismatch {
                    expected: "UDINT".to_string(),
                    actual: format!("{:?}", value),
                }),
            },
            0x00C9 => match value {
                PlcValue::Ulint(u) => Ok(u.to_le_bytes().to_vec()),
                _ => Err(crate::error::EtherNetIpError::DataTypeMismatch {
                    expected: "ULINT".to_string(),
                    actual: format!("{:?}", value),
                }),
            },
            0x00CA => match value {
                PlcValue::Real(r) => Ok(r.to_le_bytes().to_vec()),
                _ => Err(crate::error::EtherNetIpError::DataTypeMismatch {
                    expected: "REAL".to_string(),
                    actual: format!("{:?}", value),
                }),
            },
            0x00CB => match value {
                PlcValue::Lreal(l) => Ok(l.to_le_bytes().to_vec()),
                _ => Err(crate::error::EtherNetIpError::DataTypeMismatch {
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
                    _ => Err(crate::error::EtherNetIpError::DataTypeMismatch {
                        expected: "STRING".to_string(),
                        actual: format!("{:?}", value),
                    }),
                }
            }
            _ => Err(crate::error::EtherNetIpError::Protocol(format!(
                "Unsupported UDT data type for serialization: 0x{:04X}",
                member.data_type
            ))),
        }
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
}
