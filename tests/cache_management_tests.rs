// tests/cache_management_tests.rs
// =========================================================================
//
// Cache Management Tests
//
// Tests for UDT definition and tag attribute caching functionality.
//
// =========================================================================

#[cfg(test)]
mod tests {
    use rust_ethernet_ip::EipClient;
    use std::env;
    use tokio::time::{timeout, Duration};

    // Helper function to get test PLC address from environment or use default
    fn get_test_plc_address() -> String {
        env::var("TEST_PLC_ADDRESS").unwrap_or_else(|_| "192.168.0.1:44818".to_string())
    }

    #[tokio::test]
    #[ignore] // Ignore by default - requires real PLC
    async fn test_cache_clear() {
        let plc_address = get_test_plc_address();

        let mut client =
            match timeout(Duration::from_secs(10), EipClient::connect(&plc_address)).await {
                Ok(Ok(client)) => client,
                Ok(Err(e)) => {
                    eprintln!("Failed to connect: {}", e);
                    return;
                }
                Err(_) => {
                    eprintln!("Connection timeout");
                    return;
                }
            };

        // Discover tags to populate cache
        if let Err(e) = client.discover_tags().await {
            eprintln!("⚠️ Tag discovery failed: {}", e);
            return;
        }

        // Get cached attributes before clearing
        let before_clear = client.list_cached_tag_attributes().await;
        println!("✅ Cached attributes before clear: {}", before_clear.len());

        // Clear cache
        client.clear_caches().await;
        println!("✅ Cache cleared");

        // Get cached attributes after clearing
        let after_clear = client.list_cached_tag_attributes().await;
        println!("✅ Cached attributes after clear: {}", after_clear.len());

        // Cache should be empty after clearing
        assert_eq!(after_clear.len(), 0, "Cache should be empty after clearing");
    }

    #[tokio::test]
    #[ignore] // Ignore by default - requires real PLC
    async fn test_list_cached_tag_attributes() {
        let plc_address = get_test_plc_address();

        let mut client =
            match timeout(Duration::from_secs(10), EipClient::connect(&plc_address)).await {
                Ok(Ok(client)) => client,
                Ok(Err(e)) => {
                    eprintln!("Failed to connect: {}", e);
                    return;
                }
                Err(_) => {
                    eprintln!("Connection timeout");
                    return;
                }
            };

        // Discover tags to populate cache
        if let Err(e) = client.discover_tags().await {
            eprintln!("⚠️ Tag discovery failed: {}", e);
            return;
        }

        // List cached attributes
        let cached_attributes = client.list_cached_tag_attributes().await;
        println!("✅ Cached tag attributes: {}", cached_attributes.len());
        for attr in &cached_attributes {
            println!("  - {}", attr);
        }

        // Should have some cached attributes after discovery
        assert!(
            !cached_attributes.is_empty(),
            "Should have cached attributes after discovery"
        );
    }

    #[tokio::test]
    #[ignore] // Ignore by default - requires real PLC
    async fn test_cache_repopulation() {
        let plc_address = get_test_plc_address();

        let mut client =
            match timeout(Duration::from_secs(10), EipClient::connect(&plc_address)).await {
                Ok(Ok(client)) => client,
                Ok(Err(e)) => {
                    eprintln!("Failed to connect: {}", e);
                    return;
                }
                Err(_) => {
                    eprintln!("Connection timeout");
                    return;
                }
            };

        // Clear cache
        client.clear_caches().await;
        let after_clear = client.list_cached_tag_attributes().await;
        assert_eq!(after_clear.len(), 0, "Cache should be empty");

        // Repopulate cache by discovering tags
        if let Err(e) = client.discover_tags().await {
            eprintln!("⚠️ Tag discovery failed: {}", e);
            return;
        }

        // Cache should be repopulated
        let after_discovery = client.list_cached_tag_attributes().await;
        println!(
            "✅ Cached attributes after repopulation: {}",
            after_discovery.len()
        );
        assert!(
            !after_discovery.is_empty(),
            "Cache should be repopulated after discovery"
        );
    }
}
