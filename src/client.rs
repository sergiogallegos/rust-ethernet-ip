use crate::EtherNetIpStream;
use crate::batch::{BatchConfig, BatchOperation};
use crate::error::{EtherNetIpError, Result};
use crate::protocol::cip::{
    CipRequest, CipResponse, MULTIPLE_SERVICE_PACKET, READ_TAG, SendDataRequest, WRITE_TAG,
};
use crate::protocol::encap::{EncapsulationHeader, REGISTER_SESSION, UNREGISTER_SESSION};
use crate::protocol::values;
use crate::protocol::{Decode, Encode};
use crate::route::RoutePath;
use crate::subscription::TagSubscription;
use crate::tag_group::TagGroupConfig;
use crate::tag_manager::{TagManager, TagMetadata, TagPermissions, TagScope};
use crate::types::{PlcValue, UdtData};
use crate::udt::{TagAttributes, UdtDefinition, UdtManager};
use crate::{TagPath, udt};
use bytes::BytesMut;
use std::collections::HashMap;
use std::net::SocketAddr;
#[cfg(feature = "ffi")]
use std::sync::LazyLock;
use std::sync::atomic::{AtomicBool, AtomicU8, AtomicU32, AtomicU64, Ordering};
use std::sync::{Arc, Mutex as StdMutex};
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
#[cfg(feature = "ffi")]
use tokio::runtime::Runtime;
use tokio::sync::Mutex;
use tokio::time::{Duration, Instant, timeout};

mod actor;
mod batch_exec;
mod diagnostics;
mod schema_export;
mod service_layer;
mod string;
mod subscriptions;

pub use actor::{Backoff, Client, ConnectionEvent, RetryClient, RetryPolicy};

const READ_TAG_FRAGMENTED: u8 = 0x52;
const WRITE_TAG_FRAGMENTED: u8 = 0x53;
const READ_TAG_FRAGMENTED_REPLY: u8 = 0xD2;
const WRITE_TAG_FRAGMENTED_REPLY: u8 = 0xD3;
const CIP_STATUS_SUCCESS: u8 = 0x00;
const CIP_STATUS_PARTIAL_TRANSFER: u8 = 0x06;

#[derive(Debug)]
struct TagListPage {
    tags: Vec<TagAttributes>,
    last_instance_id: Option<u32>,
    partial_transfer: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct TemplateAttributes {
    structure_handle: u16,
    member_count: u16,
    definition_size_words: u32,
    structure_size_bytes: u32,
}

#[derive(Debug, Clone, Copy)]
enum DiagnosticOperation {
    Read,
    Write,
    Batch,
}

#[derive(Debug, Default)]
struct DiagnosticCounters {
    total_reads: AtomicU64,
    total_writes: AtomicU64,
    successful_reads: AtomicU64,
    successful_writes: AtomicU64,
    failed_reads: AtomicU64,
    failed_writes: AtomicU64,
    batch_operations: AtomicU64,
    partial_batch_failures: AtomicU64,
    network_errors: AtomicU64,
    protocol_errors: AtomicU64,
    timeout_errors: AtomicU64,
    tag_not_found_errors: AtomicU64,
    data_type_errors: AtomicU64,
    session_errors: AtomicU64,
    route_path_errors: AtomicU64,
    embedded_service_errors: AtomicU64,
    known_controller_limitation_errors: AtomicU64,
    retriable_errors: AtomicU64,
    non_retriable_errors: AtomicU64,
    last_successful_read_time: AtomicU64,
    last_failed_read_time: AtomicU64,
    last_successful_write_time: AtomicU64,
    last_failed_write_time: AtomicU64,
    last_error_time: AtomicU64,
    last_error_category: AtomicU8,
}

impl DiagnosticCounters {
    fn record_success(&self, operation: Option<DiagnosticOperation>) {
        let now = current_unix_seconds();
        match operation {
            Some(DiagnosticOperation::Read) => {
                self.total_reads.fetch_add(1, Ordering::Relaxed);
                self.successful_reads.fetch_add(1, Ordering::Relaxed);
                self.last_successful_read_time.store(now, Ordering::Relaxed);
            }
            Some(DiagnosticOperation::Write) => {
                self.total_writes.fetch_add(1, Ordering::Relaxed);
                self.successful_writes.fetch_add(1, Ordering::Relaxed);
                self.last_successful_write_time
                    .store(now, Ordering::Relaxed);
            }
            Some(DiagnosticOperation::Batch) => {
                self.batch_operations.fetch_add(1, Ordering::Relaxed);
            }
            None => {}
        }
    }

    fn record_cip_failure(&self, operation: Option<DiagnosticOperation>) {
        let category = crate::ErrorCategory::CipProtocol;
        self.record_operation_failure(operation, current_unix_seconds());
        self.protocol_errors.fetch_add(1, Ordering::Relaxed);
        self.non_retriable_errors.fetch_add(1, Ordering::Relaxed);
        self.store_last_error(category);
    }

    fn record_failure(&self, operation: Option<DiagnosticOperation>, error: &EtherNetIpError) {
        let now = current_unix_seconds();
        let category = diagnostic_error_category(error);
        self.record_operation_failure(operation, now);

        match category {
            crate::ErrorCategory::Network => self.network_errors.fetch_add(1, Ordering::Relaxed),
            crate::ErrorCategory::Timeout => self.timeout_errors.fetch_add(1, Ordering::Relaxed),
            crate::ErrorCategory::Session => self.session_errors.fetch_add(1, Ordering::Relaxed),
            crate::ErrorCategory::RoutePath => {
                self.route_path_errors.fetch_add(1, Ordering::Relaxed)
            }
            crate::ErrorCategory::CipProtocol => {
                self.protocol_errors.fetch_add(1, Ordering::Relaxed)
            }
            crate::ErrorCategory::BatchEmbeddedService => {
                self.embedded_service_errors.fetch_add(1, Ordering::Relaxed)
            }
            crate::ErrorCategory::KnownControllerLimitation => self
                .known_controller_limitation_errors
                .fetch_add(1, Ordering::Relaxed),
            crate::ErrorCategory::DataType => self.data_type_errors.fetch_add(1, Ordering::Relaxed),
            crate::ErrorCategory::NotFound => {
                self.tag_not_found_errors.fetch_add(1, Ordering::Relaxed)
            }
            crate::ErrorCategory::Unknown => self.protocol_errors.fetch_add(1, Ordering::Relaxed),
        };

        if error.is_retriable() || category.is_retriable() {
            self.retriable_errors.fetch_add(1, Ordering::Relaxed);
        } else {
            self.non_retriable_errors.fetch_add(1, Ordering::Relaxed);
        }
        self.store_last_error(category);
    }

    fn record_operation_failure(&self, operation: Option<DiagnosticOperation>, now: u64) {
        match operation {
            Some(DiagnosticOperation::Read) => {
                self.total_reads.fetch_add(1, Ordering::Relaxed);
                self.failed_reads.fetch_add(1, Ordering::Relaxed);
                self.last_failed_read_time.store(now, Ordering::Relaxed);
            }
            Some(DiagnosticOperation::Write) => {
                self.total_writes.fetch_add(1, Ordering::Relaxed);
                self.failed_writes.fetch_add(1, Ordering::Relaxed);
                self.last_failed_write_time.store(now, Ordering::Relaxed);
            }
            Some(DiagnosticOperation::Batch) => {
                self.batch_operations.fetch_add(1, Ordering::Relaxed);
                self.partial_batch_failures.fetch_add(1, Ordering::Relaxed);
            }
            None => {}
        }
    }

    fn store_last_error(&self, category: crate::ErrorCategory) {
        self.last_error_time
            .store(current_unix_seconds(), Ordering::Relaxed);
        self.last_error_category
            .store(error_category_to_code(category), Ordering::Relaxed);
    }

    fn operation_metrics(&self) -> crate::OperationMetrics {
        crate::OperationMetrics {
            total_reads: self.total_reads.load(Ordering::Relaxed),
            total_writes: self.total_writes.load(Ordering::Relaxed),
            successful_reads: self.successful_reads.load(Ordering::Relaxed),
            successful_writes: self.successful_writes.load(Ordering::Relaxed),
            failed_reads: self.failed_reads.load(Ordering::Relaxed),
            failed_writes: self.failed_writes.load(Ordering::Relaxed),
            batch_operations: self.batch_operations.load(Ordering::Relaxed),
            subscription_updates: 0,
            partial_batch_failures: self.partial_batch_failures.load(Ordering::Relaxed),
            last_successful_read_time: unix_seconds_to_system_time(
                self.last_successful_read_time.load(Ordering::Relaxed),
            ),
            last_failed_read_time: unix_seconds_to_system_time(
                self.last_failed_read_time.load(Ordering::Relaxed),
            ),
            last_successful_write_time: unix_seconds_to_system_time(
                self.last_successful_write_time.load(Ordering::Relaxed),
            ),
            last_failed_write_time: unix_seconds_to_system_time(
                self.last_failed_write_time.load(Ordering::Relaxed),
            ),
        }
    }

    fn error_metrics(&self) -> crate::ErrorMetrics {
        let last_error_category =
            error_category_from_code(self.last_error_category.load(Ordering::Relaxed));
        crate::ErrorMetrics {
            network_errors: self.network_errors.load(Ordering::Relaxed),
            protocol_errors: self.protocol_errors.load(Ordering::Relaxed),
            timeout_errors: self.timeout_errors.load(Ordering::Relaxed),
            tag_not_found_errors: self.tag_not_found_errors.load(Ordering::Relaxed),
            data_type_errors: self.data_type_errors.load(Ordering::Relaxed),
            session_errors: self.session_errors.load(Ordering::Relaxed),
            route_path_errors: self.route_path_errors.load(Ordering::Relaxed),
            embedded_service_errors: self.embedded_service_errors.load(Ordering::Relaxed),
            known_controller_limitation_errors: self
                .known_controller_limitation_errors
                .load(Ordering::Relaxed),
            retriable_errors: self.retriable_errors.load(Ordering::Relaxed),
            non_retriable_errors: self.non_retriable_errors.load(Ordering::Relaxed),
            last_error_time: unix_seconds_to_system_time(
                self.last_error_time.load(Ordering::Relaxed),
            ),
            last_error_message: last_error_category.map(|category| {
                format!("Most recent counted client operation failed: {category:?}")
            }),
            last_error_category,
            last_retriable_error_time: if last_error_category.is_some_and(|c| c.is_retriable()) {
                unix_seconds_to_system_time(self.last_error_time.load(Ordering::Relaxed))
            } else {
                None
            },
        }
    }
}

fn current_unix_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn unix_seconds_to_system_time(seconds: u64) -> Option<SystemTime> {
    (seconds != 0).then(|| UNIX_EPOCH + Duration::from_secs(seconds))
}

fn error_category_to_code(category: crate::ErrorCategory) -> u8 {
    match category {
        crate::ErrorCategory::Network => 1,
        crate::ErrorCategory::Timeout => 2,
        crate::ErrorCategory::Session => 3,
        crate::ErrorCategory::RoutePath => 4,
        crate::ErrorCategory::CipProtocol => 5,
        crate::ErrorCategory::BatchEmbeddedService => 6,
        crate::ErrorCategory::KnownControllerLimitation => 7,
        crate::ErrorCategory::DataType => 8,
        crate::ErrorCategory::NotFound => 9,
        crate::ErrorCategory::Unknown => 10,
    }
}

fn error_category_from_code(code: u8) -> Option<crate::ErrorCategory> {
    match code {
        1 => Some(crate::ErrorCategory::Network),
        2 => Some(crate::ErrorCategory::Timeout),
        3 => Some(crate::ErrorCategory::Session),
        4 => Some(crate::ErrorCategory::RoutePath),
        5 => Some(crate::ErrorCategory::CipProtocol),
        6 => Some(crate::ErrorCategory::BatchEmbeddedService),
        7 => Some(crate::ErrorCategory::KnownControllerLimitation),
        8 => Some(crate::ErrorCategory::DataType),
        9 => Some(crate::ErrorCategory::NotFound),
        10 => Some(crate::ErrorCategory::Unknown),
        _ => None,
    }
}

fn diagnostic_error_category(error: &EtherNetIpError) -> crate::ErrorCategory {
    match error {
        EtherNetIpError::Io(_) | EtherNetIpError::ConnectionLost(_) => {
            crate::ErrorCategory::Network
        }
        EtherNetIpError::Timeout(_) => crate::ErrorCategory::Timeout,
        EtherNetIpError::Connection(_) => crate::ErrorCategory::Session,
        EtherNetIpError::TagNotFound(_) => crate::ErrorCategory::NotFound,
        EtherNetIpError::DataTypeMismatch { .. }
        | EtherNetIpError::StringTooLong { .. }
        | EtherNetIpError::InvalidString { .. } => crate::ErrorCategory::DataType,
        EtherNetIpError::ReadError { .. }
        | EtherNetIpError::WriteError { .. }
        | EtherNetIpError::CipError { .. } => crate::ErrorCategory::CipProtocol,
        EtherNetIpError::Protocol(message)
            if message.contains("route") || message.contains("Route") =>
        {
            crate::ErrorCategory::RoutePath
        }
        EtherNetIpError::Protocol(message)
            if message.contains("Embedded service") || message.contains("Multiple Service") =>
        {
            crate::ErrorCategory::BatchEmbeddedService
        }
        EtherNetIpError::Protocol(_) | EtherNetIpError::InvalidResponse { .. } => {
            crate::ErrorCategory::CipProtocol
        }
        EtherNetIpError::Udt(_)
        | EtherNetIpError::Tag(_)
        | EtherNetIpError::Permission(_)
        | EtherNetIpError::Utf8(_)
        | EtherNetIpError::Other(_)
        | EtherNetIpError::Subscription(_)
        | EtherNetIpError::Unsupported { .. } => crate::ErrorCategory::Unknown,
    }
}

/// Global Tokio runtime for handling async operations in FFI context
#[cfg(feature = "ffi")]
pub(crate) static RUNTIME: LazyLock<std::io::Result<Runtime>> = LazyLock::new(Runtime::new);

/// High-performance EtherNet/IP client for PLC communication
///
/// This struct provides the core functionality for communicating with Allen-Bradley
/// PLCs using the EtherNet/IP protocol. It handles connection management, session
/// registration, and tag operations.
///
/// # Thread Safety
///
/// The `EipClient` is **NOT** thread-safe. For multi-threaded applications:
///
/// ```rust,no_run
/// use std::sync::Arc;
/// use tokio::sync::Mutex;
/// use rust_ethernet_ip::EipClient;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
///     // Create a thread-safe wrapper
///     let client = Arc::new(Mutex::new(EipClient::connect("192.168.1.100:44818").await?));
///
///     // Use in multiple threads
///     let client_clone = client.clone();
///     tokio::spawn(async move {
///         let mut client = client_clone.lock().await;
///         let _ = client.read_tag("Tag1").await?;
///         Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
///     });
///     Ok(())
/// }
/// ```
///
/// # Performance Characteristics
///
/// | Operation | Latency | Throughput | Memory |
/// |-----------|---------|------------|---------|
/// | Connect | 100-500ms | N/A | ~8KB |
/// | Read Tag | 1-5ms | 1,500+ ops/sec | ~2KB |
/// | Write Tag | 2-10ms | 600+ ops/sec | ~2KB |
/// | Batch Read | 5-20ms | 2,000+ ops/sec | ~4KB |
///
/// # Known Limitations
///
/// The following nested write paths have controller-specific caveats:
///
/// ## UDT Array Element Member Writes
///
/// Historical tests classified direct writes to UDT array element members
/// (e.g., `gTestUDT_Array[0].Member1_DINT`) as firmware-blocked `0x2107`
/// cases. CODEX-AM and CODEX-AV disproved the blanket rule on a
/// 5069-L330ERM fw38: scalar DINT/REAL/BOOL/INT member writes succeeded with
/// corrected paths, verified read-back, preserved sibling members, and restored
/// cleanly.
///
/// ## STRING Tags and STRING Members in UDTs
///
/// Standalone Logix `STRING` tags can be written directly with the standard structure
/// encoding. Direct writes to `STRING` members within UDTs (e.g.,
/// `gTestUDT.Member5_String`) are still rejected with `0xFF/0x2107` under the
/// current member encoding on 5069-L330ERM fw38. Whether a member-specific direct
/// encoding exists remains under CODEX-AO investigation. For those nested members,
/// write the entire UDT structure instead of writing the member path directly.
///
/// **What works:**
/// - ✅ Reading UDT array element members: `gTestUDT_Array[0].Member1_DINT` (read)
/// - ✅ Writing entire UDT array elements: `gTestUDT_Array[0]` (write full UDT)
/// - ✅ Writing UDT members (non-STRING): `gTestUDT.Member1_DINT` (write DINT/REAL/BOOL/INT members)
/// - ✅ Writing scalar UDT array element members on 5069-L330ERM fw38 with corrected paths
/// - ✅ Writing array elements: `gArray[5]` (write element of simple array)
/// - ✅ Reading STRING tags: `gTest_STRING` (read)
/// - ✅ Writing STRING tags: `gTest_STRING` (write standard top-level STRING)
/// - ✅ Reading STRING members in UDTs: `gTestUDT.Member5_String` (read)
///
/// **What doesn't work:**
/// - ❌ Writing STRING members in UDTs: `gTestUDT.Member5_String` (write) - must write entire UDT
/// - ❌ Writing program-scoped STRING members: `Program:TestProgram.gTestUDT.Member5_String` (write) - must write entire UDT
/// - ❌ Writing UDT array element STRING members under the current member encoding - must write entire UDT
///
/// **Conservative workaround:**
/// For rejected nested STRING-member writes, read the entire UDT array
/// element, modify the member in memory, then write the entire UDT array
/// element back:
///
/// ```rust,no_run
/// # async fn example() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
/// # let mut client = rust_ethernet_ip::EipClient::connect("192.168.1.100:44818").await?;
/// use rust_ethernet_ip::{PlcValue, UdtData};
///
/// // Read the entire UDT array element
/// let udt_value = client.read_tag("gTestUDT_Array[0]").await?;
/// if let PlcValue::Udt(mut udt_data) = udt_value {
///     let udt_def = client.get_udt_definition("gTestUDT_Array").await?;
///     // Convert UdtDefinition to UserDefinedType
///     let mut user_def = rust_ethernet_ip::udt::UserDefinedType::new(udt_def.name.clone());
///     for member in &udt_def.members {
///         user_def.add_member(member.clone());
///     }
///     let mut members = udt_data.parse(&user_def)?;
///     
///     // Modify the member
///     members.insert("Member1_DINT".to_string(), PlcValue::Dint(100));
///     
///     // Write the entire UDT array element back
///     let modified_udt = UdtData::from_hash_map(&members, &user_def, udt_data.symbol_id)?;
///     client.write_tag("gTestUDT_Array[0]", PlcValue::Udt(modified_udt)).await?;
/// }
/// # Ok(())
/// # }
/// ```
///
/// # Error Handling
///
/// All operations return `Result<T, EtherNetIpError>`. Common errors include:
///
/// ```rust,no_run
/// use rust_ethernet_ip::{EipClient, EtherNetIpError};
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
///     let mut client = EipClient::connect("192.168.1.100:44818").await?;
///     match client.read_tag("Tag1").await {
///         Ok(value) => println!("Tag value: {:?}", value),
///         Err(EtherNetIpError::Protocol(_)) => println!("Tag does not exist"),
///         Err(EtherNetIpError::Connection(_)) => println!("Lost connection to PLC"),
///         Err(EtherNetIpError::Timeout(_)) => println!("Operation timed out"),
///         Err(e) => println!("Other error: {}", e),
///     }
///     Ok(())
/// }
/// ```
///
/// # Examples
///
/// Basic usage:
/// ```rust,no_run
/// use rust_ethernet_ip::{EipClient, PlcValue};
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
///     let mut client = EipClient::connect("192.168.1.100:44818").await?;
///
///     // Read a boolean tag
///     let motor_running = client.read_tag("MotorRunning").await?;
///
///     // Write an integer tag
///     client.write_tag("SetPoint", PlcValue::Dint(1500)).await?;
///
///     // Read multiple tags in sequence
///     let tag1 = client.read_tag("Tag1").await?;
///     let tag2 = client.read_tag("Tag2").await?;
///     let tag3 = client.read_tag("Tag3").await?;
///     Ok(())
/// }
/// ```
///
/// Advanced usage with error recovery:
/// ```rust
/// use rust_ethernet_ip::{EipClient, PlcValue, EtherNetIpError};
/// use tokio::time::Duration;
///
/// async fn read_with_retry(client: &mut EipClient, tag: &str, retries: u32) -> Result<PlcValue, EtherNetIpError> {
///     for attempt in 0..retries {
///         match client.read_tag(tag).await {
///             Ok(value) => return Ok(value),
///             Err(EtherNetIpError::Connection(_)) => {
///                 if attempt < retries - 1 {
///                     tokio::time::sleep(Duration::from_secs(1)).await;
///                     continue;
///                 }
///                 return Err(EtherNetIpError::Protocol("Max retries exceeded".to_string()));
///             }
///             Err(e) => return Err(e),
///         }
///     }
///     Err(EtherNetIpError::Protocol("Max retries exceeded".to_string()))
/// }
/// ```
#[derive(Clone)]
pub struct EipClient {
    /// SHARED ON CLONE: network communication state.
    stream: Arc<Mutex<Box<dyn EtherNetIpStream>>>,
    /// SHARED ON CLONE: one registered session handle belongs to the shared stream.
    session_handle: Arc<AtomicU32>,
    /// SHARED ON CLONE: fail-fast marker for a stream that may contain stale response bytes.
    stream_poisoned: Arc<AtomicBool>,
    /// SHARED ON CLONE: monotonic sender_context counter for SendRRData correlation.
    sender_context_counter: Arc<AtomicU64>,
    /// SHARED ON CLONE: operation/error counters surfaced by diagnostics snapshots.
    diagnostic_counters: Arc<DiagnosticCounters>,
    /// SHARED ON CLONE: tag discovery/cache state.
    tag_manager: Arc<Mutex<TagManager>>,
    /// SHARED ON CLONE: UDT discovery/cache state.
    udt_manager: Arc<Mutex<UdtManager>>,
    /// SHARED ON CLONE: base array paths already classified as packed BOOL or non-BOOL.
    array_type_cache: Arc<StdMutex<HashMap<String, bool>>>,
    /// SHARED ON CLONE: route-path mutations must be visible through later registry lookups.
    route_path: Arc<StdMutex<Option<RoutePath>>>,
    /// SHARED ON CLONE: max packet size is cheap scalar state and may be configured through FFI.
    max_packet_size: Arc<AtomicU32>,
    /// SHARED ON CLONE: last activity timestamp.
    last_activity: Arc<Mutex<Instant>>,
    /// COPIED ON CLONE: persistent FFI config would require Arc/RwLock; current use is per-call only.
    batch_config: BatchConfig,
    /// SHARED ON CLONE: active tag subscriptions.
    subscriptions: Arc<Mutex<Vec<TagSubscription>>>,
    /// SHARED ON CLONE: registered tag-group polling definitions.
    tag_groups: Arc<Mutex<HashMap<String, TagGroupConfig>>>,
}

#[cfg(test)]
const _: fn() = || {
    fn assert_send_sync_static<T: Send + Sync + 'static>() {}
    assert_send_sync_static::<EipClient>();
};

impl std::fmt::Debug for EipClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EipClient")
            .field("session_handle", &self.session_handle())
            .field("stream_poisoned", &self.stream_poisoned())
            .field("route_path", &self.route_path_snapshot())
            .field("max_packet_size", &self.max_packet_size())
            .field("batch_config", &self.batch_config)
            .field("stream", &"<stream>")
            .field("diagnostic_counters", &"<diagnostic_counters>")
            .field("tag_manager", &"<tag_manager>")
            .field("udt_manager", &"<udt_manager>")
            .field("array_type_cache", &"<array_type_cache>")
            .field("subscriptions", &"<subscriptions>")
            .field("tag_groups", &"<tag_groups>")
            .finish()
    }
}

/// Returns the bare program name behind a caller-supplied program identifier.
///
/// [`EipClient::discover_program_tags`] accepts both spellings of the same
/// program -- `"Dashboard"` and the wire form `"Program:Dashboard"` -- so the
/// prefix has to be stripped exactly once, in one place: the request path
/// re-adds it, and [`udt::TagScope::Program`] holds the BARE name everywhere
/// else in the crate (see `schema::SchemaScope`). Normalizing here keeps the
/// two spellings from producing two different scopes for one program.
fn program_scope_name(program_name: &str) -> &str {
    program_name
        .strip_prefix("Program:")
        .unwrap_or(program_name)
}

impl EipClient {
    /// Internal constructor that initializes an EipClient from any stream
    /// that implements AsyncRead + AsyncWrite + Unpin + Send
    async fn from_stream<S>(stream: S) -> Result<Self>
    where
        S: EtherNetIpStream + 'static,
    {
        let mut client = Self {
            stream: Arc::new(Mutex::new(Box::new(stream))),
            session_handle: Arc::new(AtomicU32::new(0)),
            stream_poisoned: Arc::new(AtomicBool::new(false)),
            sender_context_counter: Arc::new(AtomicU64::new(1)),
            diagnostic_counters: Arc::new(DiagnosticCounters::default()),
            tag_manager: Arc::new(Mutex::new(TagManager::new())),
            udt_manager: Arc::new(Mutex::new(UdtManager::new())),
            array_type_cache: Arc::new(StdMutex::new(HashMap::new())),
            route_path: Arc::new(StdMutex::new(None)),
            max_packet_size: Arc::new(AtomicU32::new(4000)),
            last_activity: Arc::new(Mutex::new(Instant::now())),
            batch_config: BatchConfig::default(),
            subscriptions: Arc::new(Mutex::new(Vec::new())),
            tag_groups: Arc::new(Mutex::new(HashMap::new())),
        };
        client.register_session().await?;
        client.negotiate_packet_size().await?;
        Ok(client)
    }

    /// Connects directly to a controller.
    ///
    /// This is an alias for [`Self::connect`].
    pub async fn new(addr: &str) -> Result<Self> {
        let addr = addr
            .parse::<SocketAddr>()
            .map_err(|e| EtherNetIpError::Protocol(format!("Invalid address format: {e}")))?;
        let stream = TcpStream::connect(addr).await?;
        Self::from_stream(stream).await
    }

    /// Public async connect function for `EipClient`
    pub async fn connect(addr: &str) -> Result<Self> {
        Self::new(addr).await
    }

    #[cfg(test)]
    fn new_unconnected_for_testing() -> Self {
        let (stream, _peer) = tokio::io::duplex(64);
        Self {
            stream: Arc::new(Mutex::new(Box::new(stream))),
            session_handle: Arc::new(AtomicU32::new(0)),
            stream_poisoned: Arc::new(AtomicBool::new(false)),
            sender_context_counter: Arc::new(AtomicU64::new(1)),
            diagnostic_counters: Arc::new(DiagnosticCounters::default()),
            tag_manager: Arc::new(Mutex::new(TagManager::new())),
            udt_manager: Arc::new(Mutex::new(UdtManager::new())),
            array_type_cache: Arc::new(StdMutex::new(HashMap::new())),
            route_path: Arc::new(StdMutex::new(None)),
            max_packet_size: Arc::new(AtomicU32::new(4000)),
            last_activity: Arc::new(Mutex::new(Instant::now())),
            batch_config: BatchConfig::default(),
            subscriptions: Arc::new(Mutex::new(Vec::new())),
            tag_groups: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Registers an EtherNet/IP session with the PLC
    ///
    /// This is an internal function that implements the EtherNet/IP session
    /// registration protocol. It sends a Register Session command and
    /// processes the response to extract the session handle.
    ///
    /// # Protocol Details
    ///
    /// The Register Session command consists of:
    /// - EtherNet/IP Encapsulation Header (24 bytes)
    /// - Registration Data (4 bytes: protocol version + options)
    ///
    /// The PLC responds with:
    /// - Same header format with assigned session handle
    /// - Status code indicating success/failure
    ///
    /// # Errors
    ///
    /// - Network timeout or disconnection
    /// - Invalid response format
    /// - PLC rejection (status code non-zero)
    async fn register_session(&mut self) -> crate::error::Result<()> {
        self.ensure_stream_usable()?;
        tracing::debug!("Starting session registration...");
        let mut packet = BytesMut::with_capacity(28);
        EncapsulationHeader::new(REGISTER_SESSION, 4, 0).encode(&mut packet);
        packet.extend_from_slice(&[0x01, 0x00]); // Protocol Version: 1
        packet.extend_from_slice(&[0x00, 0x00]); // Option Flags: 0

        tracing::trace!("Sending Register Session packet: {:02X?}", packet);
        let mut stream = self.stream.lock().await;
        self.ensure_stream_usable()?;
        // Relaxed is sufficient: this flag is a scalar fail-fast marker shared
        // by clones; the stream mutex serializes actual I/O.
        self.stream_poisoned.store(true, Ordering::Relaxed);
        if let Err(e) = stream.write_all(&packet).await {
            tracing::error!("Failed to send Register Session packet: {}", e);
            return Err(EtherNetIpError::Io(e));
        }

        let mut header_buf = [0u8; 24];
        tracing::debug!("Waiting for Register Session response...");
        match timeout(Duration::from_secs(5), stream.read_exact(&mut header_buf)).await {
            Ok(Ok(_)) => {
                tracing::trace!("Received Register Session response header");
            }
            Ok(Err(e)) => {
                tracing::error!("Error reading response: {}", e);
                return Err(EtherNetIpError::Io(e));
            }
            Err(_) => {
                tracing::warn!("Timeout waiting for response");
                return Err(EtherNetIpError::Timeout(Duration::from_secs(5)));
            }
        };

        let mut header_bytes = &header_buf[..];
        let header = EncapsulationHeader::decode(&mut header_bytes)?;
        let mut body = vec![0u8; header.length as usize];
        if !body.is_empty() {
            match timeout(Duration::from_secs(5), stream.read_exact(&mut body)).await {
                Ok(Ok(_)) => {}
                Ok(Err(e)) => {
                    tracing::error!("Error reading response body: {}", e);
                    return Err(EtherNetIpError::Io(e));
                }
                Err(_) => {
                    tracing::warn!("Timeout waiting for response body");
                    return Err(EtherNetIpError::Timeout(Duration::from_secs(5)));
                }
            }
        }

        self.stream_poisoned.store(false, Ordering::Relaxed);

        // Extract session handle from response
        self.set_session_handle(header.session_handle);
        tracing::debug!("Session handle: 0x{:08X}", self.session_handle());

        // Check status
        let status = header.status;
        tracing::trace!("Status code: 0x{:08X}", status);

        if status != 0 {
            tracing::error!("Session registration failed with status: 0x{:08X}", status);
            return Err(EtherNetIpError::Protocol(format!(
                "Session registration failed with status: 0x{status:08X}"
            )));
        }

        tracing::info!("Session registration successful");
        Ok(())
    }

    /// Sets the maximum packet size for communication
    pub fn set_max_packet_size(&mut self, size: u32) {
        self.max_packet_size
            .store(size.min(4000), Ordering::Relaxed);
    }

    pub(crate) fn max_packet_size(&self) -> u32 {
        self.max_packet_size.load(Ordering::Relaxed)
    }

    pub(crate) fn session_handle(&self) -> u32 {
        self.session_handle.load(Ordering::Relaxed)
    }

    fn set_session_handle(&self, session_handle: u32) {
        self.session_handle.store(session_handle, Ordering::Relaxed);
    }

    fn stream_poisoned(&self) -> bool {
        self.stream_poisoned.load(Ordering::Relaxed)
    }

    fn next_sender_context(&self) -> [u8; 8] {
        self.sender_context_counter
            .fetch_add(1, Ordering::Relaxed)
            .to_le_bytes()
    }

    fn diagnostic_operation_for(cip_request: &[u8]) -> Option<DiagnosticOperation> {
        match cip_request.first().copied() {
            Some(READ_TAG | READ_TAG_FRAGMENTED) => Some(DiagnosticOperation::Read),
            Some(WRITE_TAG | WRITE_TAG_FRAGMENTED) => Some(DiagnosticOperation::Write),
            Some(MULTIPLE_SERVICE_PACKET) => Some(DiagnosticOperation::Batch),
            _ => None,
        }
    }

    fn ensure_stream_usable(&self) -> crate::error::Result<()> {
        if self.stream_poisoned() {
            return Err(EtherNetIpError::ConnectionLost(
                "connection stream is poisoned after an incomplete transaction; reconnect required"
                    .to_string(),
            ));
        }
        Ok(())
    }

    fn route_path_snapshot(&self) -> Option<RoutePath> {
        self.route_path
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone()
    }

    /// Discovers all tags in the PLC (including hierarchical UDT members)
    pub async fn discover_tags(&mut self) -> crate::error::Result<()> {
        let response = self
            .send_cip_request(&self.build_list_tags_request())
            .await?;

        // Extract CIP data from response and check for errors
        let cip_data = self.extract_cip_from_response(&response)?;

        // Check for CIP errors before parsing
        if let Err(e) = self.check_cip_error(&cip_data) {
            return Err(crate::error::EtherNetIpError::Protocol(format!(
                "Tag discovery failed: {}. Some PLCs may not support tag discovery. Try reading tags directly by name.",
                e
            )));
        }

        let tags = {
            let tag_manager = self.tag_manager.lock().await;
            tag_manager.parse_tag_list(&cip_data)?
        };

        tracing::debug!("Initial tag discovery found {} tags", tags.len());

        // Perform recursive drill-down discovery (similar to TypeScript implementation)
        let hierarchical_tags = {
            let tag_manager = self.tag_manager.lock().await;
            let hierarchical_tags = tag_manager.drill_down_tags(&tags).await?;
            drop(tag_manager);
            hierarchical_tags
        };

        tracing::debug!(
            "After drill-down: {} total tags discovered",
            hierarchical_tags.len()
        );

        {
            let tag_manager = self.tag_manager.lock().await;
            let mut cache = tag_manager.cache.write()?;
            for (name, metadata) in hierarchical_tags {
                cache.insert(name, metadata);
            }
        }
        Ok(())
    }

    /// Discovers UDT members for a specific structure
    pub async fn discover_udt_members(
        &mut self,
        udt_name: &str,
    ) -> crate::error::Result<Vec<(String, TagMetadata)>> {
        let definition = self.get_udt_definition(udt_name).await?;

        // Cache the definition
        {
            let tag_manager = self.tag_manager.lock().await;
            let mut definitions = tag_manager.udt_definitions.write()?;
            definitions.insert(udt_name.to_string(), definition.clone());
        }

        // Create member metadata
        let mut members = Vec::new();
        for member in &definition.members {
            let member_name = member.name.clone();
            let full_name = format!("{}.{}", udt_name, member_name);

            let metadata = TagMetadata {
                data_type: member.data_type,
                scope: TagScope::Controller,
                permissions: TagPermissions {
                    readable: true,
                    writable: true,
                },
                is_array: false,
                dimensions: Vec::new(),
                last_access: std::time::Instant::now(),
                size: member.size,
                array_info: None,
                last_updated: std::time::Instant::now(),
            };

            members.push((full_name, metadata));
        }

        Ok(members)
    }

    /// Gets cached UDT definition
    pub async fn get_udt_definition_cached(&self, udt_name: &str) -> Option<UdtDefinition> {
        let tag_manager = self.tag_manager.lock().await;
        tag_manager.get_udt_definition_cached(udt_name)
    }

    /// Lists all cached UDT definitions
    pub async fn list_udt_definitions(&self) -> Vec<String> {
        let tag_manager = self.tag_manager.lock().await;
        tag_manager.list_udt_definitions()
    }

    /// Discovers all tags with full attributes
    /// This method queries the PLC for all available tags and their detailed attributes
    pub async fn discover_tags_detailed(&mut self) -> crate::error::Result<Vec<TagAttributes>> {
        let (tags, _) = self.discover_tags_detailed_internal(false).await?;
        Ok(tags)
    }

    async fn discover_tags_detailed_internal(
        &mut self,
        best_effort: bool,
    ) -> crate::error::Result<(Vec<TagAttributes>, Vec<String>)> {
        let mut start_instance = 0u32;
        let mut tags = Vec::new();
        let mut warnings = Vec::new();

        loop {
            let request = self.build_tag_list_request_from_instance(start_instance)?;
            let response = match self.send_cip_request(&request).await {
                Ok(response) => response,
                Err(err) if best_effort && !tags.is_empty() => {
                    warnings.push(format!(
                        "Tag discovery stopped early at instance {} after transport/protocol failure: {}",
                        start_instance, err
                    ));
                    break;
                }
                Err(err) => return Err(err),
            };
            let cip_data = match self.extract_cip_from_response(&response) {
                Ok(cip_data) => cip_data,
                Err(err) if best_effort && !tags.is_empty() => {
                    warnings.push(format!(
                        "Tag discovery stopped early at instance {} after response extraction failure: {}",
                        start_instance, err
                    ));
                    break;
                }
                Err(err) => return Err(err),
            };
            let page = match self.parse_tag_list_response_page(&cip_data, udt::TagScope::Controller)
            {
                Ok(page) => page,
                Err(err) if best_effort && !tags.is_empty() => {
                    warnings.push(format!(
                        "Tag discovery stopped early at instance {} after page-parse failure: {}",
                        start_instance, err
                    ));
                    break;
                }
                Err(err) => return Err(err),
            };

            tags.extend(page.tags);

            if !page.partial_transfer {
                break;
            }

            let Some(last_instance_id) = page.last_instance_id else {
                return Err(crate::error::EtherNetIpError::Protocol(
                    "Tag discovery returned Partial transfer without a last instance ID"
                        .to_string(),
                ));
            };

            if last_instance_id == u32::MAX || last_instance_id < start_instance {
                return Err(crate::error::EtherNetIpError::Protocol(format!(
                    "Tag discovery pagination stalled at instance {}",
                    last_instance_id
                )));
            }

            start_instance = last_instance_id.saturating_add(1);
        }

        Ok((tags, warnings))
    }

    /// Discovers program-scoped tags
    /// This method discovers tags within a specific program scope
    ///
    /// The Symbol Object enumeration is PAGED: a program holding more tags than
    /// fit in one reply answers with general status `0x06` (partial transfer)
    /// and the caller must resume from the last instance id returned. This loop
    /// mirrors `discover_tags_detailed_internal`; without it a program
    /// with many tags surfaced only as `CIP Error 0x06: Partial transfer` and
    /// none of its tags were reachable.
    pub async fn discover_program_tags(
        &mut self,
        program_name: &str,
    ) -> crate::error::Result<Vec<TagAttributes>> {
        let mut start_instance = 0u32;
        let mut tags = Vec::new();
        // The scope is the caller's knowledge: the Symbol Object reply carries
        // no scope field, so it must be threaded in from the request. The
        // callers' two accepted spellings ("Dashboard" and "Program:Dashboard")
        // designate the same program and must yield the same scope.
        let scope = udt::TagScope::Program(program_scope_name(program_name).to_string());

        loop {
            let request = self.build_program_tag_list_request(program_name, start_instance)?;
            let response = self.send_cip_request(&request).await?;
            let cip_data = self.extract_cip_from_response(&response)?;

            let page = self
                .parse_tag_list_response_page(&cip_data, scope.clone())
                .map_err(|e| {
                    crate::error::EtherNetIpError::Protocol(format!(
                        "Program tag discovery failed for '{}': {}. Some PLCs may not support tag discovery. Try reading tags directly by name.",
                        program_name, e
                    ))
                })?;

            tags.extend(page.tags);

            if !page.partial_transfer {
                break;
            }

            let Some(last_instance_id) = page.last_instance_id else {
                return Err(crate::error::EtherNetIpError::Protocol(format!(
                    "Program tag discovery for '{}' returned Partial transfer without a last instance ID",
                    program_name
                )));
            };

            if last_instance_id == u32::MAX || last_instance_id < start_instance {
                return Err(crate::error::EtherNetIpError::Protocol(format!(
                    "Program tag discovery for '{}' pagination stalled at instance {}",
                    program_name, last_instance_id
                )));
            }

            start_instance = last_instance_id.saturating_add(1);
        }

        Ok(tags)
    }

    /// Lists all cached tag attributes
    pub async fn list_cached_tag_attributes(&self) -> Vec<String> {
        self.udt_manager.lock().await.list_tag_attributes()
    }

    /// Clears cached tag metadata, UDT data, and array-type classifications.
    pub async fn clear_caches(&mut self) {
        if let Err(error) = self.tag_manager.lock().await.clear_cache().await {
            tracing::warn!("failed to clear tag metadata cache: {error}");
        }
        self.udt_manager.lock().await.clear_cache();
        self.clear_array_type_cache();
    }

    /// Creates a new client with a specific route path
    pub async fn with_route_path(addr: &str, route: RoutePath) -> crate::error::Result<Self> {
        let mut client = Self::new(addr).await?;
        client.set_route_path(route);
        Ok(client)
    }

    /// Connect to a PLC using a custom stream
    ///
    /// This method allows you to provide your own stream implementation, enabling:
    /// - Wrapping streams for metrics/observability (bytes in/out)
    /// - Applying custom socket options (keepalive, timeouts, bind local address)
    /// - Reusing pre-established tunnels/connections
    /// - Using in-memory streams for deterministic testing
    ///
    /// # Arguments
    ///
    /// * `stream` - Any stream that implements `AsyncRead + AsyncWrite + Unpin + Send`
    ///
    /// # Example
    ///
    /// ```no_run
    /// use rust_ethernet_ip::EipClient;
    /// use std::io::Cursor;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// // Any AsyncRead + AsyncWrite + Unpin + Send stream can be injected.
    /// let stream = Cursor::new(Vec::<u8>::new());
    ///
    /// // Connect using the custom stream
    /// let client = EipClient::connect_with_stream(stream, None).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn connect_with_stream<S>(stream: S, route: Option<RoutePath>) -> Result<Self>
    where
        S: EtherNetIpStream + 'static,
    {
        let mut client = Self::from_stream(stream).await?;
        if let Some(route) = route {
            client.set_route_path(route);
        }
        Ok(client)
    }

    /// Sets the route path for the client
    pub fn set_route_path(&mut self, route: RoutePath) {
        self.clear_array_type_cache();
        *self
            .route_path
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = Some(route);
    }

    /// Gets the current route path
    pub fn get_route_path(&self) -> Option<RoutePath> {
        self.route_path_snapshot()
    }

    /// Removes the route path (uses direct connection)
    pub fn clear_route_path(&mut self) {
        self.clear_array_type_cache();
        *self
            .route_path
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = None;
    }

    fn cached_array_is_packed_bool(&self, array_path: &str) -> Option<bool> {
        self.array_type_cache
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .get(array_path)
            .copied()
    }

    fn cache_array_is_packed_bool(&self, array_path: &str, is_packed_bool: bool) {
        self.array_type_cache
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .insert(array_path.to_string(), is_packed_bool);
    }

    fn clear_array_type_cache(&self) {
        self.array_type_cache
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clear();
    }

    /// Gets metadata for a tag
    pub async fn get_tag_metadata(&self, tag_name: &str) -> Option<TagMetadata> {
        let tag_manager = self.tag_manager.lock().await;
        match tag_manager.cache.read() {
            Ok(cache) => cache.get(tag_name).cloned(),
            Err(_) => {
                tracing::warn!("failed to read tag metadata cache: lock poisoned");
                None
            }
        }
    }

    /// Reads a tag value from the PLC
    ///
    /// This function performs a CIP read request for the specified tag.
    /// The tag's data type is automatically determined from the PLC's response.
    ///
    /// **v0.6.0**: For UDT tags, this returns `PlcValue::Udt(UdtData)` with `symbol_id`
    /// and raw bytes. Use `UdtData::parse()` with a UDT definition to access members.
    ///
    /// # Arguments
    ///
    /// * `tag_name` - The name of the tag to read
    ///
    /// # Returns
    ///
    /// The tag's value as a `PlcValue` enum. For UDTs, this is `PlcValue::Udt(UdtData)`.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use rust_ethernet_ip::{EipClient, PlcValue};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    ///     let mut client = EipClient::connect("192.168.1.100:44818").await?;
    ///
    ///     // Read different data types
    ///     let bool_val = client.read_tag("MotorRunning").await?;
    ///     let int_val = client.read_tag("Counter").await?;
    ///     let real_val = client.read_tag("Temperature").await?;
    ///
    ///     // Read a UDT (v0.6.0: returns UdtData)
    ///     let udt_val = client.read_tag("MyUDT").await?;
    ///     if let PlcValue::Udt(udt_data) = udt_val {
    ///         let udt_def = client.get_udt_definition("MyUDT").await?;
    ///         // Convert UdtDefinition to UserDefinedType
    ///         let mut user_def = rust_ethernet_ip::udt::UserDefinedType::new(udt_def.name.clone());
    ///         for member in &udt_def.members {
    ///             user_def.add_member(member.clone());
    ///         }
    ///         let members = udt_data.parse(&user_def)?;
    ///         println!("UDT has {} members", members.len());
    ///     }
    ///
    ///     // Handle the result
    ///     match bool_val {
    ///         PlcValue::Bool(true) => println!("Motor is running"),
    ///         PlcValue::Bool(false) => println!("Motor is stopped"),
    ///         _ => println!("Unexpected data type"),
    ///     }
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Performance
    ///
    /// - Latency: 1-5ms typical
    /// - Throughput: 1,500+ ops/sec
    /// - Network: 1 request/response cycle
    ///
    /// # Error Handling
    ///
    /// Common errors:
    /// - `Protocol`: Tag doesn't exist or invalid format
    /// - `Connection`: Lost connection to PLC
    /// - `Timeout`: Operation timed out
    pub async fn read_tag(&mut self, tag_name: &str) -> crate::error::Result<PlcValue> {
        self.validate_session().await?;

        if let Some((base_path, bit_index)) = self.parse_bit_access(tag_name) {
            return self
                .read_bit_base_direct(&base_path, bit_index)
                .await
                .map(PlcValue::Bool);
        }

        // Check if this is a simple array element access (e.g., "ArrayName[0]")
        // BUT NOT if it has member access after (e.g., "ArrayName[0].Member")
        // Complex paths like "gTestUDT_Array[0].Member1_DINT" should use TagPath::parse()
        if let Some((base_name, index)) = self.parse_array_element_access(tag_name) {
            // Only use workaround if there's no member access after the array brackets
            // Find the FIRST [ and ] pair to check for member access after it
            if let Some(bracket_start) = tag_name.find('[')
                && let Some(bracket_end_rel) = tag_name[bracket_start..].find(']')
            {
                let bracket_end_abs = bracket_start + bracket_end_rel;
                let after_bracket = &tag_name[bracket_end_abs + 1..];
                tracing::debug!(
                    "Array element detected for '{}': base='{}', index={}, after_bracket='{}'",
                    tag_name,
                    base_name,
                    index,
                    after_bracket
                );
                // If there's a dot after the bracket, it's a member access - use TagPath::parse() instead
                if !after_bracket.starts_with('.') {
                    tracing::debug!(
                        "Detected simple array element access: {}[{}], using workaround",
                        base_name,
                        index
                    );
                    return self.read_array_element_workaround(&base_name, index).await;
                } else {
                    tracing::debug!(
                        "Array element '{}[{}]' has member access after bracket ('{}'), using TagPath::parse()",
                        base_name,
                        index,
                        after_bracket
                    );
                }
            }
        }

        // For complex paths (with member access, nested arrays, etc.), use TagPath::parse()
        // This handles paths like "gTestUDT_Array[0].Member1_DINT" correctly
        // Standard tag reading uses build_read_request which uses TagPath::parse()
        if let Some((parent_path, index)) = self.parse_final_array_element_access(tag_name)
            && self.detect_bool_array_path(&parent_path).await?
        {
            return self
                .read_bool_array_element_workaround(&parent_path, index)
                .await;
        }

        self.read_tag_direct(tag_name).await
    }

    async fn read_tag_direct(&mut self, tag_name: &str) -> crate::error::Result<PlcValue> {
        let response = self
            .send_cip_request(&self.build_read_request(tag_name)?)
            .await?;
        let cip_data = self.extract_cip_from_response(&response)?;
        if cip_data.get(2).copied() == Some(CIP_STATUS_PARTIAL_TRANSFER) {
            return self.read_tag_fragmented(tag_name).await;
        }
        self.parse_cip_response(&cip_data)
    }

    async fn read_tag_fragmented(&mut self, tag_name: &str) -> crate::error::Result<PlcValue> {
        let mut offset = 0u32;
        let mut reassembled = Vec::new();

        loop {
            let request = self.build_read_fragmented_request(tag_name, 1, offset)?;
            let response = self.send_cip_request(&request).await?;
            let cip_data = self.extract_unconnected_data_item(&response)?;
            let (status, fragment) = self.parse_read_fragmented_response(&cip_data)?;

            if fragment.is_empty() && status == CIP_STATUS_PARTIAL_TRANSFER {
                return Err(EtherNetIpError::Protocol(format!(
                    "Read Tag Fragmented for '{tag_name}' returned an empty partial fragment at offset {offset}"
                )));
            }

            offset = offset
                .checked_add(fragment.len() as u32)
                .ok_or_else(|| EtherNetIpError::Protocol("fragment offset overflow".to_string()))?;
            reassembled.extend_from_slice(fragment);

            if status == CIP_STATUS_SUCCESS {
                break;
            }
        }

        self.decode_type_prefixed_value(&reassembled)
    }

    /// Reads a single bit from a tag (e.g. a DINT used as a status word).
    ///
    /// Equivalent to `read_tag(&format!("{}.{}", tag_base, bit_index))` for bit paths.
    /// `bit_index` must be in 0..32 (Allen-Bradley DINT bits).
    ///
    /// # Example
    ///
    /// ```ignore
    /// let bit_5 = client.read_bit("StatusWord", 5).await?;
    /// ```
    pub async fn read_bit(&mut self, tag_base: &str, bit_index: u8) -> crate::error::Result<bool> {
        self.validate_session().await?;
        self.read_bit_base_direct(tag_base, bit_index).await
    }

    async fn read_bit_base_direct(
        &mut self,
        tag_base: &str,
        bit_index: u8,
    ) -> crate::error::Result<bool> {
        if bit_index >= 32 {
            return Err(crate::error::EtherNetIpError::Protocol(
                "bit_index must be 0..32 for DINT bit access".to_string(),
            ));
        }
        // Logix has no wire-level bit segment for atomic tags, so read the
        // parent word and extract the bit client-side.
        match self.read_tag_direct(tag_base).await? {
            PlcValue::Bool(b) => Ok(b),
            PlcValue::Dint(n) => Ok((n >> bit_index) & 1 != 0),
            other => Err(crate::error::EtherNetIpError::DataTypeMismatch {
                expected: "BOOL or DINT".to_string(),
                actual: format!("{:?}", other),
            }),
        }
    }

    /// Writes a single bit to a tag (e.g. a DINT used as a control word).
    ///
    /// Equivalent to `write_tag(&format!("{}.{}", tag_base, bit_index), PlcValue::Bool(value))`.
    /// `bit_index` must be in 0..32.
    ///
    /// # Example
    ///
    /// ```ignore
    /// client.write_bit("ControlWord", 3, true).await?;
    /// ```
    pub async fn write_bit(
        &mut self,
        tag_base: &str,
        bit_index: u8,
        value: bool,
    ) -> crate::error::Result<()> {
        self.validate_session().await?;
        self.write_bit_base_direct(tag_base, bit_index, value).await
    }

    async fn write_bit_base_direct(
        &mut self,
        tag_base: &str,
        bit_index: u8,
        value: bool,
    ) -> crate::error::Result<()> {
        if bit_index >= 32 {
            return Err(crate::error::EtherNetIpError::Protocol(
                "bit_index must be 0..32 for DINT bit access".to_string(),
            ));
        }
        // Logix cannot write a single bit of an atomic tag over CIP, so emulate
        // it with a read-modify-write of the parent word. Note: this is not
        // atomic across concurrent writers to the same word.
        match self.read_tag_direct(tag_base).await? {
            PlcValue::Dint(current) => {
                let mask = 1i32 << bit_index;
                let updated = if value {
                    current | mask
                } else {
                    current & !mask
                };
                self.write_tag_direct(tag_base, &PlcValue::Dint(updated))
                    .await
            }
            PlcValue::Bool(_) if bit_index == 0 => {
                self.write_tag_direct(tag_base, &PlcValue::Bool(value))
                    .await
            }
            other => Err(crate::error::EtherNetIpError::DataTypeMismatch {
                expected: "DINT".to_string(),
                actual: format!("{:?}", other),
            }),
        }
    }

    /// Parses array element access syntax (e.g., "ArrayName[0]") and returns (base_name, index)
    fn parse_array_element_access(&self, tag_name: &str) -> Option<(String, u32)> {
        // Look for array bracket notation
        if let Some(bracket_pos) = tag_name.rfind('[')
            && let Some(close_bracket_pos) = tag_name.rfind(']')
            && close_bracket_pos > bracket_pos
        {
            let base_name = tag_name[..bracket_pos].to_string();
            let index_str = &tag_name[bracket_pos + 1..close_bracket_pos];
            if let Ok(index) = index_str.parse::<u32>()
                && !tag_name[..bracket_pos].contains('[')
            {
                // Make sure there are no more brackets after this (multi-dimensional arrays not supported yet)
                return Some((base_name, index));
            }
        }
        None
    }

    fn has_member_suffix_after_first_array_index(&self, tag_name: &str) -> bool {
        if let Some(bracket_start) = tag_name.find('[')
            && let Some(bracket_end_rel) = tag_name[bracket_start..].find(']')
        {
            let bracket_end_abs = bracket_start + bracket_end_rel;
            return tag_name[bracket_end_abs + 1..].starts_with('.');
        }

        false
    }

    fn parse_bit_access(&self, tag_name: &str) -> Option<(String, u8)> {
        match TagPath::parse(tag_name).ok()? {
            TagPath::Bit {
                base_path,
                bit_index,
            } => Some((base_path.as_string(), bit_index)),
            _ => None,
        }
    }

    fn parse_final_array_element_access(&self, tag_name: &str) -> Option<(String, u32)> {
        match TagPath::parse(tag_name).ok()? {
            TagPath::Array { base_path, indices } if indices.len() == 1 => {
                Some((base_path.as_string(), indices[0]))
            }
            _ => None,
        }
    }

    async fn detect_bool_array_path(&mut self, array_path: &str) -> crate::error::Result<bool> {
        if let Some(is_packed_bool) = self.cached_array_is_packed_bool(array_path) {
            return Ok(is_packed_bool);
        }

        let test_response = self
            .send_cip_request(&self.build_read_request_with_count(array_path, 1)?)
            .await?;
        let test_cip_data = self.extract_cip_from_response(&test_response)?;

        if self.check_cip_error(&test_cip_data).is_err() || test_cip_data.len() < 6 {
            return Ok(false);
        }

        let test_data_type = u16::from_le_bytes([test_cip_data[4], test_cip_data[5]]);
        let is_packed_bool = test_data_type == values::BOOL_ARRAY_DWORD;
        self.cache_array_is_packed_bool(array_path, is_packed_bool);
        Ok(is_packed_bool)
    }

    fn parse_bool_array_dword_response(&self, cip_data: &[u8]) -> crate::error::Result<u32> {
        let mut response_bytes = cip_data;
        let response = CipResponse::decode(&mut response_bytes)?;
        if response.status != 0 {
            return Err(EtherNetIpError::Protocol(format!(
                "CIP Error {} when reading BOOL array DWORD: {}",
                response.status,
                self.get_cip_error_message(response.status)
            )));
        }

        if response.service != 0xCC {
            return Err(EtherNetIpError::Protocol(format!(
                "Unexpected service reply: 0x{:02X}",
                response.service
            )));
        }

        if response.data.len() < 6 {
            return Err(EtherNetIpError::Protocol(
                "BOOL array response too short for data type and DWORD".to_string(),
            ));
        }

        let data_type = u16::from_le_bytes([response.data[0], response.data[1]]);
        if data_type != values::BOOL_ARRAY_DWORD {
            return Err(EtherNetIpError::Protocol(format!(
                "Expected BOOL array DWORD data type 0x00D3, got 0x{data_type:04X}"
            )));
        }

        let value_data = &response.data[2..];

        if value_data.len() < 4 {
            return Err(EtherNetIpError::Protocol(format!(
                "BOOL array data too short: need 4 bytes (DWORD), got {} bytes",
                value_data.len()
            )));
        }

        Ok(u32::from_le_bytes([
            value_data[0],
            value_data[1],
            value_data[2],
            value_data[3],
        ]))
    }

    /// Reads a single array element using proper CIP element addressing
    ///
    /// This method uses element addressing (0x28/0x29/0x2A segments) in the Request Path
    /// to read directly from the specified array index, eliminating the need to read
    /// the entire array.
    ///
    /// Reference: 1756-PM020, Pages 603-611, 815-837 (Array Element Access Examples)
    ///
    /// # Arguments
    ///
    /// * `base_array_name` - Base name of the array (e.g., "MyArray" for "MyArray[5]")
    /// * `index` - Element index to read (0-based)
    async fn read_array_element_workaround(
        &mut self,
        base_array_name: &str,
        index: u32,
    ) -> crate::error::Result<PlcValue> {
        tracing::debug!(
            "Reading array element '{}[{}]' using element addressing",
            base_array_name,
            index
        );

        // First, detect if it's a BOOL array by reading with count=1 to check data type
        let test_response = self
            .send_cip_request(&self.build_read_request_with_count(base_array_name, 1)?)
            .await?;
        let test_cip_data = self.extract_cip_from_response(&test_response)?;

        // A Partial Transfer here means the element is a structure larger than one CIP packet
        // (e.g. a big UDT) — it cannot be a BOOL array, so skip BOOL detection and let the
        // element read below fall back to the fragmented path.
        if test_cip_data.get(2).copied() != Some(CIP_STATUS_PARTIAL_TRANSFER) {
            // Check for errors in test read
            self.check_cip_error(&test_cip_data)?;

            // Check if it's a BOOL array (data type 0x00D3 = DWORD)
            if test_cip_data.len() >= 6 {
                let test_data_type = u16::from_le_bytes([test_cip_data[4], test_cip_data[5]]);
                if test_data_type == 0x00D3 {
                    // BOOL array - use special workaround to extract the bit
                    return self
                        .read_bool_array_element_workaround(base_array_name, index)
                        .await;
                }
            }
        }

        // Use element addressing to read directly from the specified index
        // Reference: 1756-PM020, Pages 815-837 (Reading Array Element - Full Message)
        let request = self.build_read_array_request(base_array_name, index, 1);

        let response = self.send_cip_request(&request).await?;
        let cip_data = self.extract_cip_from_response(&response)?;

        // A whole array element that is a large structure (e.g. a UDT bigger than one CIP
        // packet) comes back as Partial Transfer; reassemble it via the fragmented read path,
        // mirroring read_tag_direct.
        if cip_data.get(2).copied() == Some(CIP_STATUS_PARTIAL_TRANSFER) {
            return self
                .read_tag_fragmented(&format!("{base_array_name}[{index}]"))
                .await;
        }

        // Check for errors (including extended errors)
        self.check_cip_error(&cip_data)?;

        // Parse response - should be consistent format now
        // Reference: 1756-PM020, Page 828-837 (Response format)
        self.parse_cip_response(&cip_data)
    }

    /// Special workaround for BOOL arrays: reads DWORD and extracts the specific bit
    ///
    /// Reference: 1756-PM020, Page 797-811 (BOOL Array Access)
    async fn read_bool_array_element_workaround(
        &mut self,
        base_array_name: &str,
        index: u32,
    ) -> crate::error::Result<PlcValue> {
        tracing::debug!(
            "BOOL array detected - reading DWORD and extracting bit [{}]",
            index
        );

        let dword_index = index / 32;

        // Read just 1 element (the DWORD containing 32 BOOLs)
        // Reference: 1756-PM020, Page 797-811
        let response = self
            .send_cip_request(&self.build_read_array_request(base_array_name, dword_index, 1))
            .await?;
        let cip_data = self.extract_cip_from_response(&response)?;
        let dword_value = self.parse_bool_array_dword_response(&cip_data)?;

        // Extract the specific bit
        // Each DWORD contains 32 BOOLs (bits 0-31)
        let bit_index = (index % 32) as u8;
        let bool_value = (dword_value >> bit_index) & 1 != 0;

        Ok(PlcValue::Bool(bool_value))
    }

    /// Helper function to read large arrays in chunks to avoid PLC response size limits
    ///
    /// This method uses element addressing to read specific ranges of array elements,
    /// allowing efficient reading of large arrays without reading from element 0 each time.
    ///
    /// Reference: 1756-PM020, Pages 276-315 (Read Tag Fragmented Service), 840-851 (Reading Multiple Array Elements)
    async fn read_array_in_chunks(
        &mut self,
        base_array_name: &str,
        data_type: u16,
        start_index: u32,
        target_element_count: u32,
    ) -> crate::error::Result<Vec<u8>> {
        // Determine element size and safe chunk size
        let element_size = match data_type {
            0x00C1 => 1, // BOOL
            0x00C2 => 1, // SINT
            0x00C3 => 2, // INT
            0x00C4 => 4, // DINT
            0x00C5 => 8, // LINT
            0x00C6 => 1, // USINT
            0x00C7 => 2, // UINT
            0x00C8 => 4, // UDINT
            0x00C9 => 8, // ULINT
            0x00CA => 4, // REAL
            0x00CB => 8, // LREAL
            _ => {
                return Err(EtherNetIpError::Protocol(format!(
                    "Unsupported array data type for chunked reading: 0x{:04X}",
                    data_type
                )));
            }
        };

        // Read in chunks - use 8 elements per chunk for 4-byte types to stay under 38-byte limit
        // For smaller types, we can read more elements per chunk
        let elements_per_chunk = match element_size {
            1 => 30, // 1-byte types: 30 elements = 30 bytes + 8 header = 38 bytes
            2 => 15, // 2-byte types: 15 elements = 30 bytes + 8 header = 38 bytes
            4 => 8, // 4-byte types: 8 elements = 32 bytes + 8 header = 40 bytes (may truncate to 38)
            8 => 4, // 8-byte types: 4 elements = 32 bytes + 8 header = 40 bytes
            _ => 8,
        };

        let end_index = start_index
            .checked_add(target_element_count)
            .ok_or_else(|| EtherNetIpError::Protocol("Array range overflow".to_string()))?;

        let mut all_data = Vec::new();
        let mut next_chunk_start = start_index;

        tracing::debug!(
            "Reading array '{}' in chunks: {} elements per chunk, target: {} elements",
            base_array_name,
            elements_per_chunk,
            target_element_count
        );

        while next_chunk_start < end_index {
            // Use element addressing to read specific range starting from next_chunk_start
            // Reference: 1756-PM020, Pages 840-851 (Reading Multiple Array Elements)
            let chunk_end = (next_chunk_start + elements_per_chunk as u32).min(end_index);
            let chunk_size = (chunk_end - next_chunk_start) as u16;

            tracing::trace!(
                "Reading chunk: elements {} to {} ({} elements) using element addressing",
                next_chunk_start,
                chunk_end - 1,
                chunk_size
            );

            // Use element addressing to read this specific range
            // Reference: 1756-PM020, Pages 840-851 (Reading Multiple Array Elements)
            let response = self
                .send_cip_request(&self.build_read_array_request(
                    base_array_name,
                    next_chunk_start,
                    chunk_size,
                ))
                .await?;
            let cip_data = self.extract_cip_from_response(&response)?;

            let mut response_bytes = cip_data.as_slice();
            let response = CipResponse::decode(&mut response_bytes)?;
            if response.status != 0 {
                let error_msg = self.get_cip_error_message(response.status);
                return Err(EtherNetIpError::Protocol(format!(
                    "CIP Error {} when reading chunk (elements {} to {}): {}",
                    response.status,
                    next_chunk_start,
                    chunk_end - 1,
                    error_msg
                )));
            }

            if response.service != 0xCC {
                return Err(EtherNetIpError::Protocol(format!(
                    "Unexpected service reply in chunk: 0x{:02X} (expected 0xCC)",
                    response.service
                )));
            }

            if response.data.len() < 2 {
                return Err(EtherNetIpError::Protocol(format!(
                    "Chunk response too short for data type: got {} bytes, expected at least 6",
                    cip_data.len()
                )));
            }

            let chunk_data_type = u16::from_le_bytes([response.data[0], response.data[1]]);
            if chunk_data_type != data_type {
                return Err(EtherNetIpError::Protocol(format!(
                    "Data type mismatch in chunk: expected 0x{:04X}, got 0x{:04X}",
                    data_type, chunk_data_type
                )));
            }

            // Parse response data - with element addressing, response contains the requested range
            // Reference: 1756-PM020, Page 828-837 (Response format)
            // A Logix Read Tag reply data body is [data_type u16][data...]; it does
            // not include an element-count field.
            let chunk_value_data = &response.data[2..];
            let chunk_complete_bytes = (chunk_value_data.len() / element_size) * element_size;
            let chunk_data = &chunk_value_data[..chunk_complete_bytes];

            // With element addressing, the response directly contains the requested range
            // No need to extract a portion - use all the data we received
            if !chunk_data.is_empty() {
                all_data.extend_from_slice(chunk_data);
                let elements_received = chunk_data.len() / element_size;
                next_chunk_start += elements_received as u32;

                tracing::trace!(
                    "Chunk read: {} elements ({} bytes) starting at index {}, total so far: {} elements",
                    elements_received,
                    chunk_data.len(),
                    next_chunk_start - elements_received as u32,
                    all_data.len() / element_size
                );

                // Continue reading if we haven't reached our target yet
                if next_chunk_start >= end_index {
                    tracing::trace!(
                        "Reached target element count ({}), stopping chunked read",
                        target_element_count
                    );
                    break;
                }
            } else {
                // No data received, we're done
                break;
            }
        }

        let final_element_count = all_data.len() / element_size;
        tracing::debug!(
            "Chunked read complete: {} total elements ({} bytes), target was {} elements",
            final_element_count,
            all_data.len(),
            target_element_count
        );

        if final_element_count < target_element_count as usize {
            return Err(EtherNetIpError::Protocol(format!(
                "Incomplete array read: requested {} elements, received {}",
                target_element_count, final_element_count
            )));
        }

        Ok(all_data)
    }

    fn array_element_size(data_type: u16) -> Option<usize> {
        match data_type {
            0x00C1 => Some(1), // BOOL
            0x00C2 => Some(1), // SINT
            0x00C3 => Some(2), // INT
            0x00C4 => Some(4), // DINT
            0x00C5 => Some(8), // LINT
            0x00C6 => Some(1), // USINT
            0x00C7 => Some(2), // UINT
            0x00C8 => Some(4), // UDINT
            0x00C9 => Some(8), // ULINT
            0x00CA => Some(4), // REAL
            0x00CB => Some(8), // LREAL
            _ => None,
        }
    }

    fn decode_array_bytes(
        &self,
        data_type: u16,
        bytes: &[u8],
    ) -> crate::error::Result<Vec<PlcValue>> {
        let Some(element_size) = Self::array_element_size(data_type) else {
            return Err(EtherNetIpError::Protocol(format!(
                "Unsupported data type for array decoding: 0x{:04X}",
                data_type
            )));
        };

        if !bytes.len().is_multiple_of(element_size) {
            return Err(EtherNetIpError::Protocol(format!(
                "Array payload length {} is not aligned to element size {}",
                bytes.len(),
                element_size
            )));
        }

        let mut values = Vec::with_capacity(bytes.len() / element_size);
        for chunk in bytes.chunks_exact(element_size) {
            values.push(values::decode_array_element(data_type, chunk)?);
        }

        Ok(values)
    }

    /// Read a range of elements from a basic-type PLC array.
    ///
    /// This method reads arrays in chunks under the hood to avoid PLC packet-size limits.
    /// It supports basic CIP scalar types:
    /// BOOL, SINT, INT, DINT, LINT, USINT, UINT, UDINT, ULINT, REAL, LREAL.
    ///
    /// # Arguments
    ///
    /// * `base_array_name` - Base array tag name without index (e.g., `"MyDintArray"`)
    /// * `start_index` - Starting element index
    /// * `element_count` - Number of elements to read
    ///
    /// # Returns
    ///
    /// A `Vec<PlcValue>` with one element per requested array entry.
    pub async fn read_array_range(
        &mut self,
        base_array_name: &str,
        start_index: u32,
        element_count: u32,
    ) -> crate::error::Result<Vec<PlcValue>> {
        if element_count == 0 {
            return Ok(Vec::new());
        }

        let probe_response = self
            .send_cip_request(&self.build_read_array_request(base_array_name, start_index, 1))
            .await?;
        let probe_cip = self.extract_cip_from_response(&probe_response)?;
        self.check_cip_error(&probe_cip)?;

        if probe_cip.len() < 6 {
            return Err(EtherNetIpError::Protocol(
                "Array probe response too short".to_string(),
            ));
        }

        let data_type = u16::from_le_bytes([probe_cip[4], probe_cip[5]]);
        let raw = self
            .read_array_in_chunks(base_array_name, data_type, start_index, element_count)
            .await?;
        let values = self.decode_array_bytes(data_type, &raw)?;

        if values.len() != element_count as usize {
            return Err(EtherNetIpError::Protocol(format!(
                "Array read count mismatch: requested {}, got {}",
                element_count,
                values.len()
            )));
        }

        Ok(values)
    }

    /// Writes to a single array element using direct element addressing
    ///
    /// This method uses element addressing (0x28/0x29/0x2A segments) in the Request Path
    /// to write directly to the specified array index, eliminating the need to read
    /// the entire array.
    ///
    /// Reference: 1756-PM020, Pages 855-867 (Writing to Array Element)
    ///
    /// # Arguments
    ///
    /// * `base_array_name` - Base name of the array (e.g., `"MyArray"` for `"MyArray[10]"`)
    /// * `index` - Element index to write (0-based)
    /// * `value` - The value to write
    async fn write_array_element_workaround(
        &mut self,
        base_array_name: &str,
        index: u32,
        value: PlcValue,
    ) -> crate::error::Result<()> {
        tracing::debug!(
            "Writing to array element '{}[{}]' using element addressing",
            base_array_name,
            index
        );

        // First, detect if it's a BOOL array by reading with count=1
        let test_response = self
            .send_cip_request(&self.build_read_request_with_count(base_array_name, 1)?)
            .await?;
        let test_cip_data = self.extract_cip_from_response(&test_response)?;

        // Check for errors in the test read response
        if test_cip_data.len() < 3 {
            return Err(EtherNetIpError::Protocol(
                "Test read response too short".to_string(),
            ));
        }

        // Check for errors in test read (including extended errors)
        if let Err(e) = self.check_cip_error(&test_cip_data) {
            return Err(EtherNetIpError::Protocol(format!(
                "Cannot write to array element: Test read failed: {}",
                e
            )));
        }

        // Check if we have enough data to determine the data type
        if test_cip_data.len() < 6 {
            return Err(EtherNetIpError::Protocol(
                "Test read response too short to determine data type".to_string(),
            ));
        }

        let test_data_type = u16::from_le_bytes([test_cip_data[4], test_cip_data[5]]);

        // If it's a BOOL array (0x00D3 = DWORD), handle it specially
        if test_data_type == 0x00D3 {
            return self
                .write_bool_array_element_workaround(base_array_name, index, value)
                .await;
        }

        // Get the data type and convert value to bytes
        let data_type = test_data_type;
        let value_bytes = value.to_bytes();

        // Use element addressing to write directly to the specified index
        // Reference: 1756-PM020, Pages 855-867
        let request = self.build_write_array_request_with_index(
            base_array_name,
            index,
            1, // Write 1 element
            data_type,
            &value_bytes,
        )?;

        let response = self.send_cip_request(&request).await?;
        let cip_data = self.extract_cip_from_response(&response)?;

        // Check for errors (including extended errors)
        self.check_cip_error(&cip_data)?;

        tracing::info!("Array element write completed successfully");
        Ok(())
    }

    /// Special workaround for BOOL arrays: reads DWORD, modifies bit, writes back.
    ///
    /// Note: This is a read-modify-write operation. Callers must ensure exclusive
    /// access to the client for the entire duration (the `&mut self` requirement
    /// provides this guarantee in safe Rust; FFI callers are protected by the global mutex).
    ///
    /// Reference: 1756-PM020, Page 797-811 (BOOL Array Access)
    async fn write_bool_array_element_workaround(
        &mut self,
        base_array_name: &str,
        index: u32,
        value: PlcValue,
    ) -> crate::error::Result<()> {
        tracing::debug!(
            "BOOL array element write - reading DWORD, modifying bit [{}], writing back",
            index
        );

        let dword_index = index / 32;

        // Read the DWORD
        let response = self
            .send_cip_request(&self.build_read_array_request(base_array_name, dword_index, 1))
            .await?;
        let cip_data = self.extract_cip_from_response(&response)?;

        // Get the boolean value
        let bool_value = match value {
            PlcValue::Bool(b) => b,
            _ => {
                return Err(EtherNetIpError::Protocol(
                    "Expected BOOL value for BOOL array element".to_string(),
                ));
            }
        };

        // Modify the DWORD
        let original_dword_value = self.parse_bool_array_dword_response(&cip_data)?;
        let mut dword_value = original_dword_value;

        let bit_index = (index % 32) as u8;
        if bool_value {
            dword_value |= 1u32 << bit_index;
        } else {
            dword_value &= !(1u32 << bit_index);
        }

        tracing::trace!(
            "Modified BOOL[{}] in DWORD: 0x{:08X} -> 0x{:08X} (bit {} = {})",
            index,
            original_dword_value,
            dword_value,
            bit_index,
            bool_value
        );

        // Write the DWORD back
        let write_request = self.build_write_array_request_with_index(
            base_array_name,
            dword_index,
            1,
            values::BOOL_ARRAY_DWORD,
            &dword_value.to_le_bytes(),
        )?;
        let write_response = self.send_cip_request(&write_request).await?;
        let write_cip_data = self.extract_cip_from_response(&write_response)?;

        // Check for errors (including extended errors)
        self.check_cip_error(&write_cip_data)?;

        tracing::info!("BOOL array element write completed successfully");
        Ok(())
    }

    /// Builds a CIP Write Tag Service request for array elements with element addressing
    ///
    /// This method uses proper CIP element addressing (0x28/0x29/0x2A segments) in the
    /// Request Path to write to specific array elements or ranges.
    ///
    /// Reference: 1756-PM020, Pages 603-611, 855-867 (Writing to Array Element)
    ///
    /// # Arguments
    ///
    /// * `base_array_name` - Base name of the array (e.g., `"MyArray"` for `"MyArray[10]"`)
    /// * `start_index` - Starting element index (0-based)
    /// * `element_count` - Number of elements to write
    /// * `data_type` - CIP data type code (e.g., 0x00C4 for DINT)
    /// * `data` - Raw bytes of the data to write
    ///
    /// # Example
    ///
    /// Writing value 0x12345678 to element 10 of array "MyArray":
    /// ```
    /// # async fn example() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    /// # let mut client = rust_ethernet_ip::EipClient::connect("192.168.1.100:44818").await?;
    /// let data = 0x12345678u32.to_le_bytes();
    /// let request = client.build_write_array_request_with_index(
    ///     "MyArray", 10, 1, 0x00C4, &data
    /// )?;
    /// # Ok(())
    /// # }
    /// ```
    #[cfg_attr(not(test), allow(dead_code))]
    pub fn build_write_array_request_with_index(
        &self,
        base_array_name: &str,
        start_index: u32,
        element_count: u16,
        data_type: u16,
        data: &[u8],
    ) -> crate::error::Result<Vec<u8>> {
        let mut cip_request = Vec::new();

        // Service: Write Tag Service (0x4D)
        // Reference: 1756-PM020, Page 318
        cip_request.push(0x4D);

        // Build base tag path (symbolic segment)
        // Reference: 1756-PM020, Page 894-909
        let mut full_path = self.build_base_tag_path(base_array_name);

        // Add element addressing segment
        // Reference: 1756-PM020, Pages 603-611, 870-890
        full_path.extend_from_slice(&self.build_element_id_segment(start_index));

        // Ensure path is word-aligned
        if !full_path.len().is_multiple_of(2) {
            full_path.push(0x00);
        }

        // Path size (in words)
        let path_size = (full_path.len() / 2) as u8;
        cip_request.push(path_size);
        cip_request.extend_from_slice(&full_path);

        // Request Data: Data type, element count, and data
        // Reference: 1756-PM020, Page 855-867 (Writing to Array Element - Full Message)
        cip_request.extend_from_slice(&data_type.to_le_bytes());
        cip_request.extend_from_slice(&element_count.to_le_bytes());
        cip_request.extend_from_slice(data);

        Ok(cip_request)
    }

    /// Reads a UDT through the maintained normal tag-read path.
    ///
    /// **v0.6.0**: Returns `PlcValue::Udt(UdtData)` with `symbol_id` and raw bytes.
    /// Use `UdtData::parse()` with a UDT definition to access individual members.
    ///
    /// The historical "advanced chunked" strategy ladder was retired in 1.2.0
    /// because its alternate request builders were protocol-invalid and could
    /// convert failures into fabricated empty UDT payloads. This compatibility
    /// method now delegates to `read_tag` and returns an error if the target is
    /// not decoded as a UDT. Correct fragmented UDT reads belong to the
    /// capture-gated CODEX-AO wire-format work.
    ///
    /// # Arguments
    ///
    /// * `tag_name` - The name of the UDT tag to read
    ///
    /// # Returns
    ///
    /// `PlcValue::Udt(UdtData)` containing the symbol_id and raw data bytes
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    /// # let mut client = rust_ethernet_ip::EipClient::connect("192.168.1.100:44818").await?;
    /// let udt_value = client.read_udt_chunked("Part_Data").await?;
    /// if let rust_ethernet_ip::PlcValue::Udt(udt_data) = udt_value {
    ///     println!("UDT symbol_id: {}, data size: {} bytes", udt_data.symbol_id, udt_data.data.len());
    ///     // Parse members if needed
    ///     let udt_def = client.get_udt_definition("Part_Data").await?;
    ///     // Convert UdtDefinition to UserDefinedType
    ///     let mut user_def = rust_ethernet_ip::udt::UserDefinedType::new(udt_def.name.clone());
    ///     for member in &udt_def.members {
    ///         user_def.add_member(member.clone());
    ///     }
    ///     let members = udt_data.parse(&user_def)?;
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn read_udt_chunked(&mut self, tag_name: &str) -> crate::error::Result<PlcValue> {
        self.validate_session().await?;

        match self.read_tag(tag_name).await? {
            value @ PlcValue::Udt(_) => Ok(value),
            other => Err(crate::error::EtherNetIpError::DataTypeMismatch {
                expected: "UDT".to_string(),
                actual: format!("{other:?}"),
            }),
        }
    }

    /// Reads a specific UDT member by offset
    ///
    /// This method reads a specific member of a UDT by calculating its offset
    /// and reading only that portion of the UDT.
    ///
    /// # Arguments
    ///
    /// * `udt_name` - The name of the UDT tag
    /// * `member_offset` - The byte offset of the member in the UDT
    /// * `member_size` - The size of the member in bytes
    /// * `data_type` - The data type of the member (0x00C1 for BOOL, 0x00CA for REAL, etc.)
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    /// # let mut client = rust_ethernet_ip::EipClient::connect("192.168.1.100:44818").await?;
    /// let member_value = client.read_udt_member_by_offset("MyUDT", 0, 1, 0x00C1).await?;
    /// println!("Member value: {:?}", member_value);
    /// # Ok(())
    /// # }
    /// ```
    #[deprecated(
        since = "1.2.0",
        note = "offset-based UDT member access indexed the CIP envelope, not the UDT payload; use read_udt_chunked + UdtData::parse or direct member tag reads; removal planned for 2.0"
    )]
    pub async fn read_udt_member_by_offset(
        &mut self,
        _udt_name: &str,
        _member_offset: usize,
        _member_size: usize,
        _data_type: u16,
    ) -> crate::error::Result<PlcValue> {
        Err(crate::error::EtherNetIpError::Unsupported {
            api: "read_udt_member_by_offset",
            reason: "this API indexed the full CIP reply envelope instead of the UDT payload; use read_udt_chunked + UdtData::parse or direct member tag reads instead; removal is planned for 2.0",
        })
    }

    /// Writes a specific UDT member by offset
    ///
    /// This method writes a specific member of a UDT by calculating its offset
    /// and writing only that portion of the UDT.
    ///
    /// # Arguments
    ///
    /// * `udt_name` - The name of the UDT tag
    /// * `member_offset` - The byte offset of the member in the UDT
    /// * `member_size` - The size of the member in bytes
    /// * `data_type` - The data type of the member (0x00C1 for BOOL, 0x00CA for REAL, etc.)
    /// * `value` - The value to write
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    /// # use rust_ethernet_ip::PlcValue;
    /// # let mut client = rust_ethernet_ip::EipClient::connect("192.168.1.100:44818").await?;
    /// client.write_udt_member_by_offset("MyUDT", 0, 1, 0x00C1, PlcValue::Bool(true)).await?;
    /// # Ok(())
    /// # }
    /// ```
    #[deprecated(
        since = "1.2.0",
        note = "offset-based UDT member writes round-tripped CIP envelope bytes as tag data; use write_udt_member/write_udt_array_member or direct member tag writes; removal planned for 2.0"
    )]
    pub async fn write_udt_member_by_offset(
        &mut self,
        _udt_name: &str,
        _member_offset: usize,
        _member_size: usize,
        _data_type: u16,
        _value: PlcValue,
    ) -> crate::error::Result<()> {
        Err(crate::error::EtherNetIpError::Unsupported {
            api: "write_udt_member_by_offset",
            reason: "this API read the full CIP reply envelope and wrote mutated envelope bytes back to the PLC; use write_udt_member/write_udt_array_member or direct member tag writes instead; removal is planned for 2.0",
        })
    }

    /// Gets UDT definition from the PLC
    /// This method queries the PLC for the UDT structure and caches it for future use
    pub async fn get_udt_definition(
        &mut self,
        udt_name: &str,
    ) -> crate::error::Result<UdtDefinition> {
        // Check cache first
        if let Some(cached) = self.udt_manager.lock().await.get_definition(udt_name) {
            return Ok(cached.clone());
        }

        // Get tag attributes to find template ID
        let attributes = self.get_tag_attributes(udt_name).await?;

        // If this is not a UDT, return error
        if attributes.data_type != 0x00A0 {
            return Err(crate::error::EtherNetIpError::Protocol(format!(
                "Tag '{}' is not a UDT (type: {})",
                udt_name, attributes.data_type_name
            )));
        }

        // Get template instance ID
        let template_id = attributes.template_instance_id.ok_or_else(|| {
            crate::error::EtherNetIpError::Protocol(
                "UDT template instance ID not found".to_string(),
            )
        })?;

        let (definition, _structure_size_bytes) = self
            .load_udt_definition_from_template(template_id, udt_name)
            .await?;

        Ok(definition)
    }

    async fn get_udt_definition_by_template_id(
        &mut self,
        template_id: u32,
        udt_name: &str,
    ) -> crate::error::Result<(UdtDefinition, u32)> {
        if let Some(cached) = self.udt_manager.lock().await.get_definition(udt_name) {
            return Ok((cached.clone(), 0));
        }

        self.load_udt_definition_from_template(template_id, udt_name)
            .await
    }

    async fn load_udt_definition_from_template(
        &mut self,
        template_id: u32,
        udt_name: &str,
    ) -> crate::error::Result<(UdtDefinition, u32)> {
        let (template_attributes, template_data) = self.read_udt_template(template_id).await?;
        let template = self.udt_manager.lock().await.parse_udt_template(
            template_id,
            template_attributes.member_count,
            template_attributes.structure_size_bytes,
            &template_data,
        )?;

        let definition = UdtDefinition {
            name: udt_name.to_string(),
            members: template.members,
        };

        self.udt_manager
            .lock()
            .await
            .add_definition(definition.clone());

        Ok((definition, template_attributes.structure_size_bytes))
    }

    /// Gets tag attributes (type, size, dimensions, scope) from the PLC.
    ///
    /// Use this to introspect a tag before reading or writing: discover data type,
    /// size in bytes, array dimensions, and scope (controller vs program). Results
    /// are cached per tag for the lifetime of the client.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let attrs = client.get_tag_attributes("MyTag").await?;
    /// println!("Type: {}, size: {} bytes", attrs.data_type_name, attrs.size);
    /// if !attrs.dimensions.is_empty() {
    ///     println!("Array dimensions: {:?}", attrs.dimensions);
    /// }
    /// ```
    ///
    pub async fn get_tag_attributes(
        &mut self,
        tag_name: &str,
    ) -> crate::error::Result<TagAttributes> {
        // Check cache first
        if let Some(cached) = self.udt_manager.lock().await.get_tag_attributes(tag_name) {
            return Ok(cached.clone());
        }

        // Build CIP request for Get Attribute List (Service 0x03)
        let request = self.build_get_attributes_request(tag_name)?;

        // Send request and get response
        let response = self.send_cip_request(&request).await?;
        let cip_data = self.extract_cip_from_response(&response)?;

        // Parse response
        let attributes = self.parse_attributes_response(tag_name, &cip_data)?;

        // Cache the attributes
        self.udt_manager
            .lock()
            .await
            .add_tag_attributes(attributes.clone());

        Ok(attributes)
    }

    /// Reads UDT template data from the PLC
    async fn read_udt_template(
        &mut self,
        template_id: u32,
    ) -> crate::error::Result<(TemplateAttributes, Vec<u8>)> {
        let template_attributes = self.get_template_attributes(template_id).await?;
        let read_size = template_attributes
            .definition_size_words
            .checked_mul(4)
            .and_then(|bytes| bytes.checked_sub(23))
            .ok_or_else(|| {
                crate::error::EtherNetIpError::Protocol(format!(
                    "Template {} reported invalid definition size {} words",
                    template_id, template_attributes.definition_size_words
                ))
            })?;

        let mut template_data = Vec::with_capacity(read_size as usize);
        let mut offset = 0u32;

        while offset < read_size {
            let chunk_size = (read_size - offset).min(200);
            let request = self.build_read_template_request(template_id, offset, chunk_size)?;
            let response = self.send_cip_request(&request).await?;
            let cip_data = self.extract_cip_from_response(&response)?;
            let (chunk, partial_transfer) = self.parse_template_response_chunk(&cip_data)?;

            if chunk.is_empty() {
                return Err(crate::error::EtherNetIpError::Protocol(format!(
                    "Template {} returned an empty chunk at offset {}",
                    template_id, offset
                )));
            }

            offset = offset.saturating_add(chunk.len() as u32);
            template_data.extend_from_slice(&chunk);

            if !partial_transfer && chunk.len() < chunk_size as usize {
                break;
            }
        }

        Ok((template_attributes, template_data))
    }

    async fn get_template_attributes(
        &mut self,
        template_id: u32,
    ) -> crate::error::Result<TemplateAttributes> {
        let request = self.build_get_template_attributes_request(template_id)?;
        let response = self.send_cip_request(&request).await?;
        let cip_data = self.extract_cip_from_response(&response)?;
        self.parse_template_attributes_response(template_id, &cip_data)
    }

    /// Builds CIP request for Get Attribute List (Service 0x03)
    fn build_get_attributes_request(&self, tag_name: &str) -> crate::error::Result<Vec<u8>> {
        let path = self.build_tag_path(tag_name);
        let request_data = vec![
            0x02, 0x00, // attribute count
            0x01, 0x00, // data type
            0x02, 0x00, // template/symbol instance id
        ];
        let request = CipRequest::new(0x03, path, request_data);
        let mut encoded = BytesMut::new();
        request.encode(&mut encoded)?;
        Ok(encoded.to_vec())
    }

    fn build_get_template_attributes_request(
        &self,
        template_id: u32,
    ) -> crate::error::Result<Vec<u8>> {
        let mut request = Vec::new();
        let template_id = u16::try_from(template_id).map_err(|_| {
            crate::error::EtherNetIpError::Protocol(format!(
                "Template instance {} exceeds 16-bit path encoding",
                template_id
            ))
        })?;

        request.push(0x03);
        request.push(0x03);
        request.extend_from_slice(&[0x20, 0x6C, 0x25, 0x00]);
        request.extend_from_slice(&template_id.to_le_bytes());
        request.extend_from_slice(&[0x04, 0x00]);
        request.extend_from_slice(&[0x01, 0x00]);
        request.extend_from_slice(&[0x02, 0x00]);
        request.extend_from_slice(&[0x04, 0x00]);
        request.extend_from_slice(&[0x05, 0x00]);

        Ok(request)
    }

    /// Builds CIP request for Template Read (Service 0x4C)
    fn build_read_template_request(
        &self,
        template_id: u32,
        read_offset: u32,
        read_size: u32,
    ) -> crate::error::Result<Vec<u8>> {
        let mut request = Vec::new();
        let template_id = u16::try_from(template_id).map_err(|_| {
            crate::error::EtherNetIpError::Protocol(format!(
                "Template instance {} exceeds 16-bit path encoding",
                template_id
            ))
        })?;
        let read_size = u16::try_from(read_size).map_err(|_| {
            crate::error::EtherNetIpError::Protocol(format!(
                "Template read size {} exceeds 16-bit service limit",
                read_size
            ))
        })?;

        request.push(0x4C);
        request.push(0x03);
        request.extend_from_slice(&[0x20, 0x6C, 0x25, 0x00]);
        request.extend_from_slice(&template_id.to_le_bytes());
        request.extend_from_slice(&read_offset.to_le_bytes());
        request.extend_from_slice(&read_size.to_le_bytes());

        Ok(request)
    }

    /// Parses attributes response from CIP
    fn parse_attributes_response(
        &self,
        tag_name: &str,
        response: &[u8],
    ) -> crate::error::Result<TagAttributes> {
        let mut response_bytes = response;
        let response = CipResponse::decode(&mut response_bytes)?;
        if response.service != 0x83 {
            return Err(crate::error::EtherNetIpError::Protocol(format!(
                "Unexpected Get Attribute List reply service: 0x{:02X}",
                response.service
            )));
        }

        if response.status != 0 {
            return Err(crate::error::EtherNetIpError::Protocol(format!(
                "Get Attribute List for '{}' failed: {}",
                tag_name,
                self.get_cip_error_message(response.status)
            )));
        }

        if response.data.len() < 2 {
            return Err(crate::error::EtherNetIpError::Protocol(
                "Attributes response missing attribute count".to_string(),
            ));
        }

        let attr_count = u16::from_le_bytes([response.data[0], response.data[1]]) as usize;
        let mut offset = 2;
        let mut data_type = None;
        let mut template_instance_id = None;
        let mut attr_errors = Vec::new();

        for _ in 0..attr_count {
            if response.data.len() < offset + 4 {
                return Err(crate::error::EtherNetIpError::Protocol(
                    "Attributes response truncated before attribute record header".to_string(),
                ));
            }

            let attr_id = u16::from_le_bytes([response.data[offset], response.data[offset + 1]]);
            let attr_status =
                u16::from_le_bytes([response.data[offset + 2], response.data[offset + 3]]);
            offset += 4;

            if attr_status != 0 {
                attr_errors.push(format!("attr {attr_id} status 0x{attr_status:04X}"));
                continue;
            }

            match attr_id {
                0x0001 => {
                    if response.data.len() < offset + 2 {
                        return Err(crate::error::EtherNetIpError::Protocol(
                            "Attributes response truncated in data type value".to_string(),
                        ));
                    }
                    data_type = Some(u16::from_le_bytes([
                        response.data[offset],
                        response.data[offset + 1],
                    ]));
                    offset += 2;
                }
                0x0002 => {
                    if response.data.len() < offset + 4 {
                        return Err(crate::error::EtherNetIpError::Protocol(
                            "Attributes response truncated in instance id value".to_string(),
                        ));
                    }
                    template_instance_id = Some(u32::from_le_bytes([
                        response.data[offset],
                        response.data[offset + 1],
                        response.data[offset + 2],
                        response.data[offset + 3],
                    ]));
                    offset += 4;
                }
                _ => {
                    return Err(crate::error::EtherNetIpError::Protocol(format!(
                        "Unexpected attribute id {attr_id} in Get Attribute List response"
                    )));
                }
            }
        }

        let data_type = data_type.ok_or_else(|| {
            crate::error::EtherNetIpError::Protocol(format!(
                "Get Attribute List for '{}' did not return data type{}",
                tag_name,
                if attr_errors.is_empty() {
                    String::new()
                } else {
                    format!(" ({})", attr_errors.join(", "))
                }
            ))
        })?;
        let size = Self::data_type_size(data_type);

        // Create attributes
        let attributes = TagAttributes {
            name: tag_name.to_string(),
            data_type,
            data_type_name: self.get_data_type_name(data_type),
            dimensions: Vec::new(), // Would need additional parsing
            permissions: udt::TagPermissions::ReadWrite, // Default assumption
            scope: if tag_name.contains(':') {
                let parts: Vec<&str> = tag_name.split(':').collect();
                if parts.len() >= 2 {
                    udt::TagScope::Program(parts[0].to_string())
                } else {
                    udt::TagScope::Controller
                }
            } else {
                udt::TagScope::Controller
            },
            template_instance_id,
            size,
        };

        Ok(attributes)
    }

    fn data_type_size(data_type: u16) -> u32 {
        match data_type {
            0x00C1 | 0x00C2 | 0x00C6 => 1,
            0x00C3 | 0x00C7 => 2,
            0x00C4 | 0x00C8 | 0x00CA => 4,
            0x00C5 | 0x00C9 | 0x00CB => 8,
            0x00CE => 88,
            _ => 4,
        }
    }

    fn parse_template_attributes_response(
        &self,
        template_id: u32,
        response: &[u8],
    ) -> crate::error::Result<TemplateAttributes> {
        if response.len() < 4 {
            return Err(crate::error::EtherNetIpError::Protocol(
                "Template attribute response too short".to_string(),
            ));
        }

        let general_status = response[2];
        if general_status != 0x00 {
            return Err(crate::error::EtherNetIpError::Protocol(format!(
                "Template {} attribute read failed: {}",
                template_id,
                self.get_cip_error_message(general_status)
            )));
        }

        let additional_status_words = response[3] as usize;
        let mut offset = 4 + additional_status_words * 2;
        if response.len() < offset + 2 {
            return Err(crate::error::EtherNetIpError::Protocol(
                "Template attribute response missing attribute count".to_string(),
            ));
        }

        let attr_count = u16::from_le_bytes([response[offset], response[offset + 1]]) as usize;
        offset += 2;

        let mut attributes = TemplateAttributes {
            structure_handle: 0,
            member_count: 0,
            definition_size_words: 0,
            structure_size_bytes: 0,
        };

        for _ in 0..attr_count {
            if response.len() < offset + 4 {
                return Err(crate::error::EtherNetIpError::Protocol(
                    "Template attribute response truncated".to_string(),
                ));
            }

            let attr_id = u16::from_le_bytes([response[offset], response[offset + 1]]);
            let attr_status = u16::from_le_bytes([response[offset + 2], response[offset + 3]]);
            offset += 4;

            if attr_status != 0 {
                return Err(crate::error::EtherNetIpError::Protocol(format!(
                    "Template {} attribute {} read returned status 0x{:04X}",
                    template_id, attr_id, attr_status
                )));
            }

            match attr_id {
                1 => {
                    if response.len() < offset + 2 {
                        return Err(crate::error::EtherNetIpError::Protocol(
                            "Template attribute 1 missing value".to_string(),
                        ));
                    }
                    attributes.structure_handle =
                        u16::from_le_bytes([response[offset], response[offset + 1]]);
                    offset += 2;
                }
                2 => {
                    if response.len() < offset + 2 {
                        return Err(crate::error::EtherNetIpError::Protocol(
                            "Template attribute 2 missing value".to_string(),
                        ));
                    }
                    attributes.member_count =
                        u16::from_le_bytes([response[offset], response[offset + 1]]);
                    offset += 2;
                }
                4 => {
                    if response.len() < offset + 4 {
                        return Err(crate::error::EtherNetIpError::Protocol(
                            "Template attribute 4 missing value".to_string(),
                        ));
                    }
                    attributes.definition_size_words = u32::from_le_bytes([
                        response[offset],
                        response[offset + 1],
                        response[offset + 2],
                        response[offset + 3],
                    ]);
                    offset += 4;
                }
                5 => {
                    if response.len() < offset + 4 {
                        return Err(crate::error::EtherNetIpError::Protocol(
                            "Template attribute 5 missing value".to_string(),
                        ));
                    }
                    attributes.structure_size_bytes = u32::from_le_bytes([
                        response[offset],
                        response[offset + 1],
                        response[offset + 2],
                        response[offset + 3],
                    ]);
                    offset += 4;
                }
                _ => {
                    return Err(crate::error::EtherNetIpError::Protocol(format!(
                        "Unexpected template attribute {} in response",
                        attr_id
                    )));
                }
            }
        }

        if attributes.definition_size_words == 0 {
            return Err(crate::error::EtherNetIpError::Protocol(format!(
                "Template {} reported zero definition size",
                template_id
            )));
        }

        Ok(attributes)
    }

    fn parse_template_response_chunk(
        &self,
        response: &[u8],
    ) -> crate::error::Result<(Vec<u8>, bool)> {
        if response.len() < 4 {
            return Err(crate::error::EtherNetIpError::Protocol(
                "Template response too short".to_string(),
            ));
        }

        let general_status = response[2];
        let partial_transfer = general_status == 0x06;
        if general_status != 0x00 && !partial_transfer {
            return Err(crate::error::EtherNetIpError::Protocol(format!(
                "Template read failed: {}",
                self.get_cip_error_message(general_status)
            )));
        }

        let additional_status_words = response[3] as usize;
        let data_start = 4 + additional_status_words * 2;
        if data_start > response.len() {
            return Err(crate::error::EtherNetIpError::Protocol(
                "Template response missing payload".to_string(),
            ));
        }

        Ok((response[data_start..].to_vec(), partial_transfer))
    }

    /// Gets human-readable data type name
    fn get_data_type_name(&self, data_type: u16) -> String {
        match data_type {
            0x00C1 => "BOOL".to_string(),
            0x00C2 => "SINT".to_string(),
            0x00C3 => "INT".to_string(),
            0x00C4 => "DINT".to_string(),
            0x00C5 => "LINT".to_string(),
            0x00C6 => "USINT".to_string(),
            0x00C7 => "UINT".to_string(),
            0x00C8 => "UDINT".to_string(),
            0x00C9 => "ULINT".to_string(),
            0x00CA => "REAL".to_string(),
            0x00CB => "LREAL".to_string(),
            0x00CE => "STRING".to_string(),
            0x00A0 => "UDT".to_string(),
            _ => format!("UNKNOWN(0x{:04X})", data_type),
        }
    }

    /// Builds CIP request for tag list discovery starting from a specific symbol instance.
    fn build_tag_list_request_from_instance(
        &self,
        start_instance: u32,
    ) -> crate::error::Result<Vec<u8>> {
        let start_instance = u16::try_from(start_instance).map_err(|_| {
            crate::error::EtherNetIpError::Protocol(format!(
                "Tag discovery start instance {} exceeds 16-bit Symbol Object range",
                start_instance
            ))
        })?;
        let mut request = vec![
            // Service: Get Instance Attribute List (0x55)
            0x55, // Path size: 3 words (6 bytes)
            0x03, // Path: Symbol Object (Class 0x6B), start instance
            0x20, 0x6B, 0x25, 0x00,
        ];
        request.extend_from_slice(&start_instance.to_le_bytes());

        // Attribute count
        request.extend_from_slice(&[0x02, 0x00]);

        // Attribute 1: Symbol Name (0x01)
        request.extend_from_slice(&[0x01, 0x00]);

        // Attribute 2: Symbol Type (0x02)
        request.extend_from_slice(&[0x02, 0x00]);

        Ok(request)
    }

    /// Builds CIP request for program-scoped tag list discovery, resuming the
    /// Symbol Object enumeration at `start_instance`.
    ///
    /// A program with more tags than fit in one reply answers `0x06`
    /// (partial transfer); the caller resumes from `last_instance_id + 1`, the
    /// same paging contract as [`Self::build_tag_list_request_from_instance`].
    fn build_program_tag_list_request(
        &self,
        program_name: &str,
        start_instance: u32,
    ) -> crate::error::Result<Vec<u8>> {
        let start_instance = u16::try_from(start_instance).map_err(|_| {
            crate::error::EtherNetIpError::Protocol(format!(
                "Program tag discovery start instance {} exceeds 16-bit Symbol Object range",
                start_instance
            ))
        })?;
        let scoped_program = format!("Program:{}", program_scope_name(program_name));

        let mut path = Vec::new();
        path.push(0x91);
        path.push(scoped_program.len() as u8);
        path.extend_from_slice(scoped_program.as_bytes());
        if !path.len().is_multiple_of(2) {
            path.push(0x00);
        }
        // Symbol Object (Class 0x6B), 16-bit start instance.
        path.extend_from_slice(&[0x20, 0x6B, 0x25, 0x00]);
        path.extend_from_slice(&start_instance.to_le_bytes());

        let path_words = u8::try_from(path.len() / 2).map_err(|_| {
            crate::error::EtherNetIpError::Protocol(format!(
                "Program tag discovery path too long for '{}'",
                program_name
            ))
        })?;

        let mut request = vec![
            // Service: Get Instance Attribute List (0x55)
            0x55, path_words,
        ];
        request.extend_from_slice(&path);

        // Attribute count
        request.extend_from_slice(&[0x02, 0x00]); // 2 attributes

        // Attribute 1: Symbol Name (0x01)
        request.extend_from_slice(&[0x01, 0x00]);

        // Attribute 2: Data Type (0x02)
        request.extend_from_slice(&[0x02, 0x00]);

        Ok(request)
    }

    /// Parses one page of tag discovery results from a Get Instance Attribute List response.
    ///
    /// `scope` is the scope the REQUEST was addressed to, and it is stamped onto
    /// every tag of the page. The Symbol Object reply carries no scope field --
    /// a program-scoped enumeration and a controller-scoped one are
    /// byte-identical on the wire -- so the scope is knowledge the CALLER holds
    /// and the parser cannot recover. Hardcoding it here made every tag returned
    /// by [`Self::discover_program_tags`] claim `TagScope::Controller`.
    fn parse_tag_list_response_page(
        &self,
        response: &[u8],
        scope: udt::TagScope,
    ) -> crate::error::Result<TagListPage> {
        if response.len() < 4 {
            return Err(crate::error::EtherNetIpError::Protocol(
                "Tag list response too short".to_string(),
            ));
        }

        let general_status = response[2];
        let partial_transfer = general_status == 0x06;
        if general_status != 0x00 && !partial_transfer {
            return Err(crate::error::EtherNetIpError::Protocol(format!(
                "Tag discovery failed: {}. Some PLCs may not support tag discovery. Try reading tags directly by name.",
                self.get_cip_error_message(general_status)
            )));
        }

        let additional_status_words = response[3] as usize;
        let mut offset = 4 + additional_status_words * 2;
        if response.len() == offset {
            return Ok(TagListPage {
                tags: Vec::new(),
                last_instance_id: None,
                partial_transfer: false,
            });
        }
        if response.len() < offset + 4 {
            return Err(crate::error::EtherNetIpError::Protocol(
                "Tag list response missing first entry".to_string(),
            ));
        }
        let mut tags = Vec::new();
        let mut last_instance_id = None;

        while offset + 8 <= response.len() {
            let instance_id = u32::from_le_bytes([
                response[offset],
                response[offset + 1],
                response[offset + 2],
                response[offset + 3],
            ]);
            last_instance_id = Some(instance_id);
            offset += 4;

            let name_length = u16::from_le_bytes([response[offset], response[offset + 1]]) as usize;
            offset += 2;

            if offset
                .checked_add(name_length)
                .is_none_or(|end| end > response.len())
            {
                break;
            }

            let name_bytes = &response[offset..offset + name_length];
            let tag_name = String::from_utf8_lossy(name_bytes).to_string();
            offset += name_length;

            if offset + 2 > response.len() {
                break;
            }

            let raw_tag_type = u16::from_le_bytes([response[offset], response[offset + 1]]);
            offset += 2;

            // Symbol list includes controller/program/system tags. Keep user-visible names only.
            if tag_name.starts_with("__") || tag_name.contains(':') {
                continue;
            }

            let array_dims = ((raw_tag_type & 0x6000) >> 13) as usize;
            let is_structure = (raw_tag_type & 0x8000) != 0;
            let reserved = (raw_tag_type & 0x1000) != 0;
            let type_param = raw_tag_type & 0x0FFF;
            let is_user_atomic =
                !is_structure && !reserved && (0x0001..=0x00FF).contains(&type_param);
            let is_user_structure =
                is_structure && !reserved && (0x0100..=0x0EFF).contains(&type_param);

            if !is_user_atomic && !is_user_structure {
                continue;
            }

            let data_type = if is_structure {
                0x00A0
            } else if (raw_tag_type & 0x00FF) == 0x00C1 {
                0x00C1
            } else {
                type_param
            };

            let template_instance_id = if is_structure && !reserved {
                Some(type_param as u32)
            } else {
                None
            };

            tags.push(TagAttributes {
                name: tag_name,
                data_type,
                data_type_name: if is_structure {
                    "UDT".to_string()
                } else {
                    self.get_data_type_name(data_type)
                },
                dimensions: vec![0; array_dims],
                permissions: udt::TagPermissions::ReadWrite,
                scope: scope.clone(),
                template_instance_id,
                size: 0,
            });
        }

        Ok(TagListPage {
            tags,
            last_instance_id,
            partial_transfer,
        })
    }

    /// Negotiates packet size with the PLC
    /// This method queries the PLC for its maximum supported packet size
    /// and updates the client's configuration accordingly
    async fn negotiate_packet_size(&mut self) -> crate::error::Result<()> {
        // Build CIP request for Get Attribute List (Service 0x03)
        // Query the Message Router object (Class 0x02, Instance 1) for max packet size
        let mut request = vec![
            0x03, // Service: Get Attribute List
            0x02, // Path size: 2 words (4 bytes)
            0x20, 0x02, // 8-bit class segment: Class 0x02 (Message Router)
            0x24, 0x01, // 8-bit instance segment: Instance 1
        ];
        // Attribute count
        request.extend_from_slice(&[0x01, 0x00]); // 1 attribute
        // Attribute: Max Packet Size (attribute 4 on the Message Router)
        request.extend_from_slice(&[0x04, 0x00]);

        // Send request and extract CIP from CPF response
        let response = self.send_cip_request(&request).await?;
        let cip_data = self.extract_cip_from_response(&response)?;

        // CIP response format: [Service Reply][Reserved][Status][AddtlStatusSize][...data...]
        // For Get Attribute List reply: after the 4-byte CIP header, we get:
        // [AttrCount(2)] [AttrID(2)] [Status(2)] [Value(2)]
        // The attribute value for max packet size is a UINT (2 bytes)
        if cip_data.len() >= 12 && cip_data[2] == 0x00 {
            // Skip CIP header (4 bytes) + attr count (2) + attr id (2) + attr status (2) = 10
            let max_packet_size = u16::from_le_bytes([cip_data[10], cip_data[11]]) as u32;

            // Update client's max packet size (with reasonable limits)
            self.max_packet_size
                .store(max_packet_size.clamp(504, 4000), Ordering::Relaxed);
            tracing::debug!("Negotiated packet size: {} bytes", self.max_packet_size());
        } else {
            // If negotiation fails, use default size
            self.max_packet_size.store(4000, Ordering::Relaxed);
            tracing::debug!(
                "Using default packet size: {} bytes",
                self.max_packet_size()
            );
        }

        Ok(())
    }

    /// Writes a value to a PLC tag
    ///
    /// This method automatically determines the best communication method based on the data type:
    /// - STRING values use unconnected explicit messaging with proper AB STRING format
    /// - Other data types use standard unconnected messaging
    ///
    /// **v0.6.0**: For UDT tags, pass `PlcValue::Udt(UdtData)`. The `symbol_id` must be set
    /// (typically obtained by reading the UDT first). If `symbol_id` is 0, the method will
    /// attempt to read tag attributes to get the symbol_id automatically.
    ///
    /// # Arguments
    ///
    /// * `tag_name` - The name of the tag to write to
    /// * `value` - The value to write. For UDTs, use `PlcValue::Udt(UdtData)`.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    /// # let mut client = rust_ethernet_ip::EipClient::connect("192.168.1.100:44818").await?;
    /// use rust_ethernet_ip::{PlcValue, UdtData};
    ///
    /// // Write simple types
    /// client.write_tag("Counter", PlcValue::Dint(42)).await?;
    /// client.write_tag("Message", PlcValue::String("Hello PLC".to_string())).await?;
    ///
    /// // Write UDT (v0.6.0: read first to get symbol_id, then modify and write)
    /// let udt_value = client.read_tag("MyUDT").await?;
    /// if let PlcValue::Udt(mut udt_data) = udt_value {
    ///     let udt_def = client.get_udt_definition("MyUDT").await?;
    ///     // Convert UdtDefinition to UserDefinedType
    ///     let mut user_def = rust_ethernet_ip::udt::UserDefinedType::new(udt_def.name.clone());
    ///     for member in &udt_def.members {
    ///         user_def.add_member(member.clone());
    ///     }
    ///     let mut members = udt_data.parse(&user_def)?;
    ///     members.insert("Member1".to_string(), PlcValue::Dint(100));
    ///     let modified_udt = UdtData::from_hash_map(&members, &user_def, udt_data.symbol_id)?;
    ///     client.write_tag("MyUDT", PlcValue::Udt(modified_udt)).await?;
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn write_tag(&mut self, tag_name: &str, value: PlcValue) -> crate::error::Result<()> {
        tracing::debug!(
            "Writing '{}' to tag '{}'",
            match &value {
                PlcValue::String(s) => format!("\"{s}\""),
                _ => format!("{value:?}"),
            },
            tag_name
        );

        // For UDT writes, ensure we have a valid symbol_id
        // As noted by the contributor: "to write a UDT, you typically need to read it first to get the symbol_id"
        let value = if let PlcValue::Udt(udt_data) = &value {
            if udt_data.symbol_id == 0 {
                tracing::debug!("[UDT WRITE] symbol_id is 0, reading tag to get symbol_id");
                // Read tag attributes to get symbol_id
                let attributes = self.get_tag_attributes(tag_name).await?;
                let symbol_id = attributes.template_instance_id.ok_or_else(|| {
                    crate::error::EtherNetIpError::Protocol(
                        "UDT template instance ID not found. Cannot write UDT without symbol_id."
                            .to_string(),
                    )
                })? as i32;

                // Create new UdtData with the correct symbol_id
                PlcValue::Udt(UdtData {
                    symbol_id,
                    data: udt_data.data.clone(),
                })
            } else {
                value
            }
        } else {
            value
        };

        if let Some((base_path, bit_index)) = self.parse_bit_access(tag_name) {
            return match value {
                PlcValue::Bool(bit_value) => {
                    self.write_bit_base_direct(&base_path, bit_index, bit_value)
                        .await
                }
                other => Err(crate::error::EtherNetIpError::DataTypeMismatch {
                    expected: "BOOL".to_string(),
                    actual: format!("{:?}", other),
                }),
            };
        }

        // Check if this is simple array element access (e.g., "ArrayName[0]").
        // Member paths such as "ArrayName[0].Member" must use TagPath so the
        // suffix is preserved instead of writing the whole element/base path.
        if let Some((base_name, index)) = self.parse_array_element_access(tag_name) {
            if !self.has_member_suffix_after_first_array_index(tag_name) {
                tracing::debug!(
                    "Detected array element write: {}[{}], using workaround",
                    base_name,
                    index
                );
                return self
                    .write_array_element_workaround(&base_name, index, value)
                    .await;
            }

            tracing::debug!(
                "Array element '{}[{}]' has member access, using TagPath::parse()",
                base_name,
                index
            );
        }

        if let PlcValue::Bool(_) = value
            && let Some((parent_path, index)) = self.parse_final_array_element_access(tag_name)
            && self.detect_bool_array_path(&parent_path).await?
        {
            return self
                .write_bool_array_element_workaround(&parent_path, index, value)
                .await;
        }

        self.write_tag_direct(tag_name, &value).await
    }

    async fn write_tag_direct(
        &mut self,
        tag_name: &str,
        value: &PlcValue,
    ) -> crate::error::Result<()> {
        // STRING writes must carry the *target tag's* real structure handle. The built-in Logix
        // `STRING` type (handle 0x0FCE) is the common case, but users routinely define their own
        // string types with a custom name/length (e.g. `Str82`, `Str400`); each has its own
        // structure handle, and the built-in handle is rejected with CIP 0x2107. Try the standard
        // encoding first — it is the fast path and is all the simulator models — and on a type
        // mismatch discover the target's real handle + structure size and retry. A value longer
        // than the built-in 82-byte capacity can only be a custom type, so skip straight to the
        // handle-aware path. See docs/agents/notes/ab-firmware-quirks.md (STRING Members).
        if let PlcValue::String(text) = value {
            if text.len() <= values::STANDARD_STRING_DATA_LEN {
                match self.write_tag_standard(tag_name, value).await {
                    Ok(()) => return Ok(()),
                    Err(error) if service_layer::is_2107_type_mismatch(&error) => {
                        return self.write_string_handle_aware(tag_name, text).await;
                    }
                    Err(error) => return Err(error),
                }
            }
            return self.write_string_handle_aware(tag_name, text).await;
        }
        self.write_tag_standard(tag_name, value).await
    }

    /// Single-request write ceiling for unconnected messaging. A request above this is rejected
    /// by the controller with encapsulation status 0x03, so the string path uses CIP
    /// fragmentation. Measured on CompactLogix 5069-L330ERM fw38 (494 bytes OK, 498 rejected).
    const SINGLE_PACKET_WRITE_LIMIT: usize = 494;

    /// Writes a Logix STRING using the target tag's real structure handle and structure size,
    /// discovered by reading the tag first. Handles built-in `STRING` and custom string types
    /// (own name/length) uniformly. Structures larger than one CIP packet use Write Tag
    /// Fragmented.
    async fn write_string_handle_aware(
        &mut self,
        tag_name: &str,
        value: &str,
    ) -> crate::error::Result<()> {
        // Discover the target's structure handle and total structure size (LEN + DATA + pad).
        let (handle, struct_size) = match self.read_tag(tag_name).await? {
            PlcValue::String(_) => (
                values::STANDARD_STRING_HANDLE,
                values::STANDARD_STRING_PAYLOAD_LEN,
            ),
            PlcValue::Udt(udt) if udt.data.len() >= 6 => (
                u16::from_le_bytes([udt.data[0], udt.data[1]]),
                udt.data.len() - 2,
            ),
            other => {
                return Err(EtherNetIpError::DataTypeMismatch {
                    expected: "STRING structure".to_string(),
                    actual: format!("{other:?}"),
                });
            }
        };

        // The value occupies the DATA region after the 4-byte LEN field.
        let capacity = struct_size.saturating_sub(4);
        if value.len() > capacity {
            return Err(EtherNetIpError::StringTooLong {
                max_length: capacity,
                actual_length: value.len(),
            });
        }

        // Structure payload: LEN (u32 LE) + value bytes + zero fill to the structure size.
        let mut payload = vec![0u8; struct_size];
        payload[0..4].copy_from_slice(&(value.len() as u32).to_le_bytes());
        payload[4..4 + value.len()].copy_from_slice(value.as_bytes());

        // Request data: structure type marker (0x02A0) + real handle + element count + payload.
        let mut data = Vec::with_capacity(6 + payload.len());
        data.extend_from_slice(&values::AB_UDT.to_le_bytes());
        data.extend_from_slice(&handle.to_le_bytes());
        data.extend_from_slice(&[0x01, 0x00]);
        data.extend_from_slice(&payload);

        let path = self.build_tag_path(tag_name);
        let request = CipRequest::new(WRITE_TAG, path, data);
        let mut cip_request = BytesMut::new();
        request.encode(&mut cip_request)?;

        if cip_request.len() > Self::SINGLE_PACKET_WRITE_LIMIT {
            return self
                .write_string_fragmented(tag_name, handle, &payload)
                .await;
        }

        let response = self.send_cip_request(&cip_request).await?;
        let cip_response = self.extract_cip_from_response(&response)?;
        self.check_cip_error(&cip_response)?;
        Ok(())
    }

    async fn write_string_fragmented(
        &mut self,
        tag_name: &str,
        handle: u16,
        payload: &[u8],
    ) -> crate::error::Result<()> {
        let max_fragment = self.max_write_fragment_payload_len(tag_name, handle)?;
        let mut offset = 0usize;

        while offset < payload.len() {
            let end = usize::min(offset + max_fragment, payload.len());
            let request = self.build_write_fragmented_request(
                tag_name,
                handle,
                offset as u32,
                &payload[offset..end],
            )?;
            let response = self.send_cip_request(&request).await?;
            let cip_response = self.extract_cip_from_response(&response)?;
            if cip_response.first().copied() != Some(WRITE_TAG_FRAGMENTED_REPLY) {
                return Err(EtherNetIpError::Protocol(format!(
                    "Unexpected Write Tag Fragmented reply service: 0x{:02X}",
                    cip_response.first().copied().unwrap_or(0)
                )));
            }
            self.check_cip_error(&cip_response)?;
            offset = end;
        }

        Ok(())
    }

    async fn write_tag_standard(
        &mut self,
        tag_name: &str,
        value: &PlcValue,
    ) -> crate::error::Result<()> {
        let cip_request = self.build_write_request(tag_name, value)?;

        let response = self.send_cip_request(&cip_request).await?;

        // Check write response for errors - need to extract CIP response first
        let cip_response = self.extract_cip_from_response(&response)?;

        if cip_response.len() < 3 {
            return Err(EtherNetIpError::Protocol(
                "Write response too short".to_string(),
            ));
        }

        let service_reply = cip_response[0]; // Should be 0xCD (0x4D + 0x80) for Write Tag reply
        let general_status = cip_response[2]; // CIP status code

        tracing::trace!(
            "Write response - Service: 0x{:02X}, Status: 0x{:02X}",
            service_reply,
            general_status
        );

        // Check for errors (including extended errors)
        if let Err(e) = self.check_cip_error(&cip_response) {
            tracing::error!("[WRITE] CIP Error: {}", e);
            return Err(e);
        }

        tracing::info!("Write operation completed successfully");
        Ok(())
    }

    /// Builds a CIP Write Tag Service request
    ///
    /// This creates the CIP packet for writing a value to a tag.
    /// The request includes the service code, tag path, data type, and value.
    ///
    /// For UDT writes, the data type must be Structure Tag Type (0x02A0 + Structure Handle).
    /// The Structure Handle is the template_instance_id (symbol_id) from Template Attribute 1.
    ///
    /// Reference: 1756-PM020, Page 1080 (UDT Data Layout Considerations)
    fn build_write_request(
        &self,
        tag_name: &str,
        value: &PlcValue,
    ) -> crate::error::Result<Vec<u8>> {
        tracing::debug!("Building write request for tag: '{}'", tag_name);

        // Use the same path building logic as read operations
        let path = self.build_tag_path(tag_name);

        if let PlcValue::String(string_value) = value
            && string_value.len() > values::STANDARD_STRING_DATA_LEN
        {
            return Err(EtherNetIpError::StringTooLong {
                max_length: values::STANDARD_STRING_DATA_LEN,
                actual_length: string_value.len(),
            });
        }

        let mut data = BytesMut::new();
        data.extend_from_slice(&values::write_data_type_bytes(value));
        data.extend_from_slice(&[0x01, 0x00]); // Element count: 1
        values::encode_payload(value, &mut data);

        let request = CipRequest::new(WRITE_TAG, path, data.to_vec());
        let mut cip_request = BytesMut::new();
        request.encode(&mut cip_request)?;

        tracing::trace!(
            "Built CIP write request ({} bytes): {:02X?}",
            cip_request.len(),
            cip_request
        );
        Ok(cip_request.to_vec())
    }

    fn build_read_fragmented_request(
        &self,
        tag_name: &str,
        element_count: u16,
        byte_offset: u32,
    ) -> crate::error::Result<Vec<u8>> {
        let mut data = Vec::with_capacity(6);
        data.extend_from_slice(&element_count.to_le_bytes());
        data.extend_from_slice(&byte_offset.to_le_bytes());
        let request = CipRequest::new(READ_TAG_FRAGMENTED, self.build_tag_path(tag_name), data);
        let mut cip_request = BytesMut::new();
        request.encode(&mut cip_request)?;
        Ok(cip_request.to_vec())
    }

    fn build_write_fragmented_request(
        &self,
        tag_name: &str,
        handle: u16,
        byte_offset: u32,
        fragment: &[u8],
    ) -> crate::error::Result<Vec<u8>> {
        let mut data = Vec::with_capacity(10 + fragment.len());
        data.extend_from_slice(&values::AB_UDT.to_le_bytes());
        data.extend_from_slice(&handle.to_le_bytes());
        data.extend_from_slice(&1u16.to_le_bytes());
        data.extend_from_slice(&byte_offset.to_le_bytes());
        data.extend_from_slice(fragment);
        let request = CipRequest::new(WRITE_TAG_FRAGMENTED, self.build_tag_path(tag_name), data);
        let mut cip_request = BytesMut::new();
        request.encode(&mut cip_request)?;
        Ok(cip_request.to_vec())
    }

    fn max_write_fragment_payload_len(
        &self,
        tag_name: &str,
        handle: u16,
    ) -> crate::error::Result<usize> {
        let empty_request = self.build_write_fragmented_request(tag_name, handle, 0, &[])?;
        if empty_request.len() >= Self::SINGLE_PACKET_WRITE_LIMIT {
            return Err(EtherNetIpError::Protocol(format!(
                "Write Tag Fragmented request for '{tag_name}' has {} bytes of overhead, exceeding the {}-byte single-packet limit before payload",
                empty_request.len(),
                Self::SINGLE_PACKET_WRITE_LIMIT
            )));
        }

        Ok(Self::SINGLE_PACKET_WRITE_LIMIT - empty_request.len())
    }

    /// Builds the initial controller-scoped Symbol Object enumeration request.
    pub fn build_list_tags_request(&self) -> Vec<u8> {
        tracing::debug!("Building list tags request");

        // Build path array for Symbol Object Class (0x6B)
        let path_array = vec![
            // Class segment: Symbol Object Class (0x6B)
            0x20, // Class segment identifier
            0x6B, // Symbol Object Class
            // Instance segment: Start at Instance 0
            0x25, // Instance segment identifier with 0x00
            0x00, 0x00, 0x00,
        ];

        // Request data: 2 Attributes - Attribute 1 and Attribute 2
        let request_data = vec![0x02, 0x00, 0x01, 0x00, 0x02, 0x00];

        // Build CIP Message Router request
        let request = CipRequest::new(0x55, path_array, request_data);
        let mut cip_request = BytesMut::new();
        request
            .encode(&mut cip_request)
            .expect("list-tags request path is static and valid");

        tracing::trace!(
            "Built CIP list tags request ({} bytes): {:02X?}",
            cip_request.len(),
            cip_request
        );

        cip_request.to_vec()
    }

    /// Gets a human-readable error message for a CIP status code
    ///
    /// # Arguments
    ///
    /// * `status` - The CIP status code to look up
    ///
    /// # Returns
    ///
    /// A string describing the error
    /// Parses extended CIP error codes from response data
    ///
    /// Additional status is signaled by the additional-status size field.
    /// Format: [0]=service, [1]=reserved, [2]=general_status, [3]=additional_status_size (words), [4-5]=first extended status word
    fn parse_extended_error(&self, cip_data: &[u8]) -> crate::error::Result<String> {
        if cip_data.len() < 4 {
            return Err(EtherNetIpError::Protocol(
                "CIP response too short for additional-status check".to_string(),
            ));
        }

        let additional_status_size = cip_data[3] as usize; // Size in words
        if additional_status_size == 0 {
            return Ok("Extended error (no additional status)".to_string());
        }

        let expected_len = 4 + (additional_status_size * 2);
        if cip_data.len() < expected_len {
            return Err(EtherNetIpError::Protocol(format!(
                "Additional-status response truncated: expected {expected_len} bytes, got {}",
                cip_data.len()
            )));
        }

        let extended_error_code = u16::from_le_bytes([cip_data[4], cip_data[5]]);
        let error_msg = match extended_error_code {
            0x0001 => "Connection failure (extended)".to_string(),
            0x0002 => "Resource unavailable (extended)".to_string(),
            0x0003 => "Invalid parameter value (extended)".to_string(),
            0x0004 => "Path segment error (extended)".to_string(),
            0x0005 => "Path destination unknown (extended)".to_string(),
            0x0006 => "Partial transfer (extended)".to_string(),
            0x0007 => "Connection lost (extended)".to_string(),
            0x0008 => "Service not supported (extended)".to_string(),
            0x0009 => "Invalid attribute value (extended)".to_string(),
            0x000A => "Attribute list error (extended)".to_string(),
            0x000B => "Already in requested mode/state (extended)".to_string(),
            0x000C => "Object state conflict (extended)".to_string(),
            0x000D => "Object already exists (extended)".to_string(),
            0x000E => "Attribute not settable (extended)".to_string(),
            0x000F => "Privilege violation (extended)".to_string(),
            0x0010 => "Device state conflict (extended)".to_string(),
            0x0011 => "Reply data too large (extended)".to_string(),
            0x0012 => "Fragmentation of a primitive value (extended)".to_string(),
            0x0013 => "Not enough data (extended)".to_string(),
            0x0014 => "Attribute not supported (extended)".to_string(),
            0x0015 => "Too much data (extended)".to_string(),
            0x0016 => "Object does not exist (extended)".to_string(),
            0x0017 => "Service fragmentation sequence not in progress (extended)".to_string(),
            0x0018 => "No stored attribute data (extended)".to_string(),
            0x0019 => "Store operation failure (extended)".to_string(),
            0x001A => "Routing failure, request packet too large (extended)".to_string(),
            0x001B => "Routing failure, response packet too large (extended)".to_string(),
            0x001C => "Missing attribute list entry data (extended)".to_string(),
            0x001D => "Invalid attribute value list (extended)".to_string(),
            0x001E => "Embedded service error (extended)".to_string(),
            0x001F => "Vendor specific error (extended)".to_string(),
            0x0020 => "Invalid parameter (extended)".to_string(),
            0x0021 => "Write-once value or medium already written (extended)".to_string(),
            0x0022 => "Invalid reply received (extended)".to_string(),
            0x0023 => "Buffer overflow (extended)".to_string(),
            0x0024 => "Invalid message format (extended)".to_string(),
            0x0025 => "Key failure in path (extended)".to_string(),
            0x0026 => "Path size invalid (extended)".to_string(),
            0x0027 => "Unexpected attribute in list (extended)".to_string(),
            0x0028 => "Invalid member ID (extended)".to_string(),
            0x0029 => "Member not settable (extended)".to_string(),
            0x002A => "Group 2 only server general failure (extended)".to_string(),
            0x002B => "Unknown Modbus error (extended)".to_string(),
            0x002C => "Attribute not gettable (extended)".to_string(),
            0x2107 => format!(
                "Read/Write Tag data-type mismatch extended error: 0x{extended_error_code:04X}. Raw bytes: [0x{:02X}, 0x{:02X}]. Check that the request data type matches the target tag; STRING members inside UDTs can also surface this current-encoding rejection.",
                cip_data[4], cip_data[5]
            ),
            _ => format!(
                "Unknown extended CIP error code: 0x{extended_error_code:04X}. Raw bytes: [0x{:02X}, 0x{:02X}]",
                cip_data[4], cip_data[5]
            ),
        };

        Ok(error_msg)
    }

    /// Checks CIP response for errors, including extended error codes
    /// Returns Ok(()) if no error, Err with error message if error found
    fn check_cip_error(&self, cip_data: &[u8]) -> crate::error::Result<()> {
        if cip_data.len() < 3 {
            return Err(EtherNetIpError::Protocol(
                "CIP response too short for status check".to_string(),
            ));
        }

        let general_status = cip_data[2];

        if general_status == 0x00 {
            // Success
            return Ok(());
        }

        // Additional-status words carry the extended status details.
        if cip_data.get(3).copied().unwrap_or(0) > 0 {
            let error_msg = self.parse_extended_error(cip_data)?;
            return Err(EtherNetIpError::Protocol(format!(
                "CIP Extended Error: {error_msg}"
            )));
        }

        // Regular error code
        let error_msg = self.get_cip_error_message(general_status);
        if general_status == 0x01 {
            return Err(EtherNetIpError::Connection(format!(
                "CIP Error 0x{general_status:02X}: {error_msg}"
            )));
        }
        if general_status == 0x07 {
            return Err(EtherNetIpError::ConnectionLost(format!(
                "CIP Error 0x{general_status:02X}: {error_msg}"
            )));
        }
        Err(EtherNetIpError::Protocol(format!(
            "CIP Error 0x{general_status:02X}: {error_msg}"
        )))
    }

    fn get_cip_error_message(&self, status: u8) -> String {
        match status {
            0x00 => "Success".to_string(),
            0x01 => "Connection failure".to_string(),
            0x02 => "Resource unavailable".to_string(),
            0x03 => "Invalid parameter value".to_string(),
            0x04 => "Path segment error".to_string(),
            0x05 => "Path destination unknown".to_string(),
            0x06 => "Partial transfer".to_string(),
            0x07 => "Connection lost".to_string(),
            0x08 => "Service not supported".to_string(),
            0x09 => "Invalid attribute value".to_string(),
            0x0A => "Attribute list error".to_string(),
            0x0B => "Already in requested mode/state".to_string(),
            0x0C => "Object state conflict".to_string(),
            0x0D => "Object already exists".to_string(),
            0x0E => "Attribute not settable".to_string(),
            0x0F => "Privilege violation".to_string(),
            0x10 => "Device state conflict".to_string(),
            0x11 => "Reply data too large".to_string(),
            0x12 => "Fragmentation of a primitive value".to_string(),
            0x13 => "Not enough data".to_string(),
            0x14 => "Attribute not supported".to_string(),
            0x15 => "Too much data".to_string(),
            0x16 => "Object does not exist".to_string(),
            0x17 => "Service fragmentation sequence not in progress".to_string(),
            0x18 => "No stored attribute data".to_string(),
            0x19 => "Store operation failure".to_string(),
            0x1A => "Routing failure, request packet too large".to_string(),
            0x1B => "Routing failure, response packet too large".to_string(),
            0x1C => "Missing attribute list entry data".to_string(),
            0x1D => "Invalid attribute value list".to_string(),
            0x1E => "Embedded service error".to_string(),
            0x1F => "Vendor specific error".to_string(),
            0x20 => "Invalid parameter".to_string(),
            0x21 => "Write-once value or medium already written".to_string(),
            0x22 => "Invalid reply received".to_string(),
            0x23 => "Buffer overflow".to_string(),
            0x24 => "Invalid message format".to_string(),
            0x25 => "Key failure in path".to_string(),
            0x26 => "Path size invalid".to_string(),
            0x27 => "Unexpected attribute in list".to_string(),
            0x28 => "Invalid member ID".to_string(),
            0x29 => "Member not settable".to_string(),
            0x2A => "Group 2 only server general failure".to_string(),
            0x2B => "Unknown Modbus error".to_string(),
            0x2C => "Attribute not gettable".to_string(),
            _ => format!("Unknown CIP error code: 0x{status:02X}"),
        }
    }

    fn describe_multiple_service_error(
        &self,
        general_status: u8,
        operations: &[BatchOperation],
    ) -> String {
        if general_status == 0x1E
            && operations.iter().any(|op| {
                matches!(
                    op,
                    BatchOperation::Write {
                        value: PlcValue::String(_),
                        ..
                    }
                )
            })
        {
            return "Multiple Service Response error: 0x1E (Embedded service error). A batched STRING write failed inside the controller; inspect the embedded reply for the rejected service and data-type details.".to_string();
        }

        format!("Multiple Service Response error: 0x{general_status:02X}")
    }

    async fn validate_session(&mut self) -> crate::error::Result<()> {
        let time_since_activity = self.last_activity.lock().await.elapsed();

        // Send keep-alive if it's been more than 30 seconds since last activity
        if time_since_activity > Duration::from_secs(30) {
            self.send_keep_alive().await?;
        }

        Ok(())
    }

    async fn send_keep_alive(&mut self) -> crate::error::Result<()> {
        self.ensure_stream_usable()?;
        // Send NOP command (0x0000) — a valid 24-byte EtherNet/IP packet
        // that keeps the TCP connection alive without affecting session state.
        // NOP requires no response, so we don't read one.
        let packet = vec![0u8; 24];
        // Command: NOP (0x0000) — already zero
        // Length: 0 — already zero
        // Session handle, status, context, options — all zero for NOP

        let mut stream = self.stream.lock().await;
        stream.write_all(&packet).await?;
        *self.last_activity.lock().await = Instant::now();
        Ok(())
    }

    /// Builds an Unconnected Send message wrapping a CIP request
    ///
    /// Reference: EtherNetIP_Connection_Paths_and_Routing.md
    /// The route path goes at the END of the Unconnected Send message, NOT in the CIP service request.
    ///
    /// Structure:
    /// - Service: 0x52 (Unconnected Send)
    /// - Request Path: Connection Manager (Class 0x06, Instance 1)
    /// - Priority/Time Tick: 0x0A
    /// - Timeout Ticks: 0xF0
    /// - Embedded Message Length
    /// - Embedded CIP Message (Read Tag, Write Tag, etc.) ← NO route path here
    /// - Pad byte (if message length is odd)
    /// - Route Path Size
    /// - Reserved byte
    /// - Route Path ← Route path goes HERE
    fn build_unconnected_send(&self, embedded_message: &[u8]) -> Vec<u8> {
        let mut ucmm = vec![
            // Service: Unconnected Send (0x52)
            0x52, // Request Path Size: 2 words (4 bytes) for Connection Manager
            0x02,
            // Request Path: Connection Manager (Class 0x06, Instance 1)
            0x20, // Logical Class segment
            0x06, // Class 0x06 (Connection Manager)
            0x24, // Logical Instance segment
            0x01, // Instance 1
            // Priority/Time Tick: 0x0A
            0x0A, // Timeout Ticks: 0xF0 (240 ticks)
            0xF0,
        ];

        // Embedded message length (16-bit, little-endian)
        let msg_len = embedded_message.len() as u16;
        ucmm.extend_from_slice(&msg_len.to_le_bytes());

        // The actual CIP message (Read Tag, Write Tag, etc.) - NO route path here!
        ucmm.extend_from_slice(embedded_message);

        // Pad byte if message length is odd
        if embedded_message.len() % 2 == 1 {
            ucmm.push(0x00);
        }

        // Route Path Size (in 16-bit words)
        // Get route path if configured
        let route_path_bytes = if let Some(route_path) = self.route_path_snapshot() {
            route_path.to_cip_bytes()
        } else {
            Vec::new()
        };

        let route_path_words = if route_path_bytes.is_empty() {
            0
        } else {
            (route_path_bytes.len() / 2) as u8
        };
        ucmm.push(route_path_words);

        // Reserved byte
        ucmm.push(0x00);

        // Route Path - THIS IS WHERE [0x01, slot] GOES
        if !route_path_bytes.is_empty() {
            tracing::trace!(
                "Adding route path to Unconnected Send: {:02X?} ({} bytes, {} words)",
                route_path_bytes,
                route_path_bytes.len(),
                route_path_words
            );
            ucmm.extend_from_slice(&route_path_bytes);
        }

        ucmm
    }

    /// Sends a CIP request using EtherNet/IP SendRRData.
    ///
    /// Primary mode uses Unconnected Send (0x52) wrapping. For controllers that reject
    /// this pattern for specific services, a direct-CIP fallback is attempted when:
    /// - the Unconnected Send response is `0xD2` with non-zero general status, and
    /// - no route path is configured (direct mode cannot carry a route path).
    pub async fn send_cip_request(&self, cip_request: &[u8]) -> Result<Vec<u8>> {
        tracing::trace!(
            "Sending CIP request ({} bytes): {:02X?}",
            cip_request.len(),
            cip_request
        );

        // Build Unconnected Send message wrapping the CIP request
        // Route path goes at the END of Unconnected Send, NOT in the CIP request
        let ucmm_message = self.build_unconnected_send(cip_request);
        let diagnostic_operation = Self::diagnostic_operation_for(cip_request);

        tracing::trace!(
            "Unconnected Send message ({} bytes): {:02X?}",
            ucmm_message.len(),
            &ucmm_message[..std::cmp::min(64, ucmm_message.len())]
        );

        let response_data = match self.send_rr_data_item(&ucmm_message).await {
            Ok(response_data) => response_data,
            Err(error) => {
                self.diagnostic_counters
                    .record_failure(diagnostic_operation, &error);
                return Err(error);
            }
        };

        if let Ok(raw_cip_data) = self.extract_unconnected_data_item(&response_data) {
            let use_direct_fallback = raw_cip_data.len() >= 3
                && raw_cip_data[0] == 0xD2
                && raw_cip_data[2] != 0x00
                && cip_request.first().copied() != Some(READ_TAG_FRAGMENTED)
                && self.route_path_snapshot().is_none();

            if use_direct_fallback {
                tracing::warn!(
                    "Unconnected Send returned 0xD2 status 0x{:02X}; retrying with direct CIP SendRRData fallback",
                    raw_cip_data[2]
                );
                return match self.send_rr_data_item(cip_request).await {
                    Ok(response_data) => {
                        self.diagnostic_counters
                            .record_success(diagnostic_operation);
                        Ok(response_data)
                    }
                    Err(error) => {
                        self.diagnostic_counters
                            .record_failure(diagnostic_operation, &error);
                        Err(error)
                    }
                };
            }

            if raw_cip_data.len() >= 3 && raw_cip_data[2] != 0x00 {
                self.diagnostic_counters
                    .record_cip_failure(diagnostic_operation);
            } else {
                self.diagnostic_counters
                    .record_success(diagnostic_operation);
            }
        } else {
            self.diagnostic_counters
                .record_success(diagnostic_operation);
        }

        Ok(response_data)
    }

    async fn send_rr_data_item(&self, item_data: &[u8]) -> Result<Vec<u8>> {
        let send_data = SendDataRequest::unconnected(item_data);
        let mut packet = BytesMut::new();
        let mut cpf = BytesMut::new();
        send_data.encode(&mut cpf);
        let sender_context = self.next_sender_context();
        EncapsulationHeader::send_rr_data_with_context(
            cpf.len() as u16,
            self.session_handle(),
            sender_context,
        )
        .encode(&mut packet);
        packet.extend_from_slice(&cpf);

        tracing::trace!(
            "Built packet ({} bytes): {:02X?}",
            packet.len(),
            &packet[..std::cmp::min(64, packet.len())]
        );

        // Send packet with timeout
        self.ensure_stream_usable()?;
        let mut stream = self.stream.lock().await;
        self.ensure_stream_usable()?;
        self.stream_poisoned.store(true, Ordering::Relaxed);
        if let Err(e) = stream.write_all(&packet).await {
            return Err(EtherNetIpError::Io(e));
        }

        // Read response header with timeout
        let mut header = [0u8; 24];
        match timeout(Duration::from_secs(10), stream.read_exact(&mut header)).await {
            Ok(Ok(_)) => {}
            Ok(Err(e)) => return Err(EtherNetIpError::Io(e)),
            Err(_) => return Err(EtherNetIpError::Timeout(Duration::from_secs(10))),
        }

        // Check EtherNet/IP command status
        let mut header_bytes = &header[..];
        let response_header = EncapsulationHeader::decode(&mut header_bytes)?;
        if response_header.sender_context != sender_context {
            return Err(EtherNetIpError::Protocol(format!(
                "SendRRData sender_context mismatch: expected {:02X?}, got {:02X?}",
                sender_context, response_header.sender_context
            )));
        }

        // Parse response length
        let response_length = response_header.length as usize;
        if response_header.status != 0 {
            if response_length > 0 {
                let mut response_data = vec![0u8; response_length];
                match timeout(
                    Duration::from_secs(10),
                    stream.read_exact(&mut response_data),
                )
                .await
                {
                    Ok(Ok(_)) => {}
                    Ok(Err(e)) => return Err(EtherNetIpError::Io(e)),
                    Err(_) => return Err(EtherNetIpError::Timeout(Duration::from_secs(10))),
                }
            }
            self.stream_poisoned.store(false, Ordering::Relaxed);
            return Err(EtherNetIpError::Protocol(format!(
                "EIP Command failed. Status: 0x{:08X}",
                response_header.status
            )));
        }

        if response_length == 0 {
            self.stream_poisoned.store(false, Ordering::Relaxed);
            return Ok(Vec::new());
        }

        // Read response data with timeout
        let mut response_data = vec![0u8; response_length];
        match timeout(
            Duration::from_secs(10),
            stream.read_exact(&mut response_data),
        )
        .await
        {
            Ok(Ok(_)) => {}
            Ok(Err(e)) => return Err(EtherNetIpError::Io(e)),
            Err(_) => return Err(EtherNetIpError::Timeout(Duration::from_secs(10))),
        }

        self.stream_poisoned.store(false, Ordering::Relaxed);

        // Update last activity time
        *self.last_activity.lock().await = Instant::now();

        tracing::trace!(
            "Received response ({} bytes): {:02X?}",
            response_data.len(),
            &response_data[..std::cmp::min(32, response_data.len())]
        );

        Ok(response_data)
    }

    fn extract_unconnected_data_item(&self, response: &[u8]) -> crate::error::Result<Vec<u8>> {
        let mut response = response;
        let send_data = SendDataRequest::decode(&mut response)?;
        if let Some(item) = send_data
            .items
            .into_iter()
            .find(|item| item.type_id == 0x00B2)
        {
            return Ok(item.data);
        }

        Err(EtherNetIpError::Protocol(
            "No Unconnected Data Item (0x00B2) found in response".to_string(),
        ))
    }

    fn unwrap_unconnected_send_reply(&self, cip_data: &[u8]) -> crate::error::Result<Vec<u8>> {
        if cip_data.is_empty() || cip_data[0] != 0xD2 {
            return Ok(cip_data.to_vec());
        }

        if cip_data.len() < 4 {
            return Err(EtherNetIpError::Protocol(
                "Unconnected Send reply too short".to_string(),
            ));
        }

        let general_status = cip_data[2];
        let additional_status_words = cip_data[3] as usize;
        let embedded_offset = 4 + (additional_status_words * 2);

        if general_status != 0x00 {
            let error_msg = self.get_cip_error_message(general_status);
            return Err(EtherNetIpError::Protocol(format!(
                "Unconnected Send failed (0xD2): CIP Error 0x{general_status:02X}: {error_msg}"
            )));
        }

        if embedded_offset >= cip_data.len() {
            return Err(EtherNetIpError::Protocol(
                "Unconnected Send succeeded but no embedded response payload was returned"
                    .to_string(),
            ));
        }

        Ok(cip_data[embedded_offset..].to_vec())
    }

    /// Extracts CIP data from EtherNet/IP response packet
    fn extract_cip_from_response(&self, response: &[u8]) -> crate::error::Result<Vec<u8>> {
        tracing::trace!(
            "Extracting CIP from response ({} bytes): {:02X?}",
            response.len(),
            &response[..std::cmp::min(32, response.len())]
        );
        let cip_data = self.extract_unconnected_data_item(response)?;
        tracing::trace!(
            "Found Unconnected Data Item, extracted CIP data ({} bytes)",
            cip_data.len()
        );
        tracing::trace!(
            "CIP data bytes: {:02X?}",
            &cip_data[..std::cmp::min(16, cip_data.len())]
        );
        self.unwrap_unconnected_send_reply(&cip_data)
    }

    /// Parses CIP response and converts to `PlcValue`
    fn parse_cip_response(&self, cip_response: &[u8]) -> crate::error::Result<PlcValue> {
        tracing::trace!(
            "Parsing CIP response ({} bytes): {:02X?}",
            cip_response.len(),
            cip_response
        );

        if let Err(e) = self.check_cip_error(cip_response) {
            tracing::error!("CIP Error: {}", e);
            return Err(e);
        }

        let mut response_bytes = cip_response;
        let response = CipResponse::decode(&mut response_bytes)?;

        if response.service == 0xCC {
            self.decode_type_prefixed_value(&response.data)
        } else if response.service == 0xCD {
            tracing::debug!("Write operation successful");
            Ok(PlcValue::Bool(true))
        } else {
            Err(EtherNetIpError::Protocol(format!(
                "Unknown service reply: 0x{:02X}",
                response.service
            )))
        }
    }

    fn parse_read_fragmented_response<'a>(
        &self,
        cip_response: &'a [u8],
    ) -> crate::error::Result<(u8, &'a [u8])> {
        if cip_response.len() < 4 {
            return Err(EtherNetIpError::Protocol(
                "Read Tag Fragmented response too short".to_string(),
            ));
        }

        let service = cip_response[0];
        if service != READ_TAG_FRAGMENTED_REPLY {
            return Err(EtherNetIpError::Protocol(format!(
                "Unexpected Read Tag Fragmented reply service: 0x{service:02X}"
            )));
        }

        let status = cip_response[2];
        if status != CIP_STATUS_SUCCESS && status != CIP_STATUS_PARTIAL_TRANSFER {
            self.check_cip_error(cip_response)?;
        }

        Ok((status, &cip_response[4..]))
    }

    fn decode_type_prefixed_value(&self, data: &[u8]) -> crate::error::Result<PlcValue> {
        if data.len() < 2 {
            return Err(EtherNetIpError::Protocol(
                "Read response too short for data".to_string(),
            ));
        }

        let data_type = u16::from_le_bytes([data[0], data[1]]);
        let value_data = &data[2..];
        tracing::trace!(
            "Data type: 0x{:04X}, Value data ({} bytes): {:02X?}",
            data_type,
            value_data.len(),
            value_data
        );
        Ok(values::decode_payload(data_type, value_data)?)
    }

    /// Unregisters the EtherNet/IP session with the PLC
    pub async fn unregister_session(&mut self) -> crate::error::Result<()> {
        tracing::info!("Unregistering session...");

        let mut packet = BytesMut::with_capacity(24);
        EncapsulationHeader::new(UNREGISTER_SESSION, 0, self.session_handle()).encode(&mut packet);

        self.stream
            .lock()
            .await
            .write_all(&packet)
            .await
            .map_err(EtherNetIpError::Io)?;

        tracing::info!("Session unregistered");
        Ok(())
    }

    /// Builds a CIP Read Tag Service request
    fn build_read_request(&self, tag_name: &str) -> crate::error::Result<Vec<u8>> {
        self.build_read_request_with_count(tag_name, 1)
    }

    /// Builds a CIP Read Tag Service request with specified element count
    ///
    /// Reference: 1756-PM020, Page 220-252 (Read Tag Service)
    fn build_read_request_with_count(
        &self,
        tag_name: &str,
        element_count: u16,
    ) -> crate::error::Result<Vec<u8>> {
        tracing::debug!(
            "Building read request for tag: '{}' with count: {}",
            tag_name,
            element_count
        );

        // Build the path based on tag name format
        let path = self.build_tag_path(tag_name);

        // Request Path Size (in words)
        let path_size_words = (path.len() / 2) as u8;
        tracing::debug!(
            "Path size calculation: {} bytes / 2 = {} words for tag '{}'",
            path.len(),
            path_size_words,
            tag_name
        );
        tracing::debug!(
            "Path bytes ({} bytes, {} words) for tag '{}': {:02X?}",
            path.len(),
            path_size_words,
            tag_name,
            path
        );
        let request = CipRequest::new(READ_TAG, path, element_count.to_le_bytes().to_vec());
        let mut cip_request = BytesMut::new();
        request.encode(&mut cip_request)?;

        tracing::debug!(
            "Built CIP read request ({} bytes) for tag '{}': {:02X?}",
            cip_request.len(),
            tag_name,
            cip_request
        );
        Ok(cip_request.to_vec())
    }

    /// Builds an Element ID segment for array element addressing
    ///
    /// Reference: 1756-PM020, Pages 603-611, 870-890 (Element ID Segment Size Selection)
    ///
    /// Element ID segments use different sizes based on index value:
    /// - 0-255: 8-bit Element ID (0x28 + 1 byte value)
    /// - 256-65535: 16-bit Element ID (0x29 0x00 + 2 bytes low, high)
    /// - 65536+: 32-bit Element ID (0x2A 0x00 + 4 bytes lowest to highest)
    #[cfg_attr(not(test), allow(dead_code))]
    pub fn build_element_id_segment(&self, index: u32) -> Vec<u8> {
        let mut segment = Vec::new();

        if index <= 255 {
            // 8-bit Element ID: 0x28 + index (2 bytes total)
            // Reference: 1756-PM020, Page 607, Example 1
            segment.push(0x28);
            segment.push(index as u8);
        } else if index <= 65535 {
            // 16-bit Element ID: 0x29, 0x00, low_byte, high_byte (4 bytes total)
            // Reference: 1756-PM020, Page 666-684, Example 3
            segment.push(0x29);
            segment.push(0x00); // Padding byte
            segment.extend_from_slice(&(index as u16).to_le_bytes());
        } else {
            // 32-bit Element ID: 0x2A, 0x00, byte0, byte1, byte2, byte3 (6 bytes total)
            // Reference: 1756-PM020, Page 144-146 (Element ID Segments table)
            segment.push(0x2A);
            segment.push(0x00); // Padding byte
            segment.extend_from_slice(&index.to_le_bytes());
        }

        segment
    }

    /// Builds base tag path without array element addressing
    ///
    /// Extracts the base tag name from array notation (e.g., `"MyArray[5]" -> "MyArray"`)
    /// Reference: 1756-PM020, Page 894-909 (ANSI Extended Symbol Segment Construction)
    #[cfg_attr(not(test), allow(dead_code))]
    pub fn build_base_tag_path(&self, tag_name: &str) -> Vec<u8> {
        // Parse tag path but strip array indices
        match TagPath::parse(tag_name) {
            Ok(path) => {
                // If it's an array path, get just the base
                let base_path = match &path {
                    TagPath::Array { base_path, .. } => base_path.as_ref(),
                    _ => &path,
                };
                base_path.to_cip_path().unwrap_or_else(|_| {
                    // Fallback: simple symbol segment
                    // Reference: 1756-PM020, Page 894-909
                    let mut path = Vec::new();
                    path.push(0x91); // ANSI Extended Symbol Segment
                    let name_bytes = tag_name.as_bytes();
                    path.push(name_bytes.len() as u8);
                    path.extend_from_slice(name_bytes);
                    // Pad to word boundary if odd length
                    if path.len() % 2 != 0 {
                        path.push(0x00);
                    }
                    path
                })
            }
            Err(_) => {
                // Fallback: simple symbol segment
                let mut path = Vec::new();
                path.push(0x91); // ANSI Extended Symbol Segment
                let name_bytes = tag_name.as_bytes();
                path.push(name_bytes.len() as u8);
                path.extend_from_slice(name_bytes);
                // Pad to word boundary if odd length
                if path.len() % 2 != 0 {
                    path.push(0x00);
                }
                path
            }
        }
    }

    /// Builds a CIP Read Tag Service request for array elements with element addressing
    ///
    /// This method uses proper CIP element addressing (0x28/0x29/0x2A segments) in the
    /// Request Path to read specific array elements or ranges.
    ///
    /// Reference: 1756-PM020, Pages 603-611, 815-851 (Array Element Addressing Examples)
    ///
    /// # Arguments
    ///
    /// * `base_array_name` - Base name of the array (e.g., `"MyArray"` for `"MyArray[10]"`)
    /// * `start_index` - Starting element index (0-based)
    /// * `element_count` - Number of elements to read
    ///
    /// # Example
    ///
    /// Reading elements 10-14 of array "MyArray" (5 elements):
    /// ```
    /// # async fn example() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    /// # let mut client = rust_ethernet_ip::EipClient::connect("192.168.1.100:44818").await?;
    /// let request = client.build_read_array_request("MyArray", 10, 5);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// This generates:
    /// - Request Path: `0x91 "MyArray" 0x28 0x0A` (element 10)
    /// - Request Data: `0x05 0x00` (5 elements)
    #[cfg_attr(not(test), allow(dead_code))]
    pub fn build_read_array_request(
        &self,
        base_array_name: &str,
        start_index: u32,
        element_count: u16,
    ) -> Vec<u8> {
        let mut cip_request = Vec::new();

        // Service: Read Tag Service (0x4C)
        // Reference: 1756-PM020, Page 220
        cip_request.push(0x4C);

        // Build base tag path (symbolic segment)
        // Reference: 1756-PM020, Page 894-909
        // NOTE: Route path does NOT go here - it goes at the end of Unconnected Send message
        // Reference: EtherNetIP_Connection_Paths_and_Routing.md
        let mut full_path = self.build_base_tag_path(base_array_name);

        tracing::trace!(
            "build_read_array_request: base_path for '{}' = {:02X?} ({} bytes)",
            base_array_name,
            full_path,
            full_path.len()
        );

        // Add element addressing segment
        // Reference: 1756-PM020, Pages 603-611, 870-890
        let element_segment = self.build_element_id_segment(start_index);
        tracing::trace!(
            "build_read_array_request: element_segment for index {} = {:02X?} ({} bytes)",
            start_index,
            element_segment,
            element_segment.len()
        );
        full_path.extend_from_slice(&element_segment);

        // Ensure path is word-aligned
        if !full_path.len().is_multiple_of(2) {
            full_path.push(0x00);
        }

        // Path size (in words)
        let path_size = (full_path.len() / 2) as u8;
        cip_request.push(path_size);
        cip_request.extend_from_slice(&full_path);

        // Request Data: Element count (NOT in path, but in Request Data)
        // Reference: 1756-PM020, Page 840-851 (Reading Multiple Array Elements)
        cip_request.extend_from_slice(&element_count.to_le_bytes());

        tracing::trace!(
            "build_read_array_request: final request = {:02X?} ({} bytes), path_size = {} words ({} bytes)",
            cip_request,
            cip_request.len(),
            path_size,
            full_path.len()
        );

        cip_request
    }

    /// Builds the symbolic CIP path for a tag name.
    /// Uses [`TagPath`] parsing to handle arrays, bits, UDTs, and program scope.
    ///
    /// Route-path bytes are not added here; routed requests carry the route path
    /// in the outer Unconnected Send wrapper.
    fn build_tag_path(&self, tag_name: &str) -> Vec<u8> {
        // Build the application path (tag name)
        // NOTE: Route path does NOT go here - it goes at the end of Unconnected Send message
        // Reference: EtherNetIP_Connection_Paths_and_Routing.md
        match TagPath::parse(tag_name) {
            Ok(tag_path) => {
                tracing::debug!("Parsed tag path for '{}': {:?}", tag_name, tag_path);
                // Generate CIP path using the proper parser
                match tag_path.to_cip_path() {
                    Ok(path) => {
                        tracing::debug!(
                            "TagPath generated {} bytes ({} words) for '{}': {:02X?}",
                            path.len(),
                            path.len() / 2,
                            tag_name,
                            path
                        );
                        path
                    }
                    Err(e) => {
                        tracing::warn!("TagPath.to_cip_path() failed for '{}': {}", tag_name, e);
                        // Fallback to old method if parsing fails
                        self.build_simple_tag_path_legacy(tag_name)
                    }
                }
            }
            Err(e) => {
                tracing::warn!("TagPath::parse() failed for '{}': {}", tag_name, e);
                // Fallback to old method if parsing fails
                self.build_simple_tag_path_legacy(tag_name)
            }
        }
    }

    /// Builds a simple tag path (no program prefix) - legacy method for fallback
    fn build_simple_tag_path_legacy(&self, tag_name: &str) -> Vec<u8> {
        let mut path = Vec::new();
        path.push(0x91); // ANSI Extended Symbol Segment
        path.push(tag_name.len() as u8);
        path.extend_from_slice(tag_name.as_bytes());

        // Pad to even length if necessary
        if !tag_name.len().is_multiple_of(2) {
            path.push(0x00);
        }

        path
    }
}

#[cfg(test)]
mod array_type_cache_tests {
    use super::EipClient;
    use crate::RoutePath;

    #[test]
    fn cache_preserves_positive_and_negative_array_classifications() {
        let client = EipClient::new_unconnected_for_testing();

        client.cache_array_is_packed_bool("BoolArray", true);
        client.cache_array_is_packed_bool("DintArray", false);

        assert_eq!(client.cached_array_is_packed_bool("BoolArray"), Some(true));
        assert_eq!(client.cached_array_is_packed_bool("DintArray"), Some(false));
        assert_eq!(client.cached_array_is_packed_bool("UnknownArray"), None);
    }

    #[test]
    fn cache_is_shared_across_client_clones_used_by_ffi() {
        let client = EipClient::new_unconnected_for_testing();
        let cloned = client.clone();

        client.cache_array_is_packed_bool("ControllerArray", false);
        cloned.cache_array_is_packed_bool("Program:Main.BoolArray", true);

        assert_eq!(
            cloned.cached_array_is_packed_bool("ControllerArray"),
            Some(false)
        );
        assert_eq!(
            client.cached_array_is_packed_bool("Program:Main.BoolArray"),
            Some(true)
        );
    }

    #[test]
    fn route_changes_clear_array_classifications() {
        let mut client = EipClient::new_unconnected_for_testing();
        client.cache_array_is_packed_bool("SharedName", false);

        client.set_route_path(RoutePath::new().add_slot(1));
        assert_eq!(client.cached_array_is_packed_bool("SharedName"), None);

        client.cache_array_is_packed_bool("SharedName", true);
        client.clear_route_path();
        assert_eq!(client.cached_array_is_packed_bool("SharedName"), None);
    }

    #[tokio::test]
    async fn public_cache_clear_removes_array_classifications() {
        let mut client = EipClient::new_unconnected_for_testing();
        client.cache_array_is_packed_bool("DintArray", false);

        client.clear_caches().await;

        assert_eq!(client.cached_array_is_packed_bool("DintArray"), None);
    }
}

#[cfg(test)]
mod discovery_tests {
    use super::{EipClient, TemplateAttributes};

    #[test]
    fn build_tag_list_request_rejects_instance_above_u16() {
        let client = EipClient::new_unconnected_for_testing();
        let request = client
            .build_tag_list_request_from_instance(0x12345678)
            .expect_err("instance should be rejected");

        assert!(format!("{request}").contains("exceeds 16-bit"));
    }

    #[test]
    fn build_tag_list_request_encodes_path_size_and_start_instance() {
        let client = EipClient::new_unconnected_for_testing();
        let request = client
            .build_tag_list_request_from_instance(0x5678)
            .expect("request should build");

        assert_eq!(request[0], 0x55);
        assert_eq!(request[1], 0x03);
        assert_eq!(&request[2..8], &[0x20, 0x6B, 0x25, 0x00, 0x78, 0x56]);
    }

    #[test]
    fn build_program_tag_list_request_includes_program_symbol_scope() {
        let client = EipClient::new_unconnected_for_testing();
        let request = client
            .build_program_tag_list_request("MainProgram", 0)
            .expect("request should build");

        let mut expected = vec![0x55, 0x0E, 0x91, 0x13];
        expected.extend_from_slice(b"Program:MainProgram");
        expected.push(0x00);
        expected.extend_from_slice(&[
            0x20, 0x6B, 0x25, 0x00, 0x00, 0x00, // Symbol class, instance 0
            0x02, 0x00, // attribute count
            0x01, 0x00, // Symbol Name
            0x02, 0x00, // Symbol Type
        ]);

        assert_eq!(request, expected);
    }

    #[test]
    fn build_program_tag_list_request_resumes_at_start_instance() {
        let client = EipClient::new_unconnected_for_testing();
        let request = client
            .build_program_tag_list_request("MainProgram", 0x1235)
            .expect("request should build");

        // Only the instance segment moves: the program symbolic path, the
        // service and the requested attributes are identical to instance 0.
        let mut expected = vec![0x55, 0x0E, 0x91, 0x13];
        expected.extend_from_slice(b"Program:MainProgram");
        expected.push(0x00);
        expected.extend_from_slice(&[
            0x20, 0x6B, 0x25, 0x00, 0x35, 0x12, // Symbol class, instance 0x1235 LE
            0x02, 0x00, // attribute count
            0x01, 0x00, // Symbol Name
            0x02, 0x00, // Symbol Type
        ]);

        assert_eq!(request, expected);
    }

    #[test]
    fn both_accepted_program_name_forms_yield_the_bare_scope() {
        // `discover_program_tags` accepts a program either bare or in its wire
        // form; both designate the same program, so both must stamp the same
        // `TagScope::Program("Dashboard")` -- never `Program("Program:Dashboard")`.
        // This is the exact expression `discover_program_tags` builds.
        let bare =
            super::udt::TagScope::Program(super::program_scope_name("Dashboard").to_string());
        let prefixed = super::udt::TagScope::Program(
            super::program_scope_name("Program:Dashboard").to_string(),
        );

        assert_eq!(
            bare,
            super::udt::TagScope::Program("Dashboard".to_string()),
            "a bare program name is already the scope payload"
        );
        assert_eq!(
            prefixed,
            super::udt::TagScope::Program("Dashboard".to_string()),
            "the wire prefix belongs to the request path, not to the scope payload"
        );
    }

    #[test]
    fn both_accepted_program_name_forms_build_the_same_request() {
        // The same normalization feeds the request path: prefixing an already
        // prefixed name would address `Program:Program:Dashboard`.
        let client = EipClient::new_unconnected_for_testing();
        let bare = client
            .build_program_tag_list_request("Dashboard", 0)
            .expect("request should build");
        let prefixed = client
            .build_program_tag_list_request("Program:Dashboard", 0)
            .expect("request should build");

        assert_eq!(bare, prefixed);

        let mut expected = vec![0x55, 0x0D, 0x91, 0x11];
        expected.extend_from_slice(b"Program:Dashboard");
        expected.push(0x00);
        expected.extend_from_slice(&[
            0x20, 0x6B, 0x25, 0x00, 0x00, 0x00, // Symbol class, instance 0
            0x02, 0x00, // attribute count
            0x01, 0x00, // Symbol Name
            0x02, 0x00, // Symbol Type
        ]);
        assert_eq!(bare, expected);
    }

    #[test]
    fn build_program_tag_list_request_rejects_out_of_range_start_instance() {
        let client = EipClient::new_unconnected_for_testing();
        let error = client
            .build_program_tag_list_request("MainProgram", u32::from(u16::MAX) + 1)
            .expect_err("start instance beyond the 16-bit range must be refused");

        assert!(
            error.to_string().contains("16-bit Symbol Object range"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn parse_tag_list_response_page_handles_partial_transfer() {
        let client = EipClient::new_unconnected_for_testing();
        let response = [
            0xD5, 0x00, 0x06,
            0x00, // service, reserved, partial-transfer status, no addl status
            0x34, 0x12, 0x00, 0x00, // instance id = 0x1234
            0x04, 0x00, // name length = 4
            b'R', b'a', b't', b'e', // tag name
            0xC4, 0x00, // DINT
        ];

        let page = client
            .parse_tag_list_response_page(&response, super::udt::TagScope::Controller)
            .expect("response should parse");

        assert!(page.partial_transfer);
        assert_eq!(page.last_instance_id, Some(0x1234));
        assert_eq!(page.tags.len(), 1);
        assert_eq!(page.tags[0].name, "Rate");
        assert_eq!(page.tags[0].data_type, 0x00C4);
        assert_eq!(page.tags[0].data_type_name, "DINT");
    }

    #[test]
    fn parse_tag_list_response_page_stamps_the_requested_scope() {
        // The Symbol Object reply carries NO scope field: these exact bytes are
        // what a program-scoped enumeration and a controller-scoped one both
        // return. The scope therefore has to come from the request the caller
        // issued -- hardcoding `Controller` in the parser made every tag from
        // `discover_program_tags` claim controller scope.
        let client = EipClient::new_unconnected_for_testing();
        let response = [
            0xD5, 0x00, 0x00, 0x00, // service, reserved, success, no addl status
            0x01, 0x00, 0x00, 0x00, // instance id = 1
            0x04, 0x00, // name length = 4
            b'P', b'u', b'm', b'p', // tag name
            0xC4, 0x00, // DINT
        ];

        let program = client
            .parse_tag_list_response_page(
                &response,
                super::udt::TagScope::Program("Dashboard".to_string()),
            )
            .expect("response should parse");
        assert_eq!(
            program.tags[0].scope,
            super::udt::TagScope::Program("Dashboard".to_string()),
            "a program-scoped enumeration must report the program it was addressed to"
        );

        let controller = client
            .parse_tag_list_response_page(&response, super::udt::TagScope::Controller)
            .expect("response should parse");
        assert_eq!(
            controller.tags[0].scope,
            super::udt::TagScope::Controller,
            "the SAME bytes stay controller-scoped when the request was"
        );
    }

    #[test]
    fn build_get_template_attributes_request_encodes_template_object_path() {
        let client = EipClient::new_unconnected_for_testing();
        let request = client
            .build_get_template_attributes_request(0x0456)
            .expect("request should build");

        assert_eq!(request[0], 0x03);
        assert_eq!(request[1], 0x03);
        assert_eq!(&request[2..8], &[0x20, 0x6C, 0x25, 0x00, 0x56, 0x04]);
        assert_eq!(
            &request[8..],
            &[0x04, 0x00, 0x01, 0x00, 0x02, 0x00, 0x04, 0x00, 0x05, 0x00]
        );
    }

    #[test]
    fn build_get_attributes_request_encodes_path_words_and_odd_name_padding() {
        let client = EipClient::new_unconnected_for_testing();
        let request = client
            .build_get_attributes_request("Odd")
            .expect("request should build");

        assert_eq!(
            request,
            vec![
                0x03, // Get Attribute List
                0x03, // path size: 6 bytes / 2
                0x91, 0x03, b'O', b'd', b'd', 0x00, // padded symbolic segment
                0x02, 0x00, // two attributes
                0x01, 0x00, // data type
                0x02, 0x00, // instance id
            ]
        );
    }

    #[test]
    fn parse_attributes_response_walks_attribute_records() {
        let client = EipClient::new_unconnected_for_testing();
        let response = [
            0x83, 0x00, 0x00, 0x00, // reply, reserved, success, no addl status
            0x02, 0x00, // two attribute records
            0x01, 0x00, 0x00, 0x00, 0xC4, 0x00, // attr 1 = DINT
            0x02, 0x00, 0x00, 0x00, 0x34, 0x12, 0x00, 0x00, // attr 2 = instance
        ];

        let attributes = client
            .parse_attributes_response("DINT_TAG", &response)
            .expect("response should parse");

        assert_eq!(attributes.name, "DINT_TAG");
        assert_eq!(attributes.data_type, 0x00C4);
        assert_eq!(attributes.data_type_name, "DINT");
        assert_eq!(attributes.template_instance_id, Some(0x1234));
        assert_eq!(attributes.size, 4);
    }

    #[test]
    fn extended_status_parser_uses_little_endian_additional_status() {
        let client = EipClient::new_unconnected_for_testing();
        let response = [0xCD, 0x00, 0xFF, 0x01, 0x07, 0x21];

        let err = client
            .check_cip_error(&response)
            .expect_err("extended status should be an error");
        let message = err.to_string();

        assert!(message.contains("0x2107"));
        assert!(message.contains("data-type mismatch"));
        assert!(!message.contains("(BE)"));
        assert!(!message.contains("0x0721"));
    }

    #[test]
    fn extended_status_parser_does_not_require_general_status_ff() {
        let client = EipClient::new_unconnected_for_testing();
        let response = [0xCC, 0x00, 0x01, 0x01, 0x05, 0x00];

        let err = client
            .check_cip_error(&response)
            .expect_err("additional status should be decoded");

        assert!(err.to_string().contains("Path destination unknown"));
    }

    #[test]
    fn build_read_template_request_encodes_template_read_size() {
        let client = EipClient::new_unconnected_for_testing();
        let request = client
            .build_read_template_request(0x0456, 0x0010, 0x0032)
            .expect("request should build");

        assert_eq!(request[0], 0x4C);
        assert_eq!(request[1], 0x03);
        assert_eq!(&request[2..8], &[0x20, 0x6C, 0x25, 0x00, 0x56, 0x04]);
        assert_eq!(&request[8..12], &[0x10, 0x00, 0x00, 0x00]);
        assert_eq!(&request[12..14], &[0x32, 0x00]);
    }

    #[test]
    fn parse_template_attributes_response_reads_mixed_width_values() {
        let client = EipClient::new_unconnected_for_testing();
        let response = [
            0x83, 0x00, 0x00, 0x00, // service reply, reserved, success, no addl status
            0x04, 0x00, // four attributes
            0x01, 0x00, 0x00, 0x00, 0x34, 0x12, // attr 1 = structure handle
            0x02, 0x00, 0x00, 0x00, 0x07, 0x00, // attr 2 = member count
            0x04, 0x00, 0x00, 0x00, 0x19, 0x00, 0x00, 0x00, // attr 4 = definition words
            0x05, 0x00, 0x00, 0x00, 0x58, 0x00, 0x00, 0x00, // attr 5 = structure bytes
        ];

        let attributes = client
            .parse_template_attributes_response(0x0456, &response)
            .expect("response should parse");

        assert_eq!(
            attributes,
            TemplateAttributes {
                structure_handle: 0x1234,
                member_count: 7,
                definition_size_words: 25,
                structure_size_bytes: 88,
            }
        );
    }
}

#[cfg(test)]
mod write_request_tests {
    use super::EipClient;
    use crate::PlcValue;
    use crate::protocol::values;

    #[test]
    fn build_write_request_encodes_standard_string_structure() {
        let client = EipClient::new_unconnected_for_testing();
        let request = client
            .build_write_request("Tag1", &PlcValue::String("AB".to_string()))
            .expect("STRING request should build");

        assert_eq!(
            &request[..8],
            &[0x4D, 0x03, 0x91, 0x04, b'T', b'a', b'g', b'1']
        );
        let data = &request[8..];
        assert_eq!(&data[..6], &[0xA0, 0x02, 0xCE, 0x0F, 0x01, 0x00]);
        assert_eq!(&data[6..12], &[2, 0, 0, 0, b'A', b'B']);
        assert_eq!(data.len(), 6 + values::STANDARD_STRING_PAYLOAD_LEN);
        assert!(data[12..].iter().all(|byte| *byte == 0));
    }

    #[test]
    fn build_write_request_rejects_overlong_standard_string() {
        let client = EipClient::new_unconnected_for_testing();
        let value = PlcValue::String("x".repeat(values::STANDARD_STRING_DATA_LEN + 1));
        let err = client
            .build_write_request("Tag1", &value)
            .expect_err("overlong STRING should be rejected");

        assert!(err.to_string().contains("String too long"));
    }
}

#[cfg(test)]
mod transport_tests {
    use super::{DiagnosticOperation, EipClient};
    use crate::EtherNetIpStream;
    use crate::error::EtherNetIpError;
    use std::sync::Arc;
    use std::time::Duration;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::sync::Mutex;

    fn register_response(session_handle: u32) -> Vec<u8> {
        let mut response = Vec::with_capacity(28);
        response.extend_from_slice(&0x0065u16.to_le_bytes());
        response.extend_from_slice(&4u16.to_le_bytes());
        response.extend_from_slice(&session_handle.to_le_bytes());
        response.extend_from_slice(&0u32.to_le_bytes());
        response.extend_from_slice(&[0u8; 8]);
        response.extend_from_slice(&0u32.to_le_bytes());
        response.extend_from_slice(&[0x01, 0x00, 0x00, 0x00]);
        response
    }

    async fn read_register_request(stream: &mut tokio::io::DuplexStream) {
        let mut header = [0u8; 24];
        stream
            .read_exact(&mut header)
            .await
            .expect("request header");
        let body_len = u16::from_le_bytes([header[2], header[3]]) as usize;
        let mut body = vec![0u8; body_len];
        stream.read_exact(&mut body).await.expect("request body");
    }

    #[tokio::test]
    async fn register_session_accepts_fragmented_reply() {
        let (client_stream, mut server_stream) = tokio::io::duplex(128);
        let mut client = EipClient::new_unconnected_for_testing();
        client.stream = Arc::new(Mutex::new(
            Box::new(client_stream) as Box<dyn EtherNetIpStream>
        ));

        let server = tokio::spawn(async move {
            read_register_request(&mut server_stream).await;
            let response = register_response(0x0102_0304);
            server_stream
                .write_all(&response[..10])
                .await
                .expect("first fragment");
            tokio::task::yield_now().await;
            server_stream
                .write_all(&response[10..])
                .await
                .expect("second fragment");
        });

        client
            .register_session()
            .await
            .expect("fragmented register response should parse");
        server.await.expect("server task");

        assert_eq!(client.session_handle(), 0x0102_0304);
    }

    #[tokio::test]
    async fn reregistration_updates_session_handle_across_clones() {
        let (client_stream, mut server_stream) = tokio::io::duplex(256);
        let mut client = EipClient::new_unconnected_for_testing();
        client.stream = Arc::new(Mutex::new(
            Box::new(client_stream) as Box<dyn EtherNetIpStream>
        ));
        let mut clone = client.clone();

        let server = tokio::spawn(async move {
            for handle in [0x1111_2222, 0x3333_4444] {
                read_register_request(&mut server_stream).await;
                server_stream
                    .write_all(&register_response(handle))
                    .await
                    .expect("register response");
            }
        });

        client.register_session().await.expect("first register");
        assert_eq!(client.session_handle(), 0x1111_2222);
        assert_eq!(clone.session_handle(), 0x1111_2222);

        clone.register_session().await.expect("clone re-register");
        server.await.expect("server task");

        assert_eq!(client.session_handle(), 0x3333_4444);
        assert_eq!(clone.session_handle(), 0x3333_4444);
    }

    #[tokio::test]
    async fn diagnostics_snapshot_reports_counted_operations_and_errors() {
        let client = EipClient::new_unconnected_for_testing();

        client
            .diagnostic_counters
            .record_success(Some(DiagnosticOperation::Read));
        client.diagnostic_counters.record_failure(
            Some(DiagnosticOperation::Write),
            &EtherNetIpError::Timeout(Duration::from_secs(1)),
        );
        client
            .diagnostic_counters
            .record_cip_failure(Some(DiagnosticOperation::Batch));

        let snapshot = client.get_diagnostics_snapshot().await;

        assert_eq!(snapshot.operations.total_reads, 1);
        assert_eq!(snapshot.operations.successful_reads, 1);
        assert_eq!(snapshot.operations.total_writes, 1);
        assert_eq!(snapshot.operations.failed_writes, 1);
        assert_eq!(snapshot.operations.batch_operations, 1);
        assert_eq!(snapshot.operations.partial_batch_failures, 1);
        assert_eq!(snapshot.errors.timeout_errors, 1);
        assert_eq!(snapshot.errors.protocol_errors, 1);
        assert_eq!(snapshot.errors.retriable_errors, 1);
        assert_eq!(snapshot.errors.non_retriable_errors, 1);
        assert!(snapshot.operations.last_successful_read_time.is_some());
        assert!(snapshot.operations.last_failed_write_time.is_some());
        assert!(snapshot.errors.last_error_time.is_some());
        assert!(snapshot.system_metrics_are_placeholders);
    }
}

/*
===============================================================================
END OF LIBRARY DOCUMENTATION

This file provides a complete, production-ready EtherNet/IP communication
library for Allen-Bradley PLCs. The library includes:

- Native Rust API with async support
- C FFI exports for cross-language integration
- Comprehensive error handling and validation
- Detailed documentation and examples
- Performance optimizations
- Memory safety guarantees

For usage examples, see the main.rs file or the C# integration samples.

For technical details about the EtherNet/IP protocol implementation,
refer to the inline documentation above.

Version: 1.0.0
Compatible with: CompactLogix L1x-L5x series PLCs
License: As specified in Cargo.toml
===============================================================================_
*/
