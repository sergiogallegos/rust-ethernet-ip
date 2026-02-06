/// Discover Tags and Test with Real PLC Tags
///
/// This tool discovers what tags actually exist in your PLC,
/// then tests array and UDT operations with those tags.
///
/// Run with: cargo run --example test_discover_and_verify
use rust_ethernet_ip::EipClient;

const PLC_ADDRESS: &str = "192.168.0.1:44818";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔌 Connecting to PLC at {}...", PLC_ADDRESS);
    let mut client = EipClient::connect(PLC_ADDRESS).await?;
    println!("✅ Connected successfully!\n");

    // Discover tags
    println!("📋 Discovering tags in PLC...");
    match client.discover_tags().await {
        Ok(_) => {
            println!("✅ Tag discovery completed!");
            println!("   Tags are now cached in the client.\n");
        }
        Err(e) => {
            println!("⚠️  Tag discovery failed: {}", e);
            println!("   We'll try to read tags directly.\n");
        }
    }

    // Try to get metadata for test tags to see what exists
    println!("📋 Checking for test tags...");
    let test_tag_names = vec![
        "gTestArray_DINT",
        "gTestArray_REAL",
        "gTestArray_BOOL",
        "gTestArray_Large",
        "gTestUDT",
        "gTestUDT_Array",
        "TestArray_DINT",
        "TestArray",
        "TestUDT",
    ];

    let mut found_tags = Vec::new();

    for tag_name in &test_tag_names {
        if let Some(metadata) = client.get_tag_metadata(tag_name).await {
            println!("   ✅ Found: '{}'", tag_name);
            println!(
                "      Type: 0x{:04X}, Size: {}, Scope: {:?}",
                metadata.data_type, metadata.size, metadata.scope
            );
            found_tags.push((*tag_name).to_string());
        }
    }

    if found_tags.is_empty() {
        println!("   ❌ No test tags found with expected names.");
        println!();
        println!("💡 Please verify in Studio 5000:");
        println!("   1. Tag names are EXACTLY as specified (case-sensitive)");
        println!("   2. Tags are in Controller Tags (not just Program Tags)");
        println!("   3. Tags are downloaded to PLC (not just saved in project)");
        println!("   4. Controller is in RUN mode (some tags require RUN mode)");
        println!();
        println!("   Expected controller-scoped tags:");
        for tag in &test_tag_names[..5] {
            println!("     - {}", tag);
        }
    } else {
        println!();
        println!("✅ Found {} test tag(s)!", found_tags.len());
        println!("   Testing with found tags...\n");

        // Test reading the first found tag
        if let Some(tag) = found_tags.first() {
            println!("📋 Testing read: '{}'", tag);
            match client.read_tag(tag).await {
                Ok(value) => {
                    println!("   ✅ Read successful: {:?}", value);

                    // If it's an array, try reading an element
                    if tag.contains("Array") {
                        let element_tag = format!("{}[0]", tag);
                        println!("   🧪 Testing array element: '{}'", element_tag);
                        match client.read_tag(&element_tag).await {
                            Ok(v) => {
                                println!("   ✅ Array element read successful: {:?}", v);
                            }
                            Err(e) => {
                                println!("   ⚠️  Array element read failed: {}", e);
                            }
                        }
                    }
                }
                Err(e) => {
                    println!("   ❌ Read failed: {}", e);
                }
            }
        }
    }

    println!();
    println!("═══════════════════════════════════════════════════════════");
    println!("💡 Next Steps:");
    println!("═══════════════════════════════════════════════════════════");
    println!();
    println!("If tags were found, we can test array/UDT operations.");
    println!("If no tags were found, please:");
    println!("  1. Double-check tag names in Studio 5000");
    println!("  2. Ensure tags are downloaded (not just saved)");
    println!("  3. Verify tags are in Controller Tags");
    println!();

    Ok(())
}
