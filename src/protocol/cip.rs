use bytes::{Buf, BufMut, BytesMut};

use crate::error::{EtherNetIpError, Result};

use super::{Decode, Encode};

pub(crate) const READ_TAG: u8 = 0x4C;
pub(crate) const WRITE_TAG: u8 = 0x4D;
#[allow(dead_code)]
pub(crate) const MULTIPLE_SERVICE_PACKET: u8 = 0x0A;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CipRequest {
    pub service: u8,
    pub path: Vec<u8>,
    pub data: Vec<u8>,
}

impl CipRequest {
    pub(crate) fn new(service: u8, path: Vec<u8>, data: Vec<u8>) -> Self {
        Self {
            service,
            path,
            data,
        }
    }
}

impl Encode for CipRequest {
    fn encode(&self, buf: &mut BytesMut) {
        buf.put_u8(self.service);
        buf.put_u8((self.path.len() / 2) as u8);
        buf.put_slice(&self.path);
        buf.put_slice(&self.data);
    }
}

impl Decode for CipRequest {
    fn decode(buf: &mut impl Buf) -> Result<Self> {
        if buf.remaining() < 2 {
            return Err(EtherNetIpError::Protocol(
                "CIP request too short".to_string(),
            ));
        }

        let service = buf.get_u8();
        let path_size_words = buf.get_u8() as usize;
        let path_len = path_size_words * 2;
        if buf.remaining() < path_len {
            return Err(EtherNetIpError::Protocol(
                "CIP request path truncated".to_string(),
            ));
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CipResponse {
    pub service: u8,
    pub status: u8,
    pub additional_status: Vec<u16>,
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
            return Err(EtherNetIpError::Protocol(
                "CIP response too short".to_string(),
            ));
        }

        let service = buf.get_u8();
        let _reserved = buf.get_u8();
        let status = buf.get_u8();
        let additional_status_size = buf.get_u8() as usize;
        if buf.remaining() < additional_status_size * 2 {
            return Err(EtherNetIpError::Protocol(
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CpfItem {
    pub type_id: u16,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SendDataRequest {
    pub interface_handle: u32,
    pub timeout: u16,
    pub items: Vec<CpfItem>,
}

impl SendDataRequest {
    pub(crate) fn unconnected(item_data: &[u8]) -> Self {
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
            return Err(EtherNetIpError::Protocol("CPF data too short".to_string()));
        }

        let interface_handle = buf.get_u32_le();
        let timeout = buf.get_u16_le();
        let item_count = buf.get_u16_le() as usize;
        let mut items = Vec::with_capacity(item_count);
        for _ in 0..item_count {
            if buf.remaining() < 4 {
                return Err(EtherNetIpError::Protocol(
                    "Response truncated while parsing items".to_string(),
                ));
            }
            let type_id = buf.get_u16_le();
            let item_length = buf.get_u16_le() as usize;
            if buf.remaining() < item_length {
                return Err(EtherNetIpError::Protocol("Data item truncated".to_string()));
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
