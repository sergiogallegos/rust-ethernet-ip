// tests/route_path_operations_tests.rs
// =========================================================================
//
// Route Path Operations Tests
//
// Tests for route path management operations including setting, getting,
// clearing, and creating clients with route paths.
//
// =========================================================================

#[cfg(test)]
mod tests {
    use rust_ethernet_ip::{EipClient, RoutePath};
    use std::env;
    use std::time::Duration;
    use tokio::time::timeout;

    // Helper functions (inline for now, can be moved to shared module)
    fn get_test_plc_address() -> String {
        env::var("TEST_PLC_ADDRESS").unwrap_or_else(|_| "192.168.0.1:44818".to_string())
    }

    fn get_test_plc_slot() -> u8 {
        env::var("TEST_PLC_SLOT")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(0)
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

    fn get_test_config() -> (String, u8, bool) {
        (
            get_test_plc_address(),
            get_test_plc_slot(),
            should_skip_plc_tests(),
        )
    }

    #[tokio::test]
    async fn test_set_route_path() {
        if should_skip_plc_tests() {
            tracing::debug!("Skipping test - SKIP_PLC_TESTS is set");
            return;
        }

        let (plc_address, _, _) = get_test_config();
        let mut client = match connect_to_plc(&plc_address, 10).await {
            Some(client) => client,
            None => return,
        };

        // Initially, route path should be None
        assert!(
            client.get_route_path().is_none(),
            "Route path should be None initially"
        );

        // Set a route path
        let route = RoutePath::new().add_slot(0);
        client.set_route_path(route.clone());

        // Verify route path is set
        let retrieved_route = client.get_route_path();
        assert!(retrieved_route.is_some(), "Route path should be set");
        if let Some(r) = retrieved_route {
            assert_eq!(r.slots().len(), 1, "Route path should have one slot");
            assert_eq!(r.slots()[0], 0, "Slot should be 0");
        }

        tracing::info!("Route path set and retrieved successfully");
    }

    #[tokio::test]
    async fn test_clear_route_path() {
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

        // Set a route path
        let route = RoutePath::new().add_slot(0);
        client.set_route_path(route);
        assert!(
            client.get_route_path().is_some(),
            "Route path should be set"
        );

        // Clear route path
        client.clear_route_path();
        assert!(
            client.get_route_path().is_none(),
            "Route path should be None after clearing"
        );

        tracing::info!("Route path cleared successfully");
    }

    #[tokio::test]
    async fn test_with_route_path() {
        let plc_address = get_test_plc_address();

        // Create client with route path
        let route = RoutePath::new().add_slot(0);
        let client = match timeout(
            Duration::from_secs(10),
            EipClient::with_route_path(&plc_address, route.clone()),
        )
        .await
        {
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

        // Verify route path is set
        let retrieved_route = client.get_route_path();
        assert!(retrieved_route.is_some(), "Route path should be set");
        if let Some(r) = retrieved_route {
            assert_eq!(r.slots().len(), 1, "Route path should have one slot");
            assert_eq!(r.slots()[0], 0, "Slot should be 0");
        }

        tracing::info!("Client created with route path successfully");
    }

    #[tokio::test]
    async fn test_route_path_modification() {
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

        // Set initial route path
        let route1 = RoutePath::new().add_slot(0);
        client.set_route_path(route1);
        assert!(
            client.get_route_path().is_some(),
            "Route path should be set"
        );

        // Modify route path
        let route2 = RoutePath::new().add_slot(1);
        client.set_route_path(route2);
        let retrieved_route = client.get_route_path();
        assert!(retrieved_route.is_some(), "Route path should still be set");
        if let Some(r) = retrieved_route {
            assert_eq!(r.slots()[0], 1, "Slot should be updated to 1");
        }

        tracing::info!("Route path modified successfully");
    }

    #[tokio::test]
    async fn test_route_path_with_multiple_slots() {
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

        // Set route path with multiple slots
        let route = RoutePath::new().add_slot(0).add_slot(1);
        client.set_route_path(route);

        let retrieved_route = client.get_route_path();
        assert!(retrieved_route.is_some(), "Route path should be set");
        if let Some(r) = retrieved_route {
            assert_eq!(r.slots().len(), 2, "Route path should have two slots");
            assert_eq!(r.slots()[0], 0, "First slot should be 0");
            assert_eq!(r.slots()[1], 1, "Second slot should be 1");
        }

        tracing::info!("Route path with multiple slots set successfully");
    }
}
