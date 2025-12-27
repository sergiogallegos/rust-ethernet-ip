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

use crate::error::{EtherNetIpError, Result};
use std::fmt;

/// Represents different types of tag addressing supported by Allen-Bradley PLCs
#[derive(Debug, Clone, PartialEq)]
pub enum TagPath {
    /// Simple controller-scoped tag: `MyTag`
    Controller { tag_name: String },

    /// Program-scoped tag: `Program:MainProgram.MyTag`
    Program {
        program_name: String,
        tag_name: String,
    },

    /// Array element access: `MyArray[5]` or `MyArray[1,2,3]`
    Array {
        base_path: Box<TagPath>,
        indices: Vec<u32>,
    },

    /// Bit access within a tag: "MyDINT.15"
    Bit {
        base_path: Box<TagPath>,
        bit_index: u8,
    },

    /// UDT member access: "MyUDT.Member1"
    Member {
        base_path: Box<TagPath>,
        member_name: String,
    },

    /// String length access: "MyString.LEN"
    StringLength { base_path: Box<TagPath> },

    /// String data access: "MyString.DATA[5]"
    StringData { base_path: Box<TagPath>, index: u32 },
}

impl TagPath {
    /// Parses a tag path string into a structured `TagPath`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rust_ethernet_ip::TagPath;
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
        if path.len() % 2 != 0 {
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
                if path.len() % 2 != 0 {
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
                if path.len() % 2 != 0 {
                    path.push(0x00);
                }

                // Add array indices
                // According to CIP spec, element segments in paths use format:
                // 0x28 (element segment type) + format/size byte + index value
                //
                // Based on pylogix and gologix implementations, try using 4-byte (DINT) indices
                // for DINT arrays, as the index type should match the array element type
                // Format byte: 0x04 = 4 bytes (DINT size)
                for &index in indices {
                    path.push(0x28); // Element segment type
                    path.push(0x04); // Format: 4 bytes for 32-bit index (DINT)
                    let index_u32 = index;
                    path.extend_from_slice(&index_u32.to_le_bytes());
                    // Element segment is 6 bytes total (0x28 + 0x04 + 4 bytes index) = even, no padding needed
                }
            }

            TagPath::Bit {
                base_path,
                bit_index,
            } => {
                // Build base path first
                base_path.build_cip_path(path)?;

                // Pad to even length if necessary before adding bit segment
                if path.len() % 2 != 0 {
                    path.push(0x00);
                }

                // Add bit segment
                path.push(0x29); // Bit segment
                path.push(*bit_index);
            }

            TagPath::Member {
                base_path,
                member_name,
            } => {
                // Build base path first
                base_path.build_cip_path(path)?;

                // Pad to even length if necessary before adding member segment
                if path.len() % 2 != 0 {
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
                if path.len() % 2 != 0 {
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
                if path.len() % 2 != 0 {
                    path.push(0x00);
                }

                // Add DATA member
                path.push(0x91);
                path.push(4); // "DATA".len()
                path.extend_from_slice(b"DATA");

                // Pad to even length if necessary before adding array segment
                if path.len() % 2 != 0 {
                    path.push(0x00);
                }

                // Add array index
                path.push(0x28); // Element segment
                path.push(0x04); // Size: 4 bytes for 32-bit index (DINT)
                let index_u32 = *index as u32;
                path.extend_from_slice(&index_u32.to_le_bytes());
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
            return Err(EtherNetIpError::Protocol(
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
            _ => Err(EtherNetIpError::Protocol(format!(
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
            return Err(EtherNetIpError::Protocol(
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

        // Check for special string operations
        if self.input[self.position..].starts_with("LEN") {
            self.position += 3;
            return Ok(TagPath::StringLength {
                base_path: Box::new(base_path),
            });
        }

        if self.input[self.position..].starts_with("DATA[") {
            self.position += 5; // Skip "DATA["
            let index = self.parse_number()?;
            if !self.consume_char(']') {
                return Err(EtherNetIpError::Protocol(
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
        if let Ok(bit_index) = identifier.parse::<u8>() {
            if bit_index < 32 {
                // Valid bit range for DINT
                return Ok(TagPath::Bit {
                    base_path: Box::new(base_path),
                    bit_index,
                });
            }
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
            let ch = self.input.chars().nth(self.position).unwrap();
            if ch.is_alphanumeric() || ch == '_' {
                self.position += 1;
            } else {
                break;
            }
        }

        if start == self.position {
            return Err(EtherNetIpError::Protocol("Expected identifier".to_string()));
        }

        Ok(self.input[start..self.position].to_string())
    }

    fn parse_number(&mut self) -> Result<u32> {
        let start = self.position;

        while self.position < self.input.len() {
            let ch = self.input.chars().nth(self.position).unwrap();
            if ch.is_ascii_digit() {
                self.position += 1;
            } else {
                break;
            }
        }

        if start == self.position {
            return Err(EtherNetIpError::Protocol("Expected number".to_string()));
        }

        self.input[start..self.position]
            .parse()
            .map_err(|_| EtherNetIpError::Protocol("Invalid number".to_string()))
    }

    fn peek_char(&self) -> Option<char> {
        self.input.chars().nth(self.position)
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

        // Should be: [0x91, 0x07, 'M', 'y', 'A', 'r', 'r', 'a', 'y', 0x00, 0x28, 0x04, 0x05, 0x00, 0x00, 0x00]
        // Tag segment: 0x91, length 7, "MyArray", padding
        assert_eq!(cip_path[0], 0x91); // ANSI Extended Symbol Segment
        assert_eq!(cip_path[1], 7); // Length of "MyArray"
        assert_eq!(&cip_path[2..9], b"MyArray");
        assert_eq!(cip_path[9], 0x00); // Padding

        // Array element segment: 0x28, format 0x04 (4 bytes), index 5 (little-endian u32)
        assert_eq!(cip_path[10], 0x28); // Element segment
        assert_eq!(cip_path[11], 0x04); // Format: 4 bytes (DINT)
        assert_eq!(&cip_path[12..16], &5u32.to_le_bytes()); // Index 5 as u32
        assert_eq!(cip_path.len(), 16); // Total: 9 (tag) + 1 (padding) + 6 (element segment) = 16
    }

    #[test]
    fn test_program_array_cip_path_generation() {
        let path = TagPath::parse("Program:MainProgram.ArrayTest[0]").unwrap();
        let cip_path = path.to_cip_path().unwrap();

        println!(
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

        // 3. Array element segment: 0x28, format 0x04 (4 bytes), index 0
        assert_eq!(cip_path[34], 0x28); // Element segment
        assert_eq!(cip_path[35], 0x04); // Format: 4 bytes (DINT)
        assert_eq!(&cip_path[36..40], &0u32.to_le_bytes()); // Index 0 as u32

        // Total should be 40 bytes (20 words)
        assert_eq!(cip_path.len(), 40);
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
