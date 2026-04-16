/// Comprehensive Test for Array and UDT Operations
///
/// This example tests the new array element addressing and UDT implementations
/// with a real ControlLogix PLC.
///
/// Prerequisites:
/// - PLC at 192.168.0.1 with tags created per docs/PLC_TEST_TAG_DEFINITIONS.md
/// - Controller-scoped tags: gTestArray_DINT, gTestArray_REAL, gTestArray_BOOL, etc.
/// - Program-scoped tags: Program:TestProgram.gTestArray_DINT, etc.
/// - UDT: TEST_UDT with members as specified
///
/// Run with: cargo run --example test_comprehensive_arrays_udt
use rust_ethernet_ip::{EipClient, PlcValue, RoutePath};
use std::collections::HashMap;
use std::env;

fn get_plc_address() -> String {
    env::var("TEST_PLC_ADDRESS").unwrap_or_else(|_| "192.168.0.1:44818".to_string())
}

fn get_cpu_slot() -> u8 {
    env::var("TEST_PLC_SLOT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0)
}

/// Helper function to determine which tag scope works (controller or program)
async fn detect_tag_scope(client: &mut EipClient) -> String {
    // Try controller-scoped first (simpler)
    match client.read_tag("gTestArray_DINT[0]").await {
        Ok(_) => {
            println!("✅ Using controller-scoped tags");
            return String::new(); // Empty prefix for controller-scoped
        }
        Err(_) => {}
    }

    // Try program-scoped
    match client
        .read_tag("Program:TestProgram.gTestArray_DINT[0]")
        .await
    {
        Ok(_) => {
            println!("✅ Using program-scoped tags (Program:TestProgram.*)");
            return "Program:TestProgram.".to_string();
        }
        Err(_) => {}
    }

    // Default to controller-scoped if neither works
    println!("⚠️  Could not detect scope, defaulting to controller-scoped");
    String::new()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let plc_address = get_plc_address();
    let cpu_slot = get_cpu_slot();
    println!("🔌 Connecting to ControlLogix PLC at {}...", plc_address);
    println!("   CPU Slot: {}", cpu_slot);
    println!("   Route Path: Port 1 (backplane), Slot {} (CPU)", cpu_slot);

    // Create route path for ControlLogix
    // Reference: EtherNetIP_Connection_Paths_and_Routing.md
    // Port 1 = Backplane, Slot 0 = CPU location
    let route_path = RoutePath::new().add_slot(cpu_slot);

    println!("   Route path bytes: {:02X?}", route_path.to_cip_bytes());

    // Connect with route path
    let mut client = EipClient::with_route_path(&plc_address, route_path).await?;
    println!("✅ Connected successfully!\n");

    // Detect which scope works
    let scope_prefix = detect_tag_scope(&mut client).await;
    let mut restore_values: HashMap<String, PlcValue> = HashMap::new();
    println!();

    // ========================================================================
    // Test 1: Array Element Addressing - Single Element Read (8-bit index)
    // ========================================================================
    println!("📋 Test 1: Read single array element (8-bit index)");
    println!("   Reading: gTestArray_DINT[5] (controller-scoped)");
    let _tag_name = match client.read_tag("gTestArray_DINT[5]").await {
        Ok(value) => {
            println!("   ✅ Read successful (controller-scoped): {:?}", value);
            assert!(matches!(value, PlcValue::Dint(_)));
            "gTestArray_DINT[5]".to_string()
        }
        Err(e) => {
            println!("   ⚠️  Controller-scoped failed: {}", e);
            println!("   Trying program-scoped: Program:TestProgram.gTestArray_DINT[5]");
            match client
                .read_tag("Program:TestProgram.gTestArray_DINT[5]")
                .await
            {
                Ok(value) => {
                    println!("   ✅ Read successful (program-scoped): {:?}", value);
                    assert!(matches!(value, PlcValue::Dint(_)));
                    "Program:TestProgram.gTestArray_DINT[5]".to_string()
                }
                Err(e2) => {
                    println!(
                        "   ❌ Both scopes failed. Controller: {}, Program: {}",
                        e, e2
                    );
                    return Err(format!("Failed to read array element from either scope").into());
                }
            }
        }
    };
    println!();

    // ========================================================================
    // Test 2: Array Element Addressing - Single Element Write (8-bit index)
    // ========================================================================
    println!("📋 Test 2: Write single array element (8-bit index)");
    let write_tag = format!("{}gTestArray_DINT[5]", scope_prefix);
    println!("   Writing: {} = 999", write_tag);
    if let Ok(original_value) = client.read_tag(&write_tag).await {
        restore_values.insert(write_tag.clone(), original_value);
    }
    match client.write_tag(&write_tag, PlcValue::Dint(999)).await {
        Ok(_) => {
            println!("   ✅ Write successful");

            // Read back to verify
            match client.read_tag(&write_tag).await {
                Ok(value) => {
                    println!("   ✅ Read back: {:?}", value);
                    if let PlcValue::Dint(v) = value {
                        assert_eq!(v, 999, "Value should be 999");
                        println!("   ✅ Verification passed!");
                    }
                }
                Err(e) => println!("   ⚠️  Read back failed: {}", e),
            }
        }
        Err(e) => {
            println!("   ❌ Write failed: {}", e);
        }
    }
    println!();

    // ========================================================================
    // Test 3: Array Element Addressing - Single Element Read (16-bit index)
    // ========================================================================
    println!("📋 Test 3: Read single array element (16-bit index)");
    println!("   Reading: Program:TestProgram.gTestArray_Large[300]");
    // Note: gTestArray_Large might only be in controller scope
    match client.read_tag("gTestArray_Large[300]").await {
        Ok(value) => {
            println!("   ✅ Read successful: {:?}", value);
            assert!(matches!(value, PlcValue::Dint(_)));
        }
        Err(e) => {
            println!("   ❌ Read failed: {}", e);
            println!("   ⚠️  Make sure gTestArray_Large[1000] exists");
        }
    }
    println!();

    // ========================================================================
    // Test 4: Array Element Addressing - Single Element Write (16-bit index)
    // ========================================================================
    println!("📋 Test 4: Write single array element (16-bit index)");
    println!("   Writing: gTestArray_Large[300] = 12345");
    // Note: gTestArray_Large is controller-scoped
    if let Ok(original_value) = client.read_tag("gTestArray_Large[300]").await {
        restore_values.insert("gTestArray_Large[300]".to_string(), original_value);
    }
    match client
        .write_tag("gTestArray_Large[300]", PlcValue::Dint(12345))
        .await
    {
        Ok(_) => {
            println!("   ✅ Write successful");

            // Read back to verify
            match client.read_tag("gTestArray_Large[300]").await {
                Ok(value) => {
                    println!("   ✅ Read back: {:?}", value);
                    if let PlcValue::Dint(v) = value {
                        assert_eq!(v, 12345, "Value should be 12345");
                        println!("   ✅ Verification passed!");
                    }
                }
                Err(e) => println!("   ⚠️  Read back failed: {}", e),
            }
        }
        Err(e) => {
            println!("   ❌ Write failed: {}", e);
        }
    }
    println!();

    // ========================================================================
    // Test 5: BOOL Array Element
    // ========================================================================
    println!("📋 Test 5: BOOL array element read/write");
    let bool_tag = format!("{}gTestArray_BOOL[15]", scope_prefix);
    println!("   Reading: {}", bool_tag);
    match client.read_tag(&bool_tag).await {
        Ok(value) => {
            println!("   ✅ Read successful: {:?}", value);

            // Write opposite value
            let new_value = if let PlcValue::Bool(b) = value {
                PlcValue::Bool(!b)
            } else {
                PlcValue::Bool(true)
            };
            restore_values.insert(bool_tag.clone(), value.clone());

            println!("   Writing: {} = {:?}", bool_tag, new_value);
            match client.write_tag(&bool_tag, new_value.clone()).await {
                Ok(_) => {
                    println!("   ✅ Write successful");

                    // Read back
                    match client.read_tag(&bool_tag).await {
                        Ok(read_back) => {
                            println!("   ✅ Read back: {:?}", read_back);
                            assert_eq!(read_back, new_value);
                            println!("   ✅ Verification passed!");
                        }
                        Err(e) => println!("   ⚠️  Read back failed: {}", e),
                    }
                }
                Err(e) => println!("   ❌ Write failed: {}", e),
            }
        }
        Err(e) => {
            println!("   ❌ Read failed: {}", e);
            println!("   ⚠️  Make sure gTestArray_BOOL[100] exists");
        }
    }
    println!();

    // ========================================================================
    // Test 6: Complete UDT Read
    // ========================================================================
    println!("📋 Test 6: Complete UDT read");
    let udt_tag = format!("{}gTestUDT", scope_prefix);
    println!("   Reading: {}", udt_tag);
    match client.read_tag(&udt_tag).await {
        Ok(value) => {
            if let PlcValue::Udt(udt_data) = value {
                println!("   ✅ Read successful");
                println!("   📊 UDT symbol_id: {}", udt_data.symbol_id);
                println!("   📊 UDT data length: {} bytes", udt_data.data.len());

                if udt_data.symbol_id == 0 {
                    println!("   ⚠️  Warning: symbol_id is 0 (may need tag attributes)");
                } else {
                    println!("   ✅ symbol_id is valid");
                }
            } else {
                println!("   ❌ Expected UDT, got: {:?}", value);
            }
        }
        Err(e) => {
            println!("   ❌ Read failed: {}", e);
            println!("   ⚠️  Make sure gTestUDT (TEST_UDT) exists");
        }
    }
    println!();

    // ========================================================================
    // Test 7: UDT Member Access
    // ========================================================================
    println!("📋 Test 7: UDT member access");
    let udt_member_tag = format!("{}gTestUDT.Member1_DINT", scope_prefix);
    println!("   Reading: {}", udt_member_tag);
    match client.read_tag(&udt_member_tag).await {
        Ok(value) => {
            println!("   ✅ Read successful: {:?}", value);
            assert!(matches!(value, PlcValue::Dint(_)));
        }
        Err(e) => {
            println!("   ❌ Read failed: {}", e);
        }
    }
    println!();

    // ========================================================================
    // Test 8: UDT Array Member Access
    // ========================================================================
    println!("📋 Test 8: UDT array member access");
    let udt_array_tag = format!("{}gTestUDT.Array_DINT[5]", scope_prefix);
    println!("   Reading: {}", udt_array_tag);
    match client.read_tag(&udt_array_tag).await {
        Ok(value) => {
            println!("   ✅ Read successful: {:?}", value);
            restore_values.insert(udt_array_tag.clone(), value.clone());

            // Write new value
            println!("   Writing: {} = 99", udt_array_tag);
            match client.write_tag(&udt_array_tag, PlcValue::Dint(99)).await {
                Ok(_) => {
                    println!("   ✅ Write successful");

                    // Read back
                    match client.read_tag(&udt_array_tag).await {
                        Ok(read_back) => {
                            println!("   ✅ Read back: {:?}", read_back);
                            if let PlcValue::Dint(v) = read_back {
                                assert_eq!(v, 99);
                                println!("   ✅ Verification passed!");
                            }
                        }
                        Err(e) => println!("   ⚠️  Read back failed: {}", e),
                    }
                }
                Err(e) => println!("   ❌ Write failed: {}", e),
            }
        }
        Err(e) => {
            println!("   ❌ Read failed: {}", e);
        }
    }
    println!();

    // ========================================================================
    // Test 9: Array of UDTs - Single Element
    // ========================================================================
    println!("📋 Test 9: Array of UDTs - single element");
    let udt_array_elem_tag = format!("{}gTestUDT_Array[3]", scope_prefix);
    println!("   Reading: {}", udt_array_elem_tag);
    match client.read_tag(&udt_array_elem_tag).await {
        Ok(value) => {
            if let PlcValue::Udt(udt_data) = value {
                println!("   ✅ Read successful");
                println!("   📊 UDT symbol_id: {}", udt_data.symbol_id);
                println!("   📊 UDT data length: {} bytes", udt_data.data.len());
            } else {
                println!("   ❌ Expected UDT, got: {:?}", value);
            }
        }
        Err(e) => {
            println!("   ❌ Read failed: {}", e);
            println!("   ⚠️  Make sure gTestUDT_Array[10] exists");
        }
    }
    println!();

    // ========================================================================
    // Test 10: Array of UDTs - Member Access
    // ========================================================================
    println!("📋 Test 10: Array of UDTs - member access");
    let udt_array_member_tag = format!("{}gTestUDT_Array[3].Member1_DINT", scope_prefix);
    println!("   Reading: {}", udt_array_member_tag);
    match client.read_tag(&udt_array_member_tag).await {
        Ok(value) => {
            println!("   ✅ Read successful: {:?}", value);

            // Write new value
            println!("   Writing: {} = 777", udt_array_member_tag);
            match client
                .write_tag(&udt_array_member_tag, PlcValue::Dint(777))
                .await
            {
                Ok(_) => {
                    println!("   ✅ Write successful");

                    // Read back
                    match client.read_tag(&udt_array_member_tag).await {
                        Ok(read_back) => {
                            println!("   ✅ Read back: {:?}", read_back);
                            if let PlcValue::Dint(v) = read_back {
                                assert_eq!(v, 777);
                                println!("   ✅ Verification passed!");
                            }
                        }
                        Err(e) => println!("   ⚠️  Read back failed: {}", e),
                    }
                }
                Err(e) => println!("   ❌ Write failed: {}", e),
            }
        }
        Err(e) => {
            println!("   ❌ Read failed: {}", e);
        }
    }
    println!();

    // ========================================================================
    // Test 11: Array of UDTs - Array Member Access
    // ========================================================================
    println!("📋 Test 11: Array of UDTs - array member access");
    let udt_array_nested_tag = format!("{}gTestUDT_Array[2].Array_DINT[4]", scope_prefix);
    println!("   Reading: {}", udt_array_nested_tag);
    match client.read_tag(&udt_array_nested_tag).await {
        Ok(value) => {
            println!("   ✅ Read successful: {:?}", value);
            restore_values.insert(udt_array_nested_tag.clone(), value.clone());

            // Write new value
            println!("   Writing: {} = 888", udt_array_nested_tag);
            match client
                .write_tag(&udt_array_nested_tag, PlcValue::Dint(888))
                .await
            {
                Ok(_) => {
                    println!("   ✅ Write successful");

                    // Read back
                    match client.read_tag(&udt_array_nested_tag).await {
                        Ok(read_back) => {
                            println!("   ✅ Read back: {:?}", read_back);
                            if let PlcValue::Dint(v) = read_back {
                                assert_eq!(v, 888);
                                println!("   ✅ Verification passed!");
                            }
                        }
                        Err(e) => println!("   ⚠️  Read back failed: {}", e),
                    }
                }
                Err(e) => println!("   ❌ Write failed: {}", e),
            }
        }
        Err(e) => {
            println!("   ❌ Read failed: {}", e);
        }
    }
    println!();

    // ========================================================================
    // Test 12: Program-Scoped Array
    // ========================================================================
    println!("📋 Test 12: Program-scoped array");
    println!("   Reading: Program:TestProgram.gTestArray_DINT[5]");
    match client
        .read_tag("Program:TestProgram.gTestArray_DINT[5]")
        .await
    {
        Ok(value) => {
            println!("   ✅ Read successful: {:?}", value);
            restore_values.insert(
                "Program:TestProgram.gTestArray_DINT[5]".to_string(),
                value.clone(),
            );

            // Write new value
            println!("   Writing: Program:TestProgram.gTestArray_DINT[5] = 5555");
            match client
                .write_tag(
                    "Program:TestProgram.gTestArray_DINT[5]",
                    PlcValue::Dint(5555),
                )
                .await
            {
                Ok(_) => {
                    println!("   ✅ Write successful");

                    // Read back
                    match client
                        .read_tag("Program:TestProgram.gTestArray_DINT[5]")
                        .await
                    {
                        Ok(read_back) => {
                            println!("   ✅ Read back: {:?}", read_back);
                            if let PlcValue::Dint(v) = read_back {
                                assert_eq!(v, 5555);
                                println!("   ✅ Verification passed!");
                            }
                        }
                        Err(e) => println!("   ⚠️  Read back failed: {}", e),
                    }
                }
                Err(e) => println!("   ❌ Write failed: {}", e),
            }
        }
        Err(e) => {
            println!("   ❌ Read failed: {}", e);
            println!("   ⚠️  Make sure Program:TestProgram exists with gTestArray_DINT");
        }
    }
    println!();

    // ========================================================================
    // Test 13: Program-Scoped UDT
    // ========================================================================
    println!("📋 Test 13: Program-scoped UDT");
    println!("   Reading: Program:TestProgram.gTestUDT");
    match client.read_tag("Program:TestProgram.gTestUDT").await {
        Ok(value) => {
            if let PlcValue::Udt(udt_data) = value {
                println!("   ✅ Read successful");
                println!("   📊 UDT symbol_id: {}", udt_data.symbol_id);
                println!("   📊 UDT data length: {} bytes", udt_data.data.len());
            } else {
                println!("   ❌ Expected UDT, got: {:?}", value);
            }
        }
        Err(e) => {
            println!("   ❌ Read failed: {}", e);
            println!("   ⚠️  Make sure Program:TestProgram.gTestUDT exists");
        }
    }
    println!();

    // ========================================================================
    // Summary
    // ========================================================================
    println!("═══════════════════════════════════════════════════════════");
    println!("✅ Comprehensive Array and UDT Testing Complete!");
    println!("═══════════════════════════════════════════════════════════");
    println!();
    println!("♻️  Restoring modified tags to their original values...");
    let mut restore_failures = Vec::new();
    for (tag_name, original_value) in restore_values {
        print!("   Restoring {}... ", tag_name);
        match client.write_tag(&tag_name, original_value).await {
            Ok(_) => println!("✅"),
            Err(e) => {
                println!("❌ {}", e);
                restore_failures.push((tag_name, e.to_string()));
            }
        }
    }
    if restore_failures.is_empty() {
        println!("   ✅ All modified tags restored");
    } else {
        println!("   ⚠️  Restore failures:");
        for (tag_name, error) in restore_failures {
            println!("      - {}: {}", tag_name, error);
        }
    }
    println!();
    println!("All tests that completed successfully verify:");
    println!("  ✅ Direct array element addressing (no full array read)");
    println!("  ✅ 8-bit and 16-bit element ID segments");
    println!("  ✅ UDT read/write with symbol_id");
    println!("  ✅ UDT member access");
    println!("  ✅ Array members within UDTs");
    println!("  ✅ Arrays of UDTs");
    println!("  ✅ Program-scoped tag access");
    println!();

    Ok(())
}
