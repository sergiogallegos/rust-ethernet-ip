use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};
use tokio::sync::RwLock;
use tokio::time::interval;

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
    pub last_error_time: Option<SystemTime>,
    pub last_error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthMetrics {
    pub overall_health: HealthStatus,
    pub last_health_check: SystemTime,
    pub consecutive_failures: u32,
    pub recovery_attempts: u32,
    pub system_uptime: Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Warning,
    Critical,
    Unknown,
}

/// Production monitoring system for EtherNet/IP operations
pub struct ProductionMonitor {
    metrics: Arc<RwLock<MonitoringMetrics>>,
    start_time: Instant,
    system_start_time: SystemTime,
}

impl Default for ProductionMonitor {
    fn default() -> Self {
        Self::new()
    }
}

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
                    last_error_time: None,
                    last_error_message: None,
                },
                health: HealthMetrics {
                    overall_health: HealthStatus::Unknown,
                    last_health_check: SystemTime::now(),
                    consecutive_failures: 0,
                    recovery_attempts: 0,
                    system_uptime: Duration::ZERO,
                },
            })),
            start_time: Instant::now(),
            system_start_time: SystemTime::now(),
        }
    }

    /// Record a successful read operation
    pub async fn record_read_success(&self, latency: Duration) {
        let mut metrics = self.metrics.write().await;
        metrics.operations.total_reads += 1;
        metrics.operations.successful_reads += 1;

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
        self.record_error(&mut metrics, error_type).await;
    }

    /// Record a successful write operation
    pub async fn record_write_success(&self, latency: Duration) {
        let mut metrics = self.metrics.write().await;
        metrics.operations.total_writes += 1;
        metrics.operations.successful_writes += 1;

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
        self.record_error(&mut metrics, error_type).await;
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
    async fn record_error(&self, metrics: &mut MonitoringMetrics, error_type: &str) {
        match error_type {
            "network" => metrics.errors.network_errors += 1,
            "protocol" => metrics.errors.protocol_errors += 1,
            "timeout" => metrics.errors.timeout_errors += 1,
            "tag_not_found" => metrics.errors.tag_not_found_errors += 1,
            "data_type" => metrics.errors.data_type_errors += 1,
            _ => {}
        }

        metrics.errors.last_error_time = Some(SystemTime::now());
        metrics.errors.last_error_message = Some(error_type.to_string());
        metrics.health.consecutive_failures += 1;
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

        metrics
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
        let monitor = self.clone();
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(30));
            loop {
                interval.tick().await;
                monitor.update_system_metrics().await;
            }
        });
    }

    /// Update system-level metrics
    async fn update_system_metrics(&self) {
        let mut metrics = self.metrics.write().await;

        // Update memory usage (simplified)
        metrics.performance.memory_usage_mb = self.get_memory_usage();

        // Update CPU usage (simplified)
        metrics.performance.cpu_usage_percent = self.get_cpu_usage();
    }

    /// Get current memory usage (simplified implementation)
    fn get_memory_usage(&self) -> f64 {
        // In a real implementation, you would use system APIs
        // For now, return a placeholder
        10.0
    }

    /// Get current CPU usage (simplified implementation)
    fn get_cpu_usage(&self) -> f64 {
        // In a real implementation, you would use system APIs
        // For now, return a placeholder
        5.0
    }

    /// Reset consecutive failures (call after successful recovery)
    pub async fn reset_consecutive_failures(&self) {
        let mut metrics = self.metrics.write().await;
        metrics.health.consecutive_failures = 0;
        metrics.health.recovery_attempts += 1;
    }
}

impl Clone for ProductionMonitor {
    fn clone(&self) -> Self {
        Self {
            metrics: Arc::clone(&self.metrics),
            start_time: self.start_time,
            system_start_time: self.system_start_time,
        }
    }
}
