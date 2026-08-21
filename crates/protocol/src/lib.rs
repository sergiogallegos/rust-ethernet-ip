//! EtherNet/IP encapsulation and CIP wire codecs.

use bytes::{Buf, BytesMut};

/// Common Industrial Protocol request and response codecs.
pub mod cip;
/// EtherNet/IP encapsulation packet codecs.
pub mod encap;
/// Logix value type codes and payload codecs.
pub mod values;

/// Error returned when protocol data is invalid or unsupported.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProtocolError {
    message: String,
}

impl ProtocolError {
    /// Creates a protocol error with a human-readable explanation.
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

/// Result type used by protocol codecs.
pub type Result<T> = std::result::Result<T, ProtocolError>;

/// Encodes a value into an existing byte buffer.
pub trait Encode {
    /// Appends this value's wire representation to `buf`.
    fn encode(&self, buf: &mut BytesMut);
}

/// Decodes a value from a byte buffer.
pub trait Decode: Sized {
    /// Consumes and decodes one value from `buf`.
    fn decode(buf: &mut impl Buf) -> Result<Self>;
}

#[cfg(test)]
mod tests;
