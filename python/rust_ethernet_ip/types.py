from dataclasses import dataclass


@dataclass(slots=True)
class WriteResult:
    tag_name: str
    success: bool
    error: str | None = None


@dataclass(slots=True)
class BatchWriteItem:
    tag_name: str
    value: object
    value_type: str | None = None


@dataclass(slots=True)
class DiagnosticsConnectionMetrics:
    active_connections: int
    total_connections: int
    failed_connections: int
    connection_uptime_avg_seconds: float
    last_connection_time_unix_ms: int | None


@dataclass(slots=True)
class DiagnosticsOperationMetrics:
    total_reads: int
    total_writes: int
    successful_reads: int
    successful_writes: int
    failed_reads: int
    failed_writes: int
    batch_operations: int
    subscription_updates: int
    partial_batch_failures: int
    last_successful_read_time_unix_ms: int | None
    last_failed_read_time_unix_ms: int | None
    last_successful_write_time_unix_ms: int | None
    last_failed_write_time_unix_ms: int | None


@dataclass(slots=True)
class DiagnosticsPerformanceMetrics:
    avg_read_latency_ms: float
    avg_write_latency_ms: float
    max_read_latency_ms: float
    max_write_latency_ms: float
    reads_per_second: float
    writes_per_second: float
    memory_usage_mb: float
    cpu_usage_percent: float


@dataclass(slots=True)
class DiagnosticsErrorMetrics:
    network_errors: int
    protocol_errors: int
    timeout_errors: int
    tag_not_found_errors: int
    data_type_errors: int
    session_errors: int
    route_path_errors: int
    embedded_service_errors: int
    known_controller_limitation_errors: int
    retriable_errors: int
    non_retriable_errors: int
    last_error_time_unix_ms: int | None
    last_error_message: str | None
    last_error_category: str | None
    last_retriable_error_time_unix_ms: int | None


@dataclass(slots=True)
class DiagnosticsHealthMetrics:
    overall_health: str
    health_mode: str
    last_health_check_unix_ms: int | None
    last_verified_health_check_unix_ms: int | None
    consecutive_failures: int
    recovery_attempts: int
    system_uptime_seconds: float
    last_success_time_unix_ms: int | None
    last_failure_time_unix_ms: int | None


@dataclass(slots=True)
class DiagnosticsSnapshot:
    captured_at_unix_ms: int | None
    system_metrics_are_placeholders: bool
    connections: DiagnosticsConnectionMetrics
    operations: DiagnosticsOperationMetrics
    performance: DiagnosticsPerformanceMetrics
    errors: DiagnosticsErrorMetrics
    health: DiagnosticsHealthMetrics
