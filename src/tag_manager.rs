use crate::EipClient;
use crate::error::{EtherNetIpError, Result};
use crate::udt::UdtDefinition;
use std::collections::HashMap;
use std::sync::{LazyLock, RwLock};
use std::time::{Duration, Instant};
use tracing;

static TAG_NAME_RE: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(r"^(?:Program:[A-Za-z_][A-Za-z0-9_]*\.)?[A-Za-z_][A-Za-z0-9_]*(?:\[\d+\])?(?:\.[A-Za-z_][A-Za-z0-9_]*(?:\[\d+\])?)*$")
        .expect("tag name regex pattern is a valid literal")
});

/// Represents the scope of a tag in the PLC
#[derive(Debug, Clone, PartialEq)]
pub enum TagScope {
    /// Tag in the controller scope
    Controller,
    /// Tag in a program scope
    Program(String),
    Global,
    Local,
}

/// Array information for tags
#[derive(Debug, Clone)]
pub struct ArrayInfo {
    pub dimensions: Vec<u32>,
    pub element_count: u32,
}

/// Metadata for a PLC tag
#[derive(Debug, Clone)]
pub struct TagMetadata {
    /// The data type of the tag
    pub data_type: u16,
    /// Size of the tag in bytes
    pub size: u32,
    /// Whether the tag is an array
    pub is_array: bool,
    /// Array dimensions if applicable
    pub dimensions: Vec<u32>,
    /// Access permissions for the tag
    pub permissions: TagPermissions,
    /// Scope of the tag
    pub scope: TagScope,
    /// Last time this tag was accessed
    pub last_access: Instant,
    pub array_info: Option<ArrayInfo>,
    pub last_updated: Instant,
}

/// Access permissions for a tag
#[derive(Debug, Clone, PartialEq)]
pub struct TagPermissions {
    /// Whether the tag can be read
    pub readable: bool,
    /// Whether the tag can be written
    pub writable: bool,
}

impl TagMetadata {
    /// Returns true if this tag is a structure/UDT
    pub fn is_structure(&self) -> bool {
        is_structure_type_word(self.data_type)
    }
}

fn is_structure_type_word(data_type: u16) -> bool {
    let type_code = data_type & 0x0fff;
    (data_type & 0x8000) != 0 || (0x00A0..=0x00AF).contains(&type_code) || type_code == 0x02A0
}

/// Cache for PLC tags with automatic expiration
#[derive(Debug)]
#[deprecated(
    since = "1.2.0",
    note = "TagCache was never wired into live discovery; use TagManager's built-in cache instead. The type will be removed in 2.0."
)]
pub struct TagCache {
    /// Map of tag names to their metadata
    tags: HashMap<String, (TagMetadata, Instant)>,
    /// Cache expiration time
    expiration: Duration,
}

#[expect(
    deprecated,
    reason = "CODEX-AQ keeps the compatibility implementation until 2.0 removal"
)]
impl TagCache {
    /// Creates a new tag cache with the specified expiration time
    pub fn new(expiration: Duration) -> Self {
        Self {
            tags: HashMap::new(),
            expiration,
        }
    }

    /// Updates or adds a tag to the cache
    pub fn update_tag(&mut self, name: String, metadata: TagMetadata) {
        self.tags.insert(name, (metadata, Instant::now()));
    }

    /// Gets a tag from the cache if it exists and hasn't expired
    pub fn get_tag(&self, name: &str) -> Option<&TagMetadata> {
        if let Some((metadata, timestamp)) = self.tags.get(name)
            && timestamp.elapsed() < self.expiration
        {
            return Some(metadata);
        }
        None
    }

    /// Removes expired tags from the cache
    pub fn cleanup(&mut self) {
        self.tags
            .retain(|_, (_, timestamp)| timestamp.elapsed() < self.expiration);
    }
}

/// Manager for PLC tag discovery and caching
#[derive(Debug)]
pub struct TagManager {
    pub cache: RwLock<HashMap<String, TagMetadata>>,
    cache_duration: Duration,
    pub udt_definitions: RwLock<HashMap<String, UdtDefinition>>,
}

impl TagManager {
    pub fn new() -> Self {
        Self {
            cache: RwLock::new(HashMap::new()),
            cache_duration: Duration::from_secs(300), // 5 minutes
            udt_definitions: RwLock::new(HashMap::new()),
        }
    }

    pub async fn get_metadata(&self, tag_name: &str) -> Result<Option<TagMetadata>> {
        let cache = self.cache.read()?;
        Ok(cache.get(tag_name).and_then(|metadata| {
            if metadata.last_updated.elapsed() < self.cache_duration {
                Some(metadata.clone())
            } else {
                None
            }
        }))
    }

    pub async fn update_metadata(&self, tag_name: String, metadata: TagMetadata) -> Result<()> {
        self.cache.write()?.insert(tag_name, metadata);
        Ok(())
    }

    pub async fn validate_tag(
        &self,
        tag_name: &str,
        required_permissions: &TagPermissions,
    ) -> Result<()> {
        if let Some(metadata) = self.get_metadata(tag_name).await? {
            if !metadata.permissions.readable && required_permissions.readable {
                return Err(EtherNetIpError::Permission(format!(
                    "Tag '{tag_name}' is not readable"
                )));
            }
            if !metadata.permissions.writable && required_permissions.writable {
                return Err(EtherNetIpError::Permission(format!(
                    "Tag '{tag_name}' is not writable"
                )));
            }
            Ok(())
        } else {
            Err(EtherNetIpError::Tag(format!("Tag '{tag_name}' not found")))
        }
    }

    pub async fn clear_cache(&self) -> Result<()> {
        self.cache.write()?.clear();
        Ok(())
    }

    pub async fn remove_stale_entries(&self) -> Result<()> {
        self.cache
            .write()?
            .retain(|_, metadata| metadata.last_updated.elapsed() < self.cache_duration);
        Ok(())
    }

    pub async fn discover_tags(&self, client: &mut EipClient) -> Result<()> {
        let response = client
            .send_cip_request(&client.build_list_tags_request())
            .await?;
        let tags = self.parse_tag_list(&response)?;

        // Perform hierarchical discovery for structures/UDTs
        let mut all_tags = Vec::new();
        for (name, metadata) in tags {
            all_tags.push((name, metadata));
        }

        // Discover nested tags for structures
        let hierarchical_tags = self.discover_hierarchical_tags(client, &all_tags).await?;

        let mut cache = self.cache.write()?;
        for (name, metadata) in hierarchical_tags {
            cache.insert(name, metadata);
        }
        Ok(())
    }

    /// Discovers hierarchical tags by drilling down into structures and UDTs
    async fn discover_hierarchical_tags(
        &self,
        client: &mut EipClient,
        base_tags: &[(String, TagMetadata)],
    ) -> Result<Vec<(String, TagMetadata)>> {
        let mut all_tags = Vec::new();
        let mut tag_names = std::collections::HashSet::new();

        // Add base tags first
        for (name, metadata) in base_tags {
            if self.validate_tag_name(name) {
                all_tags.push((name.clone(), metadata.clone()));
                tag_names.insert(name.clone());
            }
        }

        // Process each tag for hierarchical discovery
        for (name, metadata) in base_tags {
            if metadata.is_structure() && !metadata.is_array {
                // This is a structure/UDT, try to discover its members
                if let Ok(members) = self.discover_udt_members(client, name).await {
                    for (member_name, member_metadata) in members {
                        let full_name = format!("{}.{}", name, member_name);
                        if self.validate_tag_name(&full_name) && !tag_names.contains(&full_name) {
                            all_tags.push((full_name.clone(), member_metadata.clone()));
                            tag_names.insert(full_name.clone());

                            // Recursively discover nested structures
                            if member_metadata.is_structure()
                                && !member_metadata.is_array
                                && let Ok(nested_members) =
                                    self.discover_udt_members(client, &full_name).await
                            {
                                for (nested_name, nested_metadata) in nested_members {
                                    let nested_full_name = format!("{}.{}", full_name, nested_name);
                                    if self.validate_tag_name(&nested_full_name)
                                        && !tag_names.contains(&nested_full_name)
                                    {
                                        all_tags.push((nested_full_name.clone(), nested_metadata));
                                        tag_names.insert(nested_full_name);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        tracing::debug!(
            "Discovered {} total tags (including hierarchical)",
            all_tags.len()
        );
        Ok(all_tags)
    }

    /// Discovers members of a UDT/structure
    pub async fn discover_udt_members(
        &self,
        client: &mut EipClient,
        udt_name: &str,
    ) -> Result<Vec<(String, TagMetadata)>> {
        tracing::debug!("Discovering UDT members for: {}", udt_name);

        let udt_definition = client.get_udt_definition(udt_name).await?;

        {
            let mut definitions = self.udt_definitions.write()?;
            definitions.insert(udt_name.to_string(), udt_definition.clone());
        }

        let mut members = Vec::new();
        for member in &udt_definition.members {
            let full_name = format!("{}.{}", udt_name, member.name);
            if !self.validate_tag_name(&full_name) {
                tracing::warn!("Skipping invalid UDT member path: {}", full_name);
                continue;
            }

            let metadata = TagMetadata {
                data_type: member.data_type,
                scope: TagScope::Controller,
                permissions: TagPermissions {
                    readable: true,
                    writable: true,
                },
                is_array: false,
                dimensions: Vec::new(),
                last_access: Instant::now(),
                size: member.size,
                array_info: None,
                last_updated: Instant::now(),
            };

            tracing::trace!(
                "Found UDT member: {} (Type: 0x{:04X})",
                full_name,
                member.data_type
            );
            members.push((member.name.clone(), metadata));
        }

        Ok(members)
    }

    /// Deprecated compatibility stub retained for 1.x SemVer.
    #[deprecated(
        since = "1.2.0",
        note = "This method fabricated an invalid UDT request. Use EipClient::get_udt_definition or EipClient::discover_udt_members instead. It will be removed in 2.0."
    )]
    pub fn build_udt_definition_request(&self, _udt_name: &str) -> Result<Vec<u8>> {
        Err(EtherNetIpError::Unsupported {
            api: "TagManager::build_udt_definition_request",
            reason: "the old request builder emitted an invalid Read Tag shape; use EipClient::get_udt_definition or EipClient::discover_udt_members",
        })
    }

    /// Deprecated compatibility stub retained for 1.x SemVer.
    #[deprecated(
        since = "1.2.0",
        note = "This method fabricated UDT members from arbitrary bytes. Use EipClient::get_udt_definition or EipClient::discover_udt_members instead. It will be removed in 2.0."
    )]
    pub fn parse_udt_definition_response(
        &self,
        _response: &[u8],
        _udt_name: &str,
    ) -> Result<UdtDefinition> {
        Err(EtherNetIpError::Unsupported {
            api: "TagManager::parse_udt_definition_response",
            reason: "the old parser invented member names and fallback fields; use EipClient::get_udt_definition or EipClient::discover_udt_members",
        })
    }

    /// Validates tag name similar to the contributor's JavaScript validation
    fn validate_tag_name(&self, tag_name: &str) -> bool {
        if tag_name.is_empty() || tag_name.trim().is_empty() {
            return false;
        }

        // Check for valid characters: alphanumeric, dots, underscores
        if !TAG_NAME_RE.is_match(tag_name) {
            return false;
        }

        // Check for invalid patterns
        if tag_name.starts_with(char::is_numeric) {
            return false;
        }

        if tag_name.contains("__") || tag_name.contains("..") {
            return false;
        }

        true
    }

    /// Gets a cached UDT definition
    pub fn get_udt_definition_cached(&self, udt_name: &str) -> Option<UdtDefinition> {
        self.udt_definitions
            .read()
            .ok()
            .and_then(|definitions| definitions.get(udt_name).cloned())
    }

    /// Lists all cached UDT definitions
    pub fn list_udt_definitions(&self) -> Vec<String> {
        self.udt_definitions
            .read()
            .map(|definitions| definitions.keys().cloned().collect())
            .unwrap_or_default()
    }

    /// Clears UDT definition cache
    pub fn clear_udt_cache(&self) {
        if let Ok(mut definitions) = self.udt_definitions.write() {
            definitions.clear();
        }
    }

    pub fn parse_tag_list(&self, response: &[u8]) -> Result<Vec<(String, TagMetadata)>> {
        tracing::trace!(
            "Raw tag list response ({} bytes): {:02X?}",
            response.len(),
            response
        );

        // Check if this is a CIP error response
        if response.len() >= 3 {
            let service_reply = response[0];
            let general_status = response[2];

            // Check for error responses
            if general_status != 0x00 {
                // This is an error response, not a tag list
                let error_msg = match general_status {
                    0x01 => "Connection failure - Tag discovery may not be supported on this PLC",
                    0x04 => "Path segment error",
                    0x05 => "Path destination unknown",
                    0x16 => "Object does not exist",
                    _ => "Unknown CIP error",
                };
                return Err(crate::error::EtherNetIpError::Protocol(format!(
                    "CIP Error 0x{:02X} during tag discovery: {}. Some PLCs do not support tag discovery. Try reading tags directly by name.",
                    general_status, error_msg
                )));
            }

            // Verify this is a Get Instance Attribute List response (0xD5 = 0x55 + 0x80)
            if service_reply != 0xD5 && service_reply != 0x55 {
                // Might be a different service code, but if status is 0x00, try to parse anyway
                if general_status == 0x00 {
                    tracing::warn!(
                        "Unexpected service reply 0x{:02X}, but status is 0x00, attempting to parse",
                        service_reply
                    );
                }
            }
        }

        let mut tags = Vec::new();

        // Allen-Bradley tag list response format:
        // [ServiceReply(1)][Reserved(1)][Status(1)][AdditionalStatusSize(1)][ItemCount(4)][Items...]
        // Each item: [InstanceID(4)][NameLength(2)][Name][Type(2)][AdditionalData...]

        if response.len() < 8 {
            return Err(crate::error::EtherNetIpError::Protocol(
                "Response too short for tag list".to_string(),
            ));
        }

        // Any additional-status words sit between AdditionalStatusSize (byte 3)
        // and ItemCount, so the count is NOT at a fixed offset of 4 — it starts
        // at 4 + additional_status_size*2. On a success reply the size is 0.
        let additional_status_bytes = response[3] as usize * 2;
        let item_count_at = 4 + additional_status_bytes;
        if response.len() < item_count_at + 4 {
            return Err(crate::error::EtherNetIpError::Protocol(
                "Response too short for tag list item count".to_string(),
            ));
        }
        let item_count = u32::from_le_bytes([
            response[item_count_at],
            response[item_count_at + 1],
            response[item_count_at + 2],
            response[item_count_at + 3],
        ]);
        tracing::debug!("Detected item count: {}", item_count);

        // Items begin immediately after the 4-byte ItemCount.
        let mut offset = item_count_at + 4;

        // Parse each advertised tag entry. Truncated pages are treated as malformed
        // instead of scanning for byte patterns that may occur inside valid names.
        for item_index in 0..item_count {
            // Check if we have enough bytes for instance ID
            if offset + 4 > response.len() {
                return Err(crate::error::EtherNetIpError::Protocol(format!(
                    "Tag list ended before instance ID for item {item_index} at offset {offset}"
                )));
            }

            let instance_id = u32::from_le_bytes([
                response[offset],
                response[offset + 1],
                response[offset + 2],
                response[offset + 3],
            ]);
            offset += 4;

            // Check if we have enough bytes for name length
            if offset + 2 > response.len() {
                return Err(crate::error::EtherNetIpError::Protocol(format!(
                    "Tag list ended before name length for item {item_index} at offset {offset}"
                )));
            }

            let name_length = u16::from_le_bytes([response[offset], response[offset + 1]]) as usize;
            offset += 2;

            if name_length > 1000 || name_length == 0 {
                return Err(crate::error::EtherNetIpError::Protocol(format!(
                    "Invalid tag-list name length {} for item {} at offset {}",
                    name_length,
                    item_index,
                    offset - 2
                )));
            }

            // Check if we have enough bytes for the tag name
            if offset
                .checked_add(name_length)
                .is_none_or(|end| end > response.len())
            {
                return Err(crate::error::EtherNetIpError::Protocol(format!(
                    "Not enough bytes for tag name at offset {} (need {}, have {})",
                    offset,
                    name_length,
                    response.len() - offset
                )));
            }

            let name = String::from_utf8_lossy(&response[offset..offset + name_length]).to_string();
            offset += name_length;

            // Check if we have enough bytes for tag type
            if offset + 2 > response.len() {
                return Err(crate::error::EtherNetIpError::Protocol(format!(
                    "Tag list ended before type word for item {item_index} at offset {offset}"
                )));
            }

            let tag_type = u16::from_le_bytes([response[offset], response[offset + 1]]);
            offset += 2;

            // Parse tag type information (similar to Node.js implementation)
            let (type_code, is_structure, array_dims, _reserved) = self.parse_tag_type(tag_type);

            let is_array = array_dims > 0;
            let dimensions = Vec::new();
            let array_info = None;

            // Filter tags by type (similar to TypeScript implementation)
            if !self.is_valid_tag_type(type_code) {
                tracing::debug!(
                    "Skipping tag {} - unsupported type 0x{:04X}",
                    name,
                    type_code
                );
                continue;
            }

            let metadata = TagMetadata {
                data_type: type_code,
                scope: TagScope::Controller,
                permissions: TagPermissions {
                    readable: true,
                    writable: true,
                },
                is_array,
                dimensions,
                last_access: Instant::now(),
                size: 0,
                array_info,
                last_updated: Instant::now(),
            };

            tracing::trace!(
                "Parsed tag: {} (ID: {}, Type: 0x{:04X}, Structure: {})",
                name,
                instance_id,
                type_code,
                is_structure
            );

            tags.push((name, metadata));
        }

        tracing::debug!("Parsed {} tags from response", tags.len());
        Ok(tags)
    }

    /// Parse tag type information from the raw type value
    fn parse_tag_type(&self, tag_type: u16) -> (u16, bool, u8, bool) {
        let type_code = if (tag_type & 0x00ff) == 0xc1 {
            0x00c1
        } else {
            tag_type & 0x0fff
        };

        let is_structure = is_structure_type_word(tag_type) || is_structure_type_word(type_code);
        let array_dims = ((tag_type & 0x6000) >> 13) as u8;
        let reserved = (tag_type & 0x1000) != 0;

        (type_code, is_structure, array_dims, reserved)
    }

    /// Check if a tag type is valid for reading/writing (similar to TypeScript implementation)
    fn is_valid_tag_type(&self, type_code: u16) -> bool {
        match type_code {
            0x00C1 => true, // BOOL
            0x00C2 => true, // SINT
            0x00C3 => true, // INT
            0x00C4 => true, // DINT
            0x00C5 => true, // LINT
            0x00C6 => true, // USINT
            0x00C7 => true, // UINT
            0x00C8 => true, // UDINT
            0x00C9 => true, // ULINT
            0x00CA => true, // REAL
            0x00CB => true, // LREAL
            0x00CE => true, // STRING
            _ => is_structure_type_word(type_code),
        }
    }

    /// Recursively drill down into UDT structures (similar to TypeScript drillDown function)
    pub async fn drill_down_tags(
        &self,
        base_tags: &[(String, TagMetadata)],
    ) -> Result<Vec<(String, TagMetadata)>> {
        let mut all_tags = Vec::new();
        let mut tag_names = std::collections::HashSet::new();

        // Process each base tag
        for (tag_name, metadata) in base_tags {
            self.drill_down_recursive(&mut all_tags, &mut tag_names, tag_name, metadata, "")?;
        }

        tracing::debug!(
            "Drill down completed: {} total tags discovered",
            all_tags.len()
        );
        Ok(all_tags)
    }

    /// Recursive drill down helper (similar to TypeScript drillDown function)
    fn drill_down_recursive(
        &self,
        all_tags: &mut Vec<(String, TagMetadata)>,
        tag_names: &mut std::collections::HashSet<String>,
        tag_name: &str,
        metadata: &TagMetadata,
        previous_name: &str,
    ) -> Result<()> {
        // Skip arrays (similar to TypeScript: if (tagInfo.type.arrayDims > 0) return;)
        if metadata.is_array {
            return Ok(());
        }

        let new_name = if previous_name.is_empty() {
            tag_name.to_string()
        } else {
            format!("{}.{}", previous_name, tag_name)
        };

        // Check if this is a structure/UDT (similar to TypeScript structure check)
        if metadata.is_structure() && !metadata.is_array {
            // For now, just add the structure tag itself
            // UDT member discovery would require async calls which we'll handle separately
            if self.validate_tag_name(&new_name) && !tag_names.contains(&new_name) {
                all_tags.push((new_name.clone(), metadata.clone()));
                tag_names.insert(new_name);
            }
        } else {
            // This is a leaf tag - add it if it's a valid type
            if self.is_valid_tag_type(metadata.data_type)
                && self.validate_tag_name(&new_name)
                && !tag_names.contains(&new_name)
            {
                all_tags.push((new_name.clone(), metadata.clone()));
                tag_names.insert(new_name);
            }
        }

        Ok(())
    }
}

impl Default for TagManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::udt::UdtMember;

    #[test]
    #[expect(
        deprecated,
        reason = "CODEX-AQ keeps TagCache covered until 2.0 removal"
    )]
    fn test_tag_cache_expiration() {
        let mut cache = TagCache::new(Duration::from_secs(1));
        let metadata = TagMetadata {
            data_type: 0x00C1,
            size: 1,
            is_array: false,
            dimensions: vec![],
            permissions: TagPermissions {
                readable: true,
                writable: true,
            },
            scope: TagScope::Controller,
            last_access: Instant::now(),
            array_info: None,
            last_updated: Instant::now(),
        };

        cache.update_tag("TestTag".to_string(), metadata);
        assert!(cache.get_tag("TestTag").is_some());

        // Wait for expiration
        std::thread::sleep(Duration::from_secs(2));
        assert!(cache.get_tag("TestTag").is_none());
    }

    #[test]
    fn test_tag_metadata_is_structure() {
        // Test BOOL (not structure)
        let bool_metadata = TagMetadata {
            data_type: 0x00C1,
            size: 1,
            is_array: false,
            dimensions: vec![],
            permissions: TagPermissions {
                readable: true,
                writable: true,
            },
            scope: TagScope::Controller,
            last_access: Instant::now(),
            array_info: None,
            last_updated: Instant::now(),
        };
        assert!(!bool_metadata.is_structure());

        // Test DINT (not structure)
        let dint_metadata = TagMetadata {
            data_type: 0x00C4,
            size: 4,
            is_array: false,
            dimensions: vec![],
            permissions: TagPermissions {
                readable: true,
                writable: true,
            },
            scope: TagScope::Controller,
            last_access: Instant::now(),
            array_info: None,
            last_updated: Instant::now(),
        };
        assert!(!dint_metadata.is_structure());

        // Test UDT (structure)
        let udt_metadata = TagMetadata {
            data_type: 0x00A0,
            size: 20,
            is_array: false,
            dimensions: vec![],
            permissions: TagPermissions {
                readable: true,
                writable: true,
            },
            scope: TagScope::Controller,
            last_access: Instant::now(),
            array_info: None,
            last_updated: Instant::now(),
        };
        assert!(udt_metadata.is_structure());
    }

    #[test]
    fn test_validate_tag_name() {
        let tag_manager = TagManager::new();

        // Valid tag names
        assert!(tag_manager.validate_tag_name("ValidTag"));
        assert!(tag_manager.validate_tag_name("Valid_Tag"));
        assert!(tag_manager.validate_tag_name("Valid.Tag"));
        assert!(tag_manager.validate_tag_name("Valid123"));
        assert!(tag_manager.validate_tag_name("Valid_Tag123"));
        assert!(tag_manager.validate_tag_name("Valid.Tag123"));
        assert!(tag_manager.validate_tag_name("_Tag"));
        assert!(tag_manager.validate_tag_name("Program:Main.Tag"));
        assert!(tag_manager.validate_tag_name("Arr[3]"));
        assert!(tag_manager.validate_tag_name("Program:Main.Arr[3].Member"));

        // Invalid tag names
        assert!(!tag_manager.validate_tag_name("")); // Empty
        assert!(!tag_manager.validate_tag_name("   ")); // Whitespace only
        assert!(!tag_manager.validate_tag_name("123Invalid")); // Starts with number
        assert!(!tag_manager.validate_tag_name("Invalid..Tag")); // Double dot
        assert!(!tag_manager.validate_tag_name("Invalid-Tag")); // Invalid character
        assert!(!tag_manager.validate_tag_name("Invalid Tag")); // Space
        assert!(!tag_manager.validate_tag_name("Invalid@Tag")); // Invalid character
    }

    #[test]
    fn test_parse_tag_type() {
        let tag_manager = TagManager::new();

        // Test BOOL type
        let (type_code, is_structure, array_dims, reserved) = tag_manager.parse_tag_type(0x00C1);
        assert_eq!(type_code, 0x00C1);
        assert!(!is_structure);
        assert_eq!(array_dims, 0);
        assert!(!reserved);

        // Test DINT type
        let (type_code, is_structure, array_dims, reserved) = tag_manager.parse_tag_type(0x00C4);
        assert_eq!(type_code, 0x00C4);
        assert!(!is_structure);
        assert_eq!(array_dims, 0);
        assert!(!reserved);

        // Test structure type
        let (type_code, is_structure, array_dims, reserved) = tag_manager.parse_tag_type(0x80A0);
        assert_eq!(type_code, 0x00A0);
        assert!(is_structure);
        assert_eq!(array_dims, 0);
        assert!(!reserved);

        let (type_code, is_structure, array_dims, reserved) = tag_manager.parse_tag_type(0x82A0);
        assert_eq!(type_code, 0x02A0);
        assert!(is_structure);
        assert_eq!(array_dims, 0);
        assert!(!reserved);

        // Test array type
        let (type_code, is_structure, array_dims, reserved) = tag_manager.parse_tag_type(0x20C4);
        assert_eq!(type_code, 0x00C4);
        assert!(!is_structure);
        assert_eq!(array_dims, 1);
        assert!(!reserved);

        // Test multi-dimensional array
        let (type_code, is_structure, array_dims, reserved) = tag_manager.parse_tag_type(0x40C4);
        assert_eq!(type_code, 0x00C4);
        assert!(!is_structure);
        assert_eq!(array_dims, 2);
        assert!(!reserved);
    }

    #[test]
    fn test_udt_definition_caching() {
        let tag_manager = TagManager::new();

        // Initially no UDT definitions
        assert!(tag_manager.list_udt_definitions().is_empty());

        // Create a test UDT definition
        let udt_def = UdtDefinition {
            name: "TestUDT".to_string(),
            members: vec![
                UdtMember {
                    name: "Value1".to_string(),
                    data_type: 0x00C1,
                    offset: 0,
                    size: 1,
                },
                UdtMember {
                    name: "Value2".to_string(),
                    data_type: 0x00C4,
                    offset: 4,
                    size: 4,
                },
            ],
        };

        // Manually add to cache (simulating discovery)
        {
            let mut definitions = tag_manager
                .udt_definitions
                .write()
                .expect("test UDT definition cache lock should not be poisoned");
            definitions.insert("TestUDT".to_string(), udt_def);
        }

        // Should now be able to retrieve it
        let retrieved = tag_manager.get_udt_definition_cached("TestUDT");
        assert!(retrieved.is_some());
        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.name, "TestUDT");
        assert_eq!(retrieved.members.len(), 2);

        // Should be in the list
        let udt_list = tag_manager.list_udt_definitions();
        assert_eq!(udt_list.len(), 1);
        assert_eq!(udt_list[0], "TestUDT");

        // Clear cache
        tag_manager.clear_udt_cache();
        assert!(tag_manager.list_udt_definitions().is_empty());
        assert!(tag_manager.get_udt_definition_cached("TestUDT").is_none());
    }

    #[test]
    fn test_parse_tag_list_with_invalid_data() {
        let tag_manager = TagManager::new();

        // Test with response that has invalid name length
        let invalid_response = [
            0x00, 0x00, 0x00, 0x00, // Instance ID
            0xFF, 0xFF, // Invalid name length (65535)
            0x00, 0x00, 0x00, 0x00, // Some data
        ];

        let result = tag_manager.parse_tag_list(&invalid_response);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_tag_list_with_valid_data() {
        let tag_manager = TagManager::new();

        let valid_response = [
            0xD5, 0x00, 0x00, 0x00, // Service, reserved, success, no additional status
            0x01, 0x00, 0x00, 0x00, // Item count
            0x2A, 0x00, 0x00, 0x00, // Instance ID
            0x09, 0x00, // Name length (9)
            b'M', b'o', b't', b'o', b'r', b'D', b'a', b't', b'a', // "MotorData"
            0xC4, 0x00, // DINT type
        ];

        let tags = tag_manager.parse_tag_list(&valid_response).unwrap();
        assert_eq!(tags.len(), 1);
        assert_eq!(tags[0].0, "MotorData");
        assert_eq!(tags[0].1.data_type, 0x00C4);
        assert!(!tags[0].1.is_array);
        assert!(tags[0].1.dimensions.is_empty());
        assert!(tags[0].1.array_info.is_none());
    }

    #[test]
    fn test_tag_scope_enum() {
        // Test Controller scope
        let controller_scope = TagScope::Controller;
        assert_eq!(controller_scope, TagScope::Controller);

        // Test Program scope
        let program_scope = TagScope::Program("MainProgram".to_string());
        match program_scope {
            TagScope::Program(name) => assert_eq!(name, "MainProgram"),
            _ => panic!("Expected Program scope"),
        }

        // Test Global scope
        let global_scope = TagScope::Global;
        assert_eq!(global_scope, TagScope::Global);

        // Test Local scope
        let local_scope = TagScope::Local;
        assert_eq!(local_scope, TagScope::Local);
    }

    #[test]
    fn test_array_info() {
        let array_info = ArrayInfo {
            dimensions: vec![10, 20],
            element_count: 200,
        };

        assert_eq!(array_info.dimensions, vec![10, 20]);
        assert_eq!(array_info.element_count, 200);
    }

    #[test]
    fn test_tag_permissions() {
        let permissions = TagPermissions {
            readable: true,
            writable: false,
        };

        assert!(permissions.readable);
        assert!(!permissions.writable);
    }
}
