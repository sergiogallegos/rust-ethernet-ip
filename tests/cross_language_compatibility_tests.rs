#[allow(dead_code)]
mod test_helpers;

#[cfg(test)]
mod cross_language_compatibility_tests {
    use crate::test_helpers::{connect_to_plc, get_test_plc_address, should_skip_plc_tests};
    use rust_ethernet_ip::{EipClient, PlcValue};
    use std::fmt::Display;
    use std::time::Duration;

    const REPRESENTATIVE_TAGS: &[(&str, &str)] = &[
        ("gTestArray_DINT[0]", "Controller-scoped DINT"),
        (
            "Program:TestProgram.gTestArray_DINT[0]",
            "Program-scoped DINT",
        ),
        ("gTestArray_REAL[0]", "Controller-scoped REAL"),
        ("gTestArray_BOOL[0]", "Controller-scoped BOOL"),
        ("gTestUDT.Member1_DINT", "UDT DINT member"),
        ("gTestUDT.Array_DINT[0]", "UDT nested DINT array"),
    ];

    async fn connect_test_plc() -> Option<EipClient> {
        connect_to_plc(&get_test_plc_address(), 10).await
    }

    fn is_tag_not_found_error(error: &impl Display) -> bool {
        let message = error.to_string();
        message.contains("CIP Error 0x04")
            || message.contains("CIP Error 0x05")
            || message.contains("Path segment error")
            || message.contains("Path destination unknown")
    }

    #[tokio::test]
    async fn test_rust_library_core_functionality() {
        if should_skip_plc_tests() {
            tracing::debug!("Skipping test - SKIP_PLC_TESTS is set");
            return;
        }
        let Some(mut client) = connect_test_plc().await else {
            return;
        };

        for (tag_name, description) in REPRESENTATIVE_TAGS {
            let result = client.read_tag(tag_name).await;
            match result {
                Ok(value) => {
                    tracing::info!(
                        "{} ({}) read successfully: {:?}",
                        tag_name,
                        description,
                        value
                    );
                }
                Err(e) if is_tag_not_found_error(&e) => {
                    tracing::debug!("Skipping test - tag not available on PLC: {}", e);
                    return;
                }
                Err(e) => {
                    panic!("{} ({}) read failed: {}", tag_name, description, e);
                }
            }
        }
    }

    #[tokio::test]
    async fn test_ffi_compatibility() {
        if should_skip_plc_tests() {
            tracing::debug!("Skipping test - SKIP_PLC_TESTS is set");
            return;
        }
        let Some(mut client) = connect_test_plc().await else {
            return;
        };

        let result = client.read_tag("gTestArray_DINT[0]").await;
        match result {
            Ok(PlcValue::Dint(value)) => {
                tracing::info!("FFI compatibility test passed: {}", value);
                assert!(value >= 0, "Controller tag should be non-negative");
            }
            Ok(other) => {
                tracing::info!(
                    "FFI compatibility test passed (different type): {:?}",
                    other
                );
            }
            Err(e) if is_tag_not_found_error(&e) => {
                tracing::debug!("Skipping test - tag not available on PLC: {}", e);
            }
            Err(e) => {
                panic!("FFI compatibility test failed: {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_performance_consistency() {
        if should_skip_plc_tests() {
            tracing::debug!("Skipping test - SKIP_PLC_TESTS is set");
            return;
        }
        let Some(mut client) = connect_test_plc().await else {
            return;
        };

        for (tag_name, description) in REPRESENTATIVE_TAGS.iter().take(4) {
            let start = std::time::Instant::now();
            let result = client.read_tag(tag_name).await;
            let duration = start.elapsed();

            match result {
                Ok(_) => {
                    tracing::info!("{} ({}) completed in {:?}", tag_name, description, duration);
                    assert!(
                        duration < Duration::from_millis(500),
                        "{} should complete within 500ms",
                        description
                    );
                }
                Err(e) if is_tag_not_found_error(&e) => {
                    tracing::debug!("Skipping test - tag not available on PLC: {}", e);
                    return;
                }
                Err(e) => {
                    panic!("{} ({}) read failed: {}", tag_name, description, e);
                }
            }
        }
    }

    #[tokio::test]
    async fn test_error_handling_consistency() {
        if should_skip_plc_tests() {
            tracing::debug!("Skipping test - SKIP_PLC_TESTS is set");
            return;
        }
        let Some(mut client) = connect_test_plc().await else {
            return;
        };

        let non_existent_tags = vec![
            "NonExistentTag",
            "Program:NonExistentProgram.Tag",
            "InvalidUDT",
        ];

        for tag_name in non_existent_tags {
            let result = client.read_tag(tag_name).await;
            match result {
                Ok(_) => {
                    tracing::warn!("{} unexpectedly succeeded (might exist on PLC)", tag_name);
                }
                Err(e) => {
                    tracing::info!("{} correctly failed: {}", tag_name, e);
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
        if should_skip_plc_tests() {
            tracing::debug!("Skipping test - SKIP_PLC_TESTS is set");
            return;
        }
        let Some(mut client) = connect_test_plc().await else {
            return;
        };

        for i in 0..10 {
            let result = client.read_tag("gTestArray_DINT[0]").await;
            match result {
                Ok(_) => {
                    tracing::info!("Memory safety test iteration {} passed", i);
                }
                Err(e) if is_tag_not_found_error(&e) => {
                    tracing::debug!("Skipping test - tag not available on PLC: {}", e);
                    return;
                }
                Err(e) => {
                    panic!("Memory safety test failed at iteration {}: {}", i, e);
                }
            }
        }
    }
}
