// use std::error::Error;
use std::io;
use std::time::Duration;
use thiserror::Error;

/// Result type alias for EtherNet/IP operations
pub type Result<T> = std::result::Result<T, EtherNetIpError>;

/// Error types that can occur during EtherNet/IP communication
#[derive(Debug, Error)]
#[non_exhaustive]
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

    /// Unsupported API surface retained only for 1.x compatibility.
    #[error("Unsupported API `{api}`: {reason}")]
    Unsupported {
        /// API name that is no longer supported.
        api: &'static str,
        /// Reason the API is unsupported and the replacement to use.
        reason: &'static str,
    },

    /// The controller acknowledged a Write Tag Service request with a success
    /// status, but a read-back of the same tag shows the value was never
    /// actually applied. Observed against an Unconnected-Send-routed write on a
    /// FactoryTalk Logix Echo-emulated ControlLogix 5580 (firmware 36); not yet
    /// confirmed on physical hardware. See `write_tag_verified`.
    #[error(
        "Write to '{tag_name}' was acknowledged by the controller but not applied: expected {expected}, read back {actual}"
    )]
    WriteNotApplied {
        /// Tag path that was written.
        tag_name: String,
        /// Debug-formatted value that was written.
        expected: String,
        /// Debug-formatted value read back immediately after the write.
        actual: String,
    },
}

impl EtherNetIpError {
    /// Returns true if the error is likely retriable (e.g. timeout, connection lost).
    /// Use this to decide whether to retry an operation or reconnect.
    #[must_use]
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

impl<T> From<std::sync::PoisonError<T>> for EtherNetIpError {
    fn from(_: std::sync::PoisonError<T>) -> Self {
        EtherNetIpError::Other("lock poisoned".to_string())
    }
}

impl From<rust_ethernet_ip_tag_path::TagPathError> for EtherNetIpError {
    fn from(error: rust_ethernet_ip_tag_path::TagPathError) -> Self {
        EtherNetIpError::Protocol(error.to_string())
    }
}

impl From<rust_ethernet_ip_protocol::ProtocolError> for EtherNetIpError {
    fn from(error: rust_ethernet_ip_protocol::ProtocolError) -> Self {
        EtherNetIpError::Protocol(error.to_string())
    }
}

impl From<rust_ethernet_ip_types::TypeError> for EtherNetIpError {
    fn from(error: rust_ethernet_ip_types::TypeError) -> Self {
        EtherNetIpError::Protocol(error.to_string())
    }
}

impl From<rust_ethernet_ip_udt::UdtError> for EtherNetIpError {
    fn from(error: rust_ethernet_ip_udt::UdtError) -> Self {
        match error {
            rust_ethernet_ip_udt::UdtError::Protocol(message) => EtherNetIpError::Protocol(message),
            rust_ethernet_ip_udt::UdtError::TagNotFound(tag) => EtherNetIpError::TagNotFound(tag),
            rust_ethernet_ip_udt::UdtError::DataTypeMismatch { expected, actual } => {
                EtherNetIpError::DataTypeMismatch { expected, actual }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::assert_matches;
    use std::sync::Mutex;

    fn convert_poison_error() -> Result<()> {
        let lock = Mutex::new(());
        std::thread::scope(|scope| {
            let handle = scope.spawn(|| {
                let _guard = lock.lock().expect("test lock should not be poisoned yet");
                panic!("poison test mutex");
            });
            assert!(handle.join().is_err());
        });

        let _guard = lock.lock()?;
        Ok(())
    }

    #[test]
    fn poison_error_converts_to_other_variant() {
        let err = convert_poison_error().expect_err("poisoned mutex should convert into an error");
        assert_matches!(err, EtherNetIpError::Other(message) if message == "lock poisoned");
    }

    #[test]
    fn unsupported_error_names_api_and_reason() {
        let err = EtherNetIpError::Unsupported {
            api: "old_api",
            reason: "use new_api",
        };

        assert_eq!(err.to_string(), "Unsupported API `old_api`: use new_api");
    }
}
