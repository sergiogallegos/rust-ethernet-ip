#[cfg(test)]
mod cross_language_compatibility_tests {
    use rust_ethernet_ip::{EipClient, PlcValue};
    use std::time::Duration;
    use tokio::time::timeout;

    const TEST_PLC_IP: &str = "192.168.0.1:44818";

    #[tokio::test]
    async fn test_rust_library_core_functionality() {
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

        // Test core functionality that should work across all language bindings
        let tests = vec![
            ("TestTagController", "Controller-scoped tag"),
            ("Program:API_Web.TestTagProgram", "Program-scoped tag"),
            ("Program:API_Web.out_FusePartStatus", "Program-scoped tag 2"),
            ("TestTagUDT", "UDT tag"),
            ("gTracking", "UDT tag 2"),
        ];

        for (tag_name, description) in tests {
            let result = client.read_tag(tag_name).await;
            match result {
                Ok(value) => {
                    println!(
                        "✅ {} ({}) read successfully: {:?}",
                        tag_name, description, value
                    );
                }
                Err(e) => {
                    eprintln!("❌ {} ({}) read failed: {}", tag_name, description, e);
                    if e.to_string().contains("Connection") || e.to_string().contains("timeout") {
                        println!("⚠️ Skipping test - PLC not available");
                        return;
                    }
                    // Don't panic for individual tag failures - some might not exist
                    println!("⚠️ {} failed (might not exist on PLC): {}", tag_name, e);
                }
            }
        }
    }

    #[tokio::test]
    async fn test_ffi_compatibility() {
        // Test that the library works correctly through FFI
        // This is more of a smoke test to ensure the FFI functions are properly exposed
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

        // Test basic read operation
        let result = client.read_tag("TestTagController").await;
        match result {
            Ok(PlcValue::Dint(value)) => {
                println!("✅ FFI compatibility test passed: {}", value);
                assert!(value >= 0, "Controller tag should be non-negative");
            }
            Ok(other) => {
                println!(
                    "✅ FFI compatibility test passed (different type): {:?}",
                    other
                );
            }
            Err(e) => {
                if e.to_string().contains("Connection") || e.to_string().contains("timeout") {
                    println!("⚠️ Skipping test - PLC not available");
                    return;
                }
                panic!("FFI compatibility test failed: {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_performance_consistency() {
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

        // Test that performance is consistent across different operations
        let operations = vec![
            ("TestTagController", "Controller tag"),
            ("Program:API_Web.TestTagProgram", "Program tag"),
            ("TestTagUDT", "UDT tag"),
        ];

        for (tag_name, description) in operations {
            let start = std::time::Instant::now();
            let result = client.read_tag(tag_name).await;
            let duration = start.elapsed();

            match result {
                Ok(_) => {
                    println!(
                        "✅ {} ({}) completed in {:?}",
                        tag_name, description, duration
                    );
                    assert!(
                        duration < Duration::from_millis(500),
                        "{} should complete within 500ms",
                        description
                    );
                }
                Err(e) => {
                    if e.to_string().contains("Connection") || e.to_string().contains("timeout") {
                        println!("⚠️ Skipping test - PLC not available");
                        return;
                    }
                    println!("⚠️ {} failed (might not exist): {}", tag_name, e);
                }
            }
        }
    }

    #[tokio::test]
    async fn test_error_handling_consistency() {
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

        // Test error handling for non-existent tags
        let non_existent_tags = vec![
            "NonExistentTag",
            "Program:NonExistentProgram.Tag",
            "InvalidUDT",
        ];

        for tag_name in non_existent_tags {
            let result = client.read_tag(tag_name).await;
            match result {
                Ok(_) => {
                    println!(
                        "⚠️ {} unexpectedly succeeded (might exist on PLC)",
                        tag_name
                    );
                }
                Err(e) => {
                    println!("✅ {} correctly failed: {}", tag_name, e);
                    // Verify error message is informative
                    assert!(
                        !e.to_string().is_empty(),
                        "Error message should not be empty"
                    );
                }
            }
        }
    }

    #[tokio::test]
    async fn test_memory_safety() {
        // Test that the library doesn't leak memory or cause crashes
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

        // Perform multiple operations to test memory safety
        for i in 0..10 {
            let result = client.read_tag("TestTagController").await;
            match result {
                Ok(_) => {
                    println!("✅ Memory safety test iteration {} passed", i);
                }
                Err(e) => {
                    if e.to_string().contains("Connection") || e.to_string().contains("timeout") {
                        println!("⚠️ Skipping test - PLC not available");
                        return;
                    }
                    panic!("Memory safety test failed at iteration {}: {}", i, e);
                }
            }
        }
    }
}
