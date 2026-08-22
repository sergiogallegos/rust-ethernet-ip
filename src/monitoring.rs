//! Diagnostic metric models and the deprecated standalone monitor.

use crate::error::EtherNetIpError;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};
use tokio::sync::RwLock;

/// Production monitoring metrics for the EtherNet/IP library
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringMetrics {
    /// Connection statistics
    pub connections: ConnectionMetrics,
    /// Operation statistics
    pub operations: OperationMetrics,
    /// Performance statistics
    pub performance: PerformanceMetrics,
    /// Error statistics
    pub errors: ErrorMetrics,
    /// System health
    pub health: HealthMetrics,
}

impl MonitoringMetrics {
    /// Returns true because CPU/memory metrics in this legacy monitor are placeholders.
    pub fn system_metrics_are_placeholders(&self) -> bool {
        true
    }
}

/// Connection lifecycle counters.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionMetrics {
    /// Connections currently considered active.
    pub active_connections: u32,
    /// Successful connections recorded since startup.
    pub total_connections: u64,
    /// Failed connection attempts recorded since startup.
    pub failed_connections: u64,
    /// Average connection uptime when supplied by the caller.
    pub connection_uptime_avg: Duration,
    /// Time of the most recent successful connection.
    pub last_connection_time: Option<SystemTime>,
}

/// Tag-operation counters and timestamps.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationMetrics {
    /// Total read attempts.
    pub total_reads: u64,
    /// Total write attempts.
    pub total_writes: u64,
    /// Successful reads.
    pub successful_reads: u64,
    /// Successful writes.
    pub successful_writes: u64,
    /// Failed reads.
    pub failed_reads: u64,
    /// Failed writes.
    pub failed_writes: u64,
    /// Batch operations recorded.
    pub batch_operations: u64,
    /// Subscription updates recorded.
    pub subscription_updates: u64,
    /// Batches containing at least one failed item.
    pub partial_batch_failures: u64,
    /// Time of the most recent successful read.
    pub last_successful_read_time: Option<SystemTime>,
    /// Time of the most recent failed read.
    pub last_failed_read_time: Option<SystemTime>,
    /// Time of the most recent successful write.
    pub last_successful_write_time: Option<SystemTime>,
    /// Time of the most recent failed write.
    pub last_failed_write_time: Option<SystemTime>,
}

/// Aggregate latency and throughput measurements.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    /// Arithmetic mean read latency in milliseconds.
    pub avg_read_latency_ms: f64,
    /// Arithmetic mean write latency in milliseconds.
    pub avg_write_latency_ms: f64,
    /// Highest observed read latency in milliseconds.
    pub max_read_latency_ms: f64,
    /// Highest observed write latency in milliseconds.
    pub max_write_latency_ms: f64,
    /// Successful reads divided by monitor uptime.
    pub reads_per_second: f64,
    /// Successful writes divided by monitor uptime.
    pub writes_per_second: f64,
    /// Legacy placeholder; not measured by this monitor.
    pub memory_usage_mb: f64,
    /// Legacy placeholder; not measured by this monitor.
    pub cpu_usage_percent: f64,
}

/// Error counters grouped by actionable category.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorMetrics {
    /// Network I/O errors.
    pub network_errors: u64,
    /// CIP or other protocol errors.
    pub protocol_errors: u64,
    /// Operation timeouts.
    pub timeout_errors: u64,
    /// Missing-tag errors.
    pub tag_not_found_errors: u64,
    /// Data type or encoding errors.
    pub data_type_errors: u64,
    /// Session or connection-loss errors.
    pub session_errors: u64,
    /// Route-path errors.
    pub route_path_errors: u64,
    /// Multiple Service Packet item failures.
    pub embedded_service_errors: u64,
    /// Rejections classified as known controller limitations.
    pub known_controller_limitation_errors: u64,
    /// Errors safe to retry according to [`ErrorCategory::is_retriable`].
    pub retriable_errors: u64,
    /// Errors that should not be retried automatically.
    pub non_retriable_errors: u64,
    /// Time of the most recent error.
    pub last_error_time: Option<SystemTime>,
    /// Message from the most recent error.
    pub last_error_message: Option<String>,
    /// Category of the most recent error.
    pub last_error_category: Option<ErrorCategory>,
    /// Time of the most recent retriable error.
    pub last_retriable_error_time: Option<SystemTime>,
}

/// Current health assessment and recovery history.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthMetrics {
    /// Derived overall health state.
    pub overall_health: HealthStatus,
    /// Time the health state was last calculated.
    pub last_health_check: SystemTime,
    /// Whether health is passive or confirmed by an active request.
    pub health_mode: HealthCheckMode,
    /// Time of the most recent active health check.
    pub last_verified_health_check: Option<SystemTime>,
    /// Consecutive failures since the last success.
    pub consecutive_failures: u32,
    /// Recovery resets requested by the caller.
    pub recovery_attempts: u32,
    /// Elapsed time since monitoring began.
    pub system_uptime: Duration,
    /// Time of the most recent successful operation.
    pub last_success_time: Option<SystemTime>,
    /// Time of the most recent failed operation.
    pub last_failure_time: Option<SystemTime>,
}

/// Coarse health state derived from connection and error metrics.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub enum HealthStatus {
    /// Connected with no significant recent failure rate.
    Healthy,
    /// Elevated failure rate or repeated failures.
    Warning,
    /// High failure rate or sustained failures.
    Critical,
    /// Health cannot be established, commonly because there is no connection.
    Unknown,
}

/// Source of the current health assessment.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[non_exhaustive]
pub enum HealthCheckMode {
    /// Inferred from ordinary operation results.
    Passive,
    /// Confirmed by an explicit health request.
    Verified,
}

/// Stable error classification used by diagnostics and retry decisions.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[non_exhaustive]
pub enum ErrorCategory {
    /// Socket or other network I/O failure.
    Network,
    /// Request deadline exceeded.
    Timeout,
    /// EtherNet/IP session or connection failure.
    Session,
    /// Invalid or unreachable CIP route.
    RoutePath,
    /// General CIP protocol rejection.
    CipProtocol,
    /// Failure reported by an embedded batch service.
    BatchEmbeddedService,
    /// Recognized controller/firmware restriction.
    KnownControllerLimitation,
    /// Value type or encoding mismatch.
    DataType,
    /// Requested tag or path was not found.
    NotFound,
    /// Error did not match a stable category.
    Unknown,
}

impl ErrorCategory {
    /// Returns whether retrying may succeed without changing the request.
    pub fn is_retriable(self) -> bool {
        matches!(
            self,
            ErrorCategory::Network | ErrorCategory::Timeout | ErrorCategory::Session
        )
    }
}

/// Point-in-time diagnostic data suitable for serialization by wrappers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticsSnapshot {
    /// Time at which the snapshot was assembled.
    pub captured_at: SystemTime,
    /// Connection counters.
    pub connections: ConnectionMetrics,
    /// Operation counters.
    pub operations: OperationMetrics,
    /// Latency and throughput measurements.
    pub performance: PerformanceMetrics,
    /// Error counters and last-error details.
    pub errors: ErrorMetrics,
    /// Current health assessment.
    pub health: HealthMetrics,
    /// Whether CPU and memory fields are placeholders.
    pub system_metrics_are_placeholders: bool,
}

/// Diagnostics for controller-schema caching and bounded drift recovery.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SchemaCacheMetrics {
    /// Current clone-shared schema generation.
    pub generation: u64,
    /// Number of explicit comprehensive schema refreshes.
    pub refreshes: u64,
    /// Array-classification cache hits.
    pub array_classification_hits: u64,
    /// Array-classification cache misses.
    pub array_classification_misses: u64,
    /// Array-classification entries evicted by refresh or contradiction.
    pub array_classification_evictions: u64,
    /// Responses that contradicted cached schema assumptions.
    pub datatype_contradictions: u64,
    /// One-time read recoveries that succeeded.
    pub successful_read_recoveries: u64,
    /// One-time read recoveries that still failed.
    pub failed_read_recoveries: u64,
}

/// Production monitoring system for EtherNet/IP operations
#[deprecated(
    since = "1.2.0",
    note = "ProductionMonitor is a standalone placeholder not wired into EipClient; use EipClient diagnostics snapshots instead. The type will be removed in 2.0."
)]
pub struct ProductionMonitor {
    metrics: Arc<RwLock<MonitoringMetrics>>,
    start_time: Instant,
}

#[expect(
    deprecated,
    reason = "CODEX-AQ keeps ProductionMonitor compatibility until 2.0 removal"
)]
impl Default for ProductionMonitor {
    fn default() -> Self {
        Self::new()
    }
}

#[expect(
    deprecated,
    reason = "CODEX-AQ keeps ProductionMonitor compatibility until 2.0 removal"
)]
impl ProductionMonitor {
    /// Creates a zeroed standalone monitor.
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(RwLock::new(MonitoringMetrics {
                connections: ConnectionMetrics {
                    active_connections: 0,
                    total_connections: 0,
                    failed_connections: 0,
                    connection_uptime_avg: Duration::ZERO,
                    last_connection_time: None,
                },
                operations: OperationMetrics {
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
                performance: PerformanceMetrics {
                    avg_read_latency_ms: 0.0,
                    avg_write_latency_ms: 0.0,
                    max_read_latency_ms: 0.0,
                    max_write_latency_ms: 0.0,
                    reads_per_second: 0.0,
                    writes_per_second: 0.0,
                    memory_usage_mb: 0.0,
                    cpu_usage_percent: 0.0,
                },
                errors: ErrorMetrics {
                    network_errors: 0,
                    protocol_errors: 0,
                    timeout_errors: 0,
                    tag_not_found_errors: 0,
                    data_type_errors: 0,
                    session_errors: 0,
                    route_path_errors: 0,
                    embedded_service_errors: 0,
                    known_controller_limitation_errors: 0,
                    retriable_errors: 0,
                    non_retriable_errors: 0,
                    last_error_time: None,
                    last_error_message: None,
                    last_error_category: None,
                    last_retriable_error_time: None,
                },
                health: HealthMetrics {
                    overall_health: HealthStatus::Unknown,
                    last_health_check: SystemTime::now(),
                    health_mode: HealthCheckMode::Passive,
                    last_verified_health_check: None,
                    consecutive_failures: 0,
                    recovery_attempts: 0,
                    system_uptime: Duration::ZERO,
                    last_success_time: None,
                    last_failure_time: None,
                },
            })),
            start_time: Instant::now(),
        }
    }

    /// Record a successful read operation
    pub async fn record_read_success(&self, latency: Duration) {
        let mut metrics = self.metrics.write().await;
        metrics.operations.total_reads += 1;
        metrics.operations.successful_reads += 1;
        let now = SystemTime::now();
        metrics.operations.last_successful_read_time = Some(now);
        metrics.health.last_success_time = Some(now);
        metrics.health.consecutive_failures = 0;

        // Update latency metrics
        let latency_ms = latency.as_millis() as f64;
        metrics.performance.avg_read_latency_ms = (metrics.performance.avg_read_latency_ms
            * (metrics.operations.successful_reads - 1) as f64
            + latency_ms)
            / metrics.operations.successful_reads as f64;

        if latency_ms > metrics.performance.max_read_latency_ms {
            metrics.performance.max_read_latency_ms = latency_ms;
        }
    }

    /// Record a failed read operation
    pub async fn record_read_failure(&self, error_type: &str) {
        let mut metrics = self.metrics.write().await;
        metrics.operations.total_reads += 1;
        metrics.operations.failed_reads += 1;
        metrics.operations.last_failed_read_time = Some(SystemTime::now());
        self.record_error(&mut metrics, error_type);
    }

    /// Record a successful write operation
    pub async fn record_write_success(&self, latency: Duration) {
        let mut metrics = self.metrics.write().await;
        metrics.operations.total_writes += 1;
        metrics.operations.successful_writes += 1;
        let now = SystemTime::now();
        metrics.operations.last_successful_write_time = Some(now);
        metrics.health.last_success_time = Some(now);
        metrics.health.consecutive_failures = 0;

        // Update latency metrics
        let latency_ms = latency.as_millis() as f64;
        metrics.performance.avg_write_latency_ms = (metrics.performance.avg_write_latency_ms
            * (metrics.operations.successful_writes - 1) as f64
            + latency_ms)
            / metrics.operations.successful_writes as f64;

        if latency_ms > metrics.performance.max_write_latency_ms {
            metrics.performance.max_write_latency_ms = latency_ms;
        }
    }

    /// Record a failed write operation
    pub async fn record_write_failure(&self, error_type: &str) {
        let mut metrics = self.metrics.write().await;
        metrics.operations.total_writes += 1;
        metrics.operations.failed_writes += 1;
        metrics.operations.last_failed_write_time = Some(SystemTime::now());
        self.record_error(&mut metrics, error_type);
    }

    /// Record a partial batch failure without losing successful values.
    pub async fn record_partial_batch_failure(&self, error_type: &str) {
        let mut metrics = self.metrics.write().await;
        metrics.operations.batch_operations += 1;
        metrics.operations.partial_batch_failures += 1;
        self.record_error(&mut metrics, error_type);
    }

    /// Record a connection event
    pub async fn record_connection(&self, success: bool) {
        let mut metrics = self.metrics.write().await;
        if success {
            metrics.connections.total_connections += 1;
            metrics.connections.active_connections += 1;
            metrics.connections.last_connection_time = Some(SystemTime::now());
        } else {
            metrics.connections.failed_connections += 1;
        }
    }

    /// Record a disconnection event
    pub async fn record_disconnection(&self) {
        let mut metrics = self.metrics.write().await;
        if metrics.connections.active_connections > 0 {
            metrics.connections.active_connections -= 1;
        }
    }

    /// Record an error
    fn record_error(&self, metrics: &mut MonitoringMetrics, error_type: &str) {
        let category = Self::classify_error_type(error_type);
        let now = SystemTime::now();

        match category {
            ErrorCategory::Network => metrics.errors.network_errors += 1,
            ErrorCategory::Timeout => metrics.errors.timeout_errors += 1,
            ErrorCategory::Session => metrics.errors.session_errors += 1,
            ErrorCategory::RoutePath => metrics.errors.route_path_errors += 1,
            ErrorCategory::CipProtocol => metrics.errors.protocol_errors += 1,
            ErrorCategory::BatchEmbeddedService => {
                metrics.errors.protocol_errors += 1;
                metrics.errors.embedded_service_errors += 1;
            }
            ErrorCategory::KnownControllerLimitation => {
                metrics.errors.protocol_errors += 1;
                metrics.errors.known_controller_limitation_errors += 1;
            }
            ErrorCategory::DataType => metrics.errors.data_type_errors += 1,
            ErrorCategory::NotFound => metrics.errors.tag_not_found_errors += 1,
            ErrorCategory::Unknown => {}
        }

        if category.is_retriable() {
            metrics.errors.retriable_errors += 1;
            metrics.errors.last_retriable_error_time = Some(now);
        } else {
            metrics.errors.non_retriable_errors += 1;
        }

        metrics.errors.last_error_time = Some(now);
        metrics.errors.last_error_message = Some(error_type.to_string());
        metrics.errors.last_error_category = Some(category);
        metrics.health.consecutive_failures += 1;
        metrics.health.last_failure_time = Some(now);
    }

    /// Classifies a structured library error for diagnostics and retries.
    pub fn classify_error(error: &EtherNetIpError) -> ErrorCategory {
        match error {
            EtherNetIpError::Io(_) => ErrorCategory::Network,
            EtherNetIpError::Timeout(_) => ErrorCategory::Timeout,
            EtherNetIpError::Connection(_) | EtherNetIpError::ConnectionLost(_) => {
                ErrorCategory::Session
            }
            EtherNetIpError::TagNotFound(_) => ErrorCategory::NotFound,
            EtherNetIpError::DataTypeMismatch { .. } => ErrorCategory::DataType,
            EtherNetIpError::CipError { code, message }
            | EtherNetIpError::ReadError {
                status: code,
                message,
            }
            | EtherNetIpError::WriteError {
                status: code,
                message,
            } => Self::classify_status_and_message(Some(*code), message),
            EtherNetIpError::Protocol(message)
            | EtherNetIpError::InvalidResponse { reason: message }
            | EtherNetIpError::Other(message)
            | EtherNetIpError::Tag(message)
            | EtherNetIpError::Subscription(message)
            | EtherNetIpError::Udt(message)
            | EtherNetIpError::Permission(message)
            | EtherNetIpError::InvalidString { reason: message } => {
                Self::classify_status_and_message(None, message)
            }
            EtherNetIpError::Unsupported { .. } => ErrorCategory::CipProtocol,
            EtherNetIpError::StringTooLong { .. } => ErrorCategory::DataType,
            EtherNetIpError::Utf8(_) => ErrorCategory::DataType,
        }
    }

    /// Classifies a legacy string error name or message.
    pub fn classify_error_type(error_type: &str) -> ErrorCategory {
        match error_type {
            "network" => ErrorCategory::Network,
            "timeout" => ErrorCategory::Timeout,
            "tag_not_found" => ErrorCategory::NotFound,
            "data_type" => ErrorCategory::DataType,
            "session" => ErrorCategory::Session,
            "route_path" => ErrorCategory::RoutePath,
            "embedded_service" => ErrorCategory::BatchEmbeddedService,
            "known_controller_limitation" => ErrorCategory::KnownControllerLimitation,
            "protocol" => ErrorCategory::CipProtocol,
            other => Self::classify_status_and_message(None, other),
        }
    }

    fn classify_status_and_message(status: Option<u8>, message: &str) -> ErrorCategory {
        let lower = message.to_ascii_lowercase();

        if status == Some(0x1E) || lower.contains("embedded service error") {
            return ErrorCategory::BatchEmbeddedService;
        }
        if lower.contains("controller rejected")
            || lower.contains("does not support writing to udt array element members")
        {
            return ErrorCategory::KnownControllerLimitation;
        }
        if status == Some(0x04) || lower.contains("path segment error") || lower.contains("route") {
            return ErrorCategory::RoutePath;
        }
        if lower.contains("timed out") || lower.contains("timeout") {
            return ErrorCategory::Timeout;
        }
        if lower.contains("connection lost")
            || lower.contains("plc unreachable")
            || lower.contains("session")
            || lower.contains("keep-alive")
        {
            return ErrorCategory::Session;
        }
        if lower.contains("tag not found") {
            return ErrorCategory::NotFound;
        }
        if lower.contains("data type")
            || lower.contains("data-type")
            || lower.contains("0x2107")
            || lower.contains("invalid string")
            || lower.contains("utf-8")
        {
            return ErrorCategory::DataType;
        }
        if lower.contains("io error") || lower.contains("network") {
            return ErrorCategory::Network;
        }
        if status.is_some() || lower.contains("cip error") || lower.contains("protocol") {
            return ErrorCategory::CipProtocol;
        }

        ErrorCategory::Unknown
    }

    /// Get current metrics
    pub async fn get_metrics(&self) -> MonitoringMetrics {
        let mut metrics = self.metrics.read().await.clone();

        // Update system uptime
        metrics.health.system_uptime = self.start_time.elapsed();

        // Calculate operations per second
        let total_time = metrics.health.system_uptime.as_secs_f64();
        if total_time > 0.0 {
            metrics.performance.reads_per_second =
                metrics.operations.successful_reads as f64 / total_time;
            metrics.performance.writes_per_second =
                metrics.operations.successful_writes as f64 / total_time;
        }

        // Update health status
        metrics.health.overall_health = self.calculate_health_status(&metrics);
        metrics.health.last_health_check = SystemTime::now();
        if metrics.health.last_verified_health_check.is_none() {
            metrics.health.health_mode = HealthCheckMode::Passive;
        }

        metrics
    }

    /// Get a stable diagnostics snapshot for wrappers and service layers.
    pub async fn get_diagnostics_snapshot(&self) -> DiagnosticsSnapshot {
        let metrics = self.get_metrics().await;
        DiagnosticsSnapshot {
            captured_at: SystemTime::now(),
            connections: metrics.connections,
            operations: metrics.operations,
            performance: metrics.performance,
            errors: metrics.errors,
            health: metrics.health,
            system_metrics_are_placeholders: true,
        }
    }

    /// Calculate overall health status
    fn calculate_health_status(&self, metrics: &MonitoringMetrics) -> HealthStatus {
        let error_rate = if metrics.operations.total_reads + metrics.operations.total_writes > 0 {
            (metrics.operations.failed_reads + metrics.operations.failed_writes) as f64
                / (metrics.operations.total_reads + metrics.operations.total_writes) as f64
        } else {
            0.0
        };

        if error_rate > 0.1 || metrics.health.consecutive_failures > 10 {
            HealthStatus::Critical
        } else if error_rate > 0.05 || metrics.health.consecutive_failures > 5 {
            HealthStatus::Warning
        } else if metrics.connections.active_connections > 0 {
            HealthStatus::Healthy
        } else {
            HealthStatus::Unknown
        }
    }

    /// Start monitoring background tasks
    pub async fn start_monitoring(&self) {
        tracing::warn!(
            "ProductionMonitor::start_monitoring is deprecated and no longer spawns a placeholder metrics task"
        );
    }

    /// Reset consecutive failures (call after successful recovery)
    pub async fn reset_consecutive_failures(&self) {
        let mut metrics = self.metrics.write().await;
        metrics.health.consecutive_failures = 0;
        metrics.health.recovery_attempts += 1;
    }

    /// Record the outcome of an active, verified health check.
    pub async fn record_verified_health_check(&self, is_healthy: bool) {
        let mut metrics = self.metrics.write().await;
        let now = SystemTime::now();
        metrics.health.health_mode = HealthCheckMode::Verified;
        metrics.health.last_verified_health_check = Some(now);
        metrics.health.last_health_check = now;

        if is_healthy {
            metrics.health.last_success_time = Some(now);
            metrics.health.consecutive_failures = 0;
        } else {
            metrics.health.last_failure_time = Some(now);
            metrics.health.consecutive_failures += 1;
        }
    }
}

#[expect(
    deprecated,
    reason = "CODEX-AQ keeps ProductionMonitor compatibility until 2.0 removal"
)]
impl Clone for ProductionMonitor {
    fn clone(&self) -> Self {
        Self {
            metrics: Arc::clone(&self.metrics),
            start_time: self.start_time,
        }
    }
}

#[cfg(test)]
#[expect(
    deprecated,
    reason = "CODEX-AQ keeps ProductionMonitor unit coverage until 2.0 removal"
)]
mod tests {
    use super::*;
    use crate::error::EtherNetIpError;

    #[test]
    fn classify_timeout_and_route_path_errors() {
        assert_eq!(
            ProductionMonitor::classify_error(&EtherNetIpError::Timeout(Duration::from_secs(1))),
            ErrorCategory::Timeout
        );
        assert_eq!(
            ProductionMonitor::classify_error(&EtherNetIpError::Protocol(
                "Path segment error while resolving route".to_string()
            )),
            ErrorCategory::RoutePath
        );
    }

    #[test]
    fn classify_known_controller_limitation_and_embedded_service() {
        assert_eq!(
            ProductionMonitor::classify_error(&EtherNetIpError::Protocol(
                "Read/Write Tag data-type mismatch extended error: 0x2107".to_string()
            )),
            ErrorCategory::DataType
        );
        assert_eq!(
            ProductionMonitor::classify_error(&EtherNetIpError::WriteError {
                status: 0x1E,
                message: "Embedded service error".to_string(),
            }),
            ErrorCategory::BatchEmbeddedService
        );
    }

    #[tokio::test]
    async fn diagnostics_snapshot_distinguishes_verified_health() {
        let monitor = ProductionMonitor::new();
        monitor.record_read_success(Duration::from_millis(10)).await;

        let passive = monitor.get_diagnostics_snapshot().await;
        assert_eq!(passive.health.health_mode, HealthCheckMode::Passive);
        assert!(passive.health.last_verified_health_check.is_none());
        assert!(passive.operations.last_successful_read_time.is_some());

        monitor.record_verified_health_check(true).await;
        let verified = monitor.get_diagnostics_snapshot().await;
        assert_eq!(verified.health.health_mode, HealthCheckMode::Verified);
        assert!(verified.health.last_verified_health_check.is_some());
        assert!(verified.system_metrics_are_placeholders);
    }
}
