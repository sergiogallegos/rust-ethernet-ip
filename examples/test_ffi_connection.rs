use rust_ethernet_ip::EipClient;
use rust_ethernet_ip::PlcValue;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get IP from command line or use default
    let ip = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "192.168.0.1".to_string());

    println!("🔌 Connecting to PLC at {}...", ip);

    let mut client = match EipClient::new(&ip).await {
        Ok(c) => {
            println!("✅ Connected successfully!");
            c
        }
        Err(e) => {
            println!("❌ Connection failed: {}", e);
            return Err(e.into());
        }
    };

    // Test reading a simple tag
    println!("\n📖 Testing read operations...");

    let test_tags = vec!["gDINTTest", "gBoolTest", "gRealTest", "gStringTest"];

    for tag in &test_tags {
        println!("\n  Testing tag: {}", tag);
        match client.read_tag(tag).await {
            Ok(value) => {
                println!("  ✅ Read successful: {:?}", value);
            }
            Err(e) => {
                println!("  ❌ Read failed: {}", e);
            }
        }
    }

    // Test writing
    println!("\n📝 Testing write operations...");

    match client.write_tag("gDINTTest", PlcValue::Dint(12345)).await {
        Ok(_) => {
            println!("  ✅ Write successful");

            // Verify by reading back
            match client.read_tag("gDINTTest").await {
                Ok(value) => {
                    println!("  ✅ Read-back successful: {:?}", value);
                    if let PlcValue::Dint(v) = value {
                        if v == 12345 {
                            println!("  ✅ Value matches!");
                        } else {
                            println!("  ⚠️ Value mismatch: expected 12345, got {}", v);
                        }
                    }
                }
                Err(e) => {
                    println!("  ❌ Read-back failed: {}", e);
                }
            }
        }
        Err(e) => {
            println!("  ❌ Write failed: {}", e);
        }
    }

    println!("\n✅ Test completed!");
    Ok(())
}
