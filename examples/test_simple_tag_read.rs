/// Simple test to verify basic tag reading works
///
/// This example tries to read a simple tag to verify connectivity
/// and that tags exist on the PLC.
use rust_ethernet_ip::EipClient;

const PLC_ADDRESS: &str = "192.168.0.1:44818";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔌 Connecting to PLC at {}...", PLC_ADDRESS);
    let mut client = EipClient::connect(PLC_ADDRESS).await?;
    println!("✅ Connected successfully!\n");

    // Try reading some common system tags that should always exist
    let test_tags = vec![
        "Controller.Type",
        "Controller.MajorRev",
        "Controller.MinorRev",
        "gTestArray_DINT", // Your test tag (without index)
        "gTestUDT",        // Your UDT tag
    ];

    println!("📋 Testing tag reads:\n");
    for tag_name in test_tags {
        println!("   Trying: '{}'", tag_name);
        match client.read_tag(tag_name).await {
            Ok(value) => {
                println!("   ✅ Success: {:?}\n", value);
            }
            Err(e) => {
                println!("   ❌ Failed: {}\n", e);
            }
        }
    }

    Ok(())
}
