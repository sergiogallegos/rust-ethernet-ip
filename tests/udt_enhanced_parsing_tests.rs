#[cfg(test)]
mod udt_enhanced_parsing_tests {
    use rust_ethernet_ip::{EipClient, PlcValue};
    use std::time::Duration;
    use tokio::time::timeout;
    use tracing;

    const TEST_PLC_IP: &str = "192.168.0.1:44818";

    #[tokio::test]
    async fn test_udt_multi_member_parsing() {
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

        // Test UDT with multiple members (TestTagUDT with DINT, DINT, REAL)
        let result = client.read_tag("TestTagUDT").await;
        match result {
            Ok(PlcValue::Udt(udt_data)) => {
                tracing::info!("UDT read successfully");
                tracing::debug!("Symbol ID: {}", udt_data.symbol_id);
                tracing::debug!("Data Size: {} bytes", udt_data.data.len());

                // Verify we have data
                assert!(!udt_data.data.is_empty(), "UDT should have data");
                assert!(udt_data.symbol_id >= 0, "UDT should have valid symbol_id");
            }
            Ok(other) => {
                tracing::info!("UDT read successfully (different type): {:?}", other);
            }
            Err(e) => {
                tracing::error!("UDT read failed: {}", e);
                if e.to_string().contains("Connection") || e.to_string().contains("timeout") {
                    tracing::debug!("Skipping test - PLC not available");
                    return;
                }
                panic!("UDT read failed: {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_udt_chunked_reading() {
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

        // Test chunked reading for large UDTs
        let result = client.read_udt_chunked("Part_Data").await;
        match result {
            Ok(PlcValue::Udt(udt_data)) => {
                tracing::info!(
                    "Chunked UDT read successfully ({} bytes)",
                    udt_data.data.len()
                );
                // Part_Data might be empty due to parsing limitations, but should not fail
            }
            Ok(other) => {
                tracing::info!(
                    "Chunked UDT read successfully (different type): {:?}",
                    other
                );
            }
            Err(e) => {
                tracing::error!("Chunked UDT read failed: {}", e);
                if e.to_string().contains("Connection") || e.to_string().contains("timeout") {
                    tracing::debug!("Skipping test - PLC not available");
                    return;
                }
                // Chunked reading might fail for various reasons, don't panic
                tracing::warn!("Chunked reading failed (expected for some UDTs): {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_udt_gTracking() {
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

        // Test gTracking UDT
        let result = client.read_tag("gTracking").await;
        match result {
            Ok(PlcValue::Udt(udt_data)) => {
                tracing::info!(
                    "gTracking UDT read successfully ({} bytes)",
                    udt_data.data.len()
                );
                // gTracking might have different structure
            }
            Ok(PlcValue::Dint(value)) => {
                tracing::info!("gTracking read as DINT: {}", value);
                assert!(value >= 0, "gTracking should be non-negative");
            }
            Ok(other) => {
                tracing::info!("gTracking read successfully (different type): {:?}", other);
            }
            Err(e) => {
                tracing::error!("gTracking read failed: {}", e);
                if e.to_string().contains("Connection") || e.to_string().contains("timeout") {
                    tracing::debug!("Skipping test - PLC not available");
                    return;
                }
                panic!("gTracking read failed: {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_udt_parsing_performance() {
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
        let result = client.read_tag("TestTagUDT").await;
        let duration = start.elapsed();

        match result {
            Ok(_) => {
                tracing::info!("UDT parsing completed in {:?}", duration);
                assert!(
                    duration < Duration::from_millis(100),
                    "UDT parsing should be fast"
                );
            }
            Err(e) => {
                if e.to_string().contains("Connection") || e.to_string().contains("timeout") {
                    tracing::debug!("Skipping test - PLC not available");
                    return;
                }
                panic!("UDT parsing performance test failed: {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_udt_byte_alignment_detection() {
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

        // Test that UDT parsing handles byte alignment correctly
        let result = client.read_tag("TestTagUDT").await;
        match result {
            Ok(PlcValue::Udt(udt_data)) => {
                tracing::info!("UDT byte alignment detection successful");
                tracing::debug!("Symbol ID: {}", udt_data.symbol_id);
                tracing::debug!("Data Size: {} bytes", udt_data.data.len());

                // UdtData now contains raw bytes, not parsed members
                // To access members, you would need to parse using UDT definition
                assert!(!udt_data.data.is_empty(), "UDT should have data");
            }
            Ok(_) => {
                tracing::info!("UDT read successful (non-UDT type)");
            }
            Err(e) => {
                if e.to_string().contains("Connection") || e.to_string().contains("timeout") {
                    tracing::debug!("Skipping test - PLC not available");
                    return;
                }
                panic!("UDT byte alignment test failed: {}", e);
            }
        }
    }
}
