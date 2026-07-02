use bytes::{Buf, BufMut, BytesMut};

use crate::{Decode, Encode, ProtocolError, Result};
use rust_ethernet_ip_types::{PlcValue, UdtData};

pub const BOOL: u16 = 0x00C1;
pub const SINT: u16 = 0x00C2;
pub const INT: u16 = 0x00C3;
pub const DINT: u16 = 0x00C4;
pub const LINT: u16 = 0x00C5;
pub const USINT: u16 = 0x00C6;
pub const UINT: u16 = 0x00C7;
pub const UDINT: u16 = 0x00C8;
pub const ULINT: u16 = 0x00C9;
pub const REAL: u16 = 0x00CA;
pub const LREAL: u16 = 0x00CB;
pub const STRING: u16 = 0x00CE;
pub const ALT_STRING: u16 = 0x00DA;
pub const BOOL_ARRAY_DWORD: u16 = 0x00D3;
pub const UDT: u16 = 0x00A0;
pub const AB_UDT: u16 = 0x02A0;
pub const STANDARD_STRING_HANDLE: u16 = 0x0FCE;
pub const STANDARD_STRING_DATA_LEN: usize = 82;
pub const STANDARD_STRING_PAD_LEN: usize = 2;
pub const STANDARD_STRING_PAYLOAD_LEN: usize =
    4 + STANDARD_STRING_DATA_LEN + STANDARD_STRING_PAD_LEN;

pub fn write_data_type(value: &PlcValue) -> u16 {
    if let PlcValue::Udt(_) = value {
        value.known_data_type().unwrap_or(UDT)
    } else {
        value.get_data_type()
    }
}

pub fn write_data_type_bytes(value: &PlcValue) -> Vec<u8> {
    if matches!(value, PlcValue::String(_)) {
        let mut bytes = Vec::with_capacity(4);
        bytes.extend_from_slice(&AB_UDT.to_le_bytes());
        bytes.extend_from_slice(&STANDARD_STRING_HANDLE.to_le_bytes());
        bytes
    } else {
        write_data_type(value).to_le_bytes().to_vec()
    }
}

pub fn encode_payload(value: &PlcValue, buf: &mut BytesMut) {
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

pub fn encode_type_prefixed(value: &PlcValue, buf: &mut BytesMut) {
    buf.put_slice(&write_data_type_bytes(value));
    match value {
        PlcValue::String(v) => encode_standard_string_payload(v, buf),
        PlcValue::Udt(udt_data) => buf.put_slice(&udt_data.data),
        _ => encode_payload(value, buf),
    }
}

pub fn decode_payload(data_type: u16, value_data: &[u8]) -> Result<PlcValue> {
    match data_type {
        BOOL => {
            require_len(value_data, 1, "BOOL")?;
            Ok(PlcValue::Bool(value_data[0] != 0))
        }
        SINT => {
            require_len(value_data, 1, "SINT")?;
            Ok(PlcValue::Sint(value_data[0] as i8))
        }
        INT => {
            require_len(value_data, 2, "INT")?;
            Ok(PlcValue::Int(i16::from_le_bytes([
                value_data[0],
                value_data[1],
            ])))
        }
        DINT => {
            require_len(value_data, 4, "DINT")?;
            Ok(PlcValue::Dint(i32::from_le_bytes([
                value_data[0],
                value_data[1],
                value_data[2],
                value_data[3],
            ])))
        }
        LINT => {
            require_len(value_data, 8, "LINT")?;
            Ok(PlcValue::Lint(i64::from_le_bytes(
                value_data[..8]
                    .try_into()
                    .expect("length checked before fixed-width LINT decode"),
            )))
        }
        USINT => {
            require_len(value_data, 1, "USINT")?;
            Ok(PlcValue::Usint(value_data[0]))
        }
        UINT => {
            require_len(value_data, 2, "UINT")?;
            Ok(PlcValue::Uint(u16::from_le_bytes([
                value_data[0],
                value_data[1],
            ])))
        }
        UDINT => {
            require_len(value_data, 4, "UDINT")?;
            Ok(PlcValue::Udint(u32::from_le_bytes([
                value_data[0],
                value_data[1],
                value_data[2],
                value_data[3],
            ])))
        }
        ULINT => {
            require_len(value_data, 8, "ULINT")?;
            Ok(PlcValue::Ulint(u64::from_le_bytes(
                value_data[..8]
                    .try_into()
                    .expect("length checked before fixed-width ULINT decode"),
            )))
        }
        REAL => {
            require_len(value_data, 4, "REAL")?;
            Ok(PlcValue::Real(f32::from_le_bytes([
                value_data[0],
                value_data[1],
                value_data[2],
                value_data[3],
            ])))
        }
        LREAL => {
            require_len(value_data, 8, "LREAL")?;
            Ok(PlcValue::Lreal(f64::from_le_bytes(
                value_data[..8]
                    .try_into()
                    .expect("length checked before fixed-width LREAL decode"),
            )))
        }
        STRING => decode_dint_string(value_data),
        ALT_STRING => decode_short_string(value_data),
        AB_UDT | UDT => decode_structure_payload(value_data),
        BOOL_ARRAY_DWORD => {
            if value_data.len() >= 4 {
                Ok(PlcValue::Udint(u32::from_le_bytes([
                    value_data[0],
                    value_data[1],
                    value_data[2],
                    value_data[3],
                ])))
            } else {
                Err(ProtocolError::new(
                    "Insufficient data for DWORD value".to_string(),
                ))
            }
        }
        _ => Err(ProtocolError::new(format!(
            "Unsupported data type: 0x{data_type:04X}"
        ))),
    }
}

pub fn decode_array_element(data_type: u16, chunk: &[u8]) -> Result<PlcValue> {
    decode_payload(data_type, chunk)
}

fn encode_standard_string_payload(value: &str, buf: &mut BytesMut) {
    let string_bytes = value.as_bytes();
    let data_len = string_bytes.len().min(STANDARD_STRING_DATA_LEN);
    buf.put_u32_le(data_len as u32);
    buf.put_slice(&string_bytes[..data_len]);
    buf.resize(buf.len() + (STANDARD_STRING_DATA_LEN - data_len), 0);
    buf.resize(buf.len() + STANDARD_STRING_PAD_LEN, 0);
}

fn decode_structure_payload(value_data: &[u8]) -> Result<PlcValue> {
    if value_data.len() >= 2 {
        let handle = u16::from_le_bytes([value_data[0], value_data[1]]);
        if handle == STANDARD_STRING_HANDLE {
            return decode_standard_string_structure(value_data);
        }
    }

    Ok(PlcValue::Udt(UdtData {
        symbol_id: 0,
        data: value_data.to_vec(),
    }))
}

fn decode_standard_string_structure(value_data: &[u8]) -> Result<PlcValue> {
    let required = 2 + STANDARD_STRING_PAYLOAD_LEN;
    if value_data.len() < required {
        return Err(ProtocolError::new(format!(
            "Insufficient data for standard STRING structure: need {required} bytes, have {} bytes",
            value_data.len()
        )));
    }

    let payload = &value_data[2..];
    let length = u32::from_le_bytes([payload[0], payload[1], payload[2], payload[3]]) as usize;
    if length > STANDARD_STRING_DATA_LEN {
        return Err(ProtocolError::new(format!(
            "Invalid standard STRING length: {length} > {STANDARD_STRING_DATA_LEN}"
        )));
    }

    Ok(PlcValue::String(
        String::from_utf8_lossy(&payload[4..4 + length]).to_string(),
    ))
}

fn decode_dint_string(value_data: &[u8]) -> Result<PlcValue> {
    if value_data.len() < 4 {
        return Err(ProtocolError::new(
            "Insufficient data for STRING length field".to_string(),
        ));
    }

    let length =
        u32::from_le_bytes([value_data[0], value_data[1], value_data[2], value_data[3]]) as usize;
    if value_data.len() - 4 < length {
        return Err(ProtocolError::new(format!(
            "Insufficient data for STRING value: need {} bytes, have {} bytes",
            4 + length,
            value_data.len()
        )));
    }
    Ok(PlcValue::String(
        String::from_utf8_lossy(&value_data[4..4 + length]).to_string(),
    ))
}

fn decode_short_string(value_data: &[u8]) -> Result<PlcValue> {
    if value_data.is_empty() {
        return Ok(PlcValue::String(String::new()));
    }
    let length = value_data[0] as usize;
    if value_data.len() < 1 + length {
        return Err(ProtocolError::new(
            "Insufficient data for STRING value".to_string(),
        ));
    }
    Ok(PlcValue::String(
        String::from_utf8_lossy(&value_data[1..1 + length]).to_string(),
    ))
}

fn require_len(value_data: &[u8], min_len: usize, name: &str) -> Result<()> {
    if value_data.len() < min_len {
        let msg = if min_len == 1 {
            format!("No data for {name} value")
        } else {
            format!("Insufficient data for {name} value")
        };
        Err(ProtocolError::new(msg))
    } else {
        Ok(())
    }
}

impl Encode for PlcValue {
    fn encode(&self, buf: &mut BytesMut) {
        encode_type_prefixed(self, buf);
    }
}

impl Decode for PlcValue {
    fn decode(buf: &mut impl Buf) -> Result<Self> {
        if buf.remaining() < 2 {
            return Err(ProtocolError::new("Data too short for type".to_string()));
        }
        let data_type = buf.get_u16_le();
        let remaining = buf.copy_to_bytes(buf.remaining());
        decode_payload(data_type, &remaining)
    }
}

impl Encode for UdtData {
    fn encode(&self, buf: &mut BytesMut) {
        buf.put_slice(&self.data);
    }
}

impl Decode for UdtData {
    fn decode(buf: &mut impl Buf) -> Result<Self> {
        let data = buf.copy_to_bytes(buf.remaining()).to_vec();
        Ok(Self { symbol_id: 0, data })
    }
}
