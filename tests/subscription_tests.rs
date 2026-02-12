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
    use rust_ethernet_ip::{EipClient, SubscriptionOptions, TagSubscription};
    use std::env;
    use std::sync::Arc;
    use std::time::Duration;
    use tokio::time::timeout;
    use tracing;

    // Helper functions
    fn get_test_plc_address() -> String {
        env::var("TEST_PLC_ADDRESS").unwrap_or_else(|_| "192.168.0.1:44818".to_string())
    }

    fn should_skip_plc_tests() -> bool {
        env::var("SKIP_PLC_TESTS").is_ok()
    }

    async fn connect_to_plc(address: &str, timeout_secs: u64) -> Option<EipClient> {
        match timeout(
            Duration::from_secs(timeout_secs),
            EipClient::connect(address),
        )
        .await
        {
            Ok(Ok(client)) => Some(client),
            Ok(Err(e)) => {
                tracing::debug!("Skipping test - PLC not available at {}: {}", address, e);
                None
            }
            Err(_) => {
                tracing::debug!("Skipping test - Connection timeout to {}", address);
                None
            }
        }
    }

    #[tokio::test]
    #[ignore] // Ignore by default - requires real PLC
    async fn test_single_tag_subscription() {
        let _ = rust_ethernet_ip::try_init_tracing();

        if should_skip_plc_tests() {
            tracing::debug!("Skipping test - SKIP_PLC_TESTS is set");
            return;
        }

        let plc_address = get_test_plc_address();
        let client = match connect_to_plc(&plc_address, 10).await {
            Some(client) => client,
            None => return,
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
            Ok(subs) => {
                tracing::info!("Multiple subscriptions created: {} tags", subs.len());
                assert_eq!(subs.len(), tags.len(), "one subscription per tag");
            }
            Err(e) => {
                tracing::error!("Multiple subscriptions failed: {}", e);
            }
        }
    }

    #[tokio::test]
    #[ignore] // Ignore by default - requires real PLC
    async fn test_subscription_with_custom_options() {
        if should_skip_plc_tests() {
            tracing::debug!("Skipping test - SKIP_PLC_TESTS is set");
            return;
        }

        let plc_address = get_test_plc_address();
        let client = match connect_to_plc(&plc_address, 10).await {
            Some(client) => client,
            None => return,
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
        if should_skip_plc_tests() {
            tracing::debug!("Skipping test - SKIP_PLC_TESTS is set");
            return;
        }

        let plc_address = get_test_plc_address();
        let client = match connect_to_plc(&plc_address, 10).await {
            Some(client) => client,
            None => return,
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

    #[tokio::test]
    async fn subscription_respects_update_rate_option() {
        let options = SubscriptionOptions {
            update_rate: 250,
            change_threshold: 0.01,
            timeout: 3000,
        };
        let sub = TagSubscription::new("TestTag".to_string(), options);
        assert_eq!(sub.options.update_rate, 250);
    }

    #[tokio::test]
    async fn subscription_into_stream_produces_stream() {
        use futures_util::StreamExt;

        let options = SubscriptionOptions::default();
        let sub = Arc::new(TagSubscription::new("TestTag".to_string(), options));
        let mut stream = Box::pin(sub.clone().into_stream());
        // Stream exists; first next() will hang (no updates). Just verify we can call next with a timeout.
        let result = timeout(Duration::from_millis(50), stream.next()).await;
        assert!(
            result.is_err(),
            "stream next should timeout (no updates yet)"
        );
    }
}
