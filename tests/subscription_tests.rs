// tests/subscription_tests.rs
// =========================================================================
//
// Subscription Tests
//
// Tests for real-time tag subscriptions that provide event-driven
// notifications when tag values change.
//
// =========================================================================

mod plc_sim;

#[cfg(test)]
mod tests {
    use crate::plc_sim::{SimBehavior, SimulatedPlc};
    use rust_ethernet_ip::{
        EipClient, PlcValue, SubscriptionOptions, TagGroupSubscription, TagSubscription,
        TagSubscriptionEvent,
    };
    use std::env;
    use std::sync::Arc;
    use std::time::Duration;
    use tokio::time::timeout;
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
        let subscribe_result = client.subscribe_to_tag("gTestArray_DINT[0]", options).await;
        match subscribe_result {
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

        let subscribe_result = client.subscribe_to_tags(&tags).await;
        match subscribe_result {
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

        let subscribe_result = client.subscribe_to_tag("gTestArray_DINT[0]", options).await;
        match subscribe_result {
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
        let subscribe_result = client.subscribe_to_tag("NonExistentTag", options).await;
        match subscribe_result {
            Ok(_) => panic!("Subscription to non-existent tag should fail fast"),
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

    #[tokio::test]
    async fn abandoned_subscription_consumer_does_not_block_polling() {
        let sim = SimulatedPlc::start_with_behavior(SimBehavior {
            count_reads_as_dint_tags: vec!["DINT_TAG".to_string()],
            ..SimBehavior::default()
        })
        .await;
        let client = EipClient::connect(&sim.address.to_string())
            .await
            .expect("connect simulator");

        let subscription = client
            .subscribe_to_tag(
                "DINT_TAG",
                SubscriptionOptions {
                    update_rate: 1,
                    ..SubscriptionOptions::default()
                },
            )
            .await
            .expect("subscribe");

        timeout(Duration::from_secs(2), async {
            loop {
                if sim.read_count("DINT_TAG") > 110 {
                    break;
                }
                tokio::task::yield_now().await;
            }
        })
        .await
        .expect("polling should continue past the channel capacity");
        subscription.stop();
    }

    #[tokio::test]
    async fn stop_halts_single_tag_polling() {
        let sim = SimulatedPlc::start_with_behavior(SimBehavior {
            count_reads_as_dint_tags: vec!["DINT_TAG".to_string()],
            ..SimBehavior::default()
        })
        .await;
        let client = EipClient::connect(&sim.address.to_string())
            .await
            .expect("connect simulator");
        let subscription = client
            .subscribe_to_tag(
                "DINT_TAG",
                SubscriptionOptions {
                    update_rate: 5,
                    ..SubscriptionOptions::default()
                },
            )
            .await
            .expect("subscribe");

        timeout(Duration::from_secs(1), async {
            loop {
                if sim.read_count("DINT_TAG") >= 3 {
                    break;
                }
                tokio::task::yield_now().await;
            }
        })
        .await
        .expect("polling should start");

        subscription.stop();
        let count_after_stop = sim.read_count("DINT_TAG");
        tokio::time::sleep(Duration::from_millis(40)).await;
        assert!(
            sim.read_count("DINT_TAG") <= count_after_stop + 1,
            "polling should stop within one in-flight/read interval"
        );
    }

    #[tokio::test]
    async fn transient_single_tag_error_is_reported_and_recovers() {
        let sim = SimulatedPlc::start_with_behavior(SimBehavior {
            fail_read_once_connection_failure_on_counts: vec![("DINT_TAG".to_string(), 2)],
            ..SimBehavior::default()
        })
        .await;
        let mut client = EipClient::connect(&sim.address.to_string())
            .await
            .expect("connect simulator");
        let subscription = client
            .subscribe_to_tag(
                "DINT_TAG",
                SubscriptionOptions {
                    update_rate: 5,
                    ..SubscriptionOptions::default()
                },
            )
            .await
            .expect("subscribe");

        assert!(matches!(
            subscription.wait_for_event().await.expect("initial event"),
            TagSubscriptionEvent::Value(PlcValue::Dint(1234))
        ));
        assert!(matches!(
            timeout(Duration::from_secs(1), subscription.wait_for_event())
                .await
                .expect("error event")
                .expect("error event value"),
            TagSubscriptionEvent::Error {
                terminal: false,
                ..
            }
        ));

        client
            .write_tag("DINT_TAG", PlcValue::Dint(4321))
            .await
            .expect("write new value");
        let recovered = timeout(Duration::from_secs(1), subscription.wait_for_event())
            .await
            .expect("recovered value event")
            .expect("event value");
        assert_eq!(recovered, TagSubscriptionEvent::Value(PlcValue::Dint(4321)));
        subscription.stop();
    }

    #[tokio::test]
    async fn unsubscribe_stops_and_evicts_subscription() {
        let sim = SimulatedPlc::start().await;
        let client = EipClient::connect(&sim.address.to_string())
            .await
            .expect("connect simulator");
        let subscription = client
            .subscribe_to_tag("DINT_TAG", SubscriptionOptions::default())
            .await
            .expect("subscribe");

        assert_eq!(client.subscription_count().await, 1);
        assert!(client.unsubscribe("DINT_TAG").await);
        assert!(!subscription.is_active());
        assert_eq!(client.subscription_count().await, 0);
    }

    #[tokio::test]
    async fn direct_subscription_updates_drop_oldest_under_backpressure() {
        let subscription =
            TagSubscription::new("DINT_TAG".to_string(), SubscriptionOptions::default());

        timeout(Duration::from_secs(1), async {
            for value in 0..150 {
                subscription
                    .update_value(&PlcValue::Dint(value))
                    .await
                    .expect("update should not block");
            }
        })
        .await
        .expect("updates should not block when receiver is abandoned");

        let first_received = subscription.wait_for_update().await.expect("queued value");
        assert!(
            matches!(first_received, PlcValue::Dint(value) if value > 0),
            "oldest values should be dropped under backpressure"
        );
    }

    #[tokio::test]
    async fn tag_group_publish_drops_oldest_under_backpressure() {
        let subscription = TagGroupSubscription::new("group".to_string(), 1);

        timeout(Duration::from_secs(1), async {
            for value in 0..100 {
                subscription
                    .publish(rust_ethernet_ip::TagGroupSnapshot {
                        group_name: "group".to_string(),
                        sampled_at: std::time::SystemTime::now(),
                        values: vec![rust_ethernet_ip::TagGroupValueResult {
                            tag_name: "DINT_TAG".to_string(),
                            value: Some(PlcValue::Dint(value)),
                            error: None,
                        }],
                    })
                    .await
                    .expect("publish should not block");
            }
        })
        .await
        .expect("tag group publish should not block when receiver is abandoned");

        let event = subscription.wait_for_update().await.expect("queued event");
        let value = event.snapshot.values[0].value.clone().expect("value");
        assert!(matches!(value, PlcValue::Dint(v) if v > 0));
    }
}
