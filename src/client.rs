use crate::EtherNetIpStream;
use crate::batch::{BatchConfig, BatchError, BatchOperation, BatchResult};
use crate::error::{EtherNetIpError, Result};
use crate::protocol::cip::{CipRequest, CipResponse, READ_TAG, SendDataRequest, WRITE_TAG};
use crate::protocol::encap::{
    EncapsulationHeader, REGISTER_SESSION, SEND_RR_DATA, UNREGISTER_SESSION,
};
use crate::protocol::values;
use crate::protocol::{Decode, Encode};
use crate::route::RoutePath;
use crate::schema::SchemaExport;
use crate::subscription::{SubscriptionOptions, TagSubscription};
use crate::tag_group::{
    TagGroupConfig, TagGroupEvent, TagGroupEventKind, TagGroupFailureDiagnostic, TagGroupSnapshot,
    TagGroupSubscription, TagGroupValueResult,
};
use crate::tag_manager::{TagManager, TagMetadata, TagPermissions, TagScope};
use crate::types::{ConnectedSession, ConnectionParameters, PlcValue, UdtData};
use crate::udt::{TagAttributes, UdtDefinition, UdtManager};
use crate::{TagPath, schema, udt};
use bytes::BytesMut;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
#[cfg(feature = "ffi")]
use std::sync::LazyLock;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
#[cfg(feature = "ffi")]
use tokio::runtime::Runtime;
use tokio::sync::Mutex;
use tokio::time::{Duration, Instant, timeout};

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
    /// Stream for network communication (supports any AsyncRead + AsyncWrite + Unpin stream)
    stream: Arc<Mutex<Box<dyn EtherNetIpStream>>>,
    /// Session handle for the connection
    session_handle: u32,
    /// Tag manager for handling tag operations
    tag_manager: Arc<Mutex<TagManager>>,
    /// UDT manager for handling UDT operations
    udt_manager: Arc<Mutex<UdtManager>>,
    /// Route path for PLC communication
    route_path: Option<RoutePath>,
    /// Maximum packet size for communication
    max_packet_size: u32,
    /// Last activity timestamp
    last_activity: Arc<Mutex<Instant>>,
    /// Configuration for batch operations
    batch_config: BatchConfig,
    /// Connected session management for Class 3 operations
    connected_sessions: Arc<Mutex<HashMap<String, ConnectedSession>>>,
    /// Connection sequence counter
    connection_sequence: Arc<Mutex<u32>>,
    /// Active tag subscriptions
    subscriptions: Arc<Mutex<Vec<TagSubscription>>>,
    /// Registered tag-group polling definitions
    tag_groups: Arc<Mutex<HashMap<String, TagGroupConfig>>>,
}

impl std::fmt::Debug for EipClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EipClient")
            .field("session_handle", &self.session_handle)
            .field("route_path", &self.route_path)
            .field("max_packet_size", &self.max_packet_size)
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
            route_path: None,
            max_packet_size: 4000,
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
            route_path: None,
            max_packet_size: 4000,
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
        self.max_packet_size = size.min(4000);
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
        self.route_path = Some(route);
    }

    /// Gets the current route path
    pub fn get_route_path(&self) -> Option<&RoutePath> {
        self.route_path.as_ref()
    }

    /// Removes the route path (uses direct connection)
    pub fn clear_route_path(&mut self) {
        self.route_path = None;
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

    /// Exports a stable schema view built from the current discovery APIs.
    pub async fn export_schema(&mut self) -> crate::error::Result<SchemaExport> {
        let (discovered_tags, discovery_warnings) =
            self.discover_tags_detailed_internal(true).await?;
        let mut schema = SchemaExport::new(self.route_path.as_ref());
        schema.warnings.extend(discovery_warnings);

        let mut tags = discovered_tags;
        tags.sort_by(|a, b| a.name.cmp(&b.name));
        schema.tags = tags.iter().map(schema::SchemaTag::from).collect();

        let mut udt_sources: std::collections::HashMap<u32, (&str, u32)> =
            std::collections::HashMap::new();
        for tag in &tags {
            if tag.data_type == 0x00A0
                && let Some(template_instance_id) = tag.template_instance_id
            {
                udt_sources
                    .entry(template_instance_id)
                    .or_insert((tag.name.as_str(), tag.size));
            }
        }

        let mut template_ids: Vec<u32> = udt_sources.keys().copied().collect();
        template_ids.sort_unstable();

        for template_id in template_ids {
            let Some((tag_name, size_bytes)) = udt_sources.get(&template_id).copied() else {
                continue;
            };

            match self
                .get_udt_definition_by_template_id(template_id, tag_name)
                .await
            {
                Ok((definition, structure_size_bytes)) => {
                    schema.udts.push(schema::SchemaUdt::from_definition(
                        &definition,
                        Some(template_id),
                        if size_bytes == 0 {
                            structure_size_bytes
                        } else {
                            size_bytes
                        },
                    ));
                }
                Err(err) => schema.warnings.push(format!(
                    "Failed to resolve UDT definition for tag '{}' (template {}): {}",
                    tag_name, template_id, err
                )),
            }
        }

        schema.udts.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(schema)
    }

    /// Exports the current schema view as pretty-printed JSON.
    pub async fn export_schema_json(&mut self) -> crate::error::Result<String> {
        let schema = self.export_schema().await?;
        serde_json::to_string_pretty(&schema).map_err(|err| {
            crate::error::EtherNetIpError::Protocol(format!(
                "Failed to serialize schema export to JSON: {}",
                err
            ))
        })
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
        udt.parse_member_value(&member, member_data)
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
            self.max_packet_size = max_packet_size.clamp(504, 4000);
            tracing::debug!("Negotiated packet size: {} bytes", self.max_packet_size);
        } else {
            // If negotiation fails, use default size
            self.max_packet_size = 4000;
            tracing::debug!("Using default packet size: {} bytes", self.max_packet_size);
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

    /// Quick connection health check (no I/O).
    ///
    /// Returns `true` if the session handle is valid and there has been activity
    /// within the last 150 seconds. Use this for cheap periodic checks; for a
    /// definitive check that the PLC is still responding, use [`check_health_detailed`](Self::check_health_detailed).
    pub async fn check_health(&self) -> bool {
        self.session_handle != 0
            && self.last_activity.lock().await.elapsed() < Duration::from_secs(150)
    }

    /// Builds a lightweight diagnostics snapshot from the current client state.
    pub async fn get_diagnostics_snapshot(&self) -> crate::DiagnosticsSnapshot {
        self.build_diagnostics_snapshot(crate::HealthCheckMode::Passive, self.check_health().await)
            .await
    }

    /// Verifies the connection by sending a keep-alive (and re-registering if needed).
    ///
    /// Use this when you need to confirm the PLC is still reachable (e.g. after
    /// a long idle or before a critical operation). On failure, consider
    /// reconnecting; check [`EtherNetIpError::is_retriable`](crate::error::EtherNetIpError::is_retriable) on errors.
    pub async fn check_health_detailed(&mut self) -> crate::error::Result<bool> {
        if self.session_handle == 0 {
            return Ok(false);
        }

        // Try sending a lightweight keep-alive command
        match self.send_keep_alive().await {
            Ok(()) => Ok(true),
            Err(_) => {
                // If keep-alive fails, try re-registering the session
                match self.register_session().await {
                    Ok(()) => Ok(true),
                    Err(_) => Ok(false),
                }
            }
        }
    }

    /// Builds a verified diagnostics snapshot by actively checking PLC connectivity.
    pub async fn get_diagnostics_snapshot_detailed(
        &mut self,
    ) -> crate::error::Result<crate::DiagnosticsSnapshot> {
        let is_healthy = self.check_health_detailed().await?;
        Ok(self
            .build_diagnostics_snapshot(crate::HealthCheckMode::Verified, is_healthy)
            .await)
    }

    async fn build_diagnostics_snapshot(
        &self,
        health_mode: crate::HealthCheckMode,
        is_healthy: bool,
    ) -> crate::DiagnosticsSnapshot {
        let now = std::time::SystemTime::now();
        let session_active = self.session_handle != 0;
        let last_activity_elapsed = self.last_activity.lock().await.elapsed();
        let last_success_time = if is_healthy || session_active {
            Some(
                now.checked_sub(last_activity_elapsed)
                    .unwrap_or(std::time::SystemTime::UNIX_EPOCH),
            )
        } else {
            None
        };

        let error_category = if session_active && !is_healthy {
            Some(crate::ErrorCategory::Session)
        } else {
            None
        };

        crate::DiagnosticsSnapshot {
            captured_at: now,
            connections: crate::ConnectionMetrics {
                active_connections: if session_active { 1 } else { 0 },
                total_connections: if session_active { 1 } else { 0 },
                failed_connections: 0,
                connection_uptime_avg: Duration::ZERO,
                last_connection_time: last_success_time,
            },
            operations: crate::OperationMetrics {
                total_reads: 0,
                total_writes: 0,
                successful_reads: 0,
                successful_writes: 0,
                failed_reads: 0,
                failed_writes: 0,
                batch_operations: 0,
                subscription_updates: 0,
                partial_batch_failures: 0,
                last_successful_read_time: None,
                last_failed_read_time: None,
                last_successful_write_time: None,
                last_failed_write_time: None,
            },
            performance: crate::PerformanceMetrics {
                avg_read_latency_ms: 0.0,
                avg_write_latency_ms: 0.0,
                max_read_latency_ms: 0.0,
                max_write_latency_ms: 0.0,
                reads_per_second: 0.0,
                writes_per_second: 0.0,
                memory_usage_mb: 0.0,
                cpu_usage_percent: 0.0,
            },
            errors: crate::ErrorMetrics {
                network_errors: 0,
                protocol_errors: 0,
                timeout_errors: 0,
                tag_not_found_errors: 0,
                data_type_errors: 0,
                session_errors: if error_category == Some(crate::ErrorCategory::Session) {
                    1
                } else {
                    0
                },
                route_path_errors: 0,
                embedded_service_errors: 0,
                known_controller_limitation_errors: 0,
                retriable_errors: if error_category == Some(crate::ErrorCategory::Session) {
                    1
                } else {
                    0
                },
                non_retriable_errors: 0,
                last_error_time: if error_category.is_some() {
                    Some(now)
                } else {
                    None
                },
                last_error_message: error_category.map(|_| {
                    "Detailed health check reported session-level connectivity failure".to_string()
                }),
                last_error_category: error_category,
                last_retriable_error_time: if error_category == Some(crate::ErrorCategory::Session)
                {
                    Some(now)
                } else {
                    None
                },
            },
            health: crate::HealthMetrics {
                overall_health: if is_healthy {
                    crate::HealthStatus::Healthy
                } else if session_active {
                    crate::HealthStatus::Critical
                } else {
                    crate::HealthStatus::Unknown
                },
                last_health_check: now,
                health_mode,
                last_verified_health_check: if health_mode == crate::HealthCheckMode::Verified {
                    Some(now)
                } else {
                    None
                },
                consecutive_failures: if is_healthy { 0 } else { 1 },
                recovery_attempts: 0,
                system_uptime: Duration::ZERO,
                last_success_time,
                last_failure_time: if session_active && !is_healthy {
                    Some(now)
                } else {
                    None
                },
            },
            system_metrics_are_placeholders: true,
        }
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
        let route_path_bytes = if let Some(route_path) = &self.route_path {
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
                && self.route_path.is_none();

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
            values::decode_payload(data_type, value_data)
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

    // =========================================================================
    // BATCH OPERATIONS IMPLEMENTATION
    // =========================================================================

    /// Executes a batch of read and write operations
    ///
    /// This is the main entry point for batch operations. It takes a slice of
    /// `BatchOperation` items and executes them efficiently by grouping them
    /// into optimal CIP packets based on the current `BatchConfig`.
    ///
    /// # Arguments
    ///
    /// * `operations` - A slice of operations to execute
    ///
    /// # Returns
    ///
    /// A vector of [`BatchResult`] items, one per executed operation.
    ///
    /// When `optimize_packet_packing` is enabled, operations may be regrouped
    /// by type for execution, so result order is not guaranteed to match the
    /// original mixed-operation input order. Use [`BatchResult::operation`] to
    /// correlate each result.
    ///
    /// # Performance
    ///
    /// Batch execution primarily reduces round trips by combining multiple
    /// operations into fewer requests. Observed throughput varies significantly
    /// between simulator and real hardware, and also depends on packet sizing,
    /// controller model, route path, and tag mix.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use rust_ethernet_ip::{EipClient, BatchOperation, PlcValue};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    ///     let mut client = EipClient::connect("192.168.1.100:44818").await?;
    ///
    ///     let operations = vec![
    ///         BatchOperation::Read { tag_name: "Motor1_Speed".to_string() },
    ///         BatchOperation::Read { tag_name: "Motor2_Speed".to_string() },
    ///         BatchOperation::Write {
    ///             tag_name: "SetPoint".to_string(),
    ///             value: PlcValue::Dint(1500)
    ///         },
    ///     ];
    ///
    ///     let results = client.execute_batch(&operations).await?;
    ///
    ///     for result in results {
    ///         match result.result {
    ///             Ok(Some(value)) => println!("Read value: {:?}", value),
    ///             Ok(None) => println!("Write successful"),
    ///             Err(e) => println!("Operation failed: {}", e),
    ///         }
    ///     }
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn execute_batch(
        &mut self,
        operations: &[BatchOperation],
    ) -> crate::error::Result<Vec<BatchResult>> {
        if operations.is_empty() {
            return Ok(Vec::new());
        }

        let start_time = Instant::now();
        tracing::debug!(
            "[BATCH] Starting batch execution with {} operations",
            operations.len()
        );

        // Group operations based on configuration
        let operation_groups = if self.batch_config.optimize_packet_packing {
            self.optimize_operation_groups(operations)
        } else {
            self.sequential_operation_groups(operations)
        };

        let mut all_results = Vec::with_capacity(operations.len());

        // Execute each group
        for (group_index, group) in operation_groups.iter().enumerate() {
            tracing::debug!(
                "[BATCH] Processing group {} with {} operations",
                group_index + 1,
                group.len()
            );

            match self.execute_operation_group(group).await {
                Ok(mut group_results) => {
                    all_results.append(&mut group_results);
                }
                Err(e) => {
                    if !self.batch_config.continue_on_error {
                        return Err(e);
                    }

                    // Create error results for this group
                    for op in group {
                        let error_result = BatchResult {
                            operation: op.clone(),
                            result: Err(BatchError::NetworkError(e.to_string())),
                            execution_time_us: 0,
                        };
                        all_results.push(error_result);
                    }
                }
            }
        }

        let total_time = start_time.elapsed();
        tracing::info!(
            "[BATCH] Completed batch execution in {:?} - {} operations processed",
            total_time,
            all_results.len()
        );

        Ok(all_results)
    }

    /// Reads multiple tags in a single batch operation
    ///
    /// This is a convenience method for read-only batch operations.
    /// It's optimized for reading many tags at once.
    ///
    /// # Arguments
    ///
    /// * `tag_names` - A slice of tag names to read
    ///
    /// # Returns
    ///
    /// A vector of tuples containing `(tag_name, result)` pairs
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use rust_ethernet_ip::EipClient;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    ///     let mut client = EipClient::connect("192.168.1.100:44818").await?;
    ///
    ///     let tags = ["Motor1_Speed", "Motor2_Speed", "Temperature", "Pressure"];
    ///     let results = client.read_tags_batch(&tags).await?;
    ///
    ///     for (tag_name, result) in results {
    ///         match result {
    ///             Ok(value) => println!("{}: {:?}", tag_name, value),
    ///             Err(e) => println!("{}: Error - {}", tag_name, e),
    ///         }
    ///     }
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn read_tags_batch(
        &mut self,
        tag_names: &[&str],
    ) -> crate::error::Result<Vec<(String, std::result::Result<PlcValue, BatchError>)>> {
        let operations: Vec<BatchOperation> = tag_names
            .iter()
            .map(|&name| BatchOperation::Read {
                tag_name: name.to_string(),
            })
            .collect();

        let results = self.execute_batch(&operations).await?;

        Ok(results
            .into_iter()
            .map(|result| {
                let tag_name = match &result.operation {
                    BatchOperation::Read { tag_name } => tag_name.clone(),
                    BatchOperation::Write { .. } => {
                        unreachable!("Should only have read operations")
                    }
                };

                let value_result = match result.result {
                    Ok(Some(value)) => Ok(value),
                    Ok(None) => Err(BatchError::Other(
                        "Unexpected None result for read operation".to_string(),
                    )),
                    Err(e) => Err(e),
                };

                (tag_name, value_result)
            })
            .collect())
    }

    /// Writes multiple tag values in a single batch operation
    ///
    /// This is a convenience method for write-only batch operations.
    /// It's optimized for writing many values at once.
    ///
    /// # Arguments
    ///
    /// * `tag_values` - A slice of `(tag_name, value)` tuples to write
    ///
    /// # Returns
    ///
    /// A vector of tuples containing `(tag_name, result)` pairs
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
    ///     let writes = vec![
    ///         ("SetPoint1", PlcValue::Bool(true)),
    ///         ("SetPoint2", PlcValue::Dint(2000)),
    ///         ("EnableFlag", PlcValue::Bool(true)),
    ///     ];
    ///
    ///     let results = client.write_tags_batch(&writes).await?;
    ///
    ///     for (tag_name, result) in results {
    ///         match result {
    ///             Ok(_) => println!("{}: Write successful", tag_name),
    ///             Err(e) => println!("{}: Write failed - {}", tag_name, e),
    ///         }
    ///     }
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn write_tags_batch(
        &mut self,
        tag_values: &[(&str, PlcValue)],
    ) -> crate::error::Result<Vec<(String, std::result::Result<(), BatchError>)>> {
        let operations: Vec<BatchOperation> = tag_values
            .iter()
            .map(|(name, value)| BatchOperation::Write {
                tag_name: name.to_string(),
                value: value.clone(),
            })
            .collect();

        let results = self.execute_batch(&operations).await?;

        Ok(results
            .into_iter()
            .map(|result| {
                let tag_name = match &result.operation {
                    BatchOperation::Write { tag_name, .. } => tag_name.clone(),
                    BatchOperation::Read { .. } => {
                        unreachable!("Should only have write operations")
                    }
                };

                let write_result = match result.result {
                    Ok(None) => Ok(()),
                    Ok(Some(_)) => Err(BatchError::Other(
                        "Unexpected value result for write operation".to_string(),
                    )),
                    Err(e) => Err(e),
                };

                (tag_name, write_result)
            })
            .collect())
    }

    /// Configures batch operation settings
    ///
    /// This method allows fine-tuning of batch operation behavior,
    /// including performance optimizations and error handling.
    ///
    /// # Arguments
    ///
    /// * `config` - The new batch configuration to use
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use rust_ethernet_ip::{EipClient, BatchConfig};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    ///     let mut client = EipClient::connect("192.168.1.100:44818").await?;
    ///
    ///     let config = BatchConfig {
    ///         max_operations_per_packet: 50,
    ///         max_packet_size: 1500,
    ///         packet_timeout_ms: 5000,
    ///         continue_on_error: false,
    ///         optimize_packet_packing: true,
    ///     };
    ///
    ///     client.configure_batch_operations(config);
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn configure_batch_operations(&mut self, config: BatchConfig) {
        self.batch_config = config;
        tracing::debug!(
            "[BATCH] Updated batch configuration: max_ops={}, max_size={}, timeout={}ms",
            self.batch_config.max_operations_per_packet,
            self.batch_config.max_packet_size,
            self.batch_config.packet_timeout_ms
        );
    }

    /// Gets current batch operation configuration
    pub fn get_batch_config(&self) -> &BatchConfig {
        &self.batch_config
    }

    // =========================================================================
    // INTERNAL BATCH OPERATION HELPERS
    // =========================================================================

    /// Groups operations optimally for batch processing
    fn optimize_operation_groups(&self, operations: &[BatchOperation]) -> Vec<Vec<BatchOperation>> {
        let mut groups = Vec::new();
        let mut reads = Vec::new();
        let mut writes = Vec::new();

        // Separate reads and writes
        for op in operations {
            match op {
                BatchOperation::Read { .. } => reads.push(op.clone()),
                BatchOperation::Write { .. } => writes.push(op.clone()),
            }
        }

        // Group reads
        for chunk in reads.chunks(self.batch_config.max_operations_per_packet) {
            groups.push(chunk.to_vec());
        }

        // Group writes
        for chunk in writes.chunks(self.batch_config.max_operations_per_packet) {
            groups.push(chunk.to_vec());
        }

        groups
    }

    /// Groups operations sequentially (preserves order)
    fn sequential_operation_groups(
        &self,
        operations: &[BatchOperation],
    ) -> Vec<Vec<BatchOperation>> {
        operations
            .chunks(self.batch_config.max_operations_per_packet)
            .map(|chunk| chunk.to_vec())
            .collect()
    }

    /// Executes a single group of operations as a CIP Multiple Service Packet
    async fn execute_operation_group(
        &mut self,
        operations: &[BatchOperation],
    ) -> crate::error::Result<Vec<BatchResult>> {
        let start_time = Instant::now();
        let mut results = Vec::with_capacity(operations.len());

        // Build Multiple Service Packet request
        let cip_request = self.build_multiple_service_packet(operations)?;

        // Send request and get response
        let response = self.send_cip_request(&cip_request).await?;

        // Parse response and create results
        let parsed_results = self.parse_multiple_service_response(&response, operations)?;

        let execution_time = start_time.elapsed();

        // Create BatchResult objects
        for (i, operation) in operations.iter().enumerate() {
            let op_execution_time = execution_time.as_micros() as u64 / operations.len() as u64;

            let result = if i < parsed_results.len() {
                match &parsed_results[i] {
                    Ok(value) => Ok(value.clone()),
                    Err(e) => Err(e.clone()),
                }
            } else {
                Err(BatchError::Other(
                    "Missing result from response".to_string(),
                ))
            };

            results.push(BatchResult {
                operation: operation.clone(),
                result,
                execution_time_us: op_execution_time,
            });
        }

        Ok(results)
    }

    /// Builds a CIP Multiple Service Packet request
    fn build_multiple_service_packet(
        &self,
        operations: &[BatchOperation],
    ) -> crate::error::Result<Vec<u8>> {
        let mut packet = Vec::with_capacity(8 + (operations.len() * 2));

        // Multiple Service Packet service code
        packet.push(0x0A);

        // Request path (2 bytes for class 0x02, instance 1)
        packet.push(0x02); // Path size in words
        packet.push(0x20); // Class segment
        packet.push(0x02); // Class 0x02 (Message Router)
        packet.push(0x24); // Instance segment
        packet.push(0x01); // Instance 1

        // Number of services
        packet.extend_from_slice(&(operations.len() as u16).to_le_bytes());

        // Calculate offset table
        let mut service_requests = Vec::with_capacity(operations.len());
        let mut current_offset = 2 + (operations.len() * 2); // Start after offset table

        for operation in operations {
            // Build individual service request
            let service_request = match operation {
                BatchOperation::Read { tag_name } => self.build_read_request(tag_name)?,
                BatchOperation::Write { tag_name, value } => {
                    self.build_write_request(tag_name, value)?
                }
            };

            service_requests.push(service_request);
        }

        // Add offset table
        for service_request in &service_requests {
            packet.extend_from_slice(&(current_offset as u16).to_le_bytes());
            current_offset += service_request.len();
        }

        // Add service requests
        for service_request in service_requests {
            packet.extend_from_slice(&service_request);
        }

        tracing::trace!(
            "[BATCH] Built Multiple Service Packet ({} bytes, {} services)",
            packet.len(),
            operations.len()
        );

        Ok(packet)
    }

    /// Parses a Multiple Service Packet response
    fn parse_multiple_service_response(
        &self,
        response: &[u8],
        operations: &[BatchOperation],
    ) -> crate::error::Result<Vec<std::result::Result<Option<PlcValue>, BatchError>>> {
        if response.len() < 6 {
            return Err(crate::error::EtherNetIpError::Protocol(
                "Response too short for Multiple Service Packet".to_string(),
            ));
        }

        let mut results = Vec::new();

        tracing::trace!(
            "Raw Multiple Service Response ({} bytes): {:02X?}",
            response.len(),
            response
        );

        // First, extract the CIP data from the EtherNet/IP response
        let cip_data = match self.extract_cip_from_response(response) {
            Ok(data) => data,
            Err(e) => {
                tracing::error!("Failed to extract CIP data: {}", e);
                return Err(e);
            }
        };

        tracing::trace!(
            "Extracted CIP data ({} bytes): {:02X?}",
            cip_data.len(),
            cip_data
        );

        if cip_data.len() < 6 {
            return Err(crate::error::EtherNetIpError::Protocol(
                "CIP data too short for Multiple Service Response".to_string(),
            ));
        }

        // Parse Multiple Service Response header from CIP data:
        // [0] = Service Code (0x8A)
        // [1] = Reserved (0x00)
        // [2] = General Status (0x00 for success)
        // [3] = Additional Status Size (0x00)
        // [4-5] = Number of replies (little endian)

        let service_code = cip_data[0];
        let general_status = cip_data[2];
        let num_replies = u16::from_le_bytes([cip_data[4], cip_data[5]]) as usize;

        tracing::debug!(
            "Multiple Service Response: service=0x{:02X}, status=0x{:02X}, replies={}",
            service_code,
            general_status,
            num_replies
        );

        if general_status != 0x00 {
            return Err(crate::error::EtherNetIpError::Protocol(
                self.describe_multiple_service_error(general_status, operations),
            ));
        }

        if num_replies != operations.len() {
            return Err(crate::error::EtherNetIpError::Protocol(format!(
                "Reply count mismatch: expected {}, got {}",
                operations.len(),
                num_replies
            )));
        }

        // Read reply offsets (each is 2 bytes, little endian)
        let mut reply_offsets = Vec::new();
        let mut offset = 6; // Skip header

        for _i in 0..num_replies {
            if offset + 2 > cip_data.len() {
                return Err(crate::error::EtherNetIpError::Protocol(
                    "CIP data too short for reply offsets".to_string(),
                ));
            }
            let reply_offset =
                u16::from_le_bytes([cip_data[offset], cip_data[offset + 1]]) as usize;
            reply_offsets.push(reply_offset);
            offset += 2;
        }

        tracing::trace!("Reply offsets: {:?}", reply_offsets);

        // The reply data starts after all the offsets
        let reply_base_offset = 6 + (num_replies * 2);

        tracing::trace!("Reply base offset: {}", reply_base_offset);

        // Parse each reply
        for (i, &reply_offset) in reply_offsets.iter().enumerate() {
            // Reply offset is relative to position 4 (after service code, reserved, status, additional status size)
            let reply_start = 4 + reply_offset;

            if reply_start >= cip_data.len() {
                results.push(Err(BatchError::Other(
                    "Reply offset beyond CIP data".to_string(),
                )));
                continue;
            }

            // Calculate reply end position
            let reply_end = if i + 1 < reply_offsets.len() {
                // Not the last reply - use next reply's offset as boundary
                4 + reply_offsets[i + 1]
            } else {
                // Last reply - goes to end of CIP data
                cip_data.len()
            };

            if reply_end > cip_data.len() || reply_start >= reply_end {
                results.push(Err(BatchError::Other(
                    "Invalid reply boundaries".to_string(),
                )));
                continue;
            }

            let reply_data = &cip_data[reply_start..reply_end];

            tracing::trace!(
                "Reply {} at offset {}: start={}, end={}, len={}",
                i,
                reply_offset,
                reply_start,
                reply_end,
                reply_data.len()
            );
            tracing::trace!("Reply {} data: {:02X?}", i, reply_data);

            let result = self.parse_individual_reply(reply_data, &operations[i]);
            results.push(result);
        }

        Ok(results)
    }

    /// Parses an individual service reply within a Multiple Service Packet response
    fn parse_individual_reply(
        &self,
        reply_data: &[u8],
        operation: &BatchOperation,
    ) -> std::result::Result<Option<PlcValue>, BatchError> {
        if reply_data.len() < 4 {
            return Err(BatchError::SerializationError(
                "Reply too short".to_string(),
            ));
        }

        tracing::trace!(
            "Parsing individual reply ({} bytes): {:02X?}",
            reply_data.len(),
            reply_data
        );

        // Each individual reply in Multiple Service Response has the same format as standalone CIP response:
        // [0] = Service Code (0xCC for read response, 0xCD for write response)
        // [1] = Reserved (0x00)
        // [2] = General Status (0x00 for success)
        // [3] = Additional Status Size (0x00)
        // [4..] = Response data (for reads) or empty (for writes)

        let service_code = reply_data[0];
        let general_status = reply_data[2];

        tracing::trace!(
            "Service code: 0x{:02X}, Status: 0x{:02X}",
            service_code,
            general_status
        );

        if general_status != 0x00 {
            let error_msg = self.get_cip_error_message(general_status);
            return Err(BatchError::CipError {
                status: general_status,
                message: error_msg,
            });
        }

        match operation {
            BatchOperation::Write { .. } => {
                // Write operations return no data on success
                Ok(None)
            }
            BatchOperation::Read { .. } => {
                // Read operations return data starting at offset 4
                if reply_data.len() < 6 {
                    return Err(BatchError::SerializationError(
                        "Read reply too short for data".to_string(),
                    ));
                }

                // Parse the data directly (skip the 4-byte header)
                // Data format: [type_low, type_high, value_bytes...]
                let data = &reply_data[4..];
                tracing::trace!("Parsing data ({} bytes): {:02X?}", data.len(), data);

                if data.len() < 2 {
                    return Err(BatchError::SerializationError(
                        "Data too short for type".to_string(),
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

                if data_type == values::BOOL_ARRAY_DWORD {
                    if value_data.len() < 4 {
                        return Err(BatchError::SerializationError(
                            "Missing packed BOOL array DWORD value".to_string(),
                        ));
                    }

                    let packed_value = u32::from_le_bytes([
                        value_data[0],
                        value_data[1],
                        value_data[2],
                        value_data[3],
                    ]);

                    if let BatchOperation::Read { tag_name } = operation
                        && let Some((_base_name, index)) = self.parse_array_element_access(tag_name)
                    {
                        let bit_index = index % 32;
                        let value = (packed_value >> bit_index) & 1 != 0;
                        tracing::trace!(
                            "Parsed packed BOOL array element '{}' from DWORD 0x{:08X} using bit {} -> {}",
                            tag_name,
                            packed_value,
                            bit_index,
                            value
                        );
                        return Ok(Some(PlcValue::Bool(value)));
                    }
                }

                values::decode_payload(data_type, value_data)
                    .map(Some)
                    .map_err(|e| BatchError::SerializationError(e.to_string()))
            }
        }
    }

    /// Writes a string value using Allen-Bradley UDT component access
    /// This writes to TestString.LEN and TestString.DATA separately
    pub async fn write_ab_string_components(
        &mut self,
        tag_name: &str,
        value: &str,
    ) -> crate::error::Result<()> {
        tracing::debug!(
            "[AB STRING] Writing string '{}' to tag '{}' using component access",
            value,
            tag_name
        );

        let string_bytes = value.as_bytes();
        let string_len = string_bytes.len() as i32;

        // Step 1: Write the length to TestString.LEN
        let len_tag = format!("{tag_name}.LEN");
        tracing::debug!("Step 1: Writing length {} to {}", string_len, len_tag);

        match self.write_tag(&len_tag, PlcValue::Dint(string_len)).await {
            Ok(_) => tracing::debug!("Length written successfully"),
            Err(e) => {
                tracing::error!("Length write failed: {}", e);
                return Err(e);
            }
        }

        // Step 2: Write the string data to TestString.DATA using array access
        tracing::debug!("Step 2: Writing string data to {}.DATA", tag_name);

        // We need to write each character individually to the DATA array
        for (i, &byte) in string_bytes.iter().enumerate() {
            let data_element = format!("{tag_name}.DATA[{i}]");
            match self
                .write_tag(&data_element, PlcValue::Sint(byte as i8))
                .await
            {
                Ok(_) => tracing::trace!("wrote STRING byte at position {}", i),
                Err(e) => {
                    tracing::error!("Failed to write byte {} to position {}: {}", byte, i, e);
                    return Err(e);
                }
            }
        }

        // Step 3: Clear remaining bytes (null terminate)
        if string_bytes.len() < 82 {
            let null_element = format!("{}.DATA[{}]", tag_name, string_bytes.len());
            match self.write_tag(&null_element, PlcValue::Sint(0)).await {
                Ok(_) => tracing::debug!("String null-terminated successfully"),
                Err(e) => tracing::warn!("Could not null-terminate: {}", e),
            }
        }

        tracing::info!("AB STRING component write completed!");
        Ok(())
    }

    /// Writes a string using a single UDT write with proper AB STRING format
    pub async fn write_ab_string_udt(
        &mut self,
        tag_name: &str,
        value: &str,
    ) -> crate::error::Result<()> {
        tracing::debug!(
            "[AB STRING UDT] Writing string '{}' to tag '{}' as UDT",
            value,
            tag_name
        );

        let string_bytes = value.as_bytes();
        if string_bytes.len() > 82 {
            return Err(EtherNetIpError::Protocol(
                "String too long for Allen-Bradley STRING (max 82 chars)".to_string(),
            ));
        }

        // Build a CIP request that writes the complete AB STRING structure
        let mut cip_request = Vec::new();

        // Service: Write Tag Service (0x4D)
        cip_request.push(0x4D);

        // Request Path
        let tag_path = self.build_tag_path(tag_name);
        cip_request.push((tag_path.len() / 2) as u8); // Path size in words
        cip_request.extend_from_slice(&tag_path);

        // Data Type: Allen-Bradley STRING (0x02A0) - but write as UDT components
        cip_request.extend_from_slice(&[0xA0, 0x00]); // UDT type
        cip_request.extend_from_slice(&[0x01, 0x00]); // Element count

        // AB STRING UDT structure:
        // - DINT .LEN (4 bytes)
        // - SINT .DATA[82] (82 bytes)

        // Write .LEN field (current string length)
        let len = string_bytes.len() as u32;
        cip_request.extend_from_slice(&len.to_le_bytes());

        // Write .DATA field (82 bytes total)
        cip_request.extend_from_slice(string_bytes); // Actual string data

        // Pad with zeros to reach 82 bytes
        let padding_needed = 82 - string_bytes.len();
        cip_request.extend_from_slice(&vec![0u8; padding_needed]);

        tracing::trace!("Built UDT write request: {} bytes total", cip_request.len());

        let response = self.send_cip_request(&cip_request).await?;

        if response.len() >= 3 {
            let general_status = response[2];
            if general_status == 0x00 {
                tracing::info!("AB STRING UDT write successful!");
                Ok(())
            } else {
                let error_msg = self.get_cip_error_message(general_status);
                Err(EtherNetIpError::Protocol(format!(
                    "AB STRING UDT write failed - CIP Error 0x{general_status:02X}: {error_msg}"
                )))
            }
        } else {
            Err(EtherNetIpError::Protocol(
                "Invalid AB STRING UDT write response".to_string(),
            ))
        }
    }

    /// Establishes a Class 3 connected session for STRING operations
    ///
    /// Connected sessions are required for certain operations like STRING writes
    /// in Allen-Bradley PLCs. This implements the Forward Open CIP service.
    /// Will try multiple connection parameter configurations until one succeeds.
    async fn establish_connected_session(
        &mut self,
        session_name: &str,
    ) -> crate::error::Result<ConnectedSession> {
        tracing::debug!(
            "[CONNECTED] Establishing connected session: '{}'",
            session_name
        );
        tracing::debug!("[CONNECTED] Will try multiple parameter configurations...");

        // Generate unique connection parameters
        *self.connection_sequence.lock().await += 1;
        let connection_serial = (*self.connection_sequence.lock().await & 0xFFFF) as u16;

        // Try different configurations until one works
        for config_id in 0..=5 {
            tracing::debug!(
                "[ATTEMPT {}] Trying configuration {}:",
                config_id + 1,
                config_id
            );

            let mut session = if config_id == 0 {
                ConnectedSession::new(connection_serial)
            } else {
                ConnectedSession::with_config(connection_serial, config_id)
            };

            // Generate unique connection IDs for this attempt
            session.o_to_t_connection_id =
                0x2000_0000 + *self.connection_sequence.lock().await + (config_id as u32 * 0x1000);
            session.t_to_o_connection_id =
                0x3000_0000 + *self.connection_sequence.lock().await + (config_id as u32 * 0x1000);

            // Build Forward Open request with this configuration
            let forward_open_request = self.build_forward_open_request(&session)?;

            tracing::debug!(
                "[ATTEMPT {}] Sending Forward Open request ({} bytes)",
                config_id + 1,
                forward_open_request.len()
            );

            // Send Forward Open request
            match self.send_cip_request(&forward_open_request).await {
                Ok(response) => {
                    // Try to parse the response - DON'T clone, modify the session directly!
                    match self.parse_forward_open_response(&mut session, &response) {
                        Ok(()) => {
                            // Success! Store the session and return
                            tracing::info!("[SUCCESS] Configuration {} worked!", config_id);
                            tracing::debug!("Connection ID: 0x{:08X}", session.connection_id);
                            tracing::debug!("O->T ID: 0x{:08X}", session.o_to_t_connection_id);
                            tracing::debug!("T->O ID: 0x{:08X}", session.t_to_o_connection_id);
                            tracing::debug!(
                                "Using Connection ID: 0x{:08X} for messaging",
                                session.connection_id
                            );

                            session.is_active = true;
                            let mut sessions = self.connected_sessions.lock().await;
                            sessions.insert(session_name.to_string(), session.clone());
                            return Ok(session);
                        }
                        Err(e) => {
                            tracing::warn!(
                                "[ATTEMPT {}] Configuration {} failed: {}",
                                config_id + 1,
                                config_id,
                                e
                            );

                            // If it's a specific status error, log it
                            if e.to_string().contains("status: 0x") {
                                tracing::debug!(
                                    "Status indicates: parameter incompatibility or resource conflict"
                                );
                            }
                        }
                    }
                }
                Err(e) => {
                    tracing::warn!(
                        "[ATTEMPT {}] Network error with config {}: {}",
                        config_id + 1,
                        config_id,
                        e
                    );
                }
            }

            // Small delay between attempts
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }

        // If we get here, all configurations failed
        Err(EtherNetIpError::Protocol(
            "All connection parameter configurations failed. PLC may not support connected messaging or has reached connection limits.".to_string()
        ))
    }

    /// Builds a Forward Open CIP request for establishing connected sessions
    fn build_forward_open_request(
        &self,
        session: &ConnectedSession,
    ) -> crate::error::Result<Vec<u8>> {
        let mut request = Vec::with_capacity(50);

        // CIP Forward Open Service (0x54)
        request.push(0x54);

        // Request path length (Connection Manager object)
        request.push(0x02); // 2 words

        // Class ID: Connection Manager (0x06)
        request.push(0x20); // Logical Class segment
        request.push(0x06);

        // Instance ID: Connection Manager instance (0x01)
        request.push(0x24); // Logical Instance segment
        request.push(0x01);

        // Forward Open parameters

        // Connection Timeout Ticks (1 byte) + Timeout multiplier (1 byte)
        request.push(0x0A); // Timeout ticks (10)
        request.push(session.timeout_multiplier);

        // Originator -> Target Connection ID (4 bytes, little-endian)
        request.extend_from_slice(&session.o_to_t_connection_id.to_le_bytes());

        // Target -> Originator Connection ID (4 bytes, little-endian)
        request.extend_from_slice(&session.t_to_o_connection_id.to_le_bytes());

        // Connection Serial Number (2 bytes, little-endian)
        request.extend_from_slice(&session.connection_serial.to_le_bytes());

        // Originator Vendor ID (2 bytes, little-endian)
        request.extend_from_slice(&session.originator_vendor_id.to_le_bytes());

        // Originator Serial Number (4 bytes, little-endian)
        request.extend_from_slice(&session.originator_serial.to_le_bytes());

        // Connection Timeout Multiplier (1 byte) - repeated for target
        request.push(session.timeout_multiplier);

        // Reserved bytes (3 bytes)
        request.extend_from_slice(&[0x00, 0x00, 0x00]);

        // Originator -> Target RPI (4 bytes, little-endian, microseconds)
        request.extend_from_slice(&session.rpi.to_le_bytes());

        // Originator -> Target connection parameters (4 bytes)
        let o_to_t_params = self.encode_connection_parameters(&session.o_to_t_params);
        request.extend_from_slice(&o_to_t_params.to_le_bytes());

        // Target -> Originator RPI (4 bytes, little-endian, microseconds)
        request.extend_from_slice(&session.rpi.to_le_bytes());

        // Target -> Originator connection parameters (4 bytes)
        let t_to_o_params = self.encode_connection_parameters(&session.t_to_o_params);
        request.extend_from_slice(&t_to_o_params.to_le_bytes());

        // Transport type/trigger (1 byte) - Class 3, Application triggered
        request.push(0xA3);

        // Connection Path Size (1 byte)
        request.push(0x02); // 2 words for Message Router path

        // Connection Path - Target the Message Router
        request.push(0x20); // Logical Class segment
        request.push(0x02); // Message Router class (0x02)
        request.push(0x24); // Logical Instance segment
        request.push(0x01); // Message Router instance (0x01)

        Ok(request)
    }

    /// Encodes connection parameters into a 32-bit value
    fn encode_connection_parameters(&self, params: &ConnectionParameters) -> u32 {
        let mut encoded = 0u32;

        // Connection size (bits 0-15)
        encoded |= params.size as u32;

        // Variable flag (bit 25)
        if params.variable_size {
            encoded |= 1 << 25;
        }

        // Connection type (bits 29-30)
        encoded |= (params.connection_type as u32) << 29;

        // Priority (bits 26-27)
        encoded |= (params.priority as u32) << 26;

        encoded
    }

    /// Parses Forward Open response and updates session with connection info
    fn parse_forward_open_response(
        &self,
        session: &mut ConnectedSession,
        response: &[u8],
    ) -> crate::error::Result<()> {
        if response.len() < 2 {
            return Err(EtherNetIpError::Protocol(
                "Forward Open response too short".to_string(),
            ));
        }

        let service = response[0];
        let status = response[1];

        // Check if this is a Forward Open Reply (0xD4)
        if service != 0xD4 {
            return Err(EtherNetIpError::Protocol(format!(
                "Unexpected service in Forward Open response: 0x{service:02X}"
            )));
        }

        // Check status
        if status != 0x00 {
            let error_msg = match status {
                0x01 => "Connection failure - Resource unavailable or already exists",
                0x02 => "Invalid parameter - Connection parameters rejected",
                0x03 => "Connection timeout - PLC did not respond in time",
                0x04 => "Connection limit exceeded - Too many connections",
                0x08 => "Invalid service - Forward Open not supported",
                0x0C => "Invalid attribute - Connection parameters invalid",
                0x13 => "Path destination unknown - Target object not found",
                0x26 => "Invalid parameter value - RPI or size out of range",
                _ => &format!("Unknown status: 0x{status:02X}"),
            };
            return Err(EtherNetIpError::Protocol(format!(
                "Forward Open failed with status 0x{status:02X}: {error_msg}"
            )));
        }

        // Parse successful response
        if response.len() < 16 {
            return Err(EtherNetIpError::Protocol(
                "Forward Open response data too short".to_string(),
            ));
        }

        // CRITICAL FIX: The Forward Open response contains the actual connection IDs assigned by the PLC
        // Use the IDs returned by the PLC, not our requested ones
        let actual_o_to_t_id =
            u32::from_le_bytes([response[2], response[3], response[4], response[5]]);
        let actual_t_to_o_id =
            u32::from_le_bytes([response[6], response[7], response[8], response[9]]);

        // Update session with the actual assigned connection IDs
        session.o_to_t_connection_id = actual_o_to_t_id;
        session.t_to_o_connection_id = actual_t_to_o_id;
        session.connection_id = actual_o_to_t_id; // Use O->T as the primary connection ID

        tracing::info!("[FORWARD OPEN] Success!");
        tracing::debug!(
            "O->T Connection ID: 0x{:08X} (PLC assigned)",
            session.o_to_t_connection_id
        );
        tracing::debug!(
            "T->O Connection ID: 0x{:08X} (PLC assigned)",
            session.t_to_o_connection_id
        );
        tracing::debug!(
            "Using Connection ID: 0x{:08X} for messaging",
            session.connection_id
        );

        Ok(())
    }

    /// Writes a string using connected explicit messaging
    pub async fn write_string_connected(
        &mut self,
        tag_name: &str,
        value: &str,
    ) -> crate::error::Result<()> {
        let session_name = format!("string_write_{tag_name}");
        let mut sessions = self.connected_sessions.lock().await;

        if !sessions.contains_key(&session_name) {
            drop(sessions); // Release the lock before calling establish_connected_session
            self.establish_connected_session(&session_name).await?;
            sessions = self.connected_sessions.lock().await;
        }

        let session = sessions.get(&session_name).cloned().ok_or_else(|| {
            crate::error::EtherNetIpError::Connection(format!(
                "connected session '{session_name}' was not available after establishment"
            ))
        })?;
        let request = self.build_connected_string_write_request(tag_name, value, &session)?;

        drop(sessions); // Release the lock before sending the request
        let response = self
            .send_connected_cip_request(&request, &session, &session_name)
            .await?;

        // Check if write was successful
        if response.len() >= 2 {
            let status = response[1];
            if status == 0x00 {
                Ok(())
            } else {
                let error_msg = self.get_cip_error_message(status);
                Err(EtherNetIpError::Protocol(format!(
                    "CIP Error 0x{status:02X}: {error_msg}"
                )))
            }
        } else {
            Err(EtherNetIpError::Protocol(
                "Invalid connected string write response".to_string(),
            ))
        }
    }

    /// Builds a string write request for connected messaging
    fn build_connected_string_write_request(
        &self,
        tag_name: &str,
        value: &str,
        _session: &ConnectedSession,
    ) -> crate::error::Result<Vec<u8>> {
        let mut request = Vec::new();

        // For connected messaging, use direct CIP Write service
        // The connection is already established, so we can send the request directly

        // CIP Write Service Code
        request.push(0x4D);

        // Tag path - use simple ANSI format for connected messaging
        let tag_bytes = tag_name.as_bytes();
        let path_size_words = (2 + tag_bytes.len()).div_ceil(2); // +1 for potential padding, /2 for word count
        request.push(path_size_words as u8);

        request.push(0x91); // ANSI symbol segment
        request.push(tag_bytes.len() as u8); // Length of tag name
        request.extend_from_slice(tag_bytes);

        // Add padding byte if needed to make path even length
        if !(2 + tag_bytes.len()).is_multiple_of(2) {
            request.push(0x00);
        }

        // Data type for AB STRING
        request.extend_from_slice(&[0xCE, 0x0F]); // AB STRING data type (4046)

        // Number of elements (always 1 for a single string)
        request.extend_from_slice(&[0x01, 0x00]);

        // Build the AB STRING structure payload
        let string_bytes = value.as_bytes();
        let max_len: u16 = 82; // Standard AB STRING max length
        let current_len = string_bytes.len().min(max_len as usize) as u16;

        // STRING structure:
        // - Len (2 bytes) - number of characters used
        request.extend_from_slice(&current_len.to_le_bytes());

        // - MaxLen (2 bytes) - maximum characters allowed (typically 82)
        request.extend_from_slice(&max_len.to_le_bytes());

        // - Data[MaxLen] (82 bytes) - the character array, zero-padded
        let mut data_array = vec![0u8; max_len as usize];
        data_array[..current_len as usize].copy_from_slice(&string_bytes[..current_len as usize]);
        request.extend_from_slice(&data_array);

        tracing::trace!(
            "Built connected string write request ({} bytes) for '{}' = '{}' (len={}, maxlen={})",
            request.len(),
            tag_name,
            value,
            current_len,
            max_len
        );
        tracing::trace!("Request: {:02X?}", request);

        Ok(request)
    }

    /// Sends a CIP request using connected messaging
    async fn send_connected_cip_request(
        &mut self,
        cip_request: &[u8],
        session: &ConnectedSession,
        session_name: &str,
    ) -> crate::error::Result<Vec<u8>> {
        tracing::debug!(
            "[CONNECTED] Sending connected CIP request ({} bytes) using T->O connection ID 0x{:08X}",
            cip_request.len(),
            session.t_to_o_connection_id
        );

        let mut packet = BytesMut::new();
        EncapsulationHeader::new(SEND_RR_DATA, 0, self.session_handle).encode(&mut packet);

        // CPF (Common Packet Format) data starts here
        let cpf_start = packet.len();

        // Interface handle (4 bytes)
        packet.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]);

        // Timeout (2 bytes) - 5 seconds
        packet.extend_from_slice(&[0x05, 0x00]);

        // Item count (2 bytes) - 2 items: Address + Data
        packet.extend_from_slice(&[0x02, 0x00]);

        // Item 1: Connected Address Item (specifies which connection to use)
        packet.extend_from_slice(&[0xA1, 0x00]); // Type: Connected Address Item (0x00A1)
        packet.extend_from_slice(&[0x04, 0x00]); // Length: 4 bytes
        // Use T->O connection ID (Target to Originator) for addressing
        packet.extend_from_slice(&session.t_to_o_connection_id.to_le_bytes());

        // Item 2: Connected Data Item (contains the CIP request + sequence)
        packet.extend_from_slice(&[0xB1, 0x00]); // Type: Connected Data Item (0x00B1)
        let data_length = cip_request.len() + 2; // +2 for sequence count
        packet.extend_from_slice(&(data_length as u16).to_le_bytes()); // Length

        // Clone session_name and session before acquiring the lock
        let session_name_clone = session_name.to_string();
        let _session_clone = session.clone();

        // Get the current session mutably to increment sequence counter
        let mut sessions = self.connected_sessions.lock().await;
        let current_sequence = if let Some(session_mut) = sessions.get_mut(&session_name_clone) {
            session_mut.sequence_count += 1;
            session_mut.sequence_count
        } else {
            1 // Fallback if session not found
        };

        // Drop the lock before sending the request
        drop(sessions);

        // Sequence count (2 bytes) - incremental counter for this connection
        packet.extend_from_slice(&current_sequence.to_le_bytes());

        // CIP request data
        packet.extend_from_slice(cip_request);

        // Update packet length in header (total CPF data size)
        let cpf_length = packet.len() - cpf_start;
        packet[2..4].copy_from_slice(&(cpf_length as u16).to_le_bytes());

        tracing::trace!(
            "[CONNECTED] Sending packet ({} bytes) with sequence {}",
            packet.len(),
            current_sequence
        );

        // Send packet
        let mut stream = self.stream.lock().await;
        stream
            .write_all(&packet)
            .await
            .map_err(EtherNetIpError::Io)?;

        // Read response header
        let mut header = [0u8; 24];
        stream
            .read_exact(&mut header)
            .await
            .map_err(EtherNetIpError::Io)?;

        // Check EtherNet/IP command status
        let mut header_bytes = &header[..];
        let response_header = EncapsulationHeader::decode(&mut header_bytes)?;
        if response_header.status != 0 {
            return Err(EtherNetIpError::Protocol(format!(
                "Connected message failed with status: 0x{:08X}",
                response_header.status
            )));
        }

        // Read response data
        let response_length = response_header.length as usize;
        let mut response_data = vec![0u8; response_length];
        stream
            .read_exact(&mut response_data)
            .await
            .map_err(EtherNetIpError::Io)?;

        let mut last_activity = self.last_activity.lock().await;
        *last_activity = Instant::now();

        tracing::trace!(
            "[CONNECTED] Received response ({} bytes)",
            response_data.len()
        );

        // Extract connected CIP response
        self.extract_connected_cip_from_response(&response_data)
    }

    /// Extracts CIP data from connected response
    fn extract_connected_cip_from_response(
        &self,
        response: &[u8],
    ) -> crate::error::Result<Vec<u8>> {
        tracing::trace!(
            "[CONNECTED] Extracting CIP from connected response ({} bytes): {:02X?}",
            response.len(),
            response
        );

        if response.len() < 12 {
            return Err(EtherNetIpError::Protocol(
                "Connected response too short for CPF header".to_string(),
            ));
        }

        // Parse CPF (Common Packet Format) structure
        // [0-3]: Interface handle
        // [4-5]: Timeout
        // [6-7]: Item count
        let item_count = u16::from_le_bytes([response[6], response[7]]) as usize;
        tracing::trace!("[CONNECTED] CPF item count: {}", item_count);

        let mut pos = 8; // Start after CPF header

        // Look for Connected Data Item (0x00B1)
        for _i in 0..item_count {
            if pos + 4 > response.len() {
                return Err(EtherNetIpError::Protocol(
                    "Response truncated while parsing items".to_string(),
                ));
            }

            let item_type = u16::from_le_bytes([response[pos], response[pos + 1]]);
            let item_length = u16::from_le_bytes([response[pos + 2], response[pos + 3]]) as usize;
            pos += 4; // Skip item header

            tracing::trace!(
                "[CONNECTED] Found item: type=0x{:04X}, length={}",
                item_type,
                item_length
            );

            if pos
                .checked_add(item_length)
                .is_none_or(|end| end > response.len())
            {
                return Err(EtherNetIpError::Protocol(
                    "Connected data item truncated".to_string(),
                ));
            }

            if item_type == 0x00B1 {
                // Connected Data Item
                // Connected Data Item contains [sequence_count(2)][cip_data]
                if item_length < 2 {
                    return Err(EtherNetIpError::Protocol(
                        "Connected data item too short for sequence".to_string(),
                    ));
                }

                let sequence_count = u16::from_le_bytes([response[pos], response[pos + 1]]);
                tracing::trace!("[CONNECTED] Sequence count: {}", sequence_count);

                // Extract CIP data (skip 2-byte sequence count)
                let cip_data = response[pos + 2..pos + item_length].to_vec();
                tracing::trace!(
                    "[CONNECTED] Extracted CIP data ({} bytes): {:02X?}",
                    cip_data.len(),
                    cip_data
                );

                return Ok(cip_data);
            } else {
                // Skip this item's data
                pos += item_length;
            }
        }

        Err(EtherNetIpError::Protocol(
            "Connected Data Item (0x00B1) not found in response".to_string(),
        ))
    }

    /// Closes a specific connected session
    async fn close_connected_session(&mut self, session_name: &str) -> crate::error::Result<()> {
        if let Some(session) = self.connected_sessions.lock().await.get(session_name) {
            let session = session.clone(); // Clone to avoid borrowing issues

            // Build Forward Close request
            let forward_close_request = self.build_forward_close_request(&session)?;

            // Send Forward Close request
            let _response = self.send_cip_request(&forward_close_request).await?;

            tracing::info!("[CONNECTED] Session '{}' closed successfully", session_name);
        }

        // Remove session from our tracking
        let mut sessions = self.connected_sessions.lock().await;
        sessions.remove(session_name);

        Ok(())
    }

    /// Builds a Forward Close CIP request for terminating connected sessions
    fn build_forward_close_request(
        &self,
        session: &ConnectedSession,
    ) -> crate::error::Result<Vec<u8>> {
        let mut request = Vec::with_capacity(21);

        // CIP Forward Close Service (0x4E)
        request.push(0x4E);

        // Request path length (Connection Manager object)
        request.push(0x02); // 2 words

        // Class ID: Connection Manager (0x06)
        request.push(0x20); // Logical Class segment
        request.push(0x06);

        // Instance ID: Connection Manager instance (0x01)
        request.push(0x24); // Logical Instance segment
        request.push(0x01);

        // Forward Close parameters

        // Connection Timeout Ticks (1 byte) + Timeout multiplier (1 byte)
        request.push(0x0A); // Timeout ticks (10)
        request.push(session.timeout_multiplier);

        // Connection Serial Number (2 bytes, little-endian)
        request.extend_from_slice(&session.connection_serial.to_le_bytes());

        // Originator Vendor ID (2 bytes, little-endian)
        request.extend_from_slice(&session.originator_vendor_id.to_le_bytes());

        // Originator Serial Number (4 bytes, little-endian)
        request.extend_from_slice(&session.originator_serial.to_le_bytes());

        // Connection Path Size (1 byte)
        request.push(0x02); // 2 words for Message Router path

        // Connection Path - Target the Message Router
        request.push(0x20); // Logical Class segment
        request.push(0x02); // Message Router class (0x02)
        request.push(0x24); // Logical Instance segment
        request.push(0x01); // Message Router instance (0x01)

        Ok(request)
    }

    /// Closes all connected sessions (called during disconnect)
    async fn close_all_connected_sessions(&mut self) -> crate::error::Result<()> {
        let session_names: Vec<String> = self
            .connected_sessions
            .lock()
            .await
            .keys()
            .cloned()
            .collect();

        for session_name in session_names {
            let _ = self.close_connected_session(&session_name).await; // Ignore errors during cleanup
        }

        Ok(())
    }

    /// Writes a string using unconnected explicit messaging with proper AB STRING format
    ///
    /// This method uses standard unconnected messaging instead of connected messaging
    /// and implements the proper Allen-Bradley STRING structure as described in the
    /// provided information about `Len`, `MaxLen`, and `Data[82]` format.
    pub async fn write_string_unconnected(
        &mut self,
        tag_name: &str,
        value: &str,
    ) -> crate::error::Result<()> {
        tracing::debug!(
            "[UNCONNECTED] Writing string '{}' to tag '{}' using unconnected messaging",
            value,
            tag_name
        );

        self.validate_session().await?;

        let string_bytes = value.as_bytes();
        if string_bytes.len() > 82 {
            return Err(EtherNetIpError::Protocol(
                "String too long for Allen-Bradley STRING (max 82 chars)".to_string(),
            ));
        }

        // Build the CIP request with proper AB STRING structure
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

        // Request Path: ANSI Extended Symbol Segment for tag name
        cip_request.push(0x91); // ANSI Extended Symbol Segment
        cip_request.push(tag_bytes.len() as u8); // Tag name length
        cip_request.extend_from_slice(tag_bytes); // Tag name

        // Pad to even length if necessary
        if !tag_bytes.len().is_multiple_of(2) {
            cip_request.push(0x00);
        }

        // For write operations, we don't include data type and element count
        // The PLC infers the data type from the tag definition

        // Build Allen-Bradley STRING structure based on what we see in read responses:
        // Looking at read response: [CE, 0F, 01, 00, 00, 00, 31, 00, ...]
        // Structure appears to be:
        // - Some header/identifier (2 bytes): 0xCE, 0x0F
        // - Length (2 bytes): number of characters
        // - MaxLength or padding (2 bytes): 0x00, 0x00
        // - Data array (variable length, null terminated)

        let _current_len = string_bytes.len().min(82) as u16;

        // Build the correct Allen-Bradley STRING structure to match what the PLC expects
        // Analysis of read response: [CE, 0F, 01, 00, 00, 00, 31, 00, 00, 00, ...]
        // Structure appears to be:
        // - Header (2 bytes): 0xCE, 0x0F (Allen-Bradley STRING identifier)
        // - Length (4 bytes, DINT): Number of characters currently used
        // - Data (variable): Character data followed by padding to complete the structure

        let current_len = string_bytes.len().min(82) as u32;

        // AB STRING header/identifier - this appears to be required
        cip_request.extend_from_slice(&[0xCE, 0x0F]);

        // Length (4 bytes) - number of characters used as DINT
        cip_request.extend_from_slice(&current_len.to_le_bytes());

        // Data bytes - the actual string content
        cip_request.extend_from_slice(&string_bytes[..current_len as usize]);

        // Add padding if the total structure needs to be a specific size
        // Based on reads, it looks like there might be additional padding after the data

        tracing::trace!(
            "Built Allen-Bradley STRING write request ({} bytes) for '{}' = '{}' (len={})",
            cip_request.len(),
            tag_name,
            value,
            current_len
        );
        tracing::trace!(
            "Request structure: Service=0x4D, Path={} bytes, Header=0xCE0F, Len={} (4 bytes), Data",
            path_len * 2,
            current_len
        );

        // Send the request using standard unconnected messaging
        let response = self.send_cip_request(&cip_request).await?;

        // Extract CIP response from EtherNet/IP wrapper
        let cip_response = self.extract_cip_from_response(&response)?;

        // Check if write was successful - use correct CIP response format
        if cip_response.len() >= 3 {
            let service_reply = cip_response[0]; // Should be 0xCD (0x4D + 0x80) for Write Tag reply
            let _additional_status_size = cip_response[1]; // Additional status size (usually 0)
            let status = cip_response[2]; // CIP status code at position 2

            tracing::trace!(
                "Write response - Service: 0x{:02X}, Status: 0x{:02X}",
                service_reply,
                status
            );

            if status == 0x00 {
                tracing::info!("[UNCONNECTED] String write completed successfully");
                Ok(())
            } else {
                let error_msg = self.get_cip_error_message(status);
                tracing::error!(
                    "[UNCONNECTED] String write failed: {} (0x{:02X})",
                    error_msg,
                    status
                );
                Err(EtherNetIpError::Protocol(format!(
                    "CIP Error 0x{status:02X}: {error_msg}"
                )))
            }
        } else {
            Err(EtherNetIpError::Protocol(
                "Invalid unconnected string write response - too short".to_string(),
            ))
        }
    }

    /// Write a string value to a PLC tag using unconnected messaging
    ///
    /// # Arguments
    ///
    /// * `tag_name` - The name of the tag to write to
    /// * `value` - The string value to write (max 82 characters)
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the write was successful
    /// * `Err(EtherNetIpError)` if the write failed
    ///
    /// # Errors
    ///
    /// * `StringTooLong` - If the string is longer than 82 characters
    /// * `InvalidString` - If the string contains invalid characters
    /// * `TagNotFound` - If the tag doesn't exist
    /// * `WriteError` - If the write operation fails
    pub async fn write_string(&mut self, tag_name: &str, value: &str) -> crate::error::Result<()> {
        // Validate string length
        if value.len() > 82 {
            return Err(crate::error::EtherNetIpError::StringTooLong {
                max_length: 82,
                actual_length: value.len(),
            });
        }

        // Validate string content (ASCII only)
        if !value.is_ascii() {
            return Err(crate::error::EtherNetIpError::InvalidString {
                reason: "String contains non-ASCII characters".to_string(),
            });
        }

        // Build the string write request
        let request = self.build_string_write_request(tag_name, value)?;

        // Send the request and get the response
        let response = self.send_cip_request(&request).await?;

        // Parse the response
        let cip_response = self.extract_cip_from_response(&response)?;

        // Check for errors in the response
        if cip_response.len() < 2 {
            return Err(crate::error::EtherNetIpError::InvalidResponse {
                reason: "Response too short".to_string(),
            });
        }

        let status = cip_response[0];
        if status != 0 {
            return Err(crate::error::EtherNetIpError::WriteError {
                status,
                message: self.get_cip_error_message(status),
            });
        }

        Ok(())
    }

    /// Build a string write request packet
    fn build_string_write_request(
        &self,
        tag_name: &str,
        value: &str,
    ) -> crate::error::Result<Vec<u8>> {
        let mut request = Vec::new();

        // CIP Write Service (0x4D)
        request.push(0x4D);

        // Tag path
        let tag_path = self.build_tag_path(tag_name);
        request.extend_from_slice(&tag_path);

        // AB STRING data structure
        request.extend_from_slice(&(value.len() as u16).to_le_bytes()); // Len
        request.extend_from_slice(&82u16.to_le_bytes()); // MaxLen

        // Data[82] with padding
        let mut data = [0u8; 82];
        let bytes = value.as_bytes();
        data[..bytes.len()].copy_from_slice(bytes);
        request.extend_from_slice(&data);

        Ok(request)
    }

    /// Subscribes to a tag for real-time updates.
    ///
    /// The returned [`TagSubscription`] can be used to:
    /// - [`wait_for_update()`](TagSubscription::wait_for_update) for the next value
    /// - [`get_last_value()`](TagSubscription::get_last_value) for the latest cached value
    /// - [`into_stream()`](TagSubscription::into_stream) for an async `Stream` of updates
    ///
    /// This API validates the tag with an initial read before returning so invalid or
    /// inaccessible tags fail fast instead of surfacing only through background polling logs.
    ///
    /// The background poll loop uses [`SubscriptionOptions::update_rate`] (milliseconds) between reads.
    pub async fn subscribe_to_tag(
        &self,
        tag_path: &str,
        options: SubscriptionOptions,
    ) -> Result<TagSubscription> {
        let subscription = TagSubscription::new(tag_path.to_string(), options.clone());
        let mut validation_client = self.clone();
        let initial_value = validation_client.read_tag(tag_path).await?;
        subscription.update_value(&initial_value).await?;

        let mut subscriptions = self.subscriptions.lock().await;
        let update_rate_ms = options.update_rate;
        subscriptions.push(subscription.clone());
        drop(subscriptions);

        let tag_path = tag_path.to_string();
        let mut client = self.clone();
        tokio::spawn(async move {
            let interval = std::time::Duration::from_millis(update_rate_ms as u64);
            loop {
                match client.read_tag(&tag_path).await {
                    Ok(value) => {
                        if let Err(e) = client.update_subscription(&tag_path, &value).await {
                            tracing::error!("Error updating subscription: {}", e);
                            break;
                        }
                    }
                    Err(e) => {
                        tracing::error!("Error reading tag {}: {}", tag_path, e);
                        break;
                    }
                }
                tokio::time::sleep(interval).await;
            }
        });
        Ok(subscription)
    }

    /// Subscribes to multiple tags. Returns one [`TagSubscription`] per tag in order.
    pub async fn subscribe_to_tags(
        &self,
        tags: &[(&str, SubscriptionOptions)],
    ) -> Result<Vec<TagSubscription>> {
        let mut subs = Vec::with_capacity(tags.len());
        for (tag_name, options) in tags {
            subs.push(self.subscribe_to_tag(tag_name, options.clone()).await?);
        }
        Ok(subs)
    }

    /// Registers or replaces a named tag group for grouped polling.
    ///
    /// Tag groups are useful for HMI/SCADA-style scan classes where multiple tags
    /// share a polling interval and should be read together.
    pub async fn upsert_tag_group(
        &self,
        group_name: &str,
        tags: &[&str],
        update_rate_ms: u32,
    ) -> Result<()> {
        if group_name.trim().is_empty() {
            return Err(EtherNetIpError::Protocol(
                "Tag group name cannot be empty".to_string(),
            ));
        }
        if tags.is_empty() {
            return Err(EtherNetIpError::Protocol(
                "Tag group must contain at least one tag".to_string(),
            ));
        }
        if update_rate_ms == 0 {
            return Err(EtherNetIpError::Protocol(
                "Tag group update rate must be greater than 0 ms".to_string(),
            ));
        }

        let config = TagGroupConfig {
            name: group_name.to_string(),
            tags: tags.iter().map(|s| (*s).to_string()).collect(),
            update_rate_ms,
        };

        let mut groups = self.tag_groups.lock().await;
        groups.insert(group_name.to_string(), config);
        Ok(())
    }

    /// Removes a named tag group.
    pub async fn remove_tag_group(&self, group_name: &str) -> bool {
        let mut groups = self.tag_groups.lock().await;
        groups.remove(group_name).is_some()
    }

    /// Lists all currently registered tag groups.
    pub async fn list_tag_groups(&self) -> Vec<TagGroupConfig> {
        let groups = self.tag_groups.lock().await;
        groups.values().cloned().collect()
    }

    /// Reads all tags in a group once and returns a snapshot.
    pub async fn read_tag_group_once(&self, group_name: &str) -> Result<TagGroupSnapshot> {
        let config = {
            let groups = self.tag_groups.lock().await;
            groups.get(group_name).cloned().ok_or_else(|| {
                EtherNetIpError::Protocol(format!("Tag group '{}' is not registered", group_name))
            })?
        };

        let mut client = self.clone();
        let tag_refs: Vec<&str> = config.tags.iter().map(String::as_str).collect();
        let values = client.read_tags_batch(&tag_refs).await?;

        let mapped = values
            .into_iter()
            .map(|(tag_name, result)| match result {
                Ok(value) => TagGroupValueResult {
                    tag_name,
                    value: Some(value),
                    error: None,
                },
                Err(e) => TagGroupValueResult {
                    tag_name,
                    value: None,
                    error: Some(e.to_string()),
                },
            })
            .collect();

        Ok(TagGroupSnapshot {
            group_name: config.name,
            sampled_at: std::time::SystemTime::now(),
            values: mapped,
        })
    }

    /// Starts background polling for a registered tag group.
    ///
    /// Use the returned subscription to await snapshots and to stop polling.
    pub async fn subscribe_tag_group(&self, group_name: &str) -> Result<TagGroupSubscription> {
        let config = {
            let groups = self.tag_groups.lock().await;
            groups.get(group_name).cloned().ok_or_else(|| {
                EtherNetIpError::Protocol(format!("Tag group '{}' is not registered", group_name))
            })?
        };

        let subscription = TagGroupSubscription::new(config.name.clone(), config.update_rate_ms);
        let subscription_task = subscription.clone();
        let mut client = self.clone();
        let tags = config.tags.clone();
        let interval = std::time::Duration::from_millis(config.update_rate_ms as u64);
        let group_name_owned = config.name.clone();

        tokio::spawn(async move {
            while subscription_task.is_active() {
                let tag_refs: Vec<&str> = tags.iter().map(String::as_str).collect();
                match client.read_tags_batch(&tag_refs).await {
                    Ok(values) => {
                        let has_errors = values.iter().any(|(_, result)| result.is_err());
                        let snapshot = TagGroupSnapshot {
                            group_name: group_name_owned.clone(),
                            sampled_at: std::time::SystemTime::now(),
                            values: values
                                .into_iter()
                                .map(|(tag_name, result)| match result {
                                    Ok(value) => TagGroupValueResult {
                                        tag_name,
                                        value: Some(value),
                                        error: None,
                                    },
                                    Err(e) => TagGroupValueResult {
                                        tag_name,
                                        value: None,
                                        error: Some(e.to_string()),
                                    },
                                })
                                .collect(),
                        };

                        let event = TagGroupEvent {
                            kind: if has_errors {
                                TagGroupEventKind::PartialError
                            } else {
                                TagGroupEventKind::Data
                            },
                            snapshot,
                            error: None,
                            failure: None,
                        };

                        if let Err(e) = subscription_task.publish_event(event).await {
                            tracing::error!(
                                "Tag group '{}' publish failed: {}",
                                group_name_owned,
                                e
                            );
                            break;
                        }
                    }
                    Err(e) => {
                        tracing::error!(
                            "Tag group '{}' polling read failed: {}",
                            group_name_owned,
                            e
                        );
                        let failure_event = TagGroupEvent {
                            kind: TagGroupEventKind::ReadFailure,
                            snapshot: TagGroupSnapshot {
                                group_name: group_name_owned.clone(),
                                sampled_at: std::time::SystemTime::now(),
                                values: Vec::new(),
                            },
                            error: Some(e.to_string()),
                            failure: Some(TagGroupFailureDiagnostic::from_error(&e)),
                        };
                        if let Err(publish_error) =
                            subscription_task.publish_event(failure_event).await
                        {
                            tracing::error!(
                                "Tag group '{}' failure-event publish failed: {}",
                                group_name_owned,
                                publish_error
                            );
                            break;
                        }
                    }
                }
                tokio::time::sleep(interval).await;
            }
        });

        Ok(subscription)
    }

    async fn update_subscription(&self, tag_name: &str, value: &PlcValue) -> Result<()> {
        let subscriptions = {
            let subscriptions = self.subscriptions.lock().await;
            subscriptions.clone()
        };
        for subscription in &subscriptions {
            if subscription.tag_path == tag_name && subscription.is_active() {
                subscription.update_value(value).await?;
            }
        }
        Ok(())
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
