use bytes::{BufMut, BytesMut};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeError {
    message: String,
}

impl TypeError {
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

pub type Result<T> = std::result::Result<T, TypeError>;

pub trait UdtCodec {
    fn to_hash_map(&self, data: &[u8]) -> Result<HashMap<String, PlcValue>>;
    fn encode_hash_map(&self, values: &HashMap<String, PlcValue>) -> Result<Vec<u8>>;
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct UdtData {
    pub symbol_id: i32,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum PlcValue {
    Bool(bool),
    Sint(i8),
    Int(i16),
    Dint(i32),
    Lint(i64),
    Usint(u8),
    Uint(u16),
    Udint(u32),
    Ulint(u64),
    Real(f32),
    Lreal(f64),
    String(String),
    Udt(UdtData),
}

impl UdtData {
    pub fn parse(&self, definition: &impl UdtCodec) -> Result<HashMap<String, PlcValue>> {
        definition.to_hash_map(&self.data)
    }

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
    #[must_use]
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = BytesMut::new();
        encode_payload(self, &mut bytes);
        bytes.to_vec()
    }

    #[must_use]
    pub fn get_data_type(&self) -> u16 {
        match self {
            PlcValue::Bool(_) => 0x00C1,
            PlcValue::Sint(_) => 0x00C2,
            PlcValue::Int(_) => 0x00C3,
            PlcValue::Dint(_) => 0x00C4,
            PlcValue::Lint(_) => 0x00C5,
            PlcValue::Usint(_) => 0x00C6,
            PlcValue::Uint(_) => 0x00C7,
            PlcValue::Udint(_) => 0x00C8,
            PlcValue::Ulint(_) => 0x00C9,
            PlcValue::Real(_) => 0x00CA,
            PlcValue::Lreal(_) => 0x00CB,
            PlcValue::String(_) => 0x00CE,
            PlcValue::Udt(_) => 0x00A0,
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
        PlcValue::String(v) => {
            let length = v.len().min(82) as u32;
            buf.put_u32_le(length);
            let string_bytes = v.as_bytes();
            let data_len = string_bytes.len().min(82);
            buf.put_slice(&string_bytes[..data_len]);
        }
        PlcValue::Udt(udt_data) => buf.put_slice(&udt_data.data),
    }
}
