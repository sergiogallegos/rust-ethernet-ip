// use std::error::Error;
use std::io;
use std::time::Duration;
use thiserror::Error;

/// Result type alias for EtherNet/IP operations
pub type Result<T> = std::result::Result<T, EtherNetIpError>;

/// Error types that can occur during EtherNet/IP communication
#[derive(Debug, Error)]
pub enum EtherNetIpError {
    /// IO error (network issues, connection problems)
    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    /// Protocol error (invalid packet format, unsupported features)
    #[error("Protocol error: {0}")]
    Protocol(String),

    /// Tag not found in PLC
    #[error("Tag not found: {0}")]
    TagNotFound(String),

    /// Data type mismatch
    #[error("Data type mismatch: expected {expected}, got {actual}")]
    DataTypeMismatch { expected: String, actual: String },

    /// Write error with status code
    #[error("Write error: {message} (status: {status})")]
    WriteError { status: u8, message: String },

    /// Read error with status code
    #[error("Read error: {message} (status: {status})")]
    ReadError { status: u8, message: String },

    /// Invalid response from PLC
    #[error("Invalid response: {reason}")]
    InvalidResponse { reason: String },

    /// Timeout error
    #[error("Operation timed out after {0:?}")]
    Timeout(Duration),

    /// UDT error
    #[error("UDT error: {0}")]
    Udt(String),

    /// Connection error (PLC not responding, session issues)
    #[error("Connection error: {0}")]
    Connection(String),

    /// Connection lost (network closed, PLC unreachable)
    #[error("Connection lost: {0}")]
    ConnectionLost(String),

    /// CIP protocol error with status code (from PLC)
    #[error("CIP error 0x{code:02X}: {message}")]
    CipError { code: u8, message: String },

    /// String is too long for the PLC's string type
    #[error("String too long: max length is {max_length}, but got {actual_length}")]
    StringTooLong {
        max_length: usize,
        actual_length: usize,
    },

    /// String contains invalid characters
    #[error("Invalid string: {reason}")]
    InvalidString { reason: String },

    /// String write operation failed
    #[error("String write failed: {message} (status: {status})")]
    StringWriteError { status: u8, message: String },

    /// String read operation failed
    #[error("String read failed: {message} (status: {status})")]
    StringReadError { status: u8, message: String },

    /// Invalid string response from PLC
    #[error("Invalid string response: {reason}")]
    InvalidStringResponse { reason: String },

    /// Tag error
    #[error("Tag error: {0}")]
    Tag(String),

    /// Permission denied
    #[error("Permission denied: {0}")]
    Permission(String),

    /// UTF-8 error
    #[error("UTF-8 error: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),

    /// Other error
    #[error("Other error: {0}")]
    Other(String),

    /// Subscription error
    #[error("Subscription error: {0}")]
    Subscription(String),
}

impl EtherNetIpError {
    /// Returns true if the error is likely retriable (e.g. timeout, connection lost).
    /// Use this to decide whether to retry an operation or reconnect.
    pub fn is_retriable(&self) -> bool {
        matches!(
            self,
            EtherNetIpError::Timeout(_)
                | EtherNetIpError::Connection(_)
                | EtherNetIpError::ConnectionLost(_)
                | EtherNetIpError::Io(_)
        )
    }
}
