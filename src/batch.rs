use crate::PlcValue;

// =========================================================================
// BATCH OPERATIONS DATA STRUCTURES
// =========================================================================

/// Represents a single operation in a batch request
///
/// This enum defines the different types of operations that can be
/// performed in a batch. Each operation specifies whether it's a read
/// or write operation and includes the necessary parameters.
#[derive(Debug, Clone)]
pub enum BatchOperation {
    /// Read operation for a specific tag
    ///
    /// # Fields
    ///
    /// * `tag_name` - The name of the tag to read
    Read { tag_name: String },

    /// Write operation for a specific tag with a value
    ///
    /// # Fields
    ///
    /// * `tag_name` - The name of the tag to write
    /// * `value` - The value to write to the tag
    Write { tag_name: String, value: PlcValue },
}

/// Result of a single operation in a batch request
///
/// This structure contains the result of executing a single batch operation,
/// including success/failure status and the actual data or error information.
#[derive(Debug, Clone)]
pub struct BatchResult {
    /// The original operation that was executed
    pub operation: BatchOperation,

    /// The result of the operation
    pub result: std::result::Result<Option<PlcValue>, BatchError>,

    /// Execution time for this specific operation (in microseconds)
    pub execution_time_us: u64,
}

/// Specific error types that can occur during batch operations
///
/// This enum provides detailed error information for batch operations,
/// allowing for better error handling and diagnostics.
#[derive(Debug, Clone, thiserror::Error)]
pub enum BatchError {
    /// Tag was not found in the PLC
    #[error("Tag not found: {0}")]
    TagNotFound(String),

    /// Data type mismatch between expected and actual
    #[error("Data type mismatch: expected {expected}, got {actual}")]
    DataTypeMismatch { expected: String, actual: String },

    /// Network communication error
    #[error("Network error: {0}")]
    NetworkError(String),

    /// CIP protocol error with status code
    #[error("CIP error (0x{status:02X}): {message}")]
    CipError { status: u8, message: String },

    /// Tag name parsing error
    #[error("Tag path error: {0}")]
    TagPathError(String),

    /// Value serialization/deserialization error
    #[error("Serialization error: {0}")]
    SerializationError(String),

    /// Operation timeout
    #[error("Operation timeout")]
    Timeout,

    /// Generic error for unexpected issues
    #[error("Error: {0}")]
    Other(String),
}

/// Configuration for batch operations
///
/// This structure controls the behavior and performance characteristics
/// of batch read/write operations. Proper tuning can significantly
/// improve throughput for applications that need to process many tags.
#[derive(Debug, Clone)]
pub struct BatchConfig {
    /// Maximum number of operations to include in a single CIP packet
    ///
    /// Larger values improve performance but may exceed PLC packet size limits.
    /// Typical range: 10-50 operations per packet.
    pub max_operations_per_packet: usize,

    /// Maximum packet size in bytes for batch operations
    ///
    /// Should not exceed the PLC's maximum packet size capability.
    /// Typical values: 504 bytes (default), up to 4000 bytes for modern PLCs.
    pub max_packet_size: usize,

    /// Timeout for individual batch packets (in milliseconds)
    ///
    /// This is per-packet timeout, not per-operation.
    /// Typical range: 1000-5000 milliseconds.
    pub packet_timeout_ms: u64,

    /// Whether to continue processing other operations if one fails
    ///
    /// If true, failed operations are reported but don't stop the batch.
    /// If false, the first error stops the entire batch processing.
    pub continue_on_error: bool,

    /// Whether to optimize packet packing by grouping similar operations
    ///
    /// If true, reads and writes are grouped separately for better performance.
    /// If false, operations are processed in the order provided.
    pub optimize_packet_packing: bool,
}

impl Default for BatchConfig {
    fn default() -> Self {
        Self {
            max_operations_per_packet: 20,
            max_packet_size: 504, // Conservative default for maximum compatibility
            packet_timeout_ms: 3000,
            continue_on_error: true,
            optimize_packet_packing: true,
        }
    }
}
