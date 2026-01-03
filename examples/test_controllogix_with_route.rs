/// Test ControlLogix connection with route path
/// 
/// This example demonstrates how to connect to a ControlLogix PLC
/// with proper backplane routing. For ControlLogix, you must specify
/// the CPU slot number in the route path.
/// 
/// Setup:
/// - ControlLogix CPU in Slot 0
/// - Ethernet module (1756-EN2T) in Slot 1
/// - Connect to Ethernet module IP: 192.168.0.1
/// 
/// Route Path: Port 1 (backplane), Slot 0 (CPU location)
/// Format: [0x01, 0x00]

use rust_ethernet_ip::{EipClient, RoutePath};

const PLC_ADDRESS: &str = "192.168.0.1:44818";
const CPU_SLOT: u8 = 0; // CPU is in Slot 0

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔌 Connecting to ControlLogix PLC at {}...", PLC_ADDRESS);
    println!("   CPU Slot: {}", CPU_SLOT);
    println!("   Route Path: Port 1 (backplane), Slot {} (CPU)", CPU_SLOT);
    
    // Create route path for ControlLogix
    // Reference: EtherNetIP_Connection_Paths_and_Routing.md
    // Port 1 = Backplane, Slot 0 = CPU location
    let route_path = RoutePath::new().add_slot(CPU_SLOT);
    
    println!("   Route path bytes: {:02X?}", route_path.to_cip_bytes());
    
    // Connect with route path
    let mut client = EipClient::with_route_path(PLC_ADDRESS, route_path).await?;
    println!("✅ Connected successfully!\n");

    // Test reading tags
    let test_tags = vec![
        "gTestArray_DINT[5]",
        "gTestArray_DINT",
        "gTestUDT",
    ];

    println!("📋 Testing tag reads with route path:\n");
    for tag_name in test_tags {
        print!("   Reading: '{}' ... ", tag_name);
        match client.read_tag(tag_name).await {
            Ok(value) => {
                println!("✅ Success: {:?}", value);
            }
            Err(e) => {
                println!("❌ Failed: {}", e);
            }
        }
    }

    Ok(())
}

