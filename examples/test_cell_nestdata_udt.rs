// test_cell_nestdata_udt.rs
// =========================================================================
// Test file for reading complex nested UDT array structures
// Specifically tests: Cell_NestData[90] with nested PartData UDT
//
// This test verifies that the library can read:
// - Array of UDT elements: Cell_NestData[90]
// - Nested UDT members: Cell_NestData[90].PartData
// - Nested array members: Cell_NestData[90].PartData.PlungerInsertion[0]
//
// To run this test:
//   cargo run --example test_cell_nestdata_udt
//
// The library supports these complex paths through TagPath::parse() which
// handles nested structures, arrays, and UDT members correctly.
// =========================================================================

use rust_ethernet_ip::{EipClient, PlcValue, RoutePath};
use std::io;
use std::io::Write;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging with info level by default
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    tracing::info!(
        "╔══════════════════════════════════════════════════════════════════════════════╗"
    );
    tracing::info!(
        "║  Cell_NestData UDT Array Reading Test                                        ║"
    );
    tracing::info!(
        "║  Tests reading: Cell_NestData[90] with nested PartData UDT                   ║"
    );
    tracing::info!(
        "╚══════════════════════════════════════════════════════════════════════════════╝"
    );
    tracing::info!("");

    // Get PLC connection details
    print!("Enter PLC IP address (default: 192.168.0.1): ");
    io::stdout()
        .flush()
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;
    let mut ip = String::new();
    io::stdin()
        .read_line(&mut ip)
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;
    let ip = ip.trim();
    let ip = if ip.is_empty() { "192.168.0.1" } else { ip };

    print!("Enter CPU slot (default: 0 for CompactLogix): ");
    io::stdout()
        .flush()
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;
    let mut slot = String::new();
    io::stdin()
        .read_line(&mut slot)
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;
    let slot: u8 = slot.trim().parse().unwrap_or(0);

    // Create client and connect with route path
    tracing::info!("🔌 Connecting to PLC at {}:44818 (slot {})...", ip, slot);
    let route_path = RoutePath::new().add_slot(slot);
    let mut client = match EipClient::with_route_path(&format!("{}:44818", ip), route_path).await {
        Ok(client) => {
            tracing::info!("✅ Connected successfully!");
            client
        }
        Err(e) => {
            tracing::error!("❌ Connection failed: {}", e);
            return Err(Box::new(e) as Box<dyn std::error::Error>);
        }
    };

    tracing::info!(
        "═══════════════════════════════════════════════════════════════════════════════"
    );
    tracing::info!("TEST 1: Read entire Cell_NestData[90] UDT");
    tracing::info!(
        "═══════════════════════════════════════════════════════════════════════════════"
    );

    let tag_path = "Cell_NestData[90]";
    tracing::info!("📖 Reading: {}", tag_path);

    match client.read_tag(tag_path).await {
        Ok(value) => match value {
            PlcValue::Udt(udt_data) => {
                tracing::info!("✅ Successfully read UDT!");
                tracing::info!("   Symbol ID: {}", udt_data.symbol_id);
                tracing::info!("   Data size: {} bytes", udt_data.data.len());
                tracing::debug!(
                    "   Raw data (first 64 bytes): {:02X?}",
                    &udt_data.data[..udt_data.data.len().min(64)]
                );
            }
            other => {
                tracing::warn!("⚠️  Tag read but returned non-UDT type: {:?}", other);
            }
        },
        Err(e) => {
            tracing::error!("❌ Failed to read {}: {}", tag_path, e);
        }
    }

    tracing::info!(
        "═══════════════════════════════════════════════════════════════════════════════"
    );
    tracing::info!("TEST 2: Read nested UDT member - Cell_NestData[90].PartData");
    tracing::info!(
        "═══════════════════════════════════════════════════════════════════════════════"
    );

    let tag_path = "Cell_NestData[90].PartData";
    tracing::info!("📖 Reading: {}", tag_path);

    match client.read_tag(tag_path).await {
        Ok(value) => match value {
            PlcValue::Udt(udt_data) => {
                tracing::info!("✅ Successfully read nested UDT PartData!");
                tracing::info!("   Symbol ID: {}", udt_data.symbol_id);
                tracing::info!("   Data size: {} bytes", udt_data.data.len());
            }
            other => {
                tracing::warn!("⚠️  Tag read but returned non-UDT type: {:?}", other);
            }
        },
        Err(e) => {
            tracing::error!("❌ Failed to read {}: {}", tag_path, e);
        }
    }

    tracing::info!(
        "═══════════════════════════════════════════════════════════════════════════════"
    );
    tracing::info!("TEST 3: Read individual PartData members");
    tracing::info!(
        "═══════════════════════════════════════════════════════════════════════════════"
    );

    let members = vec![
        "Cell_NestData[90].PartData.Temp_PreHeatZone1",
        "Cell_NestData[90].PartData.Temp_PreHeatZone2",
        "Cell_NestData[90].PartData.Time_PreHeat",
        "Cell_NestData[90].PartData.Temp_HeatZone1",
        "Cell_NestData[90].PartData.Temp_HeatZone2",
        "Cell_NestData[90].PartData.Time_Heat",
        "Cell_NestData[90].PartData.Time_Cooling",
    ];

    for member in &members {
        tracing::info!("📖 Reading: {}", member);
        match client.read_tag(member).await {
            Ok(value) => {
                tracing::info!("   ✅ Value: {:?}", value);
            }
            Err(e) => {
                tracing::error!("   ❌ Error: {}", e);
            }
        }
    }

    tracing::info!(
        "═══════════════════════════════════════════════════════════════════════════════"
    );
    tracing::info!("TEST 4: Read nested array member - PlungerInsertion[0-3]");
    tracing::info!(
        "═══════════════════════════════════════════════════════════════════════════════"
    );

    for i in 0..4 {
        let tag_path = format!("Cell_NestData[90].PartData.PlungerInsertion[{}]", i);
        tracing::info!("📖 Reading: {}", tag_path);
        match client.read_tag(&tag_path).await {
            Ok(value) => {
                tracing::info!("   ✅ Value: {:?}", value);
            }
            Err(e) => {
                tracing::error!("   ❌ Error: {}", e);
            }
        }
    }

    tracing::info!(
        "═══════════════════════════════════════════════════════════════════════════════"
    );
    tracing::info!("TEST 5: Read other PartData members");
    tracing::info!(
        "═══════════════════════════════════════════════════════════════════════════════"
    );

    let other_members = vec![
        "Cell_NestData[90].PartData.Vision_AngleBody_1",
        "Cell_NestData[90].PartData.Vision_PlungerDist",
        "Cell_NestData[90].PartData.Vision_CapPres",
        "Cell_NestData[90].PartData.Vision_AngleBody_2",
        "Cell_NestData[90].PartData.Vision_PlungerDist_2",
        "Cell_NestData[90].PartData.Time_FillDecTime",
        "Cell_NestData[90].PartData.Weigh_BodyPlunger",
        "Cell_NestData[90].PartData.Weigh_Cap",
        "Cell_NestData[90].PartData.Weigh_Final",
    ];

    for member in &other_members {
        tracing::info!("📖 Reading: {}", member);
        match client.read_tag(member).await {
            Ok(value) => {
                tracing::info!("   ✅ Value: {:?}", value);
            }
            Err(e) => {
                tracing::error!("   ❌ Error: {}", e);
            }
        }
    }

    tracing::info!(
        "═══════════════════════════════════════════════════════════════════════════════"
    );
    tracing::info!("TEST 6: Read top-level Cell_NestData[90] members");
    tracing::info!(
        "═══════════════════════════════════════════════════════════════════════════════"
    );

    let top_level_members = vec![
        "Cell_NestData[90].ModelNumber",
        "Cell_NestData[90].SerialNumber",
        "Cell_NestData[90].LotNo",
        "Cell_NestData[90].LastStationWorked",
        "Cell_NestData[90].PartStatus",
        "Cell_NestData[90].StationPartStatus",
        "Cell_NestData[90].IndexPositionNestNumber",
        "Cell_NestData[90].ReworkActive",
        "Cell_NestData[90].MasterPart",
    ];

    for member in &top_level_members {
        tracing::info!("📖 Reading: {}", member);
        match client.read_tag(member).await {
            Ok(value) => {
                tracing::info!("   ✅ Value: {:?}", value);
            }
            Err(e) => {
                tracing::error!("   ❌ Error: {}", e);
            }
        }
    }

    tracing::info!(
        "═══════════════════════════════════════════════════════════════════════════════"
    );
    tracing::info!("TEST 7: Verify TagPath parsing for complex paths");
    tracing::info!(
        "═══════════════════════════════════════════════════════════════════════════════"
    );

    use rust_ethernet_ip::TagPath;

    let test_paths = vec![
        "Cell_NestData[90]",
        "Cell_NestData[90].PartData",
        "Cell_NestData[90].PartData.PlungerInsertion[0]",
        "Cell_NestData[90].PartData.Temp_PreHeatZone1",
        "Cell_NestData[90].ModelNumber",
        "Cell_NestData[90].SerialNumber",
        "Cell_NestData[90].LotNo",
    ];

    for path_str in &test_paths {
        match TagPath::parse(path_str) {
            Ok(path) => {
                tracing::info!("✅ Parsed: {} -> {:?}", path_str, path);
                match path.to_cip_path() {
                    Ok(cip_path) => {
                        tracing::info!(
                            "   CIP Path ({} bytes, {} words): {:02X?}",
                            cip_path.len(),
                            cip_path.len() / 2,
                            cip_path
                        );
                    }
                    Err(e) => {
                        tracing::error!("   ❌ Failed to generate CIP path: {}", e);
                    }
                }
            }
            Err(e) => {
                tracing::error!("❌ Failed to parse: {} - {}", path_str, e);
            }
        }
    }

    // Disconnect (client will automatically disconnect when dropped)
    tracing::info!("🔌 Disconnecting...");
    drop(client);
    tracing::info!("✅ Disconnected");

    tracing::info!(
        "═══════════════════════════════════════════════════════════════════════════════"
    );
    tracing::info!("Test completed!");
    tracing::info!(
        "═══════════════════════════════════════════════════════════════════════════════"
    );

    Ok(())
}
