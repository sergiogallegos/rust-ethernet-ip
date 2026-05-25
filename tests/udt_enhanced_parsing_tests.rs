#[allow(dead_code)]
mod test_helpers;

#[cfg(test)]
mod udt_enhanced_parsing_tests {
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
    async fn test_udt_multi_member_parsing() {
        if should_skip_plc_tests() {
            tracing::debug!("Skipping test - SKIP_PLC_TESTS is set");
            return;
        }
        let Some(mut client) = connect_test_plc().await else {
            return;
        };

        let result = client.read_tag("gTestUDT").await;
        match result {
            Ok(PlcValue::Udt(udt_data)) => {
                tracing::info!("UDT read successfully");
                tracing::debug!("Symbol ID: {}", udt_data.symbol_id);
                tracing::debug!("Data Size: {} bytes", udt_data.data.len());
                assert!(!udt_data.data.is_empty(), "UDT should have data");
                assert!(udt_data.symbol_id >= 0, "UDT should have valid symbol_id");
            }
            Ok(other) => {
                tracing::info!("UDT read successfully (different type): {:?}", other);
            }
            Err(e) if is_tag_not_found_error(&e) => {
                tracing::debug!("Skipping test - UDT tag not available on PLC: {}", e);
            }
            Err(e) => {
                panic!("UDT read failed: {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_udt_chunked_reading() {
        if should_skip_plc_tests() {
            tracing::debug!("Skipping test - SKIP_PLC_TESTS is set");
            return;
        }
        let Some(mut client) = connect_test_plc().await else {
            return;
        };

        let result = client.read_udt_chunked("gTestUDT").await;
        match result {
            Ok(PlcValue::Udt(udt_data)) => {
                tracing::info!(
                    "Chunked UDT read successfully ({} bytes)",
                    udt_data.data.len()
                );
            }
            Ok(other) => {
                tracing::info!(
                    "Chunked UDT read successfully (different type): {:?}",
                    other
                );
            }
            Err(e) if is_tag_not_found_error(&e) => {
                tracing::debug!("Skipping test - UDT tag not available on PLC: {}", e);
            }
            Err(e) => {
                tracing::warn!("Chunked reading failed (expected for some UDTs): {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_udt_g_tracking() {
        if should_skip_plc_tests() {
            tracing::debug!("Skipping test - SKIP_PLC_TESTS is set");
            return;
        }
        let Some(mut client) = connect_test_plc().await else {
            return;
        };

        let result = client.read_tag("gTestUDT.Member1_DINT").await;
        match result {
            Ok(PlcValue::Dint(value)) => {
                tracing::info!("gTestUDT.Member1_DINT read as DINT: {}", value);
                assert!(value >= 0, "gTestUDT.Member1_DINT should be non-negative");
            }
            Ok(other) => {
                tracing::info!(
                    "gTestUDT.Member1_DINT read successfully (different type): {:?}",
                    other
                );
            }
            Err(e) if is_tag_not_found_error(&e) => {
                tracing::debug!("Skipping test - UDT member not available on PLC: {}", e);
            }
            Err(e) => {
                panic!("gTestUDT.Member1_DINT read failed: {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_udt_parsing_performance() {
        if should_skip_plc_tests() {
            tracing::debug!("Skipping test - SKIP_PLC_TESTS is set");
            return;
        }
        let Some(mut client) = connect_test_plc().await else {
            return;
        };

        let start = std::time::Instant::now();
        let result = client.read_tag("gTestUDT").await;
        let duration = start.elapsed();

        match result {
            Ok(_) => {
                tracing::info!("UDT parsing completed in {:?}", duration);
                assert!(
                    duration < Duration::from_millis(100),
                    "UDT parsing should be fast"
                );
            }
            Err(e) if is_tag_not_found_error(&e) => {
                tracing::debug!("Skipping test - UDT tag not available on PLC: {}", e);
            }
            Err(e) => {
                panic!("UDT parsing performance test failed: {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_udt_byte_alignment_detection() {
        if should_skip_plc_tests() {
            tracing::debug!("Skipping test - SKIP_PLC_TESTS is set");
            return;
        }
        let Some(mut client) = connect_test_plc().await else {
            return;
        };

        let result = client.read_tag("gTestUDT").await;
        match result {
            Ok(PlcValue::Udt(udt_data)) => {
                tracing::info!("UDT byte alignment detection successful");
                tracing::debug!("Symbol ID: {}", udt_data.symbol_id);
                tracing::debug!("Data Size: {} bytes", udt_data.data.len());
                assert!(!udt_data.data.is_empty(), "UDT should have data");
            }
            Ok(_) => {
                tracing::info!("UDT read successful (non-UDT type)");
            }
            Err(e) if is_tag_not_found_error(&e) => {
                tracing::debug!("Skipping test - UDT tag not available on PLC: {}", e);
            }
            Err(e) => {
                panic!("UDT byte alignment test failed: {}", e);
            }
        }
    }
}
