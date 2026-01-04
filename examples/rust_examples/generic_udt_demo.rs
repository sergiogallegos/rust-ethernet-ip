// generic_udt_demo.rs - Generic UDT functionality demonstration
// ============================================================
//
// This example demonstrates the generic UDT features that work with any UDT structure:
// - Chunked reading for large UDTs
// - Individual UDT member access by offset
// - Complete data type support
// - Generic UDT writing functionality

use rust_ethernet_ip::EipClient;
use rust_ethernet_ip::PlcValue;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    env_logger::init();

    println!("🔧 Generic UDT Demo for Any PLC");
    println!("===============================");
    println!("Demonstrating generic UDT features that work with any UDT structure:");
    println!("- Chunked reading for large UDTs");
    println!("- Individual UDT member access by offset");
    println!("- Complete data type support");
    println!("- Generic UDT writing functionality");
    println!();

    // Connect to PLC
    let mut client = EipClient::connect("192.168.0.1:44818").await?;
    println!("✅ Connected to PLC at 192.168.0.1");

    // Test 1: Chunked UDT Reading (works with any UDT)
    println!("\n📦 Test 1: Chunked UDT Reading");
    println!("===============================");

    let udt_names = vec!["Part_Data", "MyUDT", "AnotherUDT", "TestUDT"];

    for udt_name in udt_names {
        println!("\nTesting UDT: {}", udt_name);

        match client.read_udt_chunked(udt_name).await {
            Ok(udt_value) => {
                println!("  ✅ UDT read successfully");

                if let PlcValue::Udt(udt_data) = udt_value {
                    println!("  📊 UDT Data:");
                    println!("    Symbol ID: {}", udt_data.symbol_id);
                    println!("    Data Size: {} bytes", udt_data.data.len());
                    println!("    Data Preview: {:02X?}", &udt_data.data[..udt_data.data.len().min(32)]);
                    if udt_data.data.len() > 32 {
                        println!("    ... ({} more bytes)", udt_data.data.len() - 32);
                    }
                }
            }
            Err(e) => {
                println!("  ❌ UDT read failed: {}", e);
            }
        }
    }

    // Test 2: Generic UDT Member Access by Offset
    println!("\n🔍 Test 2: Generic UDT Member Access by Offset");
    println!("=============================================");

    // Example: Reading different data types from any UDT
    let test_cases = vec![
        ("Part_Data", 0, 1, 0x00C1, "BOOL at offset 0"),
        ("Part_Data", 4, 4, 0x00CA, "REAL at offset 4"),
        ("Part_Data", 8, 4, 0x00C4, "DINT at offset 8"),
        ("MyUDT", 0, 1, 0x00C1, "BOOL at offset 0"),
        ("MyUDT", 2, 2, 0x00C2, "INT at offset 2"),
        ("TestUDT", 0, 84, 0x00CE, "STRING at offset 0"),
    ];

    for (udt_name, offset, size, data_type, description) in test_cases {
        println!(
            "\nReading {} from {} (offset: {}, size: {}, type: 0x{:04X})",
            description, udt_name, offset, size, data_type
        );

        match client
            .read_udt_member_by_offset(udt_name, offset, size, data_type)
            .await
        {
            Ok(value) => {
                println!("  ✅ SUCCESS: {:?}", value);
            }
            Err(e) => {
                println!("  ❌ FAILED: {}", e);
            }
        }
    }

    // Test 3: Generic UDT Member Writing by Offset
    println!("\n✏️ Test 3: Generic UDT Member Writing by Offset");
    println!("==============================================");

    let write_tests = vec![
        (
            "Part_Data",
            0,
            1,
            0x00C1,
            PlcValue::Bool(true),
            "BOOL at offset 0",
        ),
        (
            "Part_Data",
            4,
            4,
            0x00CA,
            PlcValue::Real(99.9),
            "REAL at offset 4",
        ),
        (
            "Part_Data",
            8,
            4,
            0x00C4,
            PlcValue::Dint(12345),
            "DINT at offset 8",
        ),
        (
            "MyUDT",
            0,
            1,
            0x00C1,
            PlcValue::Bool(false),
            "BOOL at offset 0",
        ),
        (
            "TestUDT",
            0,
            84,
            0x00CE,
            PlcValue::String("Hello World".to_string()),
            "STRING at offset 0",
        ),
    ];

    for (udt_name, offset, size, data_type, value, description) in write_tests {
        println!(
            "\nWriting {} to {} (offset: {}, size: {}, type: 0x{:04X})",
            description, udt_name, offset, size, data_type
        );

        match client
            .write_udt_member_by_offset(udt_name, offset, size, data_type, value.clone())
            .await
        {
            Ok(_) => {
                println!("  ✅ Write successful");

                // Read it back to verify
                match client
                    .read_udt_member_by_offset(udt_name, offset, size, data_type)
                    .await
                {
                    Ok(read_value) => {
                        if read_value == value {
                            println!("  ✅ Read back verification successful");
                        } else {
                            println!(
                                "  ⚠️ Read back value differs: expected {:?}, got {:?}",
                                value, read_value
                            );
                        }
                    }
                    Err(e) => {
                        println!("  ❌ Read back failed: {}", e);
                    }
                }
            }
            Err(e) => {
                println!("  ❌ Write failed: {}", e);
            }
        }
    }

    // Test 4: Data Type Support Demonstration
    println!("\n📊 Test 4: Data Type Support Demonstration");
    println!("==========================================");

    let data_types = vec![
        (0x00C1, "BOOL", PlcValue::Bool(true)),
        (0x00C2, "INT", PlcValue::Int(1234)),
        (0x00C3, "DINT", PlcValue::Dint(123456)),
        (0x00C4, "DINT", PlcValue::Dint(789012)),
        (0x00C5, "LINT", PlcValue::Lint(123456789012345)),
        (0x00C6, "WORD", PlcValue::Uint(0xABCD)),
        (0x00C7, "DWORD", PlcValue::Udint(0x12345678)),
        (0x00C8, "LWORD", PlcValue::Ulint(0x123456789ABCDEF0)),
        (0x00CA, "REAL", PlcValue::Real(3.14159)),
        (0x00CB, "LREAL", PlcValue::Lreal(2.718281828459045)),
        (0x00CE, "STRING", PlcValue::String("Hello UDT!".to_string())),
        (0x00CF, "SINT", PlcValue::Sint(-100)),
        (0x00D0, "USINT", PlcValue::Usint(200)),
        (0x00D1, "UINT", PlcValue::Uint(30000)),
        (0x00D2, "UDINT", PlcValue::Udint(4000000000)),
        (0x00D3, "ULINT", PlcValue::Ulint(5000000000000000000)),
    ];

    for (data_type, type_name, test_value) in data_types {
        println!("\nTesting {} (0x{:04X}):", type_name, data_type);

        // Test serialization
        let udt = rust_ethernet_ip::udt::UserDefinedType::new("Test".to_string());
        let member = rust_ethernet_ip::udt::UdtMember {
            name: "test".to_string(),
            data_type,
            offset: 0,
            size: 8, // Use a reasonable size
        };

        match udt.serialize_member_value(&member, &test_value) {
            Ok(serialized) => {
                println!("  ✅ Serialization successful: {} bytes", serialized.len());

                // Test deserialization
                match udt.parse_member_value(&member, &serialized) {
                    Ok(parsed_value) => {
                        if parsed_value == test_value {
                            println!("  ✅ Deserialization successful: {:?}", parsed_value);
                        } else {
                            println!(
                                "  ⚠️ Deserialization mismatch: expected {:?}, got {:?}",
                                test_value, parsed_value
                            );
                        }
                    }
                    Err(e) => {
                        println!("  ❌ Deserialization failed: {}", e);
                    }
                }
            }
            Err(e) => {
                println!("  ❌ Serialization failed: {}", e);
            }
        }
    }

    // Test 5: Error Handling
    println!("\n🚨 Test 5: Error Handling");
    println!("=========================");

    // Test invalid offset
    println!("\nTesting invalid offset:");
    match client
        .read_udt_member_by_offset("Part_Data", 9999, 1, 0x00C1)
        .await
    {
        Ok(value) => println!("  ❌ Unexpected success: {:?}", value),
        Err(e) => println!("  ✅ Expected error: {}", e),
    }

    // Test invalid data type
    println!("\nTesting invalid data type:");
    match client
        .write_udt_member_by_offset("Part_Data", 0, 1, 0x9999, PlcValue::Bool(true))
        .await
    {
        Ok(_) => println!("  ❌ Unexpected success"),
        Err(e) => println!("  ✅ Expected error: {}", e),
    }

    // Test data type mismatch
    println!("\nTesting data type mismatch:");
    match client
        .write_udt_member_by_offset("Part_Data", 0, 1, 0x00C1, PlcValue::Real(3.14))
        .await
    {
        Ok(_) => println!("  ❌ Unexpected success"),
        Err(e) => println!("  ✅ Expected error: {}", e),
    }

    // Test 6: Performance Testing
    println!("\n⚡ Test 6: Performance Testing");
    println!("=============================");

    let iterations = 100;
    let udt_name = "Part_Data";
    let offset = 0;
    let size = 1;
    let data_type = 0x00C1;

    // Test read performance
    let start = std::time::Instant::now();
    let mut success_count = 0;
    for _ in 0..iterations {
        match client
            .read_udt_member_by_offset(udt_name, offset, size, data_type)
            .await
        {
            Ok(_) => success_count += 1,
            Err(_) => {}
        }
    }
    let duration = start.elapsed();
    let ops_per_sec = (success_count as f64) / duration.as_secs_f64();

    println!("📊 Read Performance:");
    println!("  - {} operations in {:?}", success_count, duration);
    println!("  - {:.1} operations/second", ops_per_sec);
    if success_count > 0 {
        println!(
            "  - {:.2}ms average per operation",
            duration.as_millis() as f64 / success_count as f64
        );
    }

    println!("\n✅ Generic UDT Demo completed!");
    println!("\n📋 Summary of Generic Features:");
    println!("- ✅ Chunked UDT reading for any UDT structure");
    println!("- ✅ Individual UDT member access by offset (any UDT)");
    println!("- ✅ Complete data type support (BOOL, REAL, STRING, etc.)");
    println!("- ✅ Generic UDT writing functionality");
    println!("- ✅ Error handling and recovery");
    println!("- ✅ Performance testing");
    println!("\n💡 This library now works with any UDT structure!");
    println!("   Just specify the offset, size, and data type for each member.");

    Ok(())
}
