use rust_ethernet_ip::{EipClient, PlcConfig, PlcManager, PlcValue, TagScope};
use std::collections::HashMap;
use std::time::Duration;

/// Helper function to check if a PLC is available at the given address
async fn is_plc_available(address: &str) -> bool {
    EipClient::connect(address).await.is_ok()
}

#[tokio::test]
#[ignore]
async fn test_tag_discovery() {
    let address = "127.0.0.1:44818";
    if !is_plc_available(address).await {
        println!(
            "Skipping test_tag_discovery: No PLC available at {}",
            address
        );
        return;
    }

    let mut client = EipClient::connect(address).await.unwrap();

    // Discover tags
    client.discover_tags().await.unwrap();

    // Verify some common tags exist
    let metadata = client.get_tag_metadata("_IO_EM_DI00").await.unwrap();
    assert_eq!(metadata.data_type, 0x00C1); // BOOL type
    assert_eq!(metadata.scope, TagScope::Controller);

    let metadata = client.get_tag_metadata("_IO_EM_DI01").await.unwrap();
    assert_eq!(metadata.data_type, 0x00C1); // BOOL type
    assert_eq!(metadata.scope, TagScope::Controller);
}

#[tokio::test]
#[ignore]
async fn test_udt_operations() {
    let address = "127.0.0.1:44818";
    if !is_plc_available(address).await {
        println!(
            "Skipping test_udt_operations: No PLC available at {}",
            address
        );
        return;
    }

    let mut client = EipClient::connect(address).await.unwrap();

    // Read a UDT
    let udt_value = client.read_tag("MotorData").await.unwrap();
    if let PlcValue::Udt(udt_data) = udt_value {
        // UdtData now contains symbol_id and raw data bytes
        assert!(udt_data.symbol_id >= 0);
        assert!(!udt_data.data.is_empty());
    } else {
        panic!("Expected UDT value");
    }

    // Write a UDT - need to read first to get symbol_id, then modify and write back
    let udt_value = client.read_tag("MotorData").await.unwrap();
    if let PlcValue::Udt(mut udt_data) = udt_value {
        // For now, just write back the same data (proper modification would require UDT definition)
        client
            .write_tag("MotorData", PlcValue::Udt(udt_data))
            .await
            .unwrap();
    } else {
        panic!("Expected UDT value");
    }
}

#[tokio::test]
#[ignore]
async fn test_multiple_plc_connections() {
    let address1 = "127.0.0.1:44818";
    let address2 = "127.0.0.1:44819";
    if !is_plc_available(address1).await || !is_plc_available(address2).await {
        println!(
            "Skipping test_multiple_plc_connections: PLCs not available at {} or {}",
            address1, address2
        );
        return;
    }

    let mut manager = PlcManager::new();

    // Configure two PLCs
    let config1 = PlcConfig {
        address: address1.parse().unwrap(),
        max_connections: 2,
        connection_timeout: Duration::from_secs(5),
        health_check_interval: Duration::from_secs(30),
        max_packet_size: 4000,
    };

    let config2 = PlcConfig {
        address: address2.parse().unwrap(),
        max_connections: 2,
        connection_timeout: Duration::from_secs(5),
        health_check_interval: Duration::from_secs(30),
        max_packet_size: 4000,
    };

    manager.add_plc(config1.clone());
    manager.add_plc(config2.clone());

    // Get and use client1
    {
        let client1 = manager.get_connection(config1.address).await.unwrap();
        client1
            .write_tag("Tag1", PlcValue::Bool(true))
            .await
            .unwrap();
        let value1 = client1.read_tag("Tag1").await.unwrap();
        assert_eq!(value1, PlcValue::Bool(true));
    }
    // Get and use client2
    {
        let client2 = manager.get_connection(config2.address).await.unwrap();
        client2.write_tag("Tag2", PlcValue::Dint(42)).await.unwrap();
        let value2 = client2.read_tag("Tag2").await.unwrap();
        assert_eq!(value2, PlcValue::Dint(42));
    }
}

#[tokio::test]
#[ignore]
async fn test_connection_pooling() {
    let address = "127.0.0.1:44818";
    if !is_plc_available(address).await {
        println!(
            "Skipping test_connection_pooling: No PLC available at {}",
            address
        );
        return;
    }

    let mut manager = PlcManager::new();

    let config = PlcConfig {
        address: address.parse().unwrap(),
        max_connections: 2,
        connection_timeout: Duration::from_secs(5),
        health_check_interval: Duration::from_secs(30),
        max_packet_size: 4000,
    };

    manager.add_plc(config.clone());

    // Get and use client1
    {
        let client1 = manager.get_connection(config.address).await.unwrap();
        client1
            .write_tag("Tag1", PlcValue::Bool(true))
            .await
            .unwrap();
        let value1 = client1.read_tag("Tag1").await.unwrap();
        assert_eq!(value1, PlcValue::Bool(true));
    }
    // Get and use client2
    {
        let client2 = manager.get_connection(config.address).await.unwrap();
        client2
            .write_tag("Tag2", PlcValue::Bool(false))
            .await
            .unwrap();
        let value2 = client2.read_tag("Tag2").await.unwrap();
        assert_eq!(value2, PlcValue::Bool(false));
    }
}

#[tokio::test]
#[ignore]
async fn test_health_monitoring() {
    let address = "127.0.0.1:44818";
    if !is_plc_available(address).await {
        println!(
            "Skipping test_health_monitoring: No PLC available at {}",
            address
        );
        return;
    }

    let mut manager = PlcManager::new();

    let config = PlcConfig {
        address: address.parse().unwrap(),
        max_connections: 2,
        connection_timeout: Duration::from_secs(5),
        health_check_interval: Duration::from_secs(30),
        max_packet_size: 4000,
    };

    manager.add_plc(config.clone());

    // Get a connection
    let _client = manager.get_connection(config.address).await.unwrap();

    // Perform health check
    manager.check_health().await;

    // Clean up inactive connections
    manager.cleanup_connections();
}

#[tokio::test]
#[ignore]
async fn test_large_packet_support() {
    let address = "127.0.0.1:44818";
    if !is_plc_available(address).await {
        println!(
            "Skipping test_large_packet_support: No PLC available at {}",
            address
        );
        return;
    }

    let mut client = EipClient::connect(address).await.unwrap();

    // Set maximum packet size
    client.set_max_packet_size(4000);

    // Create a large string
    let large_string = "X".repeat(2000);

    // Write the large string
    client
        .write_tag("LargeString", PlcValue::String(large_string.clone()))
        .await
        .unwrap();

    // Read it back
    let value = client.read_tag("LargeString").await.unwrap();
    if let PlcValue::String(s) = value {
        assert_eq!(s, large_string);
    } else {
        panic!("Expected string value");
    }
}

#[tokio::test]
#[ignore]
async fn test_string_operations() {
    let address = "127.0.0.1:44818";
    if !is_plc_available(address).await {
        println!(
            "Skipping test_string_operations: No PLC available at {}",
            address
        );
        return;
    }

    let mut client = EipClient::connect(address).await.unwrap();

    // Test basic string operations
    let test_string = "Hello, PLC!".to_string();
    client
        .write_tag("TestString", PlcValue::String(test_string.clone()))
        .await
        .unwrap();
    let string_value = client.read_tag("TestString").await.unwrap();
    assert_eq!(string_value, PlcValue::String(test_string));

    // Test empty string
    let empty_string = "".to_string();
    client
        .write_tag("EmptyString", PlcValue::String(empty_string.clone()))
        .await
        .unwrap();
    let empty_value = client.read_tag("EmptyString").await.unwrap();
    assert_eq!(empty_value, PlcValue::String(empty_string));

    // Test string with special characters
    let special_string = "!@#$%^&*()_+-=[]{}|;:,.<>?".to_string();
    client
        .write_tag("SpecialString", PlcValue::String(special_string.clone()))
        .await
        .unwrap();
    let special_value = client.read_tag("SpecialString").await.unwrap();
    assert_eq!(special_value, PlcValue::String(special_string));

    // Test string with spaces
    let spaced_string = "Hello World with Spaces".to_string();
    client
        .write_tag("SpacedString", PlcValue::String(spaced_string.clone()))
        .await
        .unwrap();
    let spaced_value = client.read_tag("SpacedString").await.unwrap();
    assert_eq!(spaced_value, PlcValue::String(spaced_string));

    // Test string with numbers
    let number_string = "12345".to_string();
    client
        .write_tag("NumberString", PlcValue::String(number_string.clone()))
        .await
        .unwrap();
    let number_value = client.read_tag("NumberString").await.unwrap();
    assert_eq!(number_value, PlcValue::String(number_string));

    // Test string with mixed content
    let mixed_string = "Hello123!@# World".to_string();
    client
        .write_tag("MixedString", PlcValue::String(mixed_string.clone()))
        .await
        .unwrap();
    let mixed_value = client.read_tag("MixedString").await.unwrap();
    assert_eq!(mixed_value, PlcValue::String(mixed_string));
}

#[tokio::test]
#[ignore]
async fn test_string_error_handling() {
    let address = "127.0.0.1:44818";
    if !is_plc_available(address).await {
        println!(
            "Skipping test_string_error_handling: No PLC available at {}",
            address
        );
        return;
    }

    let mut client = EipClient::connect(address).await.unwrap();

    // Test string too long
    let long_string = "X".repeat(100); // Longer than 82 characters
    let result = client
        .write_tag("LongString", PlcValue::String(long_string))
        .await;
    assert!(result.is_err());

    // Test non-ASCII characters
    let non_ascii_string = "Hello 世界".to_string();
    let result = client
        .write_tag("NonAsciiString", PlcValue::String(non_ascii_string))
        .await;
    assert!(result.is_err());

    // Test string with null bytes
    let null_string = "Hello\0World".to_string();
    let result = client
        .write_tag("NullString", PlcValue::String(null_string))
        .await;
    assert!(result.is_err());

    // Test non-existent string tag
    let result = client.read_tag("NonExistentString").await;
    assert!(result.is_err());
}
