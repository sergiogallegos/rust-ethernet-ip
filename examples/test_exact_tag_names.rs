/// Test to help identify exact tag names on the PLC
/// 
/// This will try various tag name formats to find what works

use rust_ethernet_ip::EipClient;

const PLC_ADDRESS: &str = "192.168.0.1:44818";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔌 Connecting to PLC at {}...", PLC_ADDRESS);
    let mut client = EipClient::connect(PLC_ADDRESS).await?;
    println!("✅ Connected successfully!\n");

    // Try various tag name formats
    let test_tags = vec![
        // System tags (should always exist)
        "Controller",
        "Controller.Type",
        "Controller.MajorRev",
        
        // Your test tags - try different variations
        "gTestArray_DINT",
        "gTestArray_DINT[0]",
        "gTestArray_DINT[5]",
        "gTestUDT",
        
        // Try without 'g' prefix
        "TestArray_DINT",
        "TestUDT",
        
        // Try with different case
        "GTestArray_DINT",
        "GTESTARRAY_DINT",
    ];

    println!("📋 Testing various tag name formats:\n");
    for tag_name in test_tags {
        print!("   Trying: '{}' ... ", tag_name);
        match client.read_tag(tag_name).await {
            Ok(value) => {
                println!("✅ Success: {:?}", value);
            }
            Err(e) => {
                // Only show the error code, not full message
                if let Some(err_str) = e.to_string().split("CIP Error").nth(1) {
                    println!("❌ {}", err_str.trim());
                } else {
                    println!("❌ {}", e);
                }
            }
        }
    }

    println!("\n💡 If all tags fail, please check:");
    println!("   1. Tag names match exactly (case-sensitive)");
    println!("   2. Tags are downloaded to PLC (not just saved)");
    println!("   3. Controller is in RUN mode");
    println!("   4. Tags have External Access enabled");

    Ok(())
}

