//! Comprehensive Terminal-Based Demo for Rust EtherNet/IP Library
//!
//! This example demonstrates ALL features of the library in a terminal-based interface.
//! Perfect for testing and understanding all library capabilities.
//!
//! Usage:
//!   cargo run --example comprehensive_terminal_demo -- <PLC_ADDRESS>
//!
//! Example:
//!   cargo run --example comprehensive_terminal_demo -- 192.168.1.100:44818

use rust_ethernet_ip::udt::UserDefinedType;
use rust_ethernet_ip::{BatchOperation, EipClient, PlcValue, RoutePath};
use std::io::{self, Write};
use std::time::Instant;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("╔════════════════════════════════════════════════════════════════╗");
    println!("║  🦀 Rust EtherNet/IP - Comprehensive Terminal Demo v0.6.1      ║");
    println!("║  Complete Feature Testing Interface                            ║");
    println!("╚════════════════════════════════════════════════════════════════╝");
    println!();

    // Get PLC address from command line or use default
    let plc_address = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "192.168.0.1:44818".to_string());

    println!("📍 PLC Address: {}", plc_address);
    println!();

    // Ask for RoutePath configuration
    let (mut client, use_route_path) = setup_connection(&plc_address).await?;

    if !client.check_health().await {
        println!("❌ Health check failed!");
        return Ok(());
    }

    println!("✅ Connection established and healthy!");
    println!();

    // Main menu loop
    loop {
        print_menu();
        print!("Select option: ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let choice = input.trim();

        match choice {
            "1" => test_connection_management(&mut client, use_route_path).await?,
            "2" => test_tag_discovery(&mut client).await?,
            "3" => test_individual_operations(&mut client).await?,
            "4" => test_array_operations(&mut client).await?,
            "5" => test_udt_operations(&mut client).await?,
            "6" => test_string_operations(&mut client).await?,
            "7" => test_batch_operations(&mut client).await?,
            "8" => test_advanced_addressing(&mut client).await?,
            "9" => test_program_tags(&mut client).await?,
            "10" => test_cache_management(&mut client).await?,
            "11" => test_performance(&mut client).await?,
            "12" => test_health_monitoring(&mut client).await?,
            "0" => {
                println!("👋 Disconnecting...");
                client.unregister_session().await?;
                println!("✅ Disconnected. Goodbye!");
                break;
            }
            _ => println!("❌ Invalid option. Please try again."),
        }

        println!();
        println!("Press Enter to continue...");
        let mut _buffer = String::new();
        io::stdin().read_line(&mut _buffer)?;
    }

    Ok(())
}

fn print_menu() {
    println!("╔════════════════════════════════════════════════════════════════╗");
    println!("║                        MAIN MENU                               ║");
    println!("╠════════════════════════════════════════════════════════════════╣");
    println!("║  1.  Connection Management & RoutePath                        ║");
    println!("║  2.  Tag Discovery (Basic, Detailed, Program-scoped)        ║");
    println!("║  3.  Individual Tag Operations (All Data Types)               ║");
    println!("║  4.  Array Operations (Elements, Multi-dimensional)           ║");
    println!("║  5.  UDT Operations (Read, Write, Members, Discovery)        ║");
    println!("║  6.  STRING Operations (Read, Write, Components)              ║");
    println!("║  7.  Batch Operations (Read, Write, Mixed)                     ║");
    println!("║  8.  Advanced Tag Addressing (Bits, Nested, Complex)         ║");
    println!("║  9.  Program-Scoped Tags                                      ║");
    println!("║  10. Cache Management                                         ║");
    println!("║  11. Performance Testing                                      ║");
    println!("║  12. Health Monitoring                                        ║");
    println!("║  0.  Exit                                                     ║");
    println!("╚════════════════════════════════════════════════════════════════╝");
}

async fn setup_connection(
    addr: &str,
) -> Result<(EipClient, bool), Box<dyn std::error::Error + Send + Sync>> {
    print!("Use RoutePath for ControlLogix? (y/n, default=n): ");
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let use_route_path = input.trim().to_lowercase() == "y";

    let client = if use_route_path {
        print!("Enter CPU slot (0-31, default=0): ");
        io::stdout().flush()?;
        let mut slot_input = String::new();
        io::stdin().read_line(&mut slot_input)?;
        let slot = slot_input.trim().parse::<u8>().unwrap_or(0);

        let route = RoutePath::new().add_slot(slot);
        println!("🔌 Connecting with RoutePath (slot {})...", slot);
        EipClient::with_route_path(addr, route).await?
    } else {
        println!("🔌 Connecting to CompactLogix...");
        EipClient::connect(addr).await?
    };

    Ok((client, use_route_path))
}

async fn test_connection_management(
    client: &mut EipClient,
    use_route_path: bool,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("\n╔════════════════════════════════════════════════════════════════╗");
    println!("║          1. CONNECTION MANAGEMENT & ROUTEPATH                  ║");
    println!("╚════════════════════════════════════════════════════════════════╝");
    println!();

    println!("📊 Connection Status:");
    println!("  RoutePath Enabled: {}", use_route_path);
    if let Some(route) = client.get_route_path() {
        println!("  Route Path: {:?}", route);
    } else {
        println!("  Route Path: Direct connection (CompactLogix)");
    }

    println!("\n🔍 Health Check:");
    let health = client.check_health().await;
    println!(
        "  Status: {}",
        if health {
            "✅ Healthy"
        } else {
            "❌ Unhealthy"
        }
    );

    if let Ok(detailed) = client.check_health_detailed().await {
        println!(
            "  Detailed: {}",
            if detailed { "✅ OK" } else { "❌ Failed" }
        );
    }

    println!("\n⚙️ Configuration:");
    let batch_config = client.get_batch_config();
    println!(
        "  Batch Config: {} ops/packet, {} bytes max",
        batch_config.max_operations_per_packet, batch_config.max_packet_size
    );

    Ok(())
}

async fn test_tag_discovery(
    client: &mut EipClient,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("\n╔════════════════════════════════════════════════════════════════╗");
    println!("║          2. TAG DISCOVERY                                      ║");
    println!("╚════════════════════════════════════════════════════════════════╝");
    println!();

    println!("🔍 Basic Tag Discovery...");
    match client.discover_tags().await {
        Ok(_) => {
            println!("✅ Basic discovery completed");
            let cached = client.list_cached_tag_attributes().await;
            println!("  Cached tags: {}", cached.len());
            if !cached.is_empty() {
                println!("  Sample tags:");
                for tag in cached.iter().take(10) {
                    println!("    - {}", tag);
                }
            }
        }
        Err(e) => println!(
            "⚠️ Basic discovery failed: {} (some PLCs don't support this)",
            e
        ),
    }

    println!("\n🔍 Detailed Tag Discovery...");
    match client.discover_tags_detailed().await {
        Ok(tags) => {
            println!("✅ Found {} tags with full attributes", tags.len());
            if !tags.is_empty() {
                println!("\n  Sample tag details:");
                for tag in tags.iter().take(5) {
                    println!("    Tag: {}", tag.name);
                    println!(
                        "      Type: {} ({})",
                        tag.data_type_name,
                        format!("0x{:04X}", tag.data_type)
                    );
                    println!("      Size: {} bytes", tag.size);
                    println!("      Scope: {:?}", tag.scope);
                    println!("      Permissions: {:?}", tag.permissions);
                    if let Some(template_id) = tag.template_instance_id {
                        println!("      Template ID: {}", template_id);
                    }
                    println!();
                }
            }
        }
        Err(e) => println!(
            "⚠️ Detailed discovery failed: {} (some PLCs don't support this)",
            e
        ),
    }

    print!("\n🔍 Program Tag Discovery - Enter program name (or Enter to skip): ");
    io::stdout().flush()?;
    let mut program_input = String::new();
    io::stdin().read_line(&mut program_input)?;
    let program_name = program_input.trim();

    if !program_name.is_empty() {
        match client.discover_program_tags(program_name).await {
            Ok(tags) => {
                println!("✅ Found {} tags in program '{}'", tags.len(), program_name);
                for tag in tags.iter().take(10) {
                    println!("    - {} ({})", tag.name, tag.data_type_name);
                }
            }
            Err(e) => println!("⚠️ Program tag discovery failed: {}", e),
        }
    }

    Ok(())
}

async fn test_individual_operations(
    client: &mut EipClient,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("\n╔════════════════════════════════════════════════════════════════╗");
    println!("║          3. INDIVIDUAL TAG OPERATIONS                            ║");
    println!("╚════════════════════════════════════════════════════════════════╝");
    println!();

    print!("Enter tag name to test: ");
    io::stdout().flush()?;
    let mut tag_input = String::new();
    io::stdin().read_line(&mut tag_input)?;
    let tag_name = tag_input.trim();

    if tag_name.is_empty() {
        println!("⚠️ No tag name provided, skipping...");
        return Ok(());
    }

    // Read tag
    println!("\n📖 Reading tag '{}'...", tag_name);
    let start = Instant::now();
    match client.read_tag(tag_name).await {
        Ok(value) => {
            let duration = start.elapsed();
            println!("✅ Read successful in {:?}", duration);
            println!("  Value: {:?}", value);
            println!("  Data Type: 0x{:04X}", value.get_data_type());

            // Get metadata
            if let Some(metadata) = client.get_tag_metadata(tag_name).await {
                println!("  Metadata:");
                println!("    Type: 0x{:04X}", metadata.data_type);
                println!("    Size: {} bytes", metadata.size);
                println!("    Is Array: {}", metadata.is_array);
                if let Some(array_info) = metadata.array_info {
                    println!("    Array Dimensions: {:?}", array_info.dimensions);
                }
            }

            // Try to write (if writable)
            print!("\n✍️ Write new value? (Enter value or 'skip'): ");
            io::stdout().flush()?;
            let mut write_input = String::new();
            io::stdin().read_line(&mut write_input)?;
            let write_value = write_input.trim();

            if !write_value.is_empty() && write_value != "skip" {
                let new_value = parse_value(&value, write_value)?;
                println!("Writing {:?}...", new_value);
                let write_start = Instant::now();
                match client.write_tag(tag_name, new_value).await {
                    Ok(_) => {
                        let write_duration = write_start.elapsed();
                        println!("✅ Write successful in {:?}", write_duration);

                        // Verify by reading back
                        if let Ok(verify_value) = client.read_tag(tag_name).await {
                            println!("  Verified: {:?}", verify_value);
                        }
                    }
                    Err(e) => println!("❌ Write failed: {}", e),
                }
            }
        }
        Err(e) => println!("❌ Read failed: {}", e),
    }

    Ok(())
}

async fn test_array_operations(
    client: &mut EipClient,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("\n╔════════════════════════════════════════════════════════════════╗");
    println!("║          4. ARRAY OPERATIONS                                    ║");
    println!("╚════════════════════════════════════════════════════════════════╝");
    println!();

    print!("Enter array tag name (e.g., 'gTestArray_DINT'): ");
    io::stdout().flush()?;
    let mut array_input = String::new();
    io::stdin().read_line(&mut array_input)?;
    let base_array = array_input.trim();

    if base_array.is_empty() {
        println!("⚠️ No array name provided, skipping...");
        return Ok(());
    }

    print!("Enter index to test (default=0): ");
    io::stdout().flush()?;
    let mut index_input = String::new();
    io::stdin().read_line(&mut index_input)?;
    let index: usize = index_input.trim().parse().unwrap_or(0);

    let array_element = format!("{}[{}]", base_array, index);

    // Read array element
    println!("\n📖 Reading array element '{}'...", array_element);
    let start = Instant::now();
    match client.read_tag(&array_element).await {
        Ok(value) => {
            let duration = start.elapsed();
            println!("✅ Read successful in {:?}", duration);
            println!("  Value: {:?}", value);

            // Try to write
            print!("\n✍️ Write new value? (Enter value or 'skip'): ");
            io::stdout().flush()?;
            let mut write_input = String::new();
            io::stdin().read_line(&mut write_input)?;
            let write_value = write_input.trim();

            if !write_value.is_empty() && write_value != "skip" {
                let new_value = parse_value(&value, write_value)?;
                println!("Writing {:?}...", new_value);
                let write_start = Instant::now();
                match client.write_tag(&array_element, new_value).await {
                    Ok(_) => {
                        let write_duration = write_start.elapsed();
                        println!("✅ Write successful in {:?}", write_duration);
                    }
                    Err(e) => println!("❌ Write failed: {}", e),
                }
            }
        }
        Err(e) => println!("❌ Read failed: {}", e),
    }

    // Test multi-dimensional array
    print!("\n🔍 Test multi-dimensional array? (Enter '2D' tag like 'Array[1,2]' or skip): ");
    io::stdout().flush()?;
    let mut md_input = String::new();
    io::stdin().read_line(&mut md_input)?;
    let md_tag = md_input.trim();

    if !md_tag.is_empty() && md_tag != "skip" {
        println!("Reading '{}'...", md_tag);
        match client.read_tag(md_tag).await {
            Ok(value) => println!("✅ Value: {:?}", value),
            Err(e) => println!("❌ Failed: {}", e),
        }
    }

    Ok(())
}

async fn test_udt_operations(
    client: &mut EipClient,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("\n╔════════════════════════════════════════════════════════════════╗");
    println!("║          5. UDT OPERATIONS                                     ║");
    println!("╚════════════════════════════════════════════════════════════════╝");
    println!();

    print!("Enter UDT tag name (e.g., 'gTestUDT'): ");
    io::stdout().flush()?;
    let mut udt_input = String::new();
    io::stdin().read_line(&mut udt_input)?;
    let udt_name = udt_input.trim();

    if udt_name.is_empty() {
        println!("⚠️ No UDT name provided, skipping...");
        return Ok(());
    }

    // Discover UDT definition
    println!("\n🔍 Discovering UDT definition...");
    match client.get_udt_definition(udt_name).await {
        Ok(definition) => {
            println!("✅ UDT Definition found:");
            println!("  Name: {}", definition.name);
            println!("  Members: {}", definition.members.len());
            for member in &definition.members {
                println!(
                    "    - {}: Type=0x{:04X}, Offset={}, Size={}",
                    member.name, member.data_type, member.offset, member.size
                );
            }

            // Convert to UserDefinedType for parsing
            let mut user_def = UserDefinedType::new(definition.name.clone());
            for member in &definition.members {
                user_def.add_member(member.clone());
            }

            // Read UDT
            println!("\n📖 Reading UDT '{}'...", udt_name);
            let start = Instant::now();
            match client.read_tag(udt_name).await {
                Ok(PlcValue::Udt(udt_data)) => {
                    let duration = start.elapsed();
                    println!("✅ Read successful in {:?}", duration);
                    println!("  Symbol ID: {}", udt_data.symbol_id);
                    println!("  Data Size: {} bytes", udt_data.data.len());

                    // Parse UDT
                    match udt_data.parse(&user_def) {
                        Ok(members) => {
                            println!("  Parsed Members:");
                            for (name, value) in &members {
                                println!("    {}: {:?}", name, value);
                            }

                            // Test member read by name
                            print!("\n🔍 Read specific member? (Enter member name or 'skip'): ");
                            io::stdout().flush()?;
                            let mut member_input = String::new();
                            io::stdin().read_line(&mut member_input)?;
                            let member_name = member_input.trim();

                            if !member_name.is_empty() && member_name != "skip" {
                                let member_path = format!("{}.{}", udt_name, member_name);
                                match client.read_tag(&member_path).await {
                                    Ok(value) => println!("✅ {} = {:?}", member_path, value),
                                    Err(e) => println!("❌ Failed: {}", e),
                                }
                            }
                        }
                        Err(e) => println!("⚠️ Parse failed: {}", e),
                    }
                }
                Ok(other) => println!("⚠️ Expected UDT, got: {:?}", other),
                Err(e) => println!("❌ Read failed: {}", e),
            }

            // Test chunked read for large UDTs
            println!("\n📦 Testing chunked read...");
            match client.read_udt_chunked(udt_name).await {
                Ok(PlcValue::Udt(udt_data)) => {
                    println!("✅ Chunked read successful");
                    println!("  Symbol ID: {}", udt_data.symbol_id);
                    println!("  Data Size: {} bytes", udt_data.data.len());
                }
                Ok(other) => println!("⚠️ Expected UDT, got: {:?}", other),
                Err(e) => println!("⚠️ Chunked read failed: {}", e),
            }
        }
        Err(e) => println!("❌ UDT definition discovery failed: {}", e),
    }

    Ok(())
}

async fn test_string_operations(
    client: &mut EipClient,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("\n╔════════════════════════════════════════════════════════════════╗");
    println!("║          6. STRING OPERATIONS                                  ║");
    println!("╚════════════════════════════════════════════════════════════════╝");
    println!();

    print!("Enter STRING tag name (e.g., 'gTest_STRING'): ");
    io::stdout().flush()?;
    let mut string_input = String::new();
    io::stdin().read_line(&mut string_input)?;
    let string_tag = string_input.trim();

    if string_tag.is_empty() {
        println!("⚠️ No STRING tag name provided, skipping...");
        return Ok(());
    }

    // Read STRING
    println!("\n📖 Reading STRING tag '{}'...", string_tag);
    let start = Instant::now();
    match client.read_tag(string_tag).await {
        Ok(PlcValue::String(value)) => {
            let duration = start.elapsed();
            println!("✅ Read successful in {:?}", duration);
            println!("  Value: '{}'", value);
            println!("  Length: {} characters", value.len());

            // Test STRING components
            let len_tag = format!("{}.LEN", string_tag);
            println!("\n📖 Reading STRING length '{}'...", len_tag);
            match client.read_tag(&len_tag).await {
                Ok(value) => println!("✅ Length: {:?}", value),
                Err(e) => println!("⚠️ Length read failed: {}", e),
            }

            // Try to write STRING
            print!("\n✍️ Write new STRING value? (Enter value or 'skip'): ");
            io::stdout().flush()?;
            let mut write_input = String::new();
            io::stdin().read_line(&mut write_input)?;
            let write_value = write_input.trim();

            if !write_value.is_empty() && write_value != "skip" {
                println!("Writing '{}'...", write_value);
                let write_start = Instant::now();
                match client.write_string(string_tag, write_value).await {
                    Ok(_) => {
                        let write_duration = write_start.elapsed();
                        println!("✅ Write successful in {:?}", write_duration);

                        // Verify
                        if let Ok(PlcValue::String(verify)) = client.read_tag(string_tag).await {
                            println!("  Verified: '{}'", verify);
                        }
                    }
                    Err(e) => println!(
                        "❌ Write failed: {} (Note: Some PLCs restrict STRING writes)",
                        e
                    ),
                }
            }
        }
        Ok(other) => println!("⚠️ Expected STRING, got: {:?}", other),
        Err(e) => println!("❌ Read failed: {}", e),
    }

    Ok(())
}

async fn test_batch_operations(
    client: &mut EipClient,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("\n╔════════════════════════════════════════════════════════════════╗");
    println!("║          7. BATCH OPERATIONS                                  ║");
    println!("╚════════════════════════════════════════════════════════════════╝");
    println!();

    // Batch read
    println!("📖 Batch Read Test");
    let test_tags = vec![
        "gTestArray_DINT[0]",
        "gTestArray_DINT[5]",
        "gTestArray_REAL[0]",
        "gTestArray_BOOL[0]",
        "gTestArray_INT[0]",
    ];
    println!("  Reading {} tags...", test_tags.len());

    let start = Instant::now();
    match client.read_tags_batch(&test_tags).await {
        Ok(results) => {
            let duration = start.elapsed();
            println!("✅ Batch read completed in {:?}", duration);
            println!(
                "  Throughput: {:.1} ops/sec",
                test_tags.len() as f64 / duration.as_secs_f64()
            );
            for (tag, result) in &results {
                match result {
                    Ok(value) => println!("    ✅ {}: {:?}", tag, value),
                    Err(e) => println!("    ❌ {}: {}", tag, e),
                }
            }
        }
        Err(e) => println!("❌ Batch read failed: {}", e),
    }

    // Batch write
    println!("\n✍️ Batch Write Test");
    let write_tags = vec![
        ("gTestArray_DINT[5]", PlcValue::Dint(999)),
        ("gTestArray_REAL[0]", PlcValue::Real(88.8)),
        ("gTestArray_BOOL[0]", PlcValue::Bool(true)),
        ("gTestArray_INT[0]", PlcValue::Int(777)),
    ];
    println!("  Writing {} tags...", write_tags.len());

    let start = Instant::now();
    match client.write_tags_batch(&write_tags).await {
        Ok(results) => {
            let duration = start.elapsed();
            println!("✅ Batch write completed in {:?}", duration);
            println!(
                "  Throughput: {:.1} ops/sec",
                write_tags.len() as f64 / duration.as_secs_f64()
            );
            for (tag, result) in results {
                match result {
                    Ok(_) => println!("    ✅ {}: Success", tag),
                    Err(e) => println!("    ❌ {}: {}", tag, e),
                }
            }
        }
        Err(e) => println!("❌ Batch write failed: {}", e),
    }

    // Mixed batch
    println!("\n🔄 Mixed Batch Operations Test");
    let operations = vec![
        BatchOperation::Read {
            tag_name: "gTestArray_DINT[0]".to_string(),
        },
        BatchOperation::Read {
            tag_name: "gTestArray_BOOL[0]".to_string(),
        },
        BatchOperation::Write {
            tag_name: "gTestArray_DINT[5]".to_string(),
            value: PlcValue::Dint(999),
        },
        BatchOperation::Read {
            tag_name: "gTestArray_DINT[5]".to_string(),
        },
    ];

    let start = Instant::now();
    match client.execute_batch(&operations).await {
        Ok(results) => {
            let duration = start.elapsed();
            println!("✅ Mixed batch completed in {:?}", duration);
            for result in results {
                match result.operation {
                    BatchOperation::Read { tag_name } => match result.result {
                        Ok(Some(value)) => println!(
                            "    ✅ Read {}: {:?} ({:?}μs)",
                            tag_name, value, result.execution_time_us
                        ),
                        Err(e) => println!(
                            "    ❌ Read {}: {} ({:?}μs)",
                            tag_name, e, result.execution_time_us
                        ),
                        _ => {}
                    },
                    BatchOperation::Write { tag_name, .. } => match result.result {
                        Ok(_) => println!(
                            "    ✅ Write {}: Success ({:?}μs)",
                            tag_name, result.execution_time_us
                        ),
                        Err(e) => println!(
                            "    ❌ Write {}: {} ({:?}μs)",
                            tag_name, e, result.execution_time_us
                        ),
                    },
                }
            }
        }
        Err(e) => println!("❌ Mixed batch failed: {}", e),
    }

    Ok(())
}

async fn test_advanced_addressing(
    client: &mut EipClient,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("\n╔════════════════════════════════════════════════════════════════╗");
    println!("║          8. ADVANCED TAG ADDRESSING                            ║");
    println!("╚════════════════════════════════════════════════════════════════╝");
    println!();

    // Bit access
    print!("Test bit access? (Enter tag like 'StatusWord.15' or 'skip'): ");
    io::stdout().flush()?;
    let mut bit_input = String::new();
    io::stdin().read_line(&mut bit_input)?;
    let bit_tag = bit_input.trim();

    if !bit_tag.is_empty() && bit_tag != "skip" {
        println!("Reading '{}'...", bit_tag);
        match client.read_tag(bit_tag).await {
            Ok(value) => println!("✅ Value: {:?}", value),
            Err(e) => println!("❌ Failed: {}", e),
        }
    }

    // UDT member access
    print!("\nTest UDT member? (Enter path like 'gTestUDT.Member1_DINT' or 'skip'): ");
    io::stdout().flush()?;
    let mut member_input = String::new();
    io::stdin().read_line(&mut member_input)?;
    let member_path = member_input.trim();

    if !member_path.is_empty() && member_path != "skip" {
        println!("Reading '{}'...", member_path);
        match client.read_tag(member_path).await {
            Ok(value) => {
                println!("✅ Value: {:?}", value);

                // Try to write
                print!("Write new value? (Enter value or 'skip'): ");
                io::stdout().flush()?;
                let mut write_input = String::new();
                io::stdin().read_line(&mut write_input)?;
                let write_value = write_input.trim();

                if !write_value.is_empty() && write_value != "skip" {
                    let new_value = parse_value(&value, write_value)?;
                    match client.write_tag(member_path, new_value).await {
                        Ok(_) => println!("✅ Write successful"),
                        Err(e) => println!("❌ Write failed: {}", e),
                    }
                }
            }
            Err(e) => println!("❌ Failed: {}", e),
        }
    }

    // Complex nested path
    print!("\nTest complex path? (Enter path like 'Program:TestProgram.gTestArray_DINT[5]' or 'gTestUDT.Array_DINT[5]' or 'skip'): ");
    io::stdout().flush()?;
    let mut complex_input = String::new();
    io::stdin().read_line(&mut complex_input)?;
    let complex_path = complex_input.trim();

    if !complex_path.is_empty() && complex_path != "skip" {
        println!("Reading '{}'...", complex_path);
        match client.read_tag(complex_path).await {
            Ok(value) => println!("✅ Value: {:?}", value),
            Err(e) => println!("❌ Failed: {}", e),
        }
    }

    Ok(())
}

async fn test_program_tags(
    client: &mut EipClient,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("\n╔════════════════════════════════════════════════════════════════╗");
    println!("║          9. PROGRAM-SCOPED TAGS                                ║");
    println!("╚════════════════════════════════════════════════════════════════╝");
    println!();

    print!("Enter program name (e.g., 'TestProgram'): ");
    io::stdout().flush()?;
    let mut program_input = String::new();
    io::stdin().read_line(&mut program_input)?;
    let program_name = program_input.trim();

    if program_name.is_empty() {
        println!("⚠️ No program name provided, skipping...");
        return Ok(());
    }

    // Discover program tags
    println!("🔍 Discovering tags in program '{}'...", program_name);
    match client.discover_program_tags(program_name).await {
        Ok(tags) => {
            println!("✅ Found {} tags", tags.len());
            for tag in tags.iter().take(10) {
                println!("  - {} ({})", tag.name, tag.data_type_name);
            }

            // Try to read a program tag
            print!("\n📖 Read program tag? (Enter tag name or 'skip'): ");
            io::stdout().flush()?;
            let mut tag_input = String::new();
            io::stdin().read_line(&mut tag_input)?;
            let tag_name = tag_input.trim();

            if !tag_name.is_empty() && tag_name != "skip" {
                let program_tag = format!("Program:{}.{}", program_name, tag_name);
                println!("Reading '{}'...", program_tag);
                match client.read_tag(&program_tag).await {
                    Ok(value) => {
                        println!("✅ Value: {:?}", value);

                        // Try to write
                        print!("Write new value? (Enter value or 'skip'): ");
                        io::stdout().flush()?;
                        let mut write_input = String::new();
                        io::stdin().read_line(&mut write_input)?;
                        let write_value = write_input.trim();

                        if !write_value.is_empty() && write_value != "skip" {
                            let new_value = parse_value(&value, write_value)?;
                            match client.write_tag(&program_tag, new_value).await {
                                Ok(_) => println!("✅ Write successful"),
                                Err(e) => println!("❌ Write failed: {}", e),
                            }
                        }
                    }
                    Err(e) => println!("❌ Failed: {}", e),
                }
            }
        }
        Err(e) => println!("❌ Program tag discovery failed: {}", e),
    }

    Ok(())
}

async fn test_cache_management(
    client: &mut EipClient,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("\n╔════════════════════════════════════════════════════════════════╗");
    println!("║          10. CACHE MANAGEMENT                                   ║");
    println!("╚════════════════════════════════════════════════════════════════╝");
    println!();

    println!("📊 Current Cache Status:");
    let cached_tags = client.list_cached_tag_attributes().await;
    println!("  Cached Tags: {}", cached_tags.len());

    let udt_definitions = client.list_udt_definitions().await;
    println!("  Cached UDT Definitions: {}", udt_definitions.len());
    for udt in &udt_definitions {
        println!("    - {}", udt);
    }

    print!("\n🗑️ Clear all caches? (y/n): ");
    io::stdout().flush()?;
    let mut clear_input = String::new();
    io::stdin().read_line(&mut clear_input)?;
    if clear_input.trim().to_lowercase() == "y" {
        client.clear_caches().await;
        println!("✅ Caches cleared");
    }

    Ok(())
}

async fn test_performance(
    client: &mut EipClient,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("\n╔════════════════════════════════════════════════════════════════╗");
    println!("║          11. PERFORMANCE TESTING                                ║");
    println!("╚════════════════════════════════════════════════════════════════╝");
    println!();

    print!("Enter test tag name (default='gTestArray_DINT[0]'): ");
    io::stdout().flush()?;
    let mut tag_input = String::new();
    io::stdin().read_line(&mut tag_input)?;
    let test_tag = if tag_input.trim().is_empty() {
        "gTestArray_DINT[0]"
    } else {
        tag_input.trim()
    };

    print!("Number of operations (default=100): ");
    io::stdout().flush()?;
    let mut ops_input = String::new();
    io::stdin().read_line(&mut ops_input)?;
    let num_ops: usize = ops_input.trim().parse().unwrap_or(100);

    // Individual operations
    println!("\n⏱️ Individual Operations Test ({})...", num_ops);
    let start = Instant::now();
    let mut success = 0;
    for _ in 0..num_ops {
        if client.read_tag(test_tag).await.is_ok() {
            success += 1;
        }
    }
    let duration = start.elapsed();
    let ops_per_sec = success as f64 / duration.as_secs_f64();
    println!("✅ Completed: {} successful in {:?}", success, duration);
    println!("  Throughput: {:.1} ops/sec", ops_per_sec);
    println!(
        "  Average latency: {:.2} ms",
        duration.as_millis() as f64 / success as f64
    );

    // Batch operations comparison
    println!("\n⏱️ Batch Operations Test ({} tags)...", num_ops);
    // Use different array indices for batch test
    let batch_tags: Vec<String> = (0..num_ops)
        .map(|i| format!("gTestArray_DINT[{}]", i))
        .collect();
    let start = Instant::now();
    let batch_tags_str: Vec<&str> = batch_tags.iter().map(|s| s.as_str()).collect();
    match client.read_tags_batch(&batch_tags_str).await {
        Ok(results) => {
            let duration = start.elapsed();
            let success_count = results.iter().filter(|(_, r)| r.is_ok()).count();
            let ops_per_sec = success_count as f64 / duration.as_secs_f64();
            println!(
                "✅ Completed: {} successful in {:?}",
                success_count, duration
            );
            println!("  Throughput: {:.1} ops/sec", ops_per_sec);
            if success_count > 0 {
                println!(
                    "  Average latency: {:.2} ms",
                    duration.as_millis() as f64 / success_count as f64
                );
            }
            let individual_ops_per_sec = success as f64 / duration.as_secs_f64();
            if individual_ops_per_sec > 0.0 {
                println!(
                    "  Speedup: {:.1}x faster than individual",
                    ops_per_sec / individual_ops_per_sec
                );
            }
        }
        Err(e) => println!("❌ Batch test failed: {}", e),
    }

    Ok(())
}

async fn test_health_monitoring(
    client: &mut EipClient,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("\n╔════════════════════════════════════════════════════════════════╗");
    println!("║          12. HEALTH MONITORING                                 ║");
    println!("╚════════════════════════════════════════════════════════════════╝");
    println!();

    println!("🔍 Basic Health Check:");
    let health = client.check_health().await;
    println!(
        "  Status: {}",
        if health {
            "✅ Healthy"
        } else {
            "❌ Unhealthy"
        }
    );

    println!("\n🔍 Detailed Health Check:");
    match client.check_health_detailed().await {
        Ok(healthy) => {
            println!(
                "  Status: {}",
                if healthy {
                    "✅ Healthy"
                } else {
                    "❌ Unhealthy"
                }
            );
        }
        Err(e) => println!("  Error: {}", e),
    }

    println!("\n📊 Connection Information:");
    if let Some(route) = client.get_route_path() {
        println!("  Route Path: {:?}", route);
    } else {
        println!("  Route Path: Direct connection");
    }

    println!("\n⚙️ Batch Configuration:");
    let config = client.get_batch_config();
    println!(
        "  Max Operations per Packet: {}",
        config.max_operations_per_packet
    );
    println!("  Max Packet Size: {} bytes", config.max_packet_size);
    println!("  Packet Timeout: {} ms", config.packet_timeout_ms);
    println!("  Continue on Error: {}", config.continue_on_error);
    println!(
        "  Optimize Packet Packing: {}",
        config.optimize_packet_packing
    );

    Ok(())
}

fn parse_value(
    current: &PlcValue,
    input: &str,
) -> Result<PlcValue, Box<dyn std::error::Error + Send + Sync>> {
    match current {
        PlcValue::Bool(_) => Ok(PlcValue::Bool(
            input
                .parse::<bool>()
                .unwrap_or(input == "true" || input == "1"),
        )),
        PlcValue::Sint(_) => Ok(PlcValue::Sint(input.parse::<i8>()?)),
        PlcValue::Int(_) => Ok(PlcValue::Int(input.parse::<i16>()?)),
        PlcValue::Dint(_) => Ok(PlcValue::Dint(input.parse::<i32>()?)),
        PlcValue::Lint(_) => Ok(PlcValue::Lint(input.parse::<i64>()?)),
        PlcValue::Usint(_) => Ok(PlcValue::Usint(input.parse::<u8>()?)),
        PlcValue::Uint(_) => Ok(PlcValue::Uint(input.parse::<u16>()?)),
        PlcValue::Udint(_) => Ok(PlcValue::Udint(input.parse::<u32>()?)),
        PlcValue::Ulint(_) => Ok(PlcValue::Ulint(input.parse::<u64>()?)),
        PlcValue::Real(_) => Ok(PlcValue::Real(input.parse::<f32>()?)),
        PlcValue::Lreal(_) => Ok(PlcValue::Lreal(input.parse::<f64>()?)),
        PlcValue::String(_) => Ok(PlcValue::String(input.to_string())),
        _ => Err("Unsupported type for parsing".into()),
    }
}
