use bytes::{Buf, BytesMut};

pub mod cip;
pub mod encap;
pub mod values;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProtocolError {
    message: String,
}

impl ProtocolError {
    #[must_use]
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl std::fmt::Display for ProtocolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for ProtocolError {}

pub type Result<T> = std::result::Result<T, ProtocolError>;

pub trait Encode {
    fn encode(&self, buf: &mut BytesMut);
}

pub trait Decode: Sized {
    fn decode(buf: &mut impl Buf) -> Result<Self>;
}

#[cfg(test)]
mod tests;
