from __future__ import annotations

import unittest

from rust_ethernet_ip.client import _parse_diagnostics_snapshot


class DiagnosticsMappingTests(unittest.TestCase):
    def test_parse_diagnostics_snapshot_maps_nested_metrics(self) -> None:
        payload = {
            "captured_at_unix_ms": 1713550000000,
            "system_metrics_are_placeholders": True,
            "connections": {
                "active_connections": 1,
                "total_connections": 1,
                "failed_connections": 0,
                "connection_uptime_avg_seconds": 0.0,
                "last_connection_time_unix_ms": 1713550000000,
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
                "last_successful_read_time_unix_ms": None,
                "last_failed_read_time_unix_ms": None,
                "last_successful_write_time_unix_ms": None,
                "last_failed_write_time_unix_ms": None,
            },
            "performance": {
                "avg_read_latency_ms": 0.0,
                "avg_write_latency_ms": 0.0,
                "max_read_latency_ms": 0.0,
                "max_write_latency_ms": 0.0,
                "reads_per_second": 0.0,
                "writes_per_second": 0.0,
                "memory_usage_mb": 0.0,
                "cpu_usage_percent": 0.0,
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
                "last_error_time_unix_ms": None,
                "last_error_message": None,
                "last_error_category": None,
                "last_retriable_error_time_unix_ms": None,
            },
            "health": {
                "overall_health": "Healthy",
                "health_mode": "Passive",
                "last_health_check_unix_ms": 1713550000000,
                "last_verified_health_check_unix_ms": None,
                "consecutive_failures": 0,
                "recovery_attempts": 0,
                "system_uptime_seconds": 0.0,
                "last_success_time_unix_ms": 1713550000000,
                "last_failure_time_unix_ms": None,
            },
        }

        snapshot = _parse_diagnostics_snapshot(payload)
        self.assertEqual(snapshot.connections.active_connections, 1)
        self.assertEqual(snapshot.health.overall_health, "Healthy")
        self.assertEqual(snapshot.health.health_mode, "Passive")
        self.assertTrue(snapshot.system_metrics_are_placeholders)
