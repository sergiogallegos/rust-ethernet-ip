/// Simple PLC Connectivity Test
/// 
/// This example tests basic connectivity and tries to read a simple tag
/// to verify the connection works before running comprehensive tests.
/// 
/// Run with: cargo run --example test_plc_connectivity

use rust_ethernet_ip::EipClient;

const PLC_ADDRESS: &str = "192.168.0.1:44818";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔌 Connecting to PLC at {}...", PLC_ADDRESS);
    let mut client = EipClient::connect(PLC_ADDRESS).await?;
    println!("✅ Connected successfully!\n");

    // Try to discover tags
    println!("📋 Attempting tag discovery...");
    match client.discover_tags().await {
        Ok(_) => {
            println!("✅ Tag discovery successful!");
            println!("   You can now check what tags are available.\n");
        }
        Err(e) => {
            println!("⚠️  Tag discovery failed: {}", e);
            println!("   This is okay - we'll try reading specific tags.\n");
        }
    }

    // Try reading some common tags that might exist
    let test_tags = vec![
        "gTestArray_DINT",
        "gTestArray_DINT[0]",
        "gTestArray_DINT[5]",
        "TestArray",
        "TestArray[0]",
        "ArrayTest",
        "ArrayTest[0]",
        "gTestUDT",
        "TestUDT",
    ];

    println!("📋 Testing common tag names...");
    for tag in &test_tags {
        match client.read_tag(tag).await {
            Ok(value) => {
                println!("   ✅ '{}' exists: {:?}", tag, value);
            }
            Err(e) => {
                let error_str = e.to_string();
                if error_str.contains("Path destination unknown") {
                    // Tag doesn't exist - this is expected
                } else {
                    println!("   ⚠️  '{}' error: {}", tag, error_str);
                }
            }
        }
    }

    println!("\n═══════════════════════════════════════════════════════════");
    println!("💡 Next Steps:");
    println!("═══════════════════════════════════════════════════════════");
    println!();
    println!("1. Create the test tags in Studio 5000 as specified in:");
    println!("   docs/PLC_TEST_TAG_DEFINITIONS.md");
    println!();
    println!("2. Required Controller Tags:");
    println!("   - gTestArray_DINT[100] (DINT array)");
    println!("   - gTestArray_REAL[50] (REAL array)");
    println!("   - gTestArray_BOOL[100] (BOOL array)");
    println!("   - gTestArray_Large[1000] (DINT array)");
    println!("   - TEST_UDT (User-Defined Type)");
    println!("   - gTestUDT (TEST_UDT instance)");
    println!("   - gTestUDT_Array[10] (TEST_UDT array)");
    println!();
    println!("3. Required Program:");
    println!("   - Create program 'TestProgram'");
    println!("   - Add same tags under Program Tags");
    println!();
    println!("4. Download to PLC and run:");
    println!("   cargo run --example test_comprehensive_arrays_udt");
    println!();

    Ok(())
}

