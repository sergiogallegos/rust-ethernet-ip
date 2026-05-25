#[allow(dead_code)]
mod test_helpers;

#[cfg(test)]
mod program_tag_tests {
    use crate::test_helpers::{connect_to_plc, get_test_plc_address, should_skip_plc_tests};
    use rust_ethernet_ip::{EipClient, PlcValue};
    use std::fmt::Display;
    use std::time::Duration;

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
    async fn test_program_tag_reading() {
        if should_skip_plc_tests() {
            tracing::debug!("Skipping test - SKIP_PLC_TESTS is set");
            return;
        }
        let Some(mut client) = connect_test_plc().await else {
            return;
        };

        let result = client
            .read_tag("Program:TestProgram.gTestArray_DINT[0]")
            .await;
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
            Err(e) if is_tag_not_found_error(&e) => {
                tracing::debug!("Skipping test - program tag not available on PLC: {}", e);
            }
            Err(e) => {
                panic!("Program tag read failed: {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_program_tag_out_fuse() {
        if should_skip_plc_tests() {
            tracing::debug!("Skipping test - SKIP_PLC_TESTS is set");
            return;
        }
        let Some(mut client) = connect_test_plc().await else {
            return;
        };

        let result = client
            .read_tag("Program:TestProgram.gTestArray_BOOL[0]")
            .await;
        match result {
            Ok(PlcValue::Bool(_)) => {
                tracing::info!("Program BOOL tag read successfully");
            }
            Ok(other) => {
                tracing::info!("Program BOOL tag read successfully as {:?}", other);
            }
            Err(e) if is_tag_not_found_error(&e) => {
                tracing::debug!("Skipping test - program tag not available on PLC: {}", e);
            }
            Err(e) => {
                panic!("Program tag read failed: {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_program_tag_path_validation() {
        if should_skip_plc_tests() {
            tracing::debug!("Skipping test - SKIP_PLC_TESTS is set");
            return;
        }
        let Some(mut client) = connect_test_plc().await else {
            return;
        };

        let result = client.read_tag("TestProgram.gTestArray_DINT[0]").await;
        match result {
            Ok(_) => {
                tracing::warn!(
                    "Invalid program tag format unexpectedly succeeded - this might indicate PLC configuration"
                );
            }
            Err(e) => {
                tracing::info!("Invalid program tag format correctly failed: {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_program_tag_performance() {
        if should_skip_plc_tests() {
            tracing::debug!("Skipping test - SKIP_PLC_TESTS is set");
            return;
        }
        let Some(mut client) = connect_test_plc().await else {
            return;
        };

        let start = std::time::Instant::now();
        let result = client
            .read_tag("Program:TestProgram.gTestArray_DINT[0]")
            .await;
        let duration = start.elapsed();

        match result {
            Ok(_) => {
                tracing::info!("Program tag read in {:?}", duration);
                assert!(
                    duration < Duration::from_millis(100),
                    "Program tag read should be fast"
                );
            }
            Err(e) if is_tag_not_found_error(&e) => {
                tracing::debug!("Skipping test - program tag not available on PLC: {}", e);
            }
            Err(e) => {
                panic!("Program tag performance test failed: {}", e);
            }
        }
    }
}
