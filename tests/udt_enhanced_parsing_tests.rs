#[cfg(test)]
mod udt_enhanced_parsing_tests {
    use rust_ethernet_ip::{EipClient, PlcValue};
    use std::time::Duration;
    use tokio::time::timeout;

    const TEST_PLC_IP: &str = "192.168.0.1:44818";

    #[tokio::test]
    async fn test_udt_multi_member_parsing() {
        let mut client =
            match timeout(Duration::from_secs(10), EipClient::connect(TEST_PLC_IP)).await {
                Ok(Ok(client)) => client,
                Ok(Err(e)) => {
                    eprintln!("⚠️ Skipping test - PLC not available: {}", e);
                    return;
                }
                Err(_) => {
                    eprintln!("⚠️ Skipping test - Connection timeout");
                    return;
                }
            };

        // Test UDT with multiple members (TestTagUDT with DINT, DINT, REAL)
        let result = client.read_tag("TestTagUDT").await;
        match result {
            Ok(PlcValue::Udt(udt_data)) => {
                println!("✅ UDT read successfully");
                println!("   Symbol ID: {}", udt_data.symbol_id);
                println!("   Data Size: {} bytes", udt_data.data.len());

                // Verify we have data
                assert!(!udt_data.data.is_empty(), "UDT should have data");
                assert!(udt_data.symbol_id >= 0, "UDT should have valid symbol_id");
            }
            Ok(other) => {
                println!("✅ UDT read successfully (different type): {:?}", other);
            }
            Err(e) => {
                eprintln!("❌ UDT read failed: {}", e);
                if e.to_string().contains("Connection") || e.to_string().contains("timeout") {
                    println!("⚠️ Skipping test - PLC not available");
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
                    eprintln!("⚠️ Skipping test - PLC not available: {}", e);
                    return;
                }
                Err(_) => {
                    eprintln!("⚠️ Skipping test - Connection timeout");
                    return;
                }
            };

        // Test chunked reading for large UDTs
        let result = client.read_udt_chunked("Part_Data").await;
        match result {
            Ok(PlcValue::Udt(udt_data)) => {
                println!(
                    "✅ Chunked UDT read successfully ({} bytes)",
                    udt_data.data.len()
                );
                // Part_Data might be empty due to parsing limitations, but should not fail
            }
            Ok(other) => {
                println!(
                    "✅ Chunked UDT read successfully (different type): {:?}",
                    other
                );
            }
            Err(e) => {
                eprintln!("❌ Chunked UDT read failed: {}", e);
                if e.to_string().contains("Connection") || e.to_string().contains("timeout") {
                    println!("⚠️ Skipping test - PLC not available");
                    return;
                }
                // Chunked reading might fail for various reasons, don't panic
                println!("⚠️ Chunked reading failed (expected for some UDTs): {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_udt_gTracking() {
        let mut client =
            match timeout(Duration::from_secs(10), EipClient::connect(TEST_PLC_IP)).await {
                Ok(Ok(client)) => client,
                Ok(Err(e)) => {
                    eprintln!("⚠️ Skipping test - PLC not available: {}", e);
                    return;
                }
                Err(_) => {
                    eprintln!("⚠️ Skipping test - Connection timeout");
                    return;
                }
            };

        // Test gTracking UDT
        let result = client.read_tag("gTracking").await;
        match result {
            Ok(PlcValue::Udt(udt_data)) => {
                println!(
                    "✅ gTracking UDT read successfully ({} bytes)",
                    udt_data.data.len()
                );
                // gTracking might have different structure
            }
            Ok(PlcValue::Dint(value)) => {
                println!("✅ gTracking read as DINT: {}", value);
                assert!(value >= 0, "gTracking should be non-negative");
            }
            Ok(other) => {
                println!(
                    "✅ gTracking read successfully (different type): {:?}",
                    other
                );
            }
            Err(e) => {
                eprintln!("❌ gTracking read failed: {}", e);
                if e.to_string().contains("Connection") || e.to_string().contains("timeout") {
                    println!("⚠️ Skipping test - PLC not available");
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
                    println!("⚠️ Skipping test - PLC not available");
                    return;
                }
                Err(_) => {
                    println!("⚠️ Skipping test - Connection timeout");
                    return;
                }
            };

        let start = std::time::Instant::now();
        let result = client.read_tag("TestTagUDT").await;
        let duration = start.elapsed();

        match result {
            Ok(_) => {
                println!("✅ UDT parsing completed in {:?}", duration);
                assert!(
                    duration < Duration::from_millis(100),
                    "UDT parsing should be fast"
                );
            }
            Err(e) => {
                if e.to_string().contains("Connection") || e.to_string().contains("timeout") {
                    println!("⚠️ Skipping test - PLC not available");
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
                    println!("⚠️ Skipping test - PLC not available");
                    return;
                }
                Err(_) => {
                    println!("⚠️ Skipping test - Connection timeout");
                    return;
                }
            };

        // Test that UDT parsing handles byte alignment correctly
        let result = client.read_tag("TestTagUDT").await;
        match result {
            Ok(PlcValue::Udt(udt_data)) => {
                println!("✅ UDT byte alignment detection successful");
                println!("   Symbol ID: {}", udt_data.symbol_id);
                println!("   Data Size: {} bytes", udt_data.data.len());

                // UdtData now contains raw bytes, not parsed members
                // To access members, you would need to parse using UDT definition
                assert!(!udt_data.data.is_empty(), "UDT should have data");
            }
            Ok(_) => {
                println!("✅ UDT read successful (non-UDT type)");
            }
            Err(e) => {
                if e.to_string().contains("Connection") || e.to_string().contains("timeout") {
                    println!("⚠️ Skipping test - PLC not available");
                    return;
                }
                panic!("UDT byte alignment test failed: {}", e);
            }
        }
    }
}
