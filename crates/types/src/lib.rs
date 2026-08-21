//! Shared Logix value types and UDT encoding contracts.

use bytes::{BufMut, BytesMut};
use std::collections::HashMap;

const STANDARD_STRING_DATA_LEN: usize = 82;
const STANDARD_STRING_PAD_LEN: usize = 2;

/// Error returned when a PLC value or UDT payload cannot be converted.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeError {
    message: String,
}

impl TypeError {
    /// Creates an error with a human-readable explanation.
    #[must_use]
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl std::fmt::Display for TypeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for TypeError {}

/// Result type used by this crate.
pub type Result<T> = std::result::Result<T, TypeError>;

/// Converts raw UDT payloads to and from named member values.
pub trait UdtCodec {
    /// Decodes a raw UDT payload into values keyed by member name.
    fn to_hash_map(&self, data: &[u8]) -> Result<HashMap<String, PlcValue>>;
    /// Encodes values keyed by member name into a raw UDT payload.
    fn encode_hash_map(&self, values: &HashMap<String, PlcValue>) -> Result<Vec<u8>>;
}

/// Raw bytes for a Logix user-defined structure and its template symbol id.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct UdtData {
    /// Template symbol id used to identify the structure type.
    pub symbol_id: i32,
    /// Structure payload without the CIP response type prefix.
    pub data: Vec<u8>,
}

/// A typed value read from or written to a Logix tag.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum PlcValue {
    /// Logix `BOOL`.
    Bool(bool),
    /// Logix `SINT`.
    Sint(i8),
    /// Logix `INT`.
    Int(i16),
    /// Logix `DINT`.
    Dint(i32),
    /// Logix `LINT`.
    Lint(i64),
    /// Logix `USINT`.
    Usint(u8),
    /// Logix `UINT`.
    Uint(u16),
    /// Logix `UDINT`.
    Udint(u32),
    /// Logix `ULINT`.
    Ulint(u64),
    /// Logix `REAL`.
    Real(f32),
    /// Logix `LREAL`.
    Lreal(f64),
    /// A built-in or custom Logix string value.
    String(String),
    /// A raw user-defined structure value.
    Udt(UdtData),
}

impl UdtData {
    /// Decodes this payload using the supplied UDT definition or codec.
    pub fn parse(&self, definition: &impl UdtCodec) -> Result<HashMap<String, PlcValue>> {
        definition.to_hash_map(&self.data)
    }

    /// Encodes named members and associates the result with `symbol_id`.
    pub fn from_hash_map(
        members: &HashMap<String, PlcValue>,
        definition: &impl UdtCodec,
        symbol_id: i32,
    ) -> Result<Self> {
        let data = definition.encode_hash_map(members)?;
        Ok(UdtData { symbol_id, data })
    }
}

impl PlcValue {
    /// Encodes the value payload in Logix little-endian wire order.
    #[must_use]
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = BytesMut::new();
        encode_payload(self, &mut bytes);
        bytes.to_vec()
    }

    /// Returns the CIP type code, falling back to the generic structure marker.
    #[must_use]
    pub fn get_data_type(&self) -> u16 {
        self.known_data_type().unwrap_or(0x00A0)
    }

    /// Returns a precise CIP type code when this value carries enough metadata.
    ///
    /// UDT values decoded without a template symbol id return `None` instead of
    /// pretending that the generic structure marker is a concrete write type.
    #[must_use]
    pub fn known_data_type(&self) -> Option<u16> {
        match self {
            PlcValue::Bool(_) => Some(0x00C1),
            PlcValue::Sint(_) => Some(0x00C2),
            PlcValue::Int(_) => Some(0x00C3),
            PlcValue::Dint(_) => Some(0x00C4),
            PlcValue::Lint(_) => Some(0x00C5),
            PlcValue::Usint(_) => Some(0x00C6),
            PlcValue::Uint(_) => Some(0x00C7),
            PlcValue::Udint(_) => Some(0x00C8),
            PlcValue::Ulint(_) => Some(0x00C9),
            PlcValue::Real(_) => Some(0x00CA),
            PlcValue::Lreal(_) => Some(0x00CB),
            PlcValue::String(_) => Some(0x00CE),
            PlcValue::Udt(udt_data) => {
                (udt_data.symbol_id > 0).then(|| 0x02A0u16.wrapping_add(udt_data.symbol_id as u16))
            }
        }
    }
}

fn encode_payload(value: &PlcValue, buf: &mut BytesMut) {
    match value {
        PlcValue::Bool(v) => buf.put_u8(if *v { 0xFF } else { 0x00 }),
        PlcValue::Sint(v) => buf.put_i8(*v),
        PlcValue::Int(v) => buf.put_i16_le(*v),
        PlcValue::Dint(v) => buf.put_i32_le(*v),
        PlcValue::Lint(v) => buf.put_i64_le(*v),
        PlcValue::Usint(v) => buf.put_u8(*v),
        PlcValue::Uint(v) => buf.put_u16_le(*v),
        PlcValue::Udint(v) => buf.put_u32_le(*v),
        PlcValue::Ulint(v) => buf.put_u64_le(*v),
        PlcValue::Real(v) => buf.put_slice(&v.to_le_bytes()),
        PlcValue::Lreal(v) => buf.put_slice(&v.to_le_bytes()),
        PlcValue::String(v) => encode_standard_string_payload(v, buf),
        PlcValue::Udt(udt_data) => buf.put_slice(&udt_data.data),
    }
}

fn encode_standard_string_payload(value: &str, buf: &mut BytesMut) {
    let string_bytes = value.as_bytes();
    let data_len = string_bytes.len().min(STANDARD_STRING_DATA_LEN);
    buf.put_u32_le(data_len as u32);
    buf.put_slice(&string_bytes[..data_len]);
    buf.resize(buf.len() + (STANDARD_STRING_DATA_LEN - data_len), 0);
    buf.resize(buf.len() + STANDARD_STRING_PAD_LEN, 0);
}
