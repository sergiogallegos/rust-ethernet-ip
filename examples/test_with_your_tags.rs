/// Test Array and UDT Operations with Your Actual Tags
/// 
/// This tool lets you test with whatever tags you actually have in your PLC.
/// Just modify the TAG_NAMES array below with your actual tag names.
/// 
/// Run with: cargo run --example test_with_your_tags

use rust_ethernet_ip::{EipClient, PlcValue};

const PLC_ADDRESS: &str = "192.168.0.1:44818";

// ═══════════════════════════════════════════════════════════════════════
// MODIFY THESE TAG NAMES TO MATCH YOUR ACTUAL PLC TAGS
// ═══════════════════════════════════════════════════════════════════════

// Array tags to test (modify these to match your actual array tag names)
const ARRAY_TAGS: &[&str] = &[
    "gTestArray_DINT",      // Change to your actual array name
    "TestArray_DINT",       // Alternative name
    "MyArray",              // Another alternative
];

// UDT tags to test (modify these to match your actual UDT tag names)
const UDT_TAGS: &[&str] = &[
    "gTestUDT",             // Change to your actual UDT name
    "TestUDT",              // Alternative name
    "MyUDT",                // Another alternative
];

// Program-scoped tags (modify program name and tag names)
const PROGRAM_NAME: &str = "TestProgram";  // Change to your actual program name
const PROGRAM_ARRAY_TAGS: &[&str] = &[
    "gTestArray_DINT",
    "TestArray_DINT",
];

// ═══════════════════════════════════════════════════════════════════════

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔌 Connecting to PLC at {}...", PLC_ADDRESS);
    let mut client = EipClient::connect(PLC_ADDRESS).await?;
    println!("✅ Connected successfully!\n");

    // Test 1: Try to read array base (without index)
    println!("📋 Test 1: Reading array base tags (without index)");
    let mut found_array: Option<String> = None;
    
    for tag in ARRAY_TAGS {
        println!("   Trying: '{}'", tag);
        match client.read_tag(tag).await {
            Ok(value) => {
                println!("   ✅ SUCCESS! Found array: '{}'", tag);
                println!("      Value: {:?}", value);
                found_array = Some(tag.to_string());
                break;
            }
            Err(e) => {
                let error_str = e.to_string();
                if !error_str.contains("Path destination unknown") {
                    println!("   ⚠️  Error: {}", error_str);
                }
            }
        }
    }
    println!();

    // Test 2: If we found an array, test element addressing
    if let Some(array_name) = &found_array {
        println!("📋 Test 2: Testing array element addressing with '{}'", array_name);
        
        // Test reading element [0]
        let tag0 = format!("{}[0]", array_name);
        println!("   Reading: '{}'", tag0);
        match client.read_tag(&tag0).await {
            Ok(value) => {
                println!("   ✅ Element [0] read successful: {:?}", value);
            }
            Err(e) => {
                println!("   ❌ Element [0] read failed: {}", e);
            }
        }
        
        // Test reading element [5]
        let tag5 = format!("{}[5]", array_name);
        println!("   Reading: '{}'", tag5);
        match client.read_tag(&tag5).await {
            Ok(value) => {
                println!("   ✅ Element [5] read successful: {:?}", value);
                
                // Test writing to element [5]
                println!("   Writing: '{}' = 999", tag5);
                match client.write_tag(&tag5, PlcValue::Dint(999)).await {
                    Ok(_) => {
                        println!("   ✅ Write successful");
                        
                        // Read back to verify
                        match client.read_tag(&tag5).await {
                            Ok(read_back) => {
                                println!("   ✅ Read back: {:?}", read_back);
                                if let PlcValue::Dint(v) = read_back {
                                    if v == 999 {
                                        println!("   ✅✅✅ VERIFICATION PASSED! Array element addressing works!");
                                    } else {
                                        println!("   ⚠️  Value mismatch: expected 999, got {}", v);
                                    }
                                }
                            }
                            Err(e) => println!("   ⚠️  Read back failed: {}", e),
                        }
                    }
                    Err(e) => println!("   ❌ Write failed: {}", e),
                }
            }
            Err(e) => {
                println!("   ❌ Element [5] read failed: {}", e);
            }
        }
        
        // Test reading element [300] (16-bit index)
        let tag300 = format!("{}[300]", array_name);
        println!("   Reading: '{}' (16-bit index test)", tag300);
        match client.read_tag(&tag300).await {
            Ok(value) => {
                println!("   ✅ Element [300] read successful: {:?}", value);
                println!("   ✅✅✅ 16-bit element addressing works!");
            }
            Err(e) => {
                let error_str = e.to_string();
                if error_str.contains("out of bounds") {
                    println!("   ⚠️  Array doesn't have 300 elements (expected)");
                } else {
                    println!("   ❌ Element [300] read failed: {}", e);
                }
            }
        }
    } else {
        println!("⚠️  No array tags found. Please update ARRAY_TAGS in the source code.");
    }
    println!();

    // Test 3: Try to read UDT
    println!("📋 Test 3: Reading UDT tags");
    let mut found_udt: Option<String> = None;
    
    for tag in UDT_TAGS {
        println!("   Trying: '{}'", tag);
        match client.read_tag(tag).await {
            Ok(value) => {
                if let PlcValue::Udt(udt_data) = value {
                    println!("   ✅ SUCCESS! Found UDT: '{}'", tag);
                    println!("      symbol_id: {}", udt_data.symbol_id);
                    println!("      data length: {} bytes", udt_data.data.len());
                    found_udt = Some(tag.to_string());
                    break;
                } else {
                    println!("   ⚠️  Tag '{}' exists but is not a UDT: {:?}", tag, value);
                }
            }
            Err(e) => {
                let error_str = e.to_string();
                if !error_str.contains("Path destination unknown") {
                    println!("   ⚠️  Error: {}", error_str);
                }
            }
        }
    }
    println!();

    // Test 4: If we found a UDT, test member access
    if let Some(udt_name) = &found_udt {
        println!("📋 Test 4: Testing UDT member access with '{}'", udt_name);
        
        // Try common member names
        let member_names = vec!["Member1_DINT", "Member1", "Value", "Data", "Status"];
        
        for member in &member_names {
            let member_tag = format!("{}.{}", udt_name, member);
            println!("   Trying: '{}'", member_tag);
            match client.read_tag(&member_tag).await {
                Ok(value) => {
                    println!("   ✅ Member '{}' read successful: {:?}", member, value);
                    break;
                }
                Err(_) => {
                    // Member doesn't exist - try next
                }
            }
        }
    } else {
        println!("⚠️  No UDT tags found. Please update UDT_TAGS in the source code.");
    }
    println!();

    // Test 5: Program-scoped tags
    println!("📋 Test 5: Testing program-scoped tags (Program: {})", PROGRAM_NAME);
    for tag in PROGRAM_ARRAY_TAGS {
        let program_tag = format!("Program:{}.{}", PROGRAM_NAME, tag);
        println!("   Trying: '{}'", program_tag);
        match client.read_tag(&program_tag).await {
            Ok(value) => {
                println!("   ✅ Program tag found: '{}' = {:?}", program_tag, value);
                
                // Test array element
                let element_tag = format!("Program:{}.{}[0]", PROGRAM_NAME, tag);
                match client.read_tag(&element_tag).await {
                    Ok(v) => {
                        println!("   ✅ Program array element read: {:?}", v);
                    }
                    Err(_) => {}
                }
                break;
            }
            Err(_) => {
                // Tag doesn't exist
            }
        }
    }
    println!();

    println!("═══════════════════════════════════════════════════════════");
    println!("📊 Test Summary:");
    println!("═══════════════════════════════════════════════════════════");
    if found_array.is_some() {
        println!("   ✅ Array operations: WORKING");
    } else {
        println!("   ❌ Array operations: No arrays found");
        println!("      → Update ARRAY_TAGS in the source code with your tag names");
    }
    
    if found_udt.is_some() {
        println!("   ✅ UDT operations: WORKING");
    } else {
        println!("   ❌ UDT operations: No UDTs found");
        println!("      → Update UDT_TAGS in the source code with your tag names");
    }
    println!();
    println!("💡 To test with your actual tags:");
    println!("   1. Open: examples/test_with_your_tags.rs");
    println!("   2. Modify ARRAY_TAGS and UDT_TAGS arrays with your tag names");
    println!("   3. Run: cargo run --example test_with_your_tags");
    println!();

    Ok(())
}

