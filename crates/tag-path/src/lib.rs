//! Parse and encode symbolic Logix tag paths.
// tag_path.rs - Advanced Tag Path Parsing for Allen-Bradley PLCs
// =========================================================================
//
// This module provides comprehensive tag path parsing and generation for
// Allen-Bradley CompactLogix and ControlLogix PLCs, supporting:
//
// - Program-scoped tags: "Program:MainProgram.Tag1"
// - Array elements: "MyArray[5]", "MyArray[1,2,3]"
// - Bit access: "MyDINT.15" (access individual bits)
// - UDT members: "MyUDT.Member1.SubMember"
// - String operations: "MyString.LEN", "MyString.DATA[5]"
//
// =========================================================================

/// Result type returned by tag-path parsing and encoding operations.
pub type Result<T> = std::result::Result<T, TagPathError>;

/// Error produced by an invalid symbolic tag path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TagPathError {
    message: String,
}

impl TagPathError {
    /// Creates a path/protocol error with a human-readable message.
    pub fn protocol(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    /// Returns the error message.
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl std::fmt::Display for TagPathError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for TagPathError {}
use std::fmt;

/// Represents different types of tag addressing supported by Allen-Bradley PLCs
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum TagPath {
    /// Simple controller-scoped tag: `MyTag`
    Controller {
        /// Controller-scoped symbolic tag name.
        tag_name: String,
    },

    /// Program-scoped tag: `Program:MainProgram.MyTag`
    Program {
        /// Program name without the `Program:` prefix.
        program_name: String,
        /// Tag name relative to the program scope.
        tag_name: String,
    },

    /// Array element access: `MyArray[5]` or `MyArray[1,2,3]`
    Array {
        /// Path to the array value.
        base_path: Box<TagPath>,
        /// One or more zero-based Logix array indices.
        indices: Vec<u32>,
    },

    /// Bit access within a tag: "MyDINT.15"
    Bit {
        /// Path to the containing integer value.
        base_path: Box<TagPath>,
        /// Zero-based bit index.
        bit_index: u8,
    },

    /// UDT member access: "MyUDT.Member1"
    Member {
        /// Path to the containing structure.
        base_path: Box<TagPath>,
        /// Structure member name.
        member_name: String,
    },

    /// String length access: "MyString.LEN"
    StringLength {
        /// Path to the containing Logix string.
        base_path: Box<TagPath>,
    },

    /// String data access: `"MyString.DATA[5]"`
    StringData {
        /// Path to the containing Logix string.
        base_path: Box<TagPath>,
        /// Zero-based byte index in the string data array.
        index: u32,
    },
}

impl TagPath {
    /// Parses a tag path string into a structured `TagPath`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rust_ethernet_ip_tag_path::TagPath;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// // Simple controller tag
    /// let path = TagPath::parse("MyTag")?;
    ///
    /// // Program-scoped tag
    /// let path = TagPath::parse("Program:MainProgram.MyTag")?;
    ///
    /// // Array element
    /// let path = TagPath::parse("MyArray[5]")?;
    ///
    /// // Multi-dimensional array
    /// let path = TagPath::parse("Matrix[1,2,3]")?;
    ///
    /// // Bit access
    /// let path = TagPath::parse("StatusWord.15")?;
    ///
    /// // UDT member
    /// let path = TagPath::parse("MotorData.Speed")?;
    ///
    /// // Complex nested path
    /// let path = TagPath::parse("Program:Safety.Devices[2].Status.15")?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn parse(path_str: &str) -> Result<Self> {
        let parser = TagPathParser::new(path_str);
        parser.parse()
    }

    /// Converts the `TagPath` back to a string representation
    pub fn as_string(&self) -> String {
        match self {
            TagPath::Controller { tag_name } => tag_name.clone(),
            TagPath::Program {
                program_name,
                tag_name,
            } => {
                format!("Program:{program_name}.{tag_name}")
            }
            TagPath::Array { base_path, indices } => {
                let base = base_path.as_string();
                let indices_str = indices
                    .iter()
                    .map(u32::to_string)
                    .collect::<Vec<_>>()
                    .join(",");
                format!("{base}[{indices_str}]")
            }
            TagPath::Bit {
                base_path,
                bit_index,
            } => {
                format!("{base_path}.{bit_index}")
            }
            TagPath::Member {
                base_path,
                member_name,
            } => {
                format!("{base_path}.{member_name}")
            }
            TagPath::StringLength { base_path } => {
                format!("{base_path}.LEN")
            }
            TagPath::StringData { base_path, index } => {
                format!("{base_path}.DATA[{index}]")
            }
        }
    }

    /// Generates the CIP path bytes for this tag path
    ///
    /// This converts the structured tag path into the binary format
    /// required by the CIP protocol for EtherNet/IP communication.
    pub fn to_cip_path(&self) -> Result<Vec<u8>> {
        let mut path = Vec::new();
        self.build_cip_path(&mut path)?;

        // Pad to even length if necessary
        if !path.len().is_multiple_of(2) {
            path.push(0x00);
        }

        Ok(path)
    }

    /// Recursively builds the CIP path bytes
    fn build_cip_path(&self, path: &mut Vec<u8>) -> Result<()> {
        match self {
            TagPath::Controller { tag_name } => {
                // ANSI Extended Symbol Segment
                path.push(0x91);
                path.push(tag_name.len() as u8);
                path.extend_from_slice(tag_name.as_bytes());
            }

            TagPath::Program {
                program_name,
                tag_name,
            } => {
                // Program scope requires special handling
                // First add program name segment
                path.push(0x91);
                let program_path = format!("Program:{program_name}");
                path.push(program_path.len() as u8);
                path.extend_from_slice(program_path.as_bytes());

                // Pad to even length if necessary after program segment
                if !path.len().is_multiple_of(2) {
                    path.push(0x00);
                }

                // Then add tag name segment
                path.push(0x91);
                path.push(tag_name.len() as u8);
                path.extend_from_slice(tag_name.as_bytes());
            }

            TagPath::Array { base_path, indices } => {
                // Build base path first
                base_path.build_cip_path(path)?;

                // Pad to even length if necessary before adding array segments
                if !path.len().is_multiple_of(2) {
                    path.push(0x00);
                }

                for &index in indices {
                    append_element_id_segment(path, index);
                }
            }

            TagPath::Bit {
                base_path,
                bit_index: _,
            } => {
                // Allen-Bradley Logix has no CIP path segment for addressing an
                // individual bit of an atomic tag. (The previous `0x29 + 1 byte`
                // encoding was a malformed 16-bit logical-member segment, which
                // expects a pad byte plus two value bytes.) The bit is resolved
                // entirely client-side — read-modify-write for writes and a mask
                // for reads — so the wire path addresses only the parent word.
                // See `EipClient::read_bit` / `EipClient::write_bit`.
                base_path.build_cip_path(path)?;
            }

            TagPath::Member {
                base_path,
                member_name,
            } => {
                // Build base path first
                base_path.build_cip_path(path)?;

                // Pad to even length if necessary before adding member segment
                if !path.len().is_multiple_of(2) {
                    path.push(0x00);
                }

                // Add member segment
                path.push(0x91);
                path.push(member_name.len() as u8);
                path.extend_from_slice(member_name.as_bytes());
            }

            TagPath::StringLength { base_path } => {
                // Build base path first
                base_path.build_cip_path(path)?;

                // Pad to even length if necessary before adding member segment
                if !path.len().is_multiple_of(2) {
                    path.push(0x00);
                }

                // Add LEN member
                path.push(0x91);
                path.push(3); // "LEN".len()
                path.extend_from_slice(b"LEN");
            }

            TagPath::StringData { base_path, index } => {
                // Build base path first
                base_path.build_cip_path(path)?;

                // Pad to even length if necessary before adding member segment
                if !path.len().is_multiple_of(2) {
                    path.push(0x00);
                }

                // Add DATA member
                path.push(0x91);
                path.push(4); // "DATA".len()
                path.extend_from_slice(b"DATA");

                // Pad to even length if necessary before adding array segment
                if !path.len().is_multiple_of(2) {
                    path.push(0x00);
                }

                append_element_id_segment(path, *index);
            }
        }

        Ok(())
    }

    /// Returns the base tag name without any path qualifiers
    pub fn base_tag_name(&self) -> String {
        match self {
            TagPath::Controller { tag_name } => tag_name.clone(),
            TagPath::Program { tag_name, .. } => tag_name.clone(),
            TagPath::Array { base_path, .. } => base_path.base_tag_name(),
            TagPath::Bit { base_path, .. } => base_path.base_tag_name(),
            TagPath::Member { base_path, .. } => base_path.base_tag_name(),
            TagPath::StringLength { base_path } => base_path.base_tag_name(),
            TagPath::StringData { base_path, .. } => base_path.base_tag_name(),
        }
    }

    /// Returns true if this is a program-scoped tag
    pub fn is_program_scoped(&self) -> bool {
        match self {
            TagPath::Program { .. } => true,
            TagPath::Array { base_path, .. } => base_path.is_program_scoped(),
            TagPath::Bit { base_path, .. } => base_path.is_program_scoped(),
            TagPath::Member { base_path, .. } => base_path.is_program_scoped(),
            TagPath::StringLength { base_path } => base_path.is_program_scoped(),
            TagPath::StringData { base_path, .. } => base_path.is_program_scoped(),
            TagPath::Controller { .. } => false,
        }
    }

    /// Returns the program name if this is a program-scoped tag
    pub fn program_name(&self) -> Option<String> {
        match self {
            TagPath::Program { program_name, .. } => Some(program_name.clone()),
            TagPath::Array { base_path, .. } => base_path.program_name(),
            TagPath::Bit { base_path, .. } => base_path.program_name(),
            TagPath::Member { base_path, .. } => base_path.program_name(),
            TagPath::StringLength { base_path } => base_path.program_name(),
            TagPath::StringData { base_path, .. } => base_path.program_name(),
            TagPath::Controller { .. } => None,
        }
    }
}

/// Adds a CIP Element ID segment using the smallest valid operand width.
///
/// Reference: 1756-PM020, Pages 603-611, 870-890.
/// - 0..=255: 8-bit Element ID (`0x28`, value)
/// - 256..=65535: 16-bit Element ID (`0x29`, pad, low, high)
/// - 65536+: 32-bit Element ID (`0x2A`, pad, byte0..byte3)
fn append_element_id_segment(path: &mut Vec<u8>, index: u32) {
    if index <= 255 {
        path.push(0x28);
        path.push(index as u8);
    } else if index <= 65535 {
        path.push(0x29);
        path.push(0x00);
        path.extend_from_slice(&(index as u16).to_le_bytes());
    } else {
        path.push(0x2A);
        path.push(0x00);
        path.extend_from_slice(&index.to_le_bytes());
    }
}

impl fmt::Display for TagPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_string())
    }
}

/// Internal parser for tag path strings
struct TagPathParser<'a> {
    input: &'a str,
    position: usize,
}

impl<'a> TagPathParser<'a> {
    fn new(input: &'a str) -> Self {
        Self { input, position: 0 }
    }

    fn parse(mut self) -> Result<TagPath> {
        self.parse_path()
    }

    fn parse_path(&mut self) -> Result<TagPath> {
        // Check for program scope
        if self.input.starts_with("Program:") {
            self.parse_program_scoped()
        } else {
            self.parse_controller_scoped()
        }
    }

    fn parse_program_scoped(&mut self) -> Result<TagPath> {
        // Skip "Program:"
        self.position = 8;

        // Parse program name (until first dot)
        let program_name = self.parse_identifier()?;

        // Expect dot
        if !self.consume_char('.') {
            return Err(TagPathError::protocol(
                "Expected '.' after program name".to_string(),
            ));
        }

        // Parse tag name
        let tag_name = self.parse_identifier()?;

        let mut path = TagPath::Program {
            program_name,
            tag_name,
        };

        // Parse any additional qualifiers (arrays, members, bits)
        while self.position < self.input.len() {
            path = self.parse_qualifier(path)?;
        }

        Ok(path)
    }

    fn parse_controller_scoped(&mut self) -> Result<TagPath> {
        let tag_name = self.parse_identifier()?;
        let mut path = TagPath::Controller { tag_name };

        // Parse any additional qualifiers
        while self.position < self.input.len() {
            path = self.parse_qualifier(path)?;
        }

        Ok(path)
    }

    fn parse_qualifier(&mut self, base_path: TagPath) -> Result<TagPath> {
        match self.peek_char() {
            Some('[') => self.parse_array_access(base_path),
            Some('.') => self.parse_member_or_bit_access(base_path),
            _ => Err(TagPathError::protocol(format!(
                "Unexpected character at position {}",
                self.position
            ))),
        }
    }

    fn parse_array_access(&mut self, base_path: TagPath) -> Result<TagPath> {
        // Consume '['
        self.consume_char('[');

        let mut indices = Vec::new();

        // Parse first index
        indices.push(self.parse_number()?);

        // Parse additional indices separated by commas
        while self.peek_char() == Some(',') {
            self.consume_char(',');
            indices.push(self.parse_number()?);
        }

        // Expect ']'
        if !self.consume_char(']') {
            return Err(TagPathError::protocol(
                "Expected ']' after array indices".to_string(),
            ));
        }

        Ok(TagPath::Array {
            base_path: Box::new(base_path),
            indices,
        })
    }

    fn parse_member_or_bit_access(&mut self, base_path: TagPath) -> Result<TagPath> {
        // Consume '.'
        self.consume_char('.');

        // Check for special string operations — only match if it's the complete segment
        let remaining = &self.input[self.position..];
        if remaining.starts_with("LEN")
            && (remaining.len() == 3
                || remaining.as_bytes()[3] == b'.'
                || remaining.as_bytes()[3] == b'[')
        {
            self.position += 3;
            return Ok(TagPath::StringLength {
                base_path: Box::new(base_path),
            });
        }

        if remaining.starts_with("DATA[") {
            self.position += 5; // Skip "DATA["
            let index = self.parse_number()?;
            if !self.consume_char(']') {
                return Err(TagPathError::protocol(
                    "Expected ']' after DATA index".to_string(),
                ));
            }
            return Ok(TagPath::StringData {
                base_path: Box::new(base_path),
                index,
            });
        }

        // Parse identifier (could be member name or bit index)
        let identifier = self.parse_identifier()?;

        // Check if it's a numeric bit index
        if let Ok(bit_index) = identifier.parse::<u8>()
            && bit_index < 32
        {
            // Valid bit range for DINT
            return Ok(TagPath::Bit {
                base_path: Box::new(base_path),
                bit_index,
            });
        }

        // It's a member name
        Ok(TagPath::Member {
            base_path: Box::new(base_path),
            member_name: identifier,
        })
    }

    fn parse_identifier(&mut self) -> Result<String> {
        let start = self.position;

        while self.position < self.input.len() {
            let ch = self.input.as_bytes()[self.position];
            if ch.is_ascii_alphanumeric() || ch == b'_' {
                self.position += 1;
            } else {
                break;
            }
        }

        if start == self.position {
            return Err(TagPathError::protocol("Expected identifier".to_string()));
        }

        Ok(self.input[start..self.position].to_string())
    }

    fn parse_number(&mut self) -> Result<u32> {
        let start = self.position;

        while self.position < self.input.len() {
            let ch = self.input.as_bytes()[self.position];
            if ch.is_ascii_digit() {
                self.position += 1;
            } else {
                break;
            }
        }

        if start == self.position {
            return Err(TagPathError::protocol("Expected number".to_string()));
        }

        self.input[start..self.position]
            .parse()
            .map_err(|_| TagPathError::protocol("Invalid number".to_string()))
    }

    fn peek_char(&self) -> Option<char> {
        self.input
            .as_bytes()
            .get(self.position)
            .map(|byte| *byte as char)
    }

    fn consume_char(&mut self, expected: char) -> bool {
        if self.peek_char() == Some(expected) {
            self.position += 1;
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_controller_scoped_tag() {
        let path = TagPath::parse("MyTag").unwrap();
        assert_eq!(
            path,
            TagPath::Controller {
                tag_name: "MyTag".to_string()
            }
        );
        assert_eq!(path.to_string(), "MyTag");
    }

    #[test]
    fn test_program_scoped_tag() {
        let path = TagPath::parse("Program:MainProgram.MyTag").unwrap();
        assert_eq!(
            path,
            TagPath::Program {
                program_name: "MainProgram".to_string(),
                tag_name: "MyTag".to_string()
            }
        );
        assert_eq!(path.to_string(), "Program:MainProgram.MyTag");
        assert!(path.is_program_scoped());
        assert_eq!(path.program_name(), Some("MainProgram".to_string()));
    }

    #[test]
    fn test_array_access() {
        let path = TagPath::parse("MyArray[5]").unwrap();
        if let TagPath::Array { base_path, indices } = path {
            assert_eq!(
                *base_path,
                TagPath::Controller {
                    tag_name: "MyArray".to_string()
                }
            );
            assert_eq!(indices, vec![5]);
        } else {
            panic!("Expected Array path");
        }
    }

    #[test]
    fn test_multi_dimensional_array() {
        let path = TagPath::parse("Matrix[1,2,3]").unwrap();
        if let TagPath::Array { base_path, indices } = path {
            assert_eq!(
                *base_path,
                TagPath::Controller {
                    tag_name: "Matrix".to_string()
                }
            );
            assert_eq!(indices, vec![1, 2, 3]);
        } else {
            panic!("Expected Array path");
        }
    }

    #[test]
    fn test_bit_access() {
        let path = TagPath::parse("StatusWord.15").unwrap();
        if let TagPath::Bit {
            base_path,
            bit_index,
        } = path
        {
            assert_eq!(
                *base_path,
                TagPath::Controller {
                    tag_name: "StatusWord".to_string()
                }
            );
            assert_eq!(bit_index, 15);
        } else {
            panic!("Expected Bit path");
        }
    }

    #[test]
    fn test_bit_path_emits_base_path_only() {
        // Logix bits of atomic tags are resolved client-side, so the wire path
        // must address only the parent word — no (malformed) bit segment.
        let bit_path = TagPath::parse("StatusWord.15")
            .unwrap()
            .to_cip_path()
            .unwrap();
        let base_path = TagPath::parse("StatusWord").unwrap().to_cip_path().unwrap();
        assert_eq!(
            bit_path, base_path,
            "bit path should encode identically to its parent tag"
        );
        // Explicit bytes: ANSI symbol segment 0x91, len 10, then "StatusWord".
        let mut expected = vec![0x91, 0x0A];
        expected.extend_from_slice(b"StatusWord");
        assert_eq!(bit_path, expected);
        // The old 16-bit-member marker must not appear.
        assert!(!bit_path.contains(&0x29));
    }

    #[test]
    fn test_member_access() {
        let path = TagPath::parse("MotorData.Speed").unwrap();
        if let TagPath::Member {
            base_path,
            member_name,
        } = path
        {
            assert_eq!(
                *base_path,
                TagPath::Controller {
                    tag_name: "MotorData".to_string()
                }
            );
            assert_eq!(member_name, "Speed");
        } else {
            panic!("Expected Member path");
        }
    }

    #[test]
    fn test_string_length() {
        let path = TagPath::parse("MyString.LEN").unwrap();
        if let TagPath::StringLength { base_path } = path {
            assert_eq!(
                *base_path,
                TagPath::Controller {
                    tag_name: "MyString".to_string()
                }
            );
        } else {
            panic!("Expected StringLength path");
        }
    }

    #[test]
    fn test_string_data() {
        let path = TagPath::parse("MyString.DATA[5]").unwrap();
        if let TagPath::StringData { base_path, index } = path {
            assert_eq!(
                *base_path,
                TagPath::Controller {
                    tag_name: "MyString".to_string()
                }
            );
            assert_eq!(index, 5);
        } else {
            panic!("Expected StringData path");
        }
    }

    #[test]
    fn test_string_data_cip_path_uses_8_bit_element_segment() {
        let path = TagPath::parse("MyString.DATA[5]")
            .unwrap()
            .to_cip_path()
            .unwrap();

        let mut expected = vec![0x91, 0x08];
        expected.extend_from_slice(b"MyString");
        expected.extend_from_slice(&[0x91, 0x04]);
        expected.extend_from_slice(b"DATA");
        expected.extend_from_slice(&[0x28, 0x05]);

        assert_eq!(path, expected);
    }

    #[test]
    fn test_string_data_cip_path_uses_16_bit_element_segment() {
        let path = TagPath::parse("MyString.DATA[300]")
            .unwrap()
            .to_cip_path()
            .unwrap();

        let mut expected = vec![0x91, 0x08];
        expected.extend_from_slice(b"MyString");
        expected.extend_from_slice(&[0x91, 0x04]);
        expected.extend_from_slice(b"DATA");
        expected.extend_from_slice(&[0x29, 0x00]);
        expected.extend_from_slice(&300u16.to_le_bytes());

        assert_eq!(path, expected);
    }

    #[test]
    fn test_complex_nested_path() {
        let path = TagPath::parse("Program:Safety.Devices[2].Status.15").unwrap();

        // This should parse as:
        // Program:Safety.Devices -> Array[2] -> Member(Status) -> Bit(15)
        if let TagPath::Bit {
            base_path,
            bit_index,
        } = path
        {
            assert_eq!(bit_index, 15);

            if let TagPath::Member {
                base_path,
                member_name,
            } = *base_path
            {
                assert_eq!(member_name, "Status");

                if let TagPath::Array { base_path, indices } = *base_path {
                    assert_eq!(indices, vec![2]);

                    if let TagPath::Program {
                        program_name,
                        tag_name,
                    } = *base_path
                    {
                        assert_eq!(program_name, "Safety");
                        assert_eq!(tag_name, "Devices");
                    } else {
                        panic!("Expected Program path");
                    }
                } else {
                    panic!("Expected Array path");
                }
            } else {
                panic!("Expected Member path");
            }
        } else {
            panic!("Expected Bit path");
        }
    }

    #[test]
    fn test_cip_path_generation() {
        let path = TagPath::parse("MyTag").unwrap();
        let cip_path = path.to_cip_path().unwrap();

        // Should be: [0x91, 0x05, 'M', 'y', 'T', 'a', 'g', 0x00] (padded)
        assert_eq!(cip_path[0], 0x91); // ANSI Extended Symbol Segment
        assert_eq!(cip_path[1], 5); // Length of "MyTag"
        assert_eq!(&cip_path[2..7], b"MyTag");
        assert_eq!(cip_path[7], 0x00); // Padding
    }

    #[test]
    fn test_array_cip_path_generation() {
        let path = TagPath::parse("MyArray[5]").unwrap();
        let cip_path = path.to_cip_path().unwrap();

        // Should be: [0x91, 0x07, 'M', 'y', 'A', 'r', 'r', 'a', 'y', 0x00, 0x28, 0x05]
        // Tag segment: 0x91, length 7, "MyArray", padding
        assert_eq!(cip_path[0], 0x91); // ANSI Extended Symbol Segment
        assert_eq!(cip_path[1], 7); // Length of "MyArray"
        assert_eq!(&cip_path[2..9], b"MyArray");
        assert_eq!(cip_path[9], 0x00); // Padding

        // Array element segment: 0x28 (8-bit Element ID), index 5
        // Reference: 1756-PM020, Pages 603-611 (Element ID Segment Format)
        assert_eq!(cip_path[10], 0x28); // 8-bit Element ID segment
        assert_eq!(cip_path[11], 0x05); // Index 5
        assert_eq!(cip_path.len(), 12); // Total: 9 (tag) + 1 (padding) + 2 (element segment) = 12
    }

    #[test]
    fn test_program_array_cip_path_generation() {
        let path = TagPath::parse("Program:MainProgram.ArrayTest[0]").unwrap();
        let cip_path = path.to_cip_path().unwrap();

        tracing::debug!(
            "Program array CIP path ({} bytes): {:02X?}",
            cip_path.len(),
            cip_path
        );

        // Verify structure:
        // 1. Program segment: 0x91, length 19, "Program:MainProgram", padding
        assert_eq!(cip_path[0], 0x91);
        assert_eq!(cip_path[1], 19); // "Program:MainProgram".len()
        assert_eq!(&cip_path[2..21], b"Program:MainProgram");
        assert_eq!(cip_path[21], 0x00); // Padding after program segment

        // 2. Tag segment: 0x91, length 9, "ArrayTest", padding
        assert_eq!(cip_path[22], 0x91);
        assert_eq!(cip_path[23], 9); // "ArrayTest".len()
        assert_eq!(&cip_path[24..33], b"ArrayTest");
        assert_eq!(cip_path[33], 0x00); // Padding after tag segment

        // 3. Array element segment: 0x28 (8-bit Element ID), index 0
        // Reference: 1756-PM020, Pages 603-611 (Element ID Segment Format)
        assert_eq!(cip_path[34], 0x28); // 8-bit Element ID segment
        assert_eq!(cip_path[35], 0x00); // Index 0

        // Total should be 36 bytes (18 words)
        // Program segment: 20 bytes + Tag segment: 12 bytes + Element segment: 2 bytes + padding: 2 bytes = 36 bytes
        assert_eq!(cip_path.len(), 36);
    }

    #[test]
    fn test_base_tag_name() {
        let path = TagPath::parse("Program:Main.MotorData[1].Speed.15").unwrap();
        assert_eq!(path.base_tag_name(), "MotorData");
    }

    #[test]
    fn test_invalid_paths() {
        assert!(TagPath::parse("").is_err());
        assert!(TagPath::parse("Program:").is_err());
        assert!(TagPath::parse("MyArray[").is_err());
        assert!(TagPath::parse("MyArray]").is_err());
        assert!(TagPath::parse("MyTag.").is_err());
    }
}
