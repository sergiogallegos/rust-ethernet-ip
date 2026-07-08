//! EtherNet/IP client library for Allen-Bradley CompactLogix and ControlLogix PLCs.
//!
//! `rust-ethernet-ip` provides async Rust APIs for explicit EtherNet/IP and CIP
//! tag operations, plus FFI surfaces used by the repository's .NET wrapper.
//! The current released crate line is `1.2.0`.
//!
//! ## Highlights
//!
//! - Async client API via [`EipClient`]
//! - Symbolic tag addressing, including program-scoped tags, array indexing, and nested UDT paths
//! - Batch reads, writes, and mixed execution with [`BatchOperation`]
//! - Route-path support for backplane and routed topologies via [`RoutePath`]
//! - UDT discovery, schema export, diagnostics, subscriptions, and tag-group polling
//!
//! ## Quick Start
//!
//! ```no_run
//! use rust_ethernet_ip::{EipClient, PlcValue};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let mut client = EipClient::connect("192.168.1.100:44818").await?;
//!     let running = client.read_tag("Program:Main.MotorRunning").await?;
//!     client
//!         .write_tag("Program:Main.SetPoint", PlcValue::Dint(1500))
//!         .await?;
//!
//!     println!("running={running:?}");
//!     Ok(())
//! }
//! ```
//!
//! Routed example:
//!
//! ```no_run
//! use rust_ethernet_ip::{EipClient, RoutePath};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let route = RoutePath::new().add_slot(0);
//!     let _client = EipClient::with_route_path("192.168.1.100:44818", route).await?;
//!     Ok(())
//! }
//! ```
//!
//! ## Known PLC/Firmware Limits
//!
//! Real-hardware validation for the `1.2.0` release line classified some nested
//! direct write shapes as controller/firmware limitations. Follow-up probes on
//! 2026-07-02 and 2026-07-03 corrected that picture: standalone standard Logix
//! `STRING` tags write successfully when encoded as the standard
//! `0x02A0`/`0x0FCE` structure, and scalar UDT array element members
//! (DINT/REAL/BOOL/INT) write successfully when the full member path is
//! preserved.
//!
//! - Direct writes to scalar UDT array element members are confirmed on
//!   5069-L330ERM fw38 and are attempted directly by the service layer.
//! - Direct writes to `STRING` members inside UDTs still fail with
//!   `0xFF/0x2107` under the current member encoding on 5069-L330ERM fw38.
//!   Whether a member-specific direct encoding exists remains under
//!   investigation.
//!
//! These failures surface as `0x2107` Read/Write Tag data-type mismatch errors.
//!
//! Recommended pattern for rejected STRING-member cases: read-modify-write the
//! full UDT or UDT array element instead of directly writing the nested member.

#![deny(unused_must_use, unsafe_op_in_unsafe_fn)]
#![cfg_attr(not(test), warn(clippy::print_stdout, clippy::dbg_macro))]

use tokio::io::{AsyncRead, AsyncWrite};

/// Trait for streams that can be used with EipClient
///
/// This trait combines the requirements for streams used with EtherNet/IP:
/// - `AsyncRead`: Read data from the stream
/// - `AsyncWrite`: Write data to the stream
/// - `Unpin`: Required for async operations
/// - `Send`: Required for cross-thread safety
///
/// Most tokio streams (like `TcpStream`, `UnixStream`, etc.) automatically
/// implement this trait. You can also implement it for custom stream wrappers
/// to add metrics, logging, or other functionality.
///
/// # Example
///
/// ```no_run
/// use rust_ethernet_ip::EtherNetIpStream;
/// use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
///
/// // Custom stream wrapper for metrics
/// struct MetricsStream<S> {
///     inner: S,
///     bytes_read: u64,
///     bytes_written: u64,
/// }
///
/// // Most tokio streams automatically implement EtherNetIpStream
/// // For example, TcpStream implements it:
/// use tokio::net::TcpStream;
/// // TcpStream: AsyncRead + AsyncWrite + Unpin + Send ✓
/// // Therefore: TcpStream implements EtherNetIpStream ✓
/// ```
pub trait EtherNetIpStream: AsyncRead + AsyncWrite + Unpin + Send {}

impl<S> EtherNetIpStream for S where S: AsyncRead + AsyncWrite + Unpin + Send {}

pub mod batch;
pub mod client;
pub mod config; // Production-ready configuration management
pub mod error;
#[cfg(feature = "ffi")]
pub mod ffi;
pub mod fleet;
pub mod monitoring; // Enterprise-grade monitoring and health checks
pub mod plc_manager;
pub(crate) mod protocol;
pub mod route;
pub mod schema;
pub mod subscription;
pub mod tag_group;
pub mod tag_manager;
pub mod tag_path;
pub mod types;
pub mod udt;
pub mod version;

// Re-export commonly used items
pub use batch::{BatchConfig, BatchError, BatchOperation, BatchResult};
pub use client::{Backoff, Client, ConnectionEvent, EipClient, RetryClient, RetryPolicy};
#[expect(
    deprecated,
    reason = "CODEX-AQ intentionally re-exports ProductionConfig for 1.x compatibility"
)]
pub use config::ProductionConfig;
pub use config::{
    ConnectionConfig, LogFormat, LogLevel, LogRotationSchedule, LoggingConfig, MonitoringConfig,
    PerformanceConfig, PlcSpecificConfig, SecurityConfig,
};
pub use error::{EtherNetIpError, Result};
pub use fleet::{Fleet, FleetEvent};
#[expect(
    deprecated,
    reason = "CODEX-AQ intentionally re-exports ProductionMonitor for 1.x compatibility"
)]
pub use monitoring::ProductionMonitor;
pub use monitoring::{
    ConnectionMetrics, DiagnosticsSnapshot, ErrorCategory, ErrorMetrics, HealthCheckMode,
    HealthMetrics, HealthStatus, MonitoringMetrics, OperationMetrics, PerformanceMetrics,
};
#[expect(
    deprecated,
    reason = "CODEX-AQ intentionally re-exports PlcManager for 1.x compatibility"
)]
pub use plc_manager::PlcManager;
pub use plc_manager::{PlcConfig, PlcConnection};
pub use route::{RouteHop, RoutePath};
pub use schema::{
    SchemaCapabilities, SchemaDataType, SchemaExport, SchemaLibraryInfo, SchemaRoutePath,
    SchemaScope, SchemaTag, SchemaTargetInfo, SchemaUdt, SchemaUdtMember,
};
#[expect(
    deprecated,
    reason = "CODEX-AQ intentionally re-exports SubscriptionManager for 1.x compatibility"
)]
pub use subscription::SubscriptionManager;
#[expect(
    deprecated,
    reason = "CODEX-AQ intentionally re-exports RealTimeSubscriptionManager for 1.x compatibility"
)]
pub use subscription::SubscriptionManager as RealTimeSubscriptionManager;
pub use subscription::{
    SubscriptionOptions, SubscriptionOptions as RealTimeSubscriptionOptions, TagSubscription,
    TagSubscription as RealTimeSubscription, TagSubscriptionEvent,
};
pub use tag_group::{
    TagGroupConfig, TagGroupEvent, TagGroupEventKind, TagGroupFailureCategory,
    TagGroupFailureDiagnostic, TagGroupSnapshot, TagGroupSubscription, TagGroupValueResult,
};
#[expect(
    deprecated,
    reason = "CODEX-AQ intentionally re-exports TagCache for 1.x compatibility"
)]
pub use tag_manager::TagCache;
pub use tag_manager::{TagManager, TagMetadata, TagPermissions, TagScope};
pub use tag_path::TagPath;
pub use types::{PlcValue, UdtData};
pub use udt::{TagAttributes, UdtDefinition, UdtMember, UdtTemplate};

#[cfg(feature = "ffi")]
pub(crate) use client::RUNTIME;

/// Initialize tracing subscriber with environment-based filtering
///
/// This function sets up the tracing subscriber to use the `RUST_LOG` environment variable
/// for log level filtering. If not called, tracing events will be ignored.
///
/// # Examples
///
/// ```no_run
/// use rust_ethernet_ip::init_tracing;
///
/// // Initialize with default settings (reads RUST_LOG env var)
/// init_tracing();
///
/// // Or set RUST_LOG before calling:
/// // RUST_LOG=debug cargo run
/// ```
///
/// # Log Levels
///
/// Set the `RUST_LOG` environment variable to control logging:
/// - `RUST_LOG=trace` - Most verbose (all events)
/// - `RUST_LOG=debug` - Debug information
/// - `RUST_LOG=info` - Informational messages (default)
/// - `RUST_LOG=warn` - Warnings only
/// - `RUST_LOG=error` - Errors only
/// - `RUST_LOG=rust_ethernet_ip=debug` - Debug for this crate only
///
/// # Panics
///
/// This function will panic if called more than once. Use `try_init_tracing()` for
/// non-panicking initialization.
pub fn init_tracing() {
    use tracing_subscriber::EnvFilter;
    use tracing_subscriber::fmt;

    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    let subscriber = fmt::Subscriber::builder()
        .with_env_filter(filter)
        .with_target(false) // Don't show module paths by default
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("Failed to set tracing subscriber");
}

/// Try to initialize tracing subscriber (non-panicking version)
///
/// Returns `Ok(())` if initialization was successful, or an error if a subscriber
/// was already set.
pub fn try_init_tracing() -> crate::error::Result<()> {
    use tracing_subscriber::EnvFilter;
    use tracing_subscriber::fmt;

    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    let subscriber = fmt::Subscriber::builder()
        .with_env_filter(filter)
        .with_target(false)
        .finish();

    tracing::subscriber::set_global_default(subscriber)
        .map_err(|e| crate::error::EtherNetIpError::Other(e.to_string()))?;
    Ok(())
}
