from dataclasses import dataclass, field
from typing import Literal


RouteHopKind = Literal["backplane", "ethernet"]


@dataclass(frozen=True, slots=True)
class RouteHop:
    kind: RouteHopKind
    port: int
    slot: int | None = None
    address: str | None = None

    @staticmethod
    def backplane(slot: int, port: int = 1) -> "RouteHop":
        return RouteHop(kind="backplane", port=port, slot=slot)

    @staticmethod
    def ethernet(address: str, port: int = 2) -> "RouteHop":
        return RouteHop(kind="ethernet", port=port, address=address)


@dataclass(slots=True)
class RoutePath:
    slots: list[int] | None = None
    ports: list[int] | None = None
    addresses: list[str] | None = None
    hops: list[RouteHop] | None = None

    def ordered_hops(self) -> list[RouteHop]:
        if self.hops is not None:
            return list(self.hops)

        hops = [RouteHop.backplane(slot) for slot in (self.slots or [])]
        ports = self.ports or []
        for index, address in enumerate(self.addresses or []):
            port = ports[index] if index < len(ports) else 2
            hops.append(RouteHop.ethernet(address, port))
        return hops


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
class DiagnosticsSchemaCacheMetrics:
    generation: int = 0
    refreshes: int = 0
    array_classification_hits: int = 0
    array_classification_misses: int = 0
    array_classification_evictions: int = 0
    datatype_contradictions: int = 0
    successful_read_recoveries: int = 0
    failed_read_recoveries: int = 0


@dataclass(slots=True)
class DiagnosticsSnapshot:
    captured_at_unix_ms: int | None
    system_metrics_are_placeholders: bool
    connections: DiagnosticsConnectionMetrics
    operations: DiagnosticsOperationMetrics
    performance: DiagnosticsPerformanceMetrics
    errors: DiagnosticsErrorMetrics
    health: DiagnosticsHealthMetrics
    schema_cache: DiagnosticsSchemaCacheMetrics = field(default_factory=DiagnosticsSchemaCacheMetrics)
