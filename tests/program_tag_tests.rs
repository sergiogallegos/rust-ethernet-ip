#[cfg(test)]
mod program_tag_tests {
    use rust_ethernet_ip::{EipClient, PlcValue};
    use std::time::Duration;
    use tokio::time::timeout;
    use tracing;

    const TEST_PLC_IP: &str = "192.168.0.1:44818";
    const TEST_PROGRAM: &str = "API_Web";

    #[tokio::test]
    async fn test_program_tag_reading() {
        let mut client =
            match timeout(Duration::from_secs(10), EipClient::connect(TEST_PLC_IP)).await {
                Ok(Ok(client)) => client,
                Ok(Err(e)) => {
                    tracing::warn!("Skipping test - PLC not available: {}", e);
                    return;
                }
                Err(_) => {
                    tracing::warn!("Skipping test - Connection timeout");
                    return;
                }
            };

        // Test program-scoped tag reading with correct format
        let result = client.read_tag("Program:API_Web.TestTagProgram").await;
        match result {
            Ok(PlcValue::Dint(value)) => {
                tracing::info!("Program tag read successfully: {}", value);
                assert!(value >= 0, "Program tag value should be non-negative");
            }
            Ok(other) => {
                tracing::info!(
                    "Program tag read successfully (different type): {:?}",
                    other
                );
            }
            Err(e) => {
                tracing::error!("Program tag read failed: {}", e);
                // Don't fail the test if PLC is not available
                if e.to_string().contains("Connection") || e.to_string().contains("timeout") {
                    tracing::debug!("Skipping test - PLC not available");
                    return;
                }
                panic!("Program tag read failed: {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_program_tag_out_fuse() {
        let mut client =
            match timeout(Duration::from_secs(10), EipClient::connect(TEST_PLC_IP)).await {
                Ok(Ok(client)) => client,
                Ok(Err(e)) => {
                    tracing::warn!("Skipping test - PLC not available: {}", e);
                    return;
                }
                Err(_) => {
                    tracing::warn!("Skipping test - Connection timeout");
                    return;
                }
            };

        // Test another program-scoped tag
        let result = client.read_tag("Program:API_Web.out_FusePartStatus").await;
        match result {
            Ok(PlcValue::Dint(value)) => {
                tracing::info!("out_FusePartStatus read successfully: {}", value);
                assert!(
                    value >= 0,
                    "out_FusePartStatus value should be non-negative"
                );
            }
            Ok(other) => {
                tracing::info!(
                    "out_FusePartStatus read successfully (different type): {:?}",
                    other
                );
            }
            Err(e) => {
                tracing::error!("out_FusePartStatus read failed: {}", e);
                if e.to_string().contains("Connection") || e.to_string().contains("timeout") {
                    tracing::debug!("Skipping test - PLC not available");
                    return;
                }
                panic!("out_FusePartStatus read failed: {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_program_tag_path_validation() {
        // Test that old format fails (this is expected behavior)
        let mut client =
            match timeout(Duration::from_secs(10), EipClient::connect(TEST_PLC_IP)).await {
                Ok(Ok(client)) => client,
                Ok(Err(_)) => {
                    tracing::debug!("Skipping test - PLC not available");
                    return;
                }
                Err(_) => {
                    tracing::debug!("Skipping test - Connection timeout");
                    return;
                }
            };

        // Test old format (should fail)
        let result = client.read_tag("TestTagProgram").await;
        match result {
            Ok(_) => {
                tracing::warn!(
                    "Old format unexpectedly succeeded - this might indicate PLC configuration"
                );
            }
            Err(e) => {
                tracing::info!("Old format correctly failed as expected: {}", e);
                // This is expected behavior - old format should fail
            }
        }
    }

    #[tokio::test]
    async fn test_program_tag_performance() {
        let mut client =
            match timeout(Duration::from_secs(10), EipClient::connect(TEST_PLC_IP)).await {
                Ok(Ok(client)) => client,
                Ok(Err(_)) => {
                    tracing::debug!("Skipping test - PLC not available");
                    return;
                }
                Err(_) => {
                    tracing::debug!("Skipping test - Connection timeout");
                    return;
                }
            };

        let start = std::time::Instant::now();
        let result = client.read_tag("Program:API_Web.TestTagProgram").await;
        let duration = start.elapsed();

        match result {
            Ok(_) => {
                tracing::info!("Program tag read in {:?}", duration);
                assert!(
                    duration < Duration::from_millis(100),
                    "Program tag read should be fast"
                );
            }
            Err(e) => {
                if e.to_string().contains("Connection") || e.to_string().contains("timeout") {
                    tracing::debug!("Skipping test - PLC not available");
                    return;
                }
                panic!("Program tag performance test failed: {}", e);
            }
        }
    }
}
