using System.Text.Json;
using Xunit;

namespace RustEtherNetIp.Tests
{
    public class DiagnosticsSnapshotContractTests
    {
        [Fact]
        public void DiagnosticsSnapshot_Deserializes_CurrentNativeJsonShape()
        {
            const string json = """
            {
              "captured_at_unix_ms": 1713550000000,
              "system_metrics_are_placeholders": true,
              "connections": {
                "active_connections": 1,
                "total_connections": 1,
                "failed_connections": 0,
                "connection_uptime_avg_seconds": 0.0,
                "last_connection_time_unix_ms": 1713550000000
              },
              "operations": {
                "total_reads": 0,
                "total_writes": 0,
                "successful_reads": 0,
                "successful_writes": 0,
                "failed_reads": 0,
                "failed_writes": 0,
                "batch_operations": 0,
                "subscription_updates": 0,
                "partial_batch_failures": 0,
                "last_successful_read_time_unix_ms": null,
                "last_failed_read_time_unix_ms": null,
                "last_successful_write_time_unix_ms": null,
                "last_failed_write_time_unix_ms": null
              },
              "performance": {
                "avg_read_latency_ms": 0.0,
                "avg_write_latency_ms": 0.0,
                "max_read_latency_ms": 0.0,
                "max_write_latency_ms": 0.0,
                "reads_per_second": 0.0,
                "writes_per_second": 0.0,
                "memory_usage_mb": 0.0,
                "cpu_usage_percent": 0.0
              },
              "errors": {
                "network_errors": 0,
                "protocol_errors": 0,
                "timeout_errors": 0,
                "tag_not_found_errors": 0,
                "data_type_errors": 0,
                "session_errors": 0,
                "route_path_errors": 0,
                "embedded_service_errors": 0,
                "known_controller_limitation_errors": 0,
                "retriable_errors": 0,
                "non_retriable_errors": 0,
                "last_error_time_unix_ms": null,
                "last_error_message": null,
                "last_error_category": null,
                "last_retriable_error_time_unix_ms": null
              },
              "health": {
                "overall_health": "Healthy",
                "health_mode": "Passive",
                "last_health_check_unix_ms": 1713550000000,
                "last_verified_health_check_unix_ms": null,
                "consecutive_failures": 0,
                "recovery_attempts": 0,
                "system_uptime_seconds": 0.0,
                "last_success_time_unix_ms": 1713550000000,
                "last_failure_time_unix_ms": null
              }
            }
            """;

            var snapshot = JsonSerializer.Deserialize<DiagnosticsSnapshot>(json);

            Assert.NotNull(snapshot);
            Assert.True(snapshot!.SystemMetricsArePlaceholders);
            Assert.Equal(1, snapshot.Connections.ActiveConnections);
            Assert.Equal(DiagnosticsHealthStatus.Healthy, snapshot.Health.OverallHealth);
            Assert.Equal(DiagnosticsHealthMode.Passive, snapshot.Health.HealthMode);
        }
    }
}
