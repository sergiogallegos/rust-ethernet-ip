//! CIP request, response, and Common Packet Format codecs.

use bytes::{Buf, BufMut, BytesMut};

use crate::{Decode, Encode, ProtocolError, Result};

/// CIP Read Tag service code.
pub const READ_TAG: u8 = 0x4C;
/// CIP Write Tag service code.
pub const WRITE_TAG: u8 = 0x4D;
#[allow(dead_code)]
/// CIP Multiple Service Packet service code.
pub const MULTIPLE_SERVICE_PACKET: u8 = 0x0A;

/// An unconnected CIP service request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CipRequest {
    /// CIP service code.
    pub service: u8,
    /// Word-aligned encoded request path.
    pub path: Vec<u8>,
    /// Service-specific request data.
    pub data: Vec<u8>,
}

impl CipRequest {
    /// Creates a request from an encoded path and service data.
    pub fn new(service: u8, path: Vec<u8>, data: Vec<u8>) -> Self {
        Self {
            service,
            path,
            data,
        }
    }

    /// Validates path presence, alignment, and the one-byte word-count limit.
    pub fn validate(&self) -> Result<()> {
        if self.path.is_empty() {
            return Err(ProtocolError::new(format!(
                "invalid CIP request path for service 0x{:02X}: path must not be empty",
                self.service
            )));
        }

        if !self.path.len().is_multiple_of(2) {
            return Err(ProtocolError::new(format!(
                "invalid CIP request path for service 0x{:02X}: path length {} is not word-aligned",
                self.service,
                self.path.len()
            )));
        }

        let path_words = self.path.len() / 2;
        if path_words > usize::from(u8::MAX) {
            return Err(ProtocolError::new(format!(
                "invalid CIP request path for service 0x{:02X}: path length {} bytes exceeds 510-byte CIP limit",
                self.service,
                self.path.len()
            )));
        }

        Ok(())
    }

    /// Validates and appends the encoded request to `buf`.
    pub fn encode(&self, buf: &mut BytesMut) -> Result<()> {
        self.validate()?;
        buf.put_u8(self.service);
        let path_words =
            u8::try_from(self.path.len() / 2).expect("validated path word count fits in u8");
        buf.put_u8(path_words);
        buf.put_slice(&self.path);
        buf.put_slice(&self.data);
        Ok(())
    }
}

impl Decode for CipRequest {
    fn decode(buf: &mut impl Buf) -> Result<Self> {
        if buf.remaining() < 2 {
            return Err(ProtocolError::new("CIP request too short".to_string()));
        }

        let service = buf.get_u8();
        let path_size_words = buf.get_u8() as usize;
        let path_len = path_size_words * 2;
        if buf.remaining() < path_len {
            return Err(ProtocolError::new("CIP request path truncated".to_string()));
        }
        let path = buf.copy_to_bytes(path_len).to_vec();
        let data = buf.copy_to_bytes(buf.remaining()).to_vec();

        Ok(Self {
            service,
            path,
            data,
        })
    }
}

/// A decoded CIP service response.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CipResponse {
    /// Reply service code, normally the request service with bit 7 set.
    pub service: u8,
    /// CIP general status code.
    pub status: u8,
    /// Optional 16-bit extended status words.
    pub additional_status: Vec<u16>,
    /// Service-specific response data.
    pub data: Vec<u8>,
}

impl Encode for CipResponse {
    fn encode(&self, buf: &mut BytesMut) {
        buf.put_u8(self.service);
        buf.put_u8(0);
        buf.put_u8(self.status);
        buf.put_u8(self.additional_status.len() as u8);
        for status in &self.additional_status {
            buf.put_u16_le(*status);
        }
        buf.put_slice(&self.data);
    }
}

impl Decode for CipResponse {
    fn decode(buf: &mut impl Buf) -> Result<Self> {
        if buf.remaining() < 4 {
            return Err(ProtocolError::new("CIP response too short".to_string()));
        }

        let service = buf.get_u8();
        let _reserved = buf.get_u8();
        let status = buf.get_u8();
        let additional_status_size = buf.get_u8() as usize;
        if buf.remaining() < additional_status_size * 2 {
            return Err(ProtocolError::new(
                "CIP response additional status truncated".to_string(),
            ));
        }

        let mut additional_status = Vec::with_capacity(additional_status_size);
        for _ in 0..additional_status_size {
            additional_status.push(buf.get_u16_le());
        }
        let data = buf.copy_to_bytes(buf.remaining()).to_vec();

        Ok(Self {
            service,
            status,
            additional_status,
            data,
        })
    }
}

/// One Common Packet Format item.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CpfItem {
    /// CPF item type identifier.
    pub type_id: u16,
    /// Item payload.
    pub data: Vec<u8>,
}

/// Payload carried by SendRRData or SendUnitData encapsulation commands.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SendDataRequest {
    /// Encapsulation interface handle; zero for CIP.
    pub interface_handle: u32,
    /// Encapsulation timeout in seconds.
    pub timeout: u16,
    /// Common Packet Format items.
    pub items: Vec<CpfItem>,
}

impl SendDataRequest {
    /// Wraps an unconnected CIP message in the standard null-address/data items.
    pub fn unconnected(item_data: &[u8]) -> Self {
        Self {
            interface_handle: 0,
            timeout: 5,
            items: vec![
                CpfItem {
                    type_id: 0x0000,
                    data: Vec::new(),
                },
                CpfItem {
                    type_id: 0x00B2,
                    data: item_data.to_vec(),
                },
            ],
        }
    }
}

impl Encode for SendDataRequest {
    fn encode(&self, buf: &mut BytesMut) {
        buf.put_u32_le(self.interface_handle);
        buf.put_u16_le(self.timeout);
        buf.put_u16_le(self.items.len() as u16);
        for item in &self.items {
            buf.put_u16_le(item.type_id);
            buf.put_u16_le(item.data.len() as u16);
            buf.put_slice(&item.data);
        }
    }
}

impl Decode for SendDataRequest {
    fn decode(buf: &mut impl Buf) -> Result<Self> {
        if buf.remaining() < 8 {
            return Err(ProtocolError::new("CPF data too short"));
        }

        let interface_handle = buf.get_u32_le();
        let timeout = buf.get_u16_le();
        let item_count = buf.get_u16_le() as usize;
        let mut items = Vec::with_capacity(item_count);
        for _ in 0..item_count {
            if buf.remaining() < 4 {
                return Err(ProtocolError::new("Response truncated while parsing items"));
            }
            let type_id = buf.get_u16_le();
            let item_length = buf.get_u16_le() as usize;
            if buf.remaining() < item_length {
                return Err(ProtocolError::new("Data item truncated"));
            }
            items.push(CpfItem {
                type_id,
                data: buf.copy_to_bytes(item_length).to_vec(),
            });
        }

        Ok(Self {
            interface_handle,
            timeout,
            items,
        })
    }
}
