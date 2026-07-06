use super::EipClient;
use tokio::time::Duration;

impl EipClient {
    /// Quick connection health check (no I/O).
    ///
    /// Returns `true` if the session handle is valid and there has been activity
    /// within the last 150 seconds. Use this for cheap periodic checks; for a
    /// definitive check that the PLC is still responding, use [`check_health_detailed`](Self::check_health_detailed).
    pub async fn check_health(&self) -> bool {
        self.session_handle() != 0
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
        if self.session_handle() == 0 {
            return Ok(false);
        }

        // Try sending a lightweight keep-alive command
        match self.send_keep_alive().await {
            Ok(()) => Ok(true),
            Err(_) => {
                // If keep-alive fails, try re-registering the shared session.
                // The new handle is atomically visible to all clones using this stream.
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
        let session_active = self.session_handle() != 0;
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
}
