// tests/batch_operations_tests.rs
// =========================================================================
//
// Batch Operations Tests
//
// Tests for high-performance batch read/write operations that allow
// multiple tag operations in a single optimized request.
//
// =========================================================================

#[cfg(test)]
mod tests {
    use rust_ethernet_ip::{BatchConfig, BatchOperation, EipClient, PlcValue};
    use std::env;
    use tokio::time::{Duration, timeout};
    // Helper function to get test PLC address from environment or use default
    fn get_test_plc_address() -> String {
        env::var("TEST_PLC_ADDRESS").unwrap_or_else(|_| "192.168.0.1:44818".to_string())
    }

    #[tokio::test]
    #[ignore] // Ignore by default - requires real PLC
    async fn test_batch_read_operations() {
        let plc_address = get_test_plc_address();

        let mut client =
            match timeout(Duration::from_secs(10), EipClient::connect(&plc_address)).await {
                Ok(Ok(client)) => client,
                Ok(Err(e)) => {
                    tracing::error!("Failed to connect: {}", e);
                    return;
                }
                Err(_) => {
                    tracing::warn!("Connection timeout");
                    return;
                }
            };

        // Test batch read with multiple tags
        let tag_names = [
            "gTestArray_DINT[0]",
            "gTestArray_DINT[1]",
            "gTestArray_DINT[2]",
        ];

        let tag_refs = tag_names.to_vec();
        let batch_result = client.read_tags_batch(&tag_refs).await;
        match batch_result {
            Ok(results) => {
                tracing::info!("Batch read completed: {} tags", results.len());
                assert_eq!(results.len(), tag_names.len());
                for (tag_name, result) in results.iter() {
                    match result {
                        Ok(value) => {
                            tracing::info!("{}: {:?}", tag_name, value);
                            assert!(matches!(value, PlcValue::Dint(_)));
                        }
                        Err(e) => {
                            tracing::error!("{}: {}", tag_name, e);
                        }
                    }
                }
            }
            Err(e) => {
                tracing::error!("Batch read failed: {}", e);
            }
        }
    }

    #[tokio::test]
    #[ignore] // Ignore by default - requires real PLC
    async fn test_batch_write_operations() {
        let plc_address = get_test_plc_address();

        let mut client =
            match timeout(Duration::from_secs(10), EipClient::connect(&plc_address)).await {
                Ok(Ok(client)) => client,
                Ok(Err(e)) => {
                    tracing::error!("Failed to connect: {}", e);
                    return;
                }
                Err(_) => {
                    tracing::warn!("Connection timeout");
                    return;
                }
            };

        // Test batch write with multiple tags
        let tag_values = vec![
            ("gTestArray_DINT[5]", PlcValue::Dint(100)),
            ("gTestArray_DINT[6]", PlcValue::Dint(200)),
            ("gTestArray_DINT[7]", PlcValue::Dint(300)),
        ];

        let batch_result = client.write_tags_batch(&tag_values).await;
        match batch_result {
            Ok(results) => {
                tracing::info!("Batch write completed: {} tags", results.len());
                assert_eq!(results.len(), tag_values.len());
                for (tag_name, result) in results.iter() {
                    match result {
                        Ok(_) => {
                            tracing::info!("{}: Write successful", tag_name);
                        }
                        Err(e) => {
                            tracing::error!("{}: {}", tag_name, e);
                        }
                    }
                }

                // Verify by reading back
                let read_tags: Vec<&str> = tag_values.iter().map(|(tag, _)| *tag).collect();
                match client.read_tags_batch(&read_tags).await {
                    Ok(read_results) => {
                        for (i, (tag_name, result)) in read_results.iter().enumerate() {
                            match result {
                                Ok(value) => {
                                    let expected = &tag_values[i].1;
                                    assert_eq!(
                                        value, expected,
                                        "Read value should match written value for {}",
                                        tag_name
                                    );
                                    tracing::info!("Verified {}: {:?}", tag_name, value);
                                }
                                Err(e) => {
                                    tracing::error!("Failed to read back {}: {}", tag_name, e);
                                }
                            }
                        }
                    }
                    Err(e) => {
                        tracing::error!("Failed to read back batch: {}", e);
                    }
                }
            }
            Err(e) => {
                tracing::error!("Batch write failed: {}", e);
            }
        }
    }

    #[tokio::test]
    #[ignore] // Ignore by default - requires real PLC
    async fn test_mixed_batch_operations() {
        let plc_address = get_test_plc_address();

        let mut client =
            match timeout(Duration::from_secs(10), EipClient::connect(&plc_address)).await {
                Ok(Ok(client)) => client,
                Ok(Err(e)) => {
                    tracing::error!("Failed to connect: {}", e);
                    return;
                }
                Err(_) => {
                    tracing::warn!("Connection timeout");
                    return;
                }
            };

        // Test mixed batch with reads and writes
        let operations = vec![
            BatchOperation::Read {
                tag_name: "gTestArray_DINT[0]".to_string(),
            },
            BatchOperation::Read {
                tag_name: "gTestArray_DINT[1]".to_string(),
            },
            BatchOperation::Write {
                tag_name: "gTestArray_DINT[10]".to_string(),
                value: PlcValue::Dint(999),
            },
            BatchOperation::Read {
                tag_name: "gTestArray_DINT[10]".to_string(),
            },
        ];

        let batch_result = client.execute_batch(&operations).await;
        match batch_result {
            Ok(results) => {
                tracing::info!("Mixed batch completed: {} operations", results.len());
                assert_eq!(results.len(), operations.len());
                for (i, batch_result) in results.iter().enumerate() {
                    match &batch_result.result {
                        Ok(value) => {
                            tracing::info!("Operation {}: {:?}", i, value);
                        }
                        Err(e) => {
                            tracing::error!("Operation {}: {}", i, e);
                        }
                    }
                }
            }
            Err(e) => {
                tracing::error!("Mixed batch failed: {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_batch_configuration() {
        let plc_address = get_test_plc_address();

        let mut client =
            match timeout(Duration::from_secs(10), EipClient::connect(&plc_address)).await {
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

        // Test default batch configuration
        let default_config = client.get_batch_config();
        tracing::info!(
            "Default batch config: max_packet_size={}",
            default_config.max_packet_size
        );

        // Test custom batch configuration
        let custom_config = BatchConfig {
            max_operations_per_packet: 50,
            max_packet_size: 2000,
            packet_timeout_ms: 5000,
            continue_on_error: true,
            optimize_packet_packing: true,
        };
        client.configure_batch_operations(custom_config.clone());

        let updated_config = client.get_batch_config();
        assert_eq!(
            updated_config.max_packet_size,
            custom_config.max_packet_size
        );
        assert_eq!(
            updated_config.max_operations_per_packet,
            custom_config.max_operations_per_packet
        );
        tracing::info!("Batch configuration updated successfully");
    }

    #[tokio::test]
    #[ignore] // Ignore by default - requires real PLC
    async fn test_batch_performance() {
        let plc_address = get_test_plc_address();

        let mut client =
            match timeout(Duration::from_secs(10), EipClient::connect(&plc_address)).await {
                Ok(Ok(client)) => client,
                Ok(Err(e)) => {
                    tracing::error!("Failed to connect: {}", e);
                    return;
                }
                Err(_) => {
                    tracing::warn!("Connection timeout");
                    return;
                }
            };

        // Test batch performance vs individual operations
        let tag_names: Vec<String> = (0..10).map(|i| format!("gTestArray_DINT[{}]", i)).collect();
        let tag_names_refs: Vec<&str> = tag_names.iter().map(|s| s.as_str()).collect();

        // Time individual operations
        let start_individual = std::time::Instant::now();
        for tag_name in &tag_names_refs {
            let _ = client.read_tag(tag_name).await;
        }
        let individual_duration = start_individual.elapsed();

        // Time batch operation
        let start_batch = std::time::Instant::now();
        let _ = client.read_tags_batch(&tag_names_refs).await;
        let batch_duration = start_batch.elapsed();

        tracing::info!("Individual operations: {:?}", individual_duration);
        tracing::info!("Batch operation: {:?}", batch_duration);
        tracing::info!(
            "Speedup: {:.2}x",
            individual_duration.as_secs_f64() / batch_duration.as_secs_f64()
        );

        // Batch should be faster
        assert!(
            batch_duration < individual_duration,
            "Batch operation should be faster"
        );
    }

    #[tokio::test]
    #[ignore] // Ignore by default - requires real PLC
    async fn test_batch_error_handling() {
        let plc_address = get_test_plc_address();

        let mut client =
            match timeout(Duration::from_secs(10), EipClient::connect(&plc_address)).await {
                Ok(Ok(client)) => client,
                Ok(Err(e)) => {
                    tracing::error!("Failed to connect: {}", e);
                    return;
                }
                Err(_) => {
                    tracing::warn!("Connection timeout");
                    return;
                }
            };

        // Test batch with some invalid tags
        let tag_names = [
            "gTestArray_DINT[0]",    // Valid
            "NonExistentTag",        // Invalid
            "gTestArray_DINT[1]",    // Valid
            "AnotherNonExistentTag", // Invalid
        ];

        let tag_refs = tag_names.to_vec();
        let batch_result = client.read_tags_batch(&tag_refs).await;
        match batch_result {
            Ok(results) => {
                tracing::info!("Batch read with errors completed: {} tags", results.len());
                for (tag_name, result) in results.iter() {
                    match result {
                        Ok(value) => {
                            tracing::info!("{}: {:?}", tag_name, value);
                        }
                        Err(e) => {
                            tracing::warn!("{}: {} (expected error)", tag_name, e);
                        }
                    }
                }
            }
            Err(e) => {
                tracing::error!("Batch read failed: {}", e);
            }
        }
    }
}
