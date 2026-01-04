/// Tag Verification and Diagnostic Tool
///
/// This tool helps verify tag names and test basic connectivity.
/// It tries different variations of tag names to find what works.
///
/// Run with: cargo run --example test_tag_verification
use rust_ethernet_ip::EipClient;

const PLC_ADDRESS: &str = "192.168.0.1:44818";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔌 Connecting to PLC at {}...", PLC_ADDRESS);
    let mut client = EipClient::connect(PLC_ADDRESS).await?;
    println!("✅ Connected successfully!\n");

    // Test 1: Try reading base array name (without index)
    println!("📋 Test 1: Reading base array name (without index)");
    let base_tags = vec![
        "gTestArray_DINT",
        "TestArray_DINT",
        "gTestArray",
        "TestArray",
    ];

    for tag in &base_tags {
        println!("   Trying: '{}'", tag);
        match client.read_tag(tag).await {
            Ok(value) => {
                println!("   ✅ SUCCESS! '{}' exists: {:?}", tag, value);
                println!("   📊 This means the tag exists - we can test array access now");
                break;
            }
            Err(e) => {
                let error_str = e.to_string();
                if error_str.contains("Path destination unknown") {
                    // Tag doesn't exist
                } else {
                    println!("   ⚠️  Error: {}", error_str);
                }
            }
        }
    }
    println!();

    // Test 2: Try reading array element with index 0
    println!("📋 Test 2: Reading array element [0]");
    let array_tags = vec![
        "gTestArray_DINT[0]",
        "TestArray_DINT[0]",
        "gTestArray[0]",
        "TestArray[0]",
    ];

    for tag in &array_tags {
        println!("   Trying: '{}'", tag);
        match client.read_tag(tag).await {
            Ok(value) => {
                println!("   ✅ SUCCESS! '{}' exists: {:?}", tag, value);
                println!("   📊 Array element addressing is working!");

                // Try reading element [5] to test direct addressing
                let base_name = tag.split('[').next().unwrap();
                let test_tag = format!("{}[5]", base_name);
                println!("   🧪 Testing element [5]: '{}'", test_tag);
                match client.read_tag(&test_tag).await {
                    Ok(v) => {
                        println!("   ✅ Element [5] read successful: {:?}", v);
                    }
                    Err(e) => {
                        println!("   ⚠️  Element [5] failed: {}", e);
                    }
                }
                break;
            }
            Err(e) => {
                let error_str = e.to_string();
                if error_str.contains("Path destination unknown") {
                    // Tag doesn't exist
                } else {
                    println!("   ⚠️  Error: {}", error_str);
                }
            }
        }
    }
    println!();

    // Test 3: Try UDT
    println!("📋 Test 3: Reading UDT");
    let udt_tags = vec!["gTestUDT", "TestUDT", "gUDT", "UDT"];

    for tag in &udt_tags {
        println!("   Trying: '{}'", tag);
        match client.read_tag(tag).await {
            Ok(value) => {
                println!("   ✅ SUCCESS! '{}' exists: {:?}", tag, value);
                if let rust_ethernet_ip::PlcValue::Udt(udt_data) = value {
                    println!("   📊 UDT symbol_id: {}", udt_data.symbol_id);
                    println!("   📊 UDT data length: {} bytes", udt_data.data.len());
                }
                break;
            }
            Err(e) => {
                let error_str = e.to_string();
                if error_str.contains("Path destination unknown") {
                    // Tag doesn't exist
                } else {
                    println!("   ⚠️  Error: {}", error_str);
                }
            }
        }
    }
    println!();

    // Test 4: Try program-scoped tags
    println!("📋 Test 4: Reading program-scoped tags");
    let program_names = vec!["TestProgram", "MainProgram", "Main", "Program1"];

    for prog_name in &program_names {
        let test_tag = format!("Program:{}", prog_name);
        println!("   Trying program: '{}'", test_tag);

        // Try to read a simple tag in the program
        let test_tags = vec![
            format!("Program:{}.gTestArray_DINT", prog_name),
            format!("Program:{}.TestArray_DINT", prog_name),
            format!("Program:{}.gTestArray_DINT[0]", prog_name),
        ];

        let mut found = false;
        for tag in &test_tags {
            match client.read_tag(tag).await {
                Ok(value) => {
                    println!("   ✅ SUCCESS! '{}' exists: {:?}", tag, value);
                    found = true;
                    break;
                }
                Err(e) => {
                    let error_str = e.to_string();
                    if !error_str.contains("Path destination unknown") {
                        println!("   ⚠️  '{}' error: {}", tag, error_str);
                    }
                }
            }
        }
        if found {
            break;
        }
    }
    println!();

    println!("═══════════════════════════════════════════════════════════");
    println!("💡 Diagnostic Summary:");
    println!("═══════════════════════════════════════════════════════════");
    println!();
    println!("If all tests failed with 'Path destination unknown':");
    println!("  1. Verify tag names match EXACTLY (case-sensitive)");
    println!("  2. Ensure tags are downloaded to the PLC (not just saved)");
    println!("  3. Check that tags are in Controller Tags (not Program Tags)");
    println!("  4. Verify the program name if using program-scoped tags");
    println!();
    println!("If you found working tags, update the test script with those names.");
    println!();

    Ok(())
}
