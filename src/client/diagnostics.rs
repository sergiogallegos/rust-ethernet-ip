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

        let operations = self.diagnostic_counters.operation_metrics();
        let mut errors = self.diagnostic_counters.error_metrics();
        if error_category == Some(crate::ErrorCategory::Session) {
            errors.session_errors += 1;
            errors.retriable_errors += 1;
            if errors.last_error_time.is_none() {
                errors.last_error_time = Some(now);
                errors.last_error_message = Some(
                    "Detailed health check reported session-level connectivity failure".to_string(),
                );
                errors.last_error_category = error_category;
                errors.last_retriable_error_time = Some(now);
            }
        }

        crate::DiagnosticsSnapshot {
            captured_at: now,
            connections: crate::ConnectionMetrics {
                active_connections: if session_active { 1 } else { 0 },
                total_connections: if session_active { 1 } else { 0 },
                failed_connections: 0,
                connection_uptime_avg: Duration::ZERO,
                last_connection_time: last_success_time,
            },
            operations,
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
            errors,
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
