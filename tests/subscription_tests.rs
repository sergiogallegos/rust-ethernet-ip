// tests/subscription_tests.rs
// =========================================================================
//
// Subscription Tests
//
// Tests for real-time tag subscriptions that provide event-driven
// notifications when tag values change.
//
// =========================================================================

#[cfg(test)]
mod tests {
    use rust_ethernet_ip::{EipClient, PlcValue, SubscriptionOptions};
    use std::env;
    use tokio::time::{timeout, Duration};
    use tracing;

    // Helper function to get test PLC address from environment or use default
    fn get_test_plc_address() -> String {
        env::var("TEST_PLC_ADDRESS").unwrap_or_else(|_| "192.168.0.1:44818".to_string())
    }

    #[tokio::test]
    #[ignore] // Ignore by default - requires real PLC
    async fn test_single_tag_subscription() {
        let _ = rust_ethernet_ip::try_init_tracing();
        let plc_address = get_test_plc_address();

        let client = match timeout(Duration::from_secs(10), EipClient::connect(&plc_address)).await
        {
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

        // Subscribe to a tag with default options
        let options = SubscriptionOptions::default();
        match client.subscribe_to_tag("gTestArray_DINT[0]", options).await {
            Ok(subscription) => {
                tracing::info!("Subscription created: {:?}", subscription);
                // Note: In a real test, you would wait for value changes
                // and verify the callback is called
            }
            Err(e) => {
                tracing::error!("Subscription failed: {}", e);
            }
        }
    }

    #[tokio::test]
    #[ignore] // Ignore by default - requires real PLC
    async fn test_multiple_tag_subscriptions() {
        let plc_address = get_test_plc_address();

        let client = match timeout(Duration::from_secs(10), EipClient::connect(&plc_address)).await
        {
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

        // Subscribe to multiple tags
        let tags = vec![
            ("gTestArray_DINT[0]", SubscriptionOptions::default()),
            ("gTestArray_DINT[1]", SubscriptionOptions::default()),
            ("gTestArray_DINT[2]", SubscriptionOptions::default()),
        ];

        match client.subscribe_to_tags(&tags).await {
            Ok(_) => {
                tracing::info!("Multiple subscriptions created: {} tags", tags.len());
            }
            Err(e) => {
                tracing::error!("Multiple subscriptions failed: {}", e);
            }
        }
    }

    #[tokio::test]
    #[ignore] // Ignore by default - requires real PLC
    async fn test_subscription_with_custom_options() {
        let plc_address = get_test_plc_address();

        let client = match timeout(Duration::from_secs(10), EipClient::connect(&plc_address)).await
        {
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

        // Subscribe with custom options
        let options = SubscriptionOptions {
            update_rate: 100,        // milliseconds
            change_threshold: 0.001, // 0.1% change threshold
            timeout: 5000,           // milliseconds
        };

        match client.subscribe_to_tag("gTestArray_DINT[0]", options).await {
            Ok(subscription) => {
                tracing::info!(
                    "Subscription with custom options created: {:?}",
                    subscription
                );
            }
            Err(e) => {
                tracing::error!("Subscription with custom options failed: {}", e);
            }
        }
    }

    #[tokio::test]
    #[ignore] // Ignore by default - requires real PLC
    async fn test_subscription_error_handling() {
        let plc_address = get_test_plc_address();

        let client = match timeout(Duration::from_secs(10), EipClient::connect(&plc_address)).await
        {
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

        // Try to subscribe to non-existent tag
        let options = SubscriptionOptions::default();
        match client.subscribe_to_tag("NonExistentTag", options).await {
            Ok(_) => {
                tracing::warn!("Subscription to non-existent tag unexpectedly succeeded");
            }
            Err(e) => {
                tracing::info!("Subscription correctly failed for non-existent tag: {}", e);
            }
        }
    }
}
