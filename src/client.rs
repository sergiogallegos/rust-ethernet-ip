use crate::EtherNetIpStream;
use crate::batch::{BatchConfig, BatchOperation};
use crate::error::{EtherNetIpError, Result};
use crate::protocol::cip::{CipRequest, CipResponse, READ_TAG, SendDataRequest, WRITE_TAG};
use crate::protocol::encap::{EncapsulationHeader, REGISTER_SESSION, UNREGISTER_SESSION};
use crate::protocol::values;
use crate::protocol::{Decode, Encode};
use crate::route::RoutePath;
use crate::subscription::TagSubscription;
use crate::tag_group::TagGroupConfig;
use crate::tag_manager::{TagManager, TagMetadata, TagPermissions, TagScope};
use crate::types::{ConnectedSession, PlcValue, UdtData};
use crate::udt::{TagAttributes, UdtDefinition, UdtManager};
use crate::{TagPath, udt};
use bytes::BytesMut;
use std::collections::HashMap;
use std::net::SocketAddr;
#[cfg(feature = "ffi")]
use std::sync::LazyLock;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex as StdMutex};
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
/// The following operations are **not supported** due to PLC firmware limitations:
///
/// ## UDT Array Element Member Writes
///
/// **Cannot write directly to UDT array element members** (e.g., `gTestUDT_Array[0].Member1_DINT`).
/// This is a PLC firmware limitation, not a library bug. The PLC returns CIP Error 0x2107
/// (Vendor Specific Error) when attempting to write to such paths.
///
/// ## STRING Tags and STRING Members in UDTs
///
/// **Cannot write directly to STRING tags or STRING members in UDTs**.
/// This is a PLC firmware limitation (CIP Error 0x2107). Both simple STRING tags
/// (e.g., `gTest_STRING`) and STRING members within UDTs (e.g., `gTestUDT.Member5_String`)
/// cannot be written directly. STRING values must be written as part of the entire UDT
/// structure, not as individual tags or members.
///
/// **What works:**
/// - ✅ Reading UDT array element members: `gTestUDT_Array[0].Member1_DINT` (read)
/// - ✅ Writing entire UDT array elements: `gTestUDT_Array[0]` (write full UDT)
/// - ✅ Writing UDT members (non-STRING): `gTestUDT.Member1_DINT` (write DINT/REAL/BOOL/INT members)
/// - ✅ Writing array elements: `gArray[5]` (write element of simple array)
/// - ✅ Reading STRING tags: `gTest_STRING` (read)
/// - ✅ Reading STRING members in UDTs: `gTestUDT.Member5_String` (read)
///
/// **What doesn't work:**
/// - ❌ Writing UDT array element members: `gTestUDT_Array[0].Member1_DINT` (write)
/// - ❌ Writing program-scoped UDT array element members: `Program:TestProgram.gTestUDT_Array[0].Member1_DINT` (write)
/// - ❌ Writing simple STRING tags: `gTest_STRING` (write) - PLC limitation
/// - ❌ Writing program-scoped STRING tags: `Program:TestProgram.gTest_STRING` (write) - PLC limitation
/// - ❌ Writing STRING members in UDTs: `gTestUDT.Member5_String` (write) - must write entire UDT
/// - ❌ Writing program-scoped STRING members: `Program:TestProgram.gTestUDT.Member5_String` (write) - must write entire UDT
///
/// **Workaround:**
/// To modify a UDT array element member, read the entire UDT array element, modify the member
/// in memory, then write the entire UDT array element back:
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
    /// COPIED ON CLONE: set during construction before FFI registry insertion; never mutate post-insert.
    session_handle: u32,
    /// SHARED ON CLONE: tag discovery/cache state.
    tag_manager: Arc<Mutex<TagManager>>,
    /// SHARED ON CLONE: UDT discovery/cache state.
    udt_manager: Arc<Mutex<UdtManager>>,
    /// SHARED ON CLONE: route-path mutations must be visible through later registry lookups.
    route_path: Arc<StdMutex<Option<RoutePath>>>,
    /// SHARED ON CLONE: max packet size is cheap scalar state and may be configured through FFI.
    max_packet_size: Arc<AtomicU32>,
    /// SHARED ON CLONE: last activity timestamp.
    last_activity: Arc<Mutex<Instant>>,
    /// COPIED ON CLONE: persistent FFI config would require Arc/RwLock; current use is per-call only.
    batch_config: BatchConfig,
    /// SHARED ON CLONE: Class 3 connected session state.
    connected_sessions: Arc<Mutex<HashMap<String, ConnectedSession>>>,
    /// SHARED ON CLONE: connection sequence counter.
    connection_sequence: Arc<Mutex<u32>>,
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
            .field("session_handle", &self.session_handle)
            .field("route_path", &self.route_path_snapshot())
            .field("max_packet_size", &self.max_packet_size())
            .field("batch_config", &self.batch_config)
            .field("stream", &"<stream>")
            .field("tag_manager", &"<tag_manager>")
            .field("udt_manager", &"<udt_manager>")
            .field("connected_sessions", &"<connected_sessions>")
            .field("subscriptions", &"<subscriptions>")
            .field("tag_groups", &"<tag_groups>")
            .finish()
    }
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
            session_handle: 0,
            tag_manager: Arc::new(Mutex::new(TagManager::new())),
            udt_manager: Arc::new(Mutex::new(UdtManager::new())),
            route_path: Arc::new(StdMutex::new(None)),
            max_packet_size: Arc::new(AtomicU32::new(4000)),
            last_activity: Arc::new(Mutex::new(Instant::now())),
            batch_config: BatchConfig::default(),
            connected_sessions: Arc::new(Mutex::new(HashMap::new())),
            connection_sequence: Arc::new(Mutex::new(1)),
            subscriptions: Arc::new(Mutex::new(Vec::new())),
            tag_groups: Arc::new(Mutex::new(HashMap::new())),
        };
        client.register_session().await?;
        client.negotiate_packet_size().await?;
        Ok(client)
    }

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
            session_handle: 0,
            tag_manager: Arc::new(Mutex::new(TagManager::new())),
            udt_manager: Arc::new(Mutex::new(UdtManager::new())),
            route_path: Arc::new(StdMutex::new(None)),
            max_packet_size: Arc::new(AtomicU32::new(4000)),
            last_activity: Arc::new(Mutex::new(Instant::now())),
            batch_config: BatchConfig::default(),
            connected_sessions: Arc::new(Mutex::new(HashMap::new())),
            connection_sequence: Arc::new(Mutex::new(1)),
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
        tracing::debug!("Starting session registration...");
        let mut packet = BytesMut::with_capacity(28);
        EncapsulationHeader::new(REGISTER_SESSION, 4, 0).encode(&mut packet);
        packet.extend_from_slice(&[0x01, 0x00]); // Protocol Version: 1
        packet.extend_from_slice(&[0x00, 0x00]); // Option Flags: 0

        tracing::trace!("Sending Register Session packet: {:02X?}", packet);
        self.stream
            .lock()
            .await
            .write_all(&packet)
            .await
            .map_err(|e| {
                tracing::error!("Failed to send Register Session packet: {}", e);
                EtherNetIpError::Io(e)
            })?;

        let mut buf = [0u8; 1024];
        tracing::debug!("Waiting for Register Session response...");
        let n = match timeout(
            Duration::from_secs(5),
            self.stream.lock().await.read(&mut buf),
        )
        .await
        {
            Ok(Ok(n)) => {
                tracing::trace!("Received {} bytes in response", n);
                n
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

        if n < 28 {
            tracing::error!("Response too short: {} bytes (expected 28)", n);
            return Err(EtherNetIpError::Protocol("Response too short".to_string()));
        }

        let mut response = &buf[..n];
        let header = EncapsulationHeader::decode(&mut response)?;

        // Extract session handle from response
        self.session_handle = header.session_handle;
        tracing::debug!("Session handle: 0x{:08X}", self.session_handle);

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
        // Build CIP request to get UDT definition
        let cip_request = {
            let tag_manager = self.tag_manager.lock().await;
            tag_manager.build_udt_definition_request(udt_name)?
        };

        // Send the request
        let response = self.send_cip_request(&cip_request).await?;

        // Parse the UDT definition from response
        let definition = {
            let tag_manager = self.tag_manager.lock().await;
            tag_manager.parse_udt_definition_response(&response, udt_name)?
        };

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
            let page = match self.parse_tag_list_response_page(&cip_data) {
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
    pub async fn discover_program_tags(
        &mut self,
        program_name: &str,
    ) -> crate::error::Result<Vec<TagAttributes>> {
        // Build CIP request for program-scoped tag list
        let request = self.build_program_tag_list_request(program_name)?;
        let response = self.send_cip_request(&request).await?;

        // Extract CIP data from response and check for errors
        let cip_data = self.extract_cip_from_response(&response)?;

        // Check for CIP errors before parsing
        if let Err(e) = self.check_cip_error(&cip_data) {
            return Err(crate::error::EtherNetIpError::Protocol(format!(
                "Program tag discovery failed for '{}': {}. Some PLCs may not support tag discovery. Try reading tags directly by name.",
                program_name, e
            )));
        }

        // Parse response
        self.parse_tag_list_response(&cip_data)
    }

    /// Lists all cached tag attributes
    pub async fn list_cached_tag_attributes(&self) -> Vec<String> {
        self.udt_manager.lock().await.list_tag_attributes()
    }

    /// Clears cached tag metadata and UDT-related caches.
    pub async fn clear_caches(&mut self) {
        if let Err(error) = self.tag_manager.lock().await.clear_cache().await {
            tracing::warn!("failed to clear tag metadata cache: {error}");
        }
        self.udt_manager.lock().await.clear_cache();
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
        *self
            .route_path
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = None;
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

        let response = self
            .send_cip_request(&self.build_read_request(tag_name)?)
            .await?;
        let cip_data = self.extract_cip_from_response(&response)?;
        self.parse_cip_response(&cip_data)
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
        if bit_index >= 32 {
            return Err(crate::error::EtherNetIpError::Protocol(
                "bit_index must be 0..32 for DINT bit access".to_string(),
            ));
        }
        let path = format!("{}.{}", tag_base, bit_index);
        match self.read_tag(&path).await? {
            PlcValue::Bool(b) => Ok(b),
            PlcValue::Dint(n) => {
                // Some PLCs/simulators return the full DINT for bit paths; extract the bit
                Ok((n >> bit_index) & 1 != 0)
            }
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
        if bit_index >= 32 {
            return Err(crate::error::EtherNetIpError::Protocol(
                "bit_index must be 0..32 for DINT bit access".to_string(),
            ));
        }
        let path = format!("{}.{}", tag_base, bit_index);
        self.write_tag(&path, PlcValue::Bool(value)).await
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

    fn parse_final_array_element_access(&self, tag_name: &str) -> Option<(String, u32)> {
        match TagPath::parse(tag_name).ok()? {
            TagPath::Array { base_path, indices } if indices.len() == 1 => {
                Some((base_path.as_string(), indices[0]))
            }
            _ => None,
        }
    }

    async fn detect_bool_array_path(&mut self, array_path: &str) -> crate::error::Result<bool> {
        let test_response = self
            .send_cip_request(&self.build_read_request_with_count(array_path, 1)?)
            .await?;
        let test_cip_data = self.extract_cip_from_response(&test_response)?;

        if self.check_cip_error(&test_cip_data).is_err() || test_cip_data.len() < 6 {
            return Ok(false);
        }

        let test_data_type = u16::from_le_bytes([test_cip_data[4], test_cip_data[5]]);
        Ok(test_data_type == values::BOOL_ARRAY_DWORD)
    }

    fn parse_bool_array_dword_response(&self, cip_data: &[u8]) -> crate::error::Result<u32> {
        if cip_data.len() < 6 {
            return Err(EtherNetIpError::Protocol(
                "BOOL array response too short".to_string(),
            ));
        }

        self.check_cip_error(cip_data)?;

        let service_reply = cip_data[0];
        if service_reply != 0xCC {
            return Err(EtherNetIpError::Protocol(format!(
                "Unexpected service reply: 0x{service_reply:02X}"
            )));
        }

        let data_type = u16::from_le_bytes([cip_data[4], cip_data[5]]);
        if data_type != values::BOOL_ARRAY_DWORD {
            return Err(EtherNetIpError::Protocol(format!(
                "Expected BOOL array DWORD data type 0x00D3, got 0x{data_type:04X}"
            )));
        }

        let value_data = if cip_data.len() >= 12 {
            &cip_data[8..]
        } else if cip_data.len() >= 10 {
            &cip_data[6..]
        } else {
            return Err(EtherNetIpError::Protocol(
                "BOOL array response too short for data".to_string(),
            ));
        };

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

        // Use element addressing to read directly from the specified index
        // Reference: 1756-PM020, Pages 815-837 (Reading Array Element - Full Message)
        let request = self.build_read_array_request(base_array_name, index, 1);

        let response = self.send_cip_request(&request).await?;
        let cip_data = self.extract_cip_from_response(&response)?;

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

            if cip_data.len() < 8 {
                // Response too short - might be an error or empty response
                // Check if it's a CIP error response
                if cip_data.len() >= 3 {
                    let general_status = cip_data[2];
                    if general_status != 0x00 {
                        let error_msg = self.get_cip_error_message(general_status);
                        return Err(EtherNetIpError::Protocol(format!(
                            "CIP Error {} when reading chunk (elements {} to {}): {}",
                            general_status,
                            next_chunk_start,
                            chunk_end - 1,
                            error_msg
                        )));
                    }
                }
                return Err(EtherNetIpError::Protocol(format!(
                    "Chunk response too short: got {} bytes, expected at least 8 (requested {} elements starting at {})",
                    cip_data.len(),
                    chunk_size,
                    next_chunk_start
                )));
            }

            // Check for CIP errors in the response
            if cip_data.len() >= 3 {
                let general_status = cip_data[2];
                if general_status != 0x00 {
                    let error_msg = self.get_cip_error_message(general_status);
                    return Err(EtherNetIpError::Protocol(format!(
                        "CIP Error {} when reading chunk (elements {} to {}): {}",
                        general_status,
                        next_chunk_start,
                        chunk_end - 1,
                        error_msg
                    )));
                }
            }

            // Check service reply
            if !cip_data.is_empty() && cip_data[0] != 0xCC {
                return Err(EtherNetIpError::Protocol(format!(
                    "Unexpected service reply in chunk: 0x{:02X} (expected 0xCC)",
                    cip_data[0]
                )));
            }

            if cip_data.len() < 6 {
                return Err(EtherNetIpError::Protocol(format!(
                    "Chunk response too short for data type: got {} bytes, expected at least 6",
                    cip_data.len()
                )));
            }

            let chunk_data_type = u16::from_le_bytes([cip_data[4], cip_data[5]]);
            if chunk_data_type != data_type {
                return Err(EtherNetIpError::Protocol(format!(
                    "Data type mismatch in chunk: expected 0x{:04X}, got 0x{:04X}",
                    data_type, chunk_data_type
                )));
            }

            // Parse response data - with element addressing, response contains the requested range
            // Reference: 1756-PM020, Page 828-837 (Response format)
            let value_data_start = if cip_data.len() >= 8 {
                // Standard format: [service][reserved][status][status_size][data_type(2)][element_count(2)][data...]
                8
            } else {
                6
            };

            let chunk_value_data = &cip_data[value_data_start..];
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

    /// Builds a write request for an entire array (legacy method - writes from element 0)
    ///
    /// Reference: 1756-PM020, Page 318-357 (Write Tag Service)
    #[allow(dead_code)]
    fn build_write_array_request(
        &self,
        tag_name: &str,
        data_type: u16,
        element_count: u16,
        data: &[u8],
    ) -> crate::error::Result<Vec<u8>> {
        let mut cip_request = Vec::new();

        // Service: Write Tag Service (0x4D)
        // Reference: 1756-PM020, Page 318
        cip_request.push(0x4D);

        // Build the path
        let path = self.build_tag_path(tag_name);
        cip_request.push((path.len() / 2) as u8);
        cip_request.extend_from_slice(&path);

        // Data type and element count
        // Reference: 1756-PM020, Page 335-337 (Request Data format)
        cip_request.extend_from_slice(&data_type.to_le_bytes());
        cip_request.extend_from_slice(&element_count.to_le_bytes());

        // Array data
        cip_request.extend_from_slice(data);

        Ok(cip_request)
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

    /// Reads a UDT with advanced chunked reading to handle large structures
    ///
    /// **v0.6.0**: Returns `PlcValue::Udt(UdtData)` with `symbol_id` and raw bytes.
    /// Use `UdtData::parse()` with a UDT definition to access individual members.
    ///
    /// This method uses multiple strategies to handle large UDTs that exceed
    /// the maximum packet size, including intelligent chunking and member discovery.
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

        tracing::debug!("[CHUNKED] Starting advanced UDT reading for: {}", tag_name);

        // Strategy 1: Try normal read first
        match self.read_tag(tag_name).await {
            Ok(value) => {
                tracing::debug!("[CHUNKED] Normal read successful");
                return Ok(value);
            }
            Err(crate::error::EtherNetIpError::Protocol(msg))
                if msg.contains("Partial transfer") =>
            {
                tracing::debug!("[CHUNKED] Partial transfer detected, using advanced chunking");
            }
            Err(e) => {
                tracing::warn!("[CHUNKED] Normal read failed: {}", e);
                return Err(e);
            }
        }

        // Strategy 2: Advanced chunked reading with multiple approaches
        self.read_udt_advanced_chunked(tag_name).await
    }

    /// Advanced chunked UDT reading with multiple strategies
    async fn read_udt_advanced_chunked(
        &mut self,
        tag_name: &str,
    ) -> crate::error::Result<PlcValue> {
        tracing::debug!("[ADVANCED] Using multiple strategies for large UDT");

        // Strategy A: Try different chunk sizes
        let chunk_sizes = vec![512, 256, 128, 64, 32, 16, 8, 4];

        for chunk_size in chunk_sizes {
            tracing::trace!("[ADVANCED] Trying chunk size: {}", chunk_size);

            match self.read_udt_with_chunk_size(tag_name, chunk_size).await {
                Ok(udt_value) => {
                    tracing::debug!("[ADVANCED] Success with chunk size {}", chunk_size);
                    return Ok(udt_value);
                }
                Err(e) => {
                    tracing::trace!("[ADVANCED] Chunk size {} failed: {}", chunk_size, e);
                    continue;
                }
            }
        }

        // Strategy B: Try member-by-member discovery
        tracing::debug!("[ADVANCED] Trying member-by-member discovery");
        match self.read_udt_member_discovery(tag_name).await {
            Ok(udt_value) => {
                tracing::debug!("[ADVANCED] Member discovery successful");
                return Ok(udt_value);
            }
            Err(e) => {
                tracing::warn!("[ADVANCED] Member discovery failed: {}", e);
            }
        }

        // Strategy C: Try progressive reading
        tracing::debug!("[ADVANCED] Trying progressive reading");
        match self.read_udt_progressive(tag_name).await {
            Ok(udt_value) => {
                tracing::debug!("[ADVANCED] Progressive reading successful");
                return Ok(udt_value);
            }
            Err(e) => {
                tracing::warn!("[ADVANCED] Progressive reading failed: {}", e);
            }
        }

        // Strategy D: Fallback - try to get at least the symbol_id
        tracing::warn!("[ADVANCED] All strategies failed, using fallback");
        // Try to get tag attributes for symbol_id
        let symbol_id = self
            .get_tag_attributes(tag_name)
            .await
            .ok()
            .and_then(|attr| attr.template_instance_id)
            .unwrap_or(0) as i32;

        // Return empty UDT data with error indication
        Ok(PlcValue::Udt(UdtData {
            symbol_id,
            data: vec![], // Empty data indicates read failure
        }))
    }

    /// Try reading UDT with specific chunk size
    async fn read_udt_with_chunk_size(
        &mut self,
        tag_name: &str,
        mut chunk_size: usize,
    ) -> crate::error::Result<PlcValue> {
        let mut all_data = Vec::new();
        let mut offset = 0;
        let mut consecutive_failures = 0;
        const MAX_FAILURES: usize = 3;

        loop {
            match self
                .read_udt_chunk_advanced(tag_name, offset, chunk_size)
                .await
            {
                Ok(chunk_data) => {
                    if chunk_data.is_empty() {
                        break; // No more data
                    }

                    all_data.extend_from_slice(&chunk_data);
                    offset += chunk_data.len();
                    consecutive_failures = 0;

                    tracing::trace!(
                        "[CHUNK] Read {} bytes at offset {}, total: {}",
                        chunk_data.len(),
                        offset - chunk_data.len(),
                        all_data.len()
                    );

                    // If we got less data than requested, we might be done
                    if chunk_data.len() < chunk_size {
                        break;
                    }
                }
                Err(e) => {
                    consecutive_failures += 1;
                    tracing::warn!(
                        "[CHUNK] Chunk read failed (attempt {}): {}",
                        consecutive_failures,
                        e
                    );

                    if consecutive_failures >= MAX_FAILURES {
                        break;
                    }

                    // Try smaller chunk by reducing size and continuing
                    if chunk_size > 4 {
                        chunk_size /= 2;
                        continue;
                    }
                }
            }
        }

        if all_data.is_empty() {
            return Err(crate::error::EtherNetIpError::Protocol(
                "No data read from UDT".to_string(),
            ));
        }

        tracing::debug!("[CHUNK] Total data collected: {} bytes", all_data.len());

        // Get symbol_id from tag attributes
        let symbol_id = self
            .get_tag_attributes(tag_name)
            .await
            .ok()
            .and_then(|attr| attr.template_instance_id)
            .unwrap_or(0) as i32;

        // Return raw UDT data (generic approach - no parsing)
        Ok(PlcValue::Udt(UdtData {
            symbol_id,
            data: all_data,
        }))
    }

    /// Advanced chunk reading with better error handling
    async fn read_udt_chunk_advanced(
        &mut self,
        tag_name: &str,
        offset: usize,
        size: usize,
    ) -> crate::error::Result<Vec<u8>> {
        // Build a more sophisticated read request
        let mut request = Vec::new();

        // Service: Read Tag (0x4C)
        request.push(0x4C);

        // Use TagPath::parse() to correctly handle complex paths like Cell_NestData[90].PartData
        let tag_path = self.build_tag_path(tag_name);

        // Path size (in words)
        let path_size = (tag_path.len() / 2) as u8;
        request.push(path_size);

        // Path: use properly parsed tag path
        request.extend_from_slice(&tag_path);

        // For UDTs, we need to use a different approach than array indexing
        // Try to read as raw data with offset
        if offset > 0 {
            // Use element path for offset
            request.push(0x28); // Element symbol
            request.push(0x02); // 2 bytes for offset
            request.extend_from_slice(&(offset as u16).to_le_bytes());
        }

        // Element count
        request.push(0x28); // Element count symbol
        request.push(0x02); // 2 bytes for count
        request.extend_from_slice(&(size as u16).to_le_bytes());

        // Data type - try as raw bytes first
        request.push(0x00);
        request.push(0x01);

        // Send the request
        let response = self.send_cip_request(&request).await?;
        let cip_data = self.extract_cip_from_response(&response)?;

        // Parse the response
        if cip_data.len() < 2 {
            return Ok(Vec::new()); // No data
        }

        let _data_type = u16::from_le_bytes([cip_data[0], cip_data[1]]);
        let data = &cip_data[2..];

        Ok(data.to_vec())
    }

    /// Try to read UDT as raw data with symbol_id
    ///
    /// This is a generic approach that works for any UDT without requiring
    /// knowledge of member names. It reads the raw bytes and gets the
    /// symbol_id (template instance ID) from tag attributes.
    async fn read_udt_member_discovery(
        &mut self,
        tag_name: &str,
    ) -> crate::error::Result<PlcValue> {
        tracing::debug!("[DISCOVERY] Reading UDT as raw data for: {}", tag_name);

        // Get tag attributes to retrieve symbol_id (template_instance_id)
        let attributes = self.get_tag_attributes(tag_name).await?;

        let symbol_id = attributes.template_instance_id.ok_or_else(|| {
            crate::error::EtherNetIpError::Protocol(
                "UDT template instance ID not found in tag attributes".to_string(),
            )
        })?;

        // Read raw UDT data
        let raw_data = self.read_tag_raw(tag_name).await?;

        tracing::debug!(
            "[DISCOVERY] Read {} bytes of UDT data with symbol_id: {}",
            raw_data.len(),
            symbol_id
        );

        Ok(PlcValue::Udt(UdtData {
            symbol_id: symbol_id as i32,
            data: raw_data,
        }))
    }

    /// Progressive reading - try to read UDT in progressively smaller chunks
    async fn read_udt_progressive(&mut self, tag_name: &str) -> crate::error::Result<PlcValue> {
        tracing::debug!("[PROGRESSIVE] Starting progressive reading");

        // Start with a small chunk and gradually increase
        let mut chunk_size = 4;
        let mut all_data = Vec::new();
        let mut offset = 0;

        while chunk_size <= 512 {
            match self
                .read_udt_chunk_advanced(tag_name, offset, chunk_size)
                .await
            {
                Ok(chunk_data) => {
                    if chunk_data.is_empty() {
                        break;
                    }

                    all_data.extend_from_slice(&chunk_data);
                    offset += chunk_data.len();

                    tracing::trace!(
                        "[PROGRESSIVE] Read {} bytes with chunk size {}",
                        chunk_data.len(),
                        chunk_size
                    );

                    // If we got the full chunk, try a larger one next time
                    if chunk_data.len() == chunk_size {
                        chunk_size = (chunk_size * 2).min(512);
                    }
                }
                Err(_) => {
                    // Reduce chunk size and try again
                    chunk_size /= 2;
                    if chunk_size < 4 {
                        break;
                    }
                }
            }
        }

        if all_data.is_empty() {
            return Err(crate::error::EtherNetIpError::Protocol(
                "Progressive reading failed".to_string(),
            ));
        }

        tracing::debug!("[PROGRESSIVE] Collected {} bytes total", all_data.len());

        // Get symbol_id from tag attributes
        let symbol_id = self
            .get_tag_attributes(tag_name)
            .await
            .ok()
            .and_then(|attr| attr.template_instance_id)
            .unwrap_or(0) as i32;

        // Return raw UDT data (generic approach - no parsing)
        Ok(PlcValue::Udt(UdtData {
            symbol_id,
            data: all_data,
        }))
    }

    /// Reads a UDT in chunks to handle large structures
    #[allow(dead_code)]
    async fn read_udt_in_chunks(&mut self, tag_name: &str) -> crate::error::Result<PlcValue> {
        const MAX_CHUNK_SIZE: usize = 1000; // Conservative chunk size
        let mut all_data = Vec::new();
        let mut offset = 0;
        let mut chunk_size = MAX_CHUNK_SIZE;

        loop {
            // Try to read a chunk
            match self.read_udt_chunk(tag_name, offset, chunk_size).await {
                Ok(chunk_data) => {
                    all_data.extend_from_slice(&chunk_data);
                    offset += chunk_data.len();

                    // If we got less data than requested, we're done
                    if chunk_data.len() < chunk_size {
                        break;
                    }
                }
                Err(crate::error::EtherNetIpError::Protocol(msg))
                    if msg.contains("Partial transfer") =>
                {
                    // Reduce chunk size and try again
                    chunk_size /= 2;
                    if chunk_size < 100 {
                        return Err(crate::error::EtherNetIpError::Protocol(
                            "UDT too large even for smallest chunk size".to_string(),
                        ));
                    }
                    continue;
                }
                Err(e) => return Err(e),
            }
        }

        // Get symbol_id from tag attributes
        let symbol_id = self
            .get_tag_attributes(tag_name)
            .await
            .ok()
            .and_then(|attr| attr.template_instance_id)
            .unwrap_or(0) as i32;

        // Return raw UDT data (generic approach - no parsing)
        Ok(PlcValue::Udt(UdtData {
            symbol_id,
            data: all_data,
        }))
    }

    /// Reads a specific chunk of a UDT
    #[allow(dead_code)]
    async fn read_udt_chunk(
        &mut self,
        tag_name: &str,
        offset: usize,
        size: usize,
    ) -> crate::error::Result<Vec<u8>> {
        // Build a read request for a specific range
        let mut request = Vec::new();

        // Service: Read Tag (0x4C)
        request.push(0x4C);

        // Path size (in words) - tag name + array index
        let path_size = 2 + tag_name.len().div_ceil(2); // Round up for word alignment
        request.push(path_size as u8);

        // Path: tag name
        request.extend_from_slice(tag_name.as_bytes());
        if !tag_name.len().is_multiple_of(2) {
            request.push(0); // Pad to word boundary
        }

        // Array index for chunk reading
        request.push(0x28); // Array index symbol
        request.push(0x02); // 2 bytes for index
        request.extend_from_slice(&(offset as u16).to_le_bytes());

        // Element count
        request.push(0x28); // Element count symbol
        request.push(0x02); // 2 bytes for count
        request.extend_from_slice(&(size as u16).to_le_bytes());

        // Data type (assume DINT for raw data)
        request.push(0x00);
        request.push(0x01);

        // Send the request
        let response = self.send_cip_request(&request).await?;
        let cip_data = self.extract_cip_from_response(&response)?;

        // Parse the response to get raw data
        if cip_data.len() < 2 {
            return Err(crate::error::EtherNetIpError::Protocol(
                "Response too short".to_string(),
            ));
        }

        let _data_type = u16::from_le_bytes([cip_data[0], cip_data[1]]);
        let data = &cip_data[2..];

        Ok(data.to_vec())
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
    pub async fn read_udt_member_by_offset(
        &mut self,
        udt_name: &str,
        member_offset: usize,
        member_size: usize,
        data_type: u16,
    ) -> crate::error::Result<PlcValue> {
        self.validate_session().await?;

        // Read the UDT data
        let udt_data = self.read_tag_raw(udt_name).await?;

        // Extract the member data
        if member_offset + member_size > udt_data.len() {
            return Err(crate::error::EtherNetIpError::Protocol(format!(
                "Member data incomplete: offset {} + size {} > UDT size {}",
                member_offset,
                member_size,
                udt_data.len()
            )));
        }

        let member_data = &udt_data[member_offset..member_offset + member_size];

        // Parse the member value using the data type
        let member = crate::udt::UdtMember {
            name: "temp".to_string(),
            data_type,
            offset: member_offset as u32,
            size: member_size as u32,
        };

        let udt = crate::udt::UserDefinedType::new(udt_name.to_string());
        Ok(udt.parse_member_value(&member, member_data)?)
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
    pub async fn write_udt_member_by_offset(
        &mut self,
        udt_name: &str,
        member_offset: usize,
        member_size: usize,
        data_type: u16,
        value: PlcValue,
    ) -> crate::error::Result<()> {
        self.validate_session().await?;

        // Read the current UDT data
        let mut udt_data = self.read_tag_raw(udt_name).await?;

        // Check bounds
        if member_offset + member_size > udt_data.len() {
            return Err(crate::error::EtherNetIpError::Protocol(format!(
                "Member data incomplete: offset {} + size {} > UDT size {}",
                member_offset,
                member_size,
                udt_data.len()
            )));
        }

        // Serialize the value
        let member = crate::udt::UdtMember {
            name: "temp".to_string(),
            data_type,
            offset: member_offset as u32,
            size: member_size as u32,
        };

        let udt = crate::udt::UserDefinedType::new(udt_name.to_string());
        let member_data = udt.serialize_member_value(&member, &value)?;

        // Update the UDT data
        let end_offset = member_offset + member_data.len();
        if end_offset <= udt_data.len() {
            udt_data[member_offset..end_offset].copy_from_slice(&member_data);
        } else {
            return Err(crate::error::EtherNetIpError::Protocol(format!(
                "Member data exceeds UDT size: {} > {}",
                end_offset,
                udt_data.len()
            )));
        }

        // Write the updated UDT data back
        self.write_tag_raw(udt_name, &udt_data).await
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
        let mut request = Vec::new();

        // Service: Get Attribute List (0x03)
        request.push(0x03);

        // Path: Tag name (ANSI extended symbolic segment)
        let tag_bytes = tag_name.as_bytes();
        request.push(0x91); // ANSI extended symbolic segment
        request.push(tag_bytes.len() as u8);
        request.extend_from_slice(tag_bytes);

        // Attribute count
        request.extend_from_slice(&[0x02, 0x00]); // 2 attributes

        // Attribute 1: Data Type (0x01)
        request.extend_from_slice(&[0x01, 0x00]);

        // Attribute 2: Template Instance ID (0x02)
        request.extend_from_slice(&[0x02, 0x00]);

        Ok(request)
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
        if response.len() < 8 {
            return Err(crate::error::EtherNetIpError::Protocol(
                "Attributes response too short".to_string(),
            ));
        }

        let mut offset = 0;

        // Parse data type
        let data_type = u16::from_le_bytes([response[offset], response[offset + 1]]);
        offset += 2;

        // Parse size
        let size = u32::from_le_bytes([
            response[offset],
            response[offset + 1],
            response[offset + 2],
            response[offset + 3],
        ]);
        offset += 4;

        // Parse template instance ID (if present)
        let template_instance_id = if response.len() > offset + 4 {
            Some(u32::from_le_bytes([
                response[offset],
                response[offset + 1],
                response[offset + 2],
                response[offset + 3],
            ]))
        } else {
            None
        };

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

    /// Builds CIP request for program-scoped tag list discovery
    fn build_program_tag_list_request(&self, _program_name: &str) -> crate::error::Result<Vec<u8>> {
        let mut request = vec![
            // Service: Get Instance Attribute List (0x55)
            0x55, // Path size: 3 words (6 bytes)
            0x03, // Path: Program Object (Class 0x6C), instance 0 placeholder.
            0x20, 0x6C, 0x25,
        ];
        request.extend_from_slice(&[0x00, 0x00, 0x00]);

        // Attribute count
        request.extend_from_slice(&[0x02, 0x00]); // 2 attributes

        // Attribute 1: Symbol Name (0x01)
        request.extend_from_slice(&[0x01, 0x00]);

        // Attribute 2: Data Type (0x02)
        request.extend_from_slice(&[0x02, 0x00]);

        Ok(request)
    }

    /// Parses one page of tag discovery results from a Get Instance Attribute List response.
    fn parse_tag_list_response_page(&self, response: &[u8]) -> crate::error::Result<TagListPage> {
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
                scope: udt::TagScope::Controller,
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

    /// Parses tag list response from CIP
    fn parse_tag_list_response(&self, response: &[u8]) -> crate::error::Result<Vec<TagAttributes>> {
        Ok(self.parse_tag_list_response_page(response)?.tags)
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

        // Check if this is array element access (e.g., "ArrayName[0]")
        if let Some((base_name, index)) = self.parse_array_element_access(tag_name) {
            tracing::debug!(
                "Detected array element write: {}[{}], using workaround",
                base_name,
                index
            );
            return self
                .write_array_element_workaround(&base_name, index, value)
                .await;
        }

        if let PlcValue::Bool(_) = value
            && let Some((parent_path, index)) = self.parse_final_array_element_access(tag_name)
            && self.detect_bool_array_path(&parent_path).await?
        {
            return self
                .write_bool_array_element_workaround(&parent_path, index, value)
                .await;
        }

        // Use specialized AB STRING format for STRING writes (required for proper Allen-Bradley STRING handling)
        // All data types including strings now use the standard write path
        // The PlcValue::to_bytes() method handles the correct format for each type

        // Use standard unconnected messaging for other data types
        let cip_request = self.build_write_request(tag_name, &value)?;

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

    /// Builds a write request specifically for Allen-Bradley string format
    fn _build_ab_string_write_request(
        &self,
        tag_name: &str,
        value: &PlcValue,
    ) -> crate::error::Result<Vec<u8>> {
        if let PlcValue::String(string_value) = value {
            tracing::debug!(
                "Building correct Allen-Bradley string write request for tag: '{}'",
                tag_name
            );

            let mut cip_request = Vec::new();

            // Service: Write Tag Service (0x4D)
            cip_request.push(0x4D);

            // Request Path Size (in words)
            let tag_bytes = tag_name.as_bytes();
            let path_len = if tag_bytes.len().is_multiple_of(2) {
                tag_bytes.len() + 2
            } else {
                tag_bytes.len() + 3
            } / 2;
            cip_request.push(path_len as u8);

            // Request Path
            cip_request.push(0x91); // ANSI Extended Symbol
            cip_request.push(tag_bytes.len() as u8);
            cip_request.extend_from_slice(tag_bytes);

            // Pad to word boundary if needed
            if !tag_bytes.len().is_multiple_of(2) {
                cip_request.push(0x00);
            }

            // Data Type: Allen-Bradley STRING (0x02A0)
            cip_request.extend_from_slice(&[0xA0, 0x02]);

            // Element Count (always 1 for single string)
            cip_request.extend_from_slice(&[0x01, 0x00]);

            // Build the correct AB STRING structure
            let string_bytes = string_value.as_bytes();
            let max_len: u16 = 82; // Standard AB STRING max length
            let current_len = string_bytes.len().min(max_len as usize) as u16;

            // AB STRING structure:
            // - Len (2 bytes) - number of characters used
            cip_request.extend_from_slice(&current_len.to_le_bytes());

            // - MaxLen (2 bytes) - maximum characters allowed (typically 82)
            cip_request.extend_from_slice(&max_len.to_le_bytes());

            // - Data[MaxLen] (82 bytes) - the character array, zero-padded
            let mut data_array = vec![0u8; max_len as usize];
            data_array[..current_len as usize]
                .copy_from_slice(&string_bytes[..current_len as usize]);
            cip_request.extend_from_slice(&data_array);

            tracing::trace!(
                "Built correct AB string write request ({} bytes): len={}, maxlen={}, data_len={}",
                cip_request.len(),
                current_len,
                max_len,
                string_bytes.len()
            );
            tracing::trace!(
                "First 32 bytes: {:02X?}",
                &cip_request[..std::cmp::min(32, cip_request.len())]
            );

            Ok(cip_request)
        } else {
            Err(EtherNetIpError::Protocol(
                "Expected string value for Allen-Bradley string write".to_string(),
            ))
        }
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

        let mut data = BytesMut::new();
        data.extend_from_slice(&values::write_data_type(value).to_le_bytes());
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

    /// Builds a raw write request with pre-serialized data
    fn build_write_request_raw(
        &self,
        tag_name: &str,
        data: &[u8],
    ) -> crate::error::Result<Vec<u8>> {
        let path = self.build_tag_path(tag_name);
        let request = CipRequest::new(WRITE_TAG, path, data.to_vec());
        let mut cip_request = BytesMut::new();
        request.encode(&mut cip_request)?;
        Ok(cip_request.to_vec())
    }

    /// Serializes a `PlcValue` into bytes for transmission
    #[allow(dead_code)]
    fn serialize_value(&self, value: &PlcValue) -> crate::error::Result<Vec<u8>> {
        let mut data = BytesMut::new();
        value.encode(&mut data);
        Ok(data.to_vec())
    }

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
    /// When general_status is 0xFF, the error code is in the additional status field.
    /// Format: [0]=service, [1]=reserved, [2]=0xFF, [3]=additional_status_size (words), [4-5]=extended_error_code
    fn parse_extended_error(&self, cip_data: &[u8]) -> crate::error::Result<String> {
        if cip_data.len() < 6 {
            return Err(EtherNetIpError::Protocol(
                "Extended error response too short".to_string(),
            ));
        }

        let additional_status_size = cip_data[3] as usize; // Size in words
        if additional_status_size == 0 || cip_data.len() < 4 + (additional_status_size * 2) {
            return Ok("Extended error (no additional status)".to_string());
        }

        // Extended error code is in the additional status field (2 bytes)
        // Try both little-endian and big-endian interpretations
        let extended_error_code_le = u16::from_le_bytes([cip_data[4], cip_data[5]]);
        let extended_error_code_be = u16::from_be_bytes([cip_data[4], cip_data[5]]);

        // Map extended error codes (these are the same as regular error codes but in extended format)
        // Try little-endian first (standard CIP format)
        let error_msg = match extended_error_code_le {
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
            // Try big-endian interpretation if little-endian doesn't match
            _ => {
                // Try big-endian interpretation
                match extended_error_code_be {
                    0x0001 => "Connection failure (extended, BE)".to_string(),
                    0x0002 => "Resource unavailable (extended, BE)".to_string(),
                    0x0003 => "Invalid parameter value (extended, BE)".to_string(),
                    0x0004 => "Path segment error (extended, BE)".to_string(),
                    0x0005 => "Path destination unknown (extended, BE)".to_string(),
                    0x0006 => "Partial transfer (extended, BE)".to_string(),
                    0x0007 => "Connection lost (extended, BE)".to_string(),
                    0x0008 => "Service not supported (extended, BE)".to_string(),
                    0x0009 => "Invalid attribute value (extended, BE)".to_string(),
                    0x000A => "Attribute list error (extended, BE)".to_string(),
                    0x000B => "Already in requested mode/state (extended, BE)".to_string(),
                    0x000C => "Object state conflict (extended, BE)".to_string(),
                    0x000D => "Object already exists (extended, BE)".to_string(),
                    0x000E => "Attribute not settable (extended, BE)".to_string(),
                    0x000F => "Privilege violation (extended, BE)".to_string(),
                    0x0010 => "Device state conflict (extended, BE)".to_string(),
                    0x0011 => "Reply data too large (extended, BE)".to_string(),
                    0x0012 => "Fragmentation of a primitive value (extended, BE)".to_string(),
                    0x0013 => "Not enough data (extended, BE)".to_string(),
                    0x0014 => "Attribute not supported (extended, BE)".to_string(),
                    0x0015 => "Too much data (extended, BE)".to_string(),
                    0x0016 => "Object does not exist (extended, BE)".to_string(),
                    0x0017 => {
                        "Service fragmentation sequence not in progress (extended, BE)".to_string()
                    }
                    0x0018 => "No stored attribute data (extended, BE)".to_string(),
                    0x0019 => "Store operation failure (extended, BE)".to_string(),
                    0x001A => {
                        "Routing failure, request packet too large (extended, BE)".to_string()
                    }
                    0x001B => {
                        "Routing failure, response packet too large (extended, BE)".to_string()
                    }
                    0x001C => "Missing attribute list entry data (extended, BE)".to_string(),
                    0x001D => "Invalid attribute value list (extended, BE)".to_string(),
                    0x001E => "Embedded service error (extended, BE)".to_string(),
                    0x001F => "Vendor specific error (extended, BE)".to_string(),
                    0x0020 => "Invalid parameter (extended, BE)".to_string(),
                    0x0021 => {
                        "Write-once value or medium already written (extended, BE)".to_string()
                    }
                    0x0022 => "Invalid reply received (extended, BE)".to_string(),
                    0x0023 => "Buffer overflow (extended, BE)".to_string(),
                    0x0024 => "Invalid message format (extended, BE)".to_string(),
                    0x0025 => "Key failure in path (extended, BE)".to_string(),
                    0x0026 => "Path size invalid (extended, BE)".to_string(),
                    0x0027 => "Unexpected attribute in list (extended, BE)".to_string(),
                    0x0028 => "Invalid member ID (extended, BE)".to_string(),
                    0x0029 => "Member not settable (extended, BE)".to_string(),
                    0x002A => "Group 2 only server general failure (extended, BE)".to_string(),
                    0x002B => "Unknown Modbus error (extended, BE)".to_string(),
                    0x002C => "Attribute not gettable (extended, BE)".to_string(),
                    // Check if it's a vendor-specific or composite error
                    _ if extended_error_code_le == 0x2107 || extended_error_code_be == 0x2107 => {
                        // 0x2107 might be a composite error or vendor-specific
                        // Bytes are [0x07, 0x21] - could be error 0x07 (Connection lost) with additional info 0x21
                        // Or could be a vendor-specific extended error
                        format!(
                            "Vendor-specific or composite extended error: 0x{extended_error_code_le:04X} (LE) / 0x{extended_error_code_be:04X} (BE). Raw bytes: [0x{:02X}, 0x{:02X}]. This may indicate the PLC does not support writing to UDT array element members directly.",
                            cip_data[4], cip_data[5]
                        )
                    }
                    _ => format!(
                        "Unknown extended CIP error code: 0x{extended_error_code_le:04X} (LE) / 0x{extended_error_code_be:04X} (BE). Raw bytes: [0x{:02X}, 0x{:02X}]",
                        cip_data[4], cip_data[5]
                    ),
                }
            }
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

        // Check for extended error (0xFF indicates extended error code)
        if general_status == 0xFF {
            let error_msg = self.parse_extended_error(cip_data)?;
            return Err(EtherNetIpError::Protocol(format!(
                "CIP Extended Error: {error_msg}"
            )));
        }

        // Regular error code
        let error_msg = self.get_cip_error_message(general_status);
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
            return "Multiple Service Response error: 0x1E (Embedded service error). On CompactLogix/ControlLogix this commonly indicates the controller rejected a direct STRING write in the batch request; treat it as a PLC firmware limitation, not a protocol bug.".to_string();
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

    /// Reads raw data from a tag
    async fn read_tag_raw(&mut self, tag_name: &str) -> crate::error::Result<Vec<u8>> {
        let response = self
            .send_cip_request(&self.build_read_request(tag_name)?)
            .await?;
        self.extract_cip_from_response(&response)
    }

    /// Writes raw data to a tag
    #[allow(dead_code)]
    async fn write_tag_raw(&mut self, tag_name: &str, data: &[u8]) -> crate::error::Result<()> {
        let request = self.build_write_request_raw(tag_name, data)?;
        let response = self.send_cip_request(&request).await?;

        // Check write response for errors
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

        tracing::info!("Write completed successfully");
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

        tracing::trace!(
            "Unconnected Send message ({} bytes): {:02X?}",
            ucmm_message.len(),
            &ucmm_message[..std::cmp::min(64, ucmm_message.len())]
        );

        let response_data = self.send_rr_data_item(&ucmm_message).await?;

        if let Ok(raw_cip_data) = self.extract_unconnected_data_item(&response_data) {
            let use_direct_fallback = raw_cip_data.len() >= 3
                && raw_cip_data[0] == 0xD2
                && raw_cip_data[2] != 0x00
                && self.route_path_snapshot().is_none();

            if use_direct_fallback {
                tracing::warn!(
                    "Unconnected Send returned 0xD2 status 0x{:02X}; retrying with direct CIP SendRRData fallback",
                    raw_cip_data[2]
                );
                return self.send_rr_data_item(cip_request).await;
            }
        }

        Ok(response_data)
    }

    async fn send_rr_data_item(&self, item_data: &[u8]) -> Result<Vec<u8>> {
        let send_data = SendDataRequest::unconnected(item_data);
        let mut packet = BytesMut::new();
        let mut cpf = BytesMut::new();
        send_data.encode(&mut cpf);
        EncapsulationHeader::send_rr_data(cpf.len() as u16, self.session_handle)
            .encode(&mut packet);
        packet.extend_from_slice(&cpf);

        tracing::trace!(
            "Built packet ({} bytes): {:02X?}",
            packet.len(),
            &packet[..std::cmp::min(64, packet.len())]
        );

        // Send packet with timeout
        let mut stream = self.stream.lock().await;
        stream
            .write_all(&packet)
            .await
            .map_err(EtherNetIpError::Io)?;

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
        if response_header.status != 0 {
            return Err(EtherNetIpError::Protocol(format!(
                "EIP Command failed. Status: 0x{:08X}",
                response_header.status
            )));
        }

        // Parse response length
        let response_length = response_header.length as usize;
        if response_length == 0 {
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
            if response.data.len() < 2 {
                return Err(EtherNetIpError::Protocol(
                    "Read response too short for data".to_string(),
                ));
            }

            let data_type = u16::from_le_bytes([response.data[0], response.data[1]]);
            let value_data = &response.data[2..];
            tracing::trace!(
                "Data type: 0x{:04X}, Value data ({} bytes): {:02X?}",
                data_type,
                value_data.len(),
                value_data
            );
            Ok(values::decode_payload(data_type, value_data)?)
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

    /// Unregisters the EtherNet/IP session with the PLC
    pub async fn unregister_session(&mut self) -> crate::error::Result<()> {
        tracing::info!("Unregistering session and cleaning up connections...");

        // Close all connected sessions first
        let _ = self.close_all_connected_sessions().await;

        let mut packet = BytesMut::with_capacity(24);
        EncapsulationHeader::new(UNREGISTER_SESSION, 0, self.session_handle).encode(&mut packet);

        self.stream
            .lock()
            .await
            .write_all(&packet)
            .await
            .map_err(EtherNetIpError::Io)?;

        tracing::info!("Session unregistered and all connections closed");
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

    async fn _get_connected_session(
        &mut self,
        session_name: &str,
    ) -> crate::error::Result<ConnectedSession> {
        // First check if we already have a session
        {
            let sessions = self.connected_sessions.lock().await;
            if let Some(session) = sessions.get(session_name) {
                return Ok(session.clone());
            }
        }

        // If we don't have a session, establish a new one
        let session = self.establish_connected_session(session_name).await?;

        // Store the new session
        let mut sessions = self.connected_sessions.lock().await;
        sessions.insert(session_name.to_string(), session.clone());

        Ok(session)
    }

    /// Enhanced UDT structure parser - tries multiple parsing strategies
    #[allow(dead_code)]
    fn parse_udt_structure(&self, data: &[u8]) -> crate::error::Result<PlcValue> {
        tracing::debug!("Parsing UDT structure with {} bytes", data.len());

        // Strategy 1: Try to parse as TestTagUDT structure (DINT, DINT, REAL)
        if data.len() >= 12 {
            let _offset = 0;

            // Try different byte alignments and interpretations
            for alignment in 0..4 {
                if alignment + 12 <= data.len() {
                    let aligned_data = &data[alignment..];

                    // Parse first DINT
                    if aligned_data.len() >= 4 {
                        let dint1_bytes = [
                            aligned_data[0],
                            aligned_data[1],
                            aligned_data[2],
                            aligned_data[3],
                        ];
                        let dint1_value = i32::from_le_bytes(dint1_bytes);

                        // Parse second DINT
                        if aligned_data.len() >= 8 {
                            let dint2_bytes = [
                                aligned_data[4],
                                aligned_data[5],
                                aligned_data[6],
                                aligned_data[7],
                            ];
                            let dint2_value = i32::from_le_bytes(dint2_bytes);

                            // Parse REAL
                            if aligned_data.len() >= 12 {
                                let real_bytes = [
                                    aligned_data[8],
                                    aligned_data[9],
                                    aligned_data[10],
                                    aligned_data[11],
                                ];
                                let real_value = f32::from_le_bytes(real_bytes);

                                tracing::trace!(
                                    "Alignment {}: DINT1={}, DINT2={}, REAL={}",
                                    alignment,
                                    dint1_value,
                                    dint2_value,
                                    real_value
                                );

                                // Check if this looks like reasonable values
                                if self.is_reasonable_udt_values(
                                    dint1_value,
                                    dint2_value,
                                    real_value,
                                ) {
                                    // Legacy parsing - return raw data with symbol_id=0
                                    // Note: These methods are deprecated in favor of generic UdtData approach
                                    tracing::debug!(
                                        "Found reasonable UDT values at alignment {}",
                                        alignment
                                    );
                                    return Ok(PlcValue::Udt(UdtData {
                                        symbol_id: 0, // Not available in this context
                                        data: data.to_vec(),
                                    }));
                                }
                            }
                        }
                    }
                }
            }
        }

        // Strategy 2: Try to parse as simple packed structure
        if data.len() >= 4 {
            // Try different interpretations of the data
            let interpretations = vec![
                ("DINT_at_start", 0, 4),
                ("DINT_at_end", data.len().saturating_sub(4), data.len()),
                ("DINT_middle", data.len() / 2, data.len() / 2 + 4),
            ];

            for (name, start, end) in interpretations {
                if end <= data.len() && end > start {
                    let bytes = &data[start..end];
                    if bytes.len() == 4 {
                        let dint_value =
                            i32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
                        tracing::trace!("{}: DINT = {}", name, dint_value);

                        if self.is_reasonable_value(dint_value) {
                            // Legacy parsing - return raw data with symbol_id=0
                            return Ok(PlcValue::Udt(UdtData {
                                symbol_id: 0, // Not available in this context
                                data: data.to_vec(),
                            }));
                        }
                    }
                }
            }
        }

        Err(crate::error::EtherNetIpError::Protocol(
            "Could not parse UDT structure".to_string(),
        ))
    }

    /// Simple UDT parser fallback
    /// Note: This is a legacy method. New code should use generic UdtData approach.
    #[allow(dead_code)]
    fn parse_udt_simple(&self, data: &[u8]) -> crate::error::Result<PlcValue> {
        // Legacy parsing - return raw data with symbol_id=0
        Ok(PlcValue::Udt(UdtData {
            symbol_id: 0, // Not available in this context
            data: data.to_vec(),
        }))
    }

    /// Check if UDT values look reasonable
    #[allow(dead_code)]
    fn is_reasonable_udt_values(&self, dint1: i32, dint2: i32, real: f32) -> bool {
        // Check for reasonable ranges
        let dint1_reasonable = (-1000..=1000).contains(&dint1);
        let dint2_reasonable = (-1000..=1000).contains(&dint2);
        let real_reasonable = (-1000.0..=1000.0).contains(&real) && real.is_finite();

        tracing::trace!(
            "Reasonableness check: DINT1={} ({}), DINT2={} ({}), REAL={} ({})",
            dint1,
            dint1_reasonable,
            dint2,
            dint2_reasonable,
            real,
            real_reasonable
        );

        dint1_reasonable && dint2_reasonable && real_reasonable
    }

    /// Check if a single value looks reasonable
    #[allow(dead_code)]
    fn is_reasonable_value(&self, value: i32) -> bool {
        (-1000..=1000).contains(&value)
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
            .parse_tag_list_response_page(&response)
            .expect("response should parse");

        assert!(page.partial_transfer);
        assert_eq!(page.last_instance_id, Some(0x1234));
        assert_eq!(page.tags.len(), 1);
        assert_eq!(page.tags[0].name, "Rate");
        assert_eq!(page.tags[0].data_type, 0x00C4);
        assert_eq!(page.tags[0].data_type_name, "DINT");
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
