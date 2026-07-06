use bytes::{Buf, BufMut, BytesMut};

use crate::{Decode, Encode, ProtocolError, Result};

pub const REGISTER_SESSION: u16 = 0x0065;
pub const UNREGISTER_SESSION: u16 = 0x0066;
pub const SEND_RR_DATA: u16 = 0x006F;
#[allow(dead_code)]
pub const SEND_UNIT_DATA: u16 = 0x0070;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EncapsulationHeader {
    pub command: u16,
    pub length: u16,
    pub session_handle: u32,
    pub status: u32,
    pub sender_context: [u8; 8],
    pub options: u32,
}

impl EncapsulationHeader {
    pub fn new(command: u16, length: u16, session_handle: u32) -> Self {
        Self {
            command,
            length,
            session_handle,
            status: 0,
            sender_context: [0; 8],
            options: 0,
        }
    }

    pub fn send_rr_data(length: u16, session_handle: u32) -> Self {
        Self {
            sender_context: [0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08],
            ..Self::new(SEND_RR_DATA, length, session_handle)
        }
    }

    pub fn send_rr_data_with_context(
        length: u16,
        session_handle: u32,
        sender_context: [u8; 8],
    ) -> Self {
        Self {
            sender_context,
            ..Self::new(SEND_RR_DATA, length, session_handle)
        }
    }
}

impl Encode for EncapsulationHeader {
    fn encode(&self, buf: &mut BytesMut) {
        buf.put_u16_le(self.command);
        buf.put_u16_le(self.length);
        buf.put_u32_le(self.session_handle);
        buf.put_u32_le(self.status);
        buf.put_slice(&self.sender_context);
        buf.put_u32_le(self.options);
    }
}

impl Decode for EncapsulationHeader {
    fn decode(buf: &mut impl Buf) -> Result<Self> {
        if buf.remaining() < 24 {
            return Err(ProtocolError::new(
                "Encapsulation header too short".to_string(),
            ));
        }

        let command = buf.get_u16_le();
        let length = buf.get_u16_le();
        let session_handle = buf.get_u32_le();
        let status = buf.get_u32_le();
        let mut sender_context = [0u8; 8];
        buf.copy_to_slice(&mut sender_context);
        let options = buf.get_u32_le();

        Ok(Self {
            command,
            length,
            session_handle,
            status,
            sender_context,
            options,
        })
    }
}
