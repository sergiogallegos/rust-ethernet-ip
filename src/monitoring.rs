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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionMetrics {
    pub active_connections: u32,
    pub total_connections: u64,
    pub failed_connections: u64,
    pub connection_uptime_avg: Duration,
    pub last_connection_time: Option<SystemTime>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationMetrics {
    pub total_reads: u64,
    pub total_writes: u64,
    pub successful_reads: u64,
    pub successful_writes: u64,
    pub failed_reads: u64,
    pub failed_writes: u64,
    pub batch_operations: u64,
    pub subscription_updates: u64,
    pub partial_batch_failures: u64,
    pub last_successful_read_time: Option<SystemTime>,
    pub last_failed_read_time: Option<SystemTime>,
    pub last_successful_write_time: Option<SystemTime>,
    pub last_failed_write_time: Option<SystemTime>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub avg_read_latency_ms: f64,
    pub avg_write_latency_ms: f64,
    pub max_read_latency_ms: f64,
    pub max_write_latency_ms: f64,
    pub reads_per_second: f64,
    pub writes_per_second: f64,
    pub memory_usage_mb: f64,
    pub cpu_usage_percent: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorMetrics {
    pub network_errors: u64,
    pub protocol_errors: u64,
    pub timeout_errors: u64,
    pub tag_not_found_errors: u64,
    pub data_type_errors: u64,
    pub session_errors: u64,
    pub route_path_errors: u64,
    pub embedded_service_errors: u64,
    pub known_controller_limitation_errors: u64,
    pub retriable_errors: u64,
    pub non_retriable_errors: u64,
    pub last_error_time: Option<SystemTime>,
    pub last_error_message: Option<String>,
    pub last_error_category: Option<ErrorCategory>,
    pub last_retriable_error_time: Option<SystemTime>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthMetrics {
    pub overall_health: HealthStatus,
    pub last_health_check: SystemTime,
    pub health_mode: HealthCheckMode,
    pub last_verified_health_check: Option<SystemTime>,
    pub consecutive_failures: u32,
    pub recovery_attempts: u32,
    pub system_uptime: Duration,
    pub last_success_time: Option<SystemTime>,
    pub last_failure_time: Option<SystemTime>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub enum HealthStatus {
    Healthy,
    Warning,
    Critical,
    Unknown,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[non_exhaustive]
pub enum HealthCheckMode {
    Passive,
    Verified,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[non_exhaustive]
pub enum ErrorCategory {
    Network,
    Timeout,
    Session,
    RoutePath,
    CipProtocol,
    BatchEmbeddedService,
    KnownControllerLimitation,
    DataType,
    NotFound,
    Unknown,
}

impl ErrorCategory {
    pub fn is_retriable(self) -> bool {
        matches!(
            self,
            ErrorCategory::Network | ErrorCategory::Timeout | ErrorCategory::Session
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticsSnapshot {
    pub captured_at: SystemTime,
    pub connections: ConnectionMetrics,
    pub operations: OperationMetrics,
    pub performance: PerformanceMetrics,
    pub errors: ErrorMetrics,
    pub health: HealthMetrics,
    pub system_metrics_are_placeholders: bool,
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
            EtherNetIpError::WriteNotApplied { .. } => ErrorCategory::CipProtocol,
        }
    }

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
