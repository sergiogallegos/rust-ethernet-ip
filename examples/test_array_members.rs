// examples/test_array_members.rs
// =========================================================================
//
// Array Member Reading Test
//
// This example tests reading array members to verify the fix for
// "CIP Error 4: Path segment error" when reading array elements.
//
// Usage:
//   cargo run --example test_array_members -- <PLC_IP:PORT> <ARRAY_TAG_NAME>
//
// Examples:
//   cargo run --example test_array_members -- 192.168.1.100:44818 MyArray
//   cargo run --example test_array_members -- 192.168.1.100:44818 Program:MainProgram.DataArray
//
// =========================================================================

use rust_ethernet_ip::{EipClient, PlcValue};
use std::env;
use tokio::time::{Duration, timeout};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Array Member Reading Test");
    println!("============================\n");

    // Parse command line arguments
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: cargo run --example test_array_members -- <PLC_IP:PORT> [ARRAY_TAG_NAME]");
        println!("\nExamples:");
        println!("  cargo run --example test_array_members -- 192.168.1.100:44818 MyArray");
        println!(
            "  cargo run --example test_array_members -- 192.168.1.100:44818 Program:MainProgram.DataArray"
        );
        return Ok(());
    }

    let plc_address = &args[1];
    let array_tag_name = args.get(2).map(|s| s.as_str()).unwrap_or("TestArray");

    println!("📡 Connecting to PLC at {}", plc_address);
    println!("🔍 Testing array: {}\n", array_tag_name);

    // Connect to PLC
    let mut client = match timeout(Duration::from_secs(10), EipClient::connect(plc_address)).await {
        Ok(Ok(client)) => {
            println!("✅ Connected successfully!\n");
            client
        }
        Ok(Err(e)) => {
            eprintln!("❌ Failed to connect: {}", e);
            return Err(e.into());
        }
        Err(_) => {
            eprintln!("❌ Connection timeout");
            return Err("Connection timeout".into());
        }
    };

    // Test 0: Read base array tag (without index) to verify basic connectivity
    println!("📊 Test 0: Reading Base Array Tag (No Index)");
    println!("----------------------------------------------");
    print!("  Reading {} (base tag, no index)... ", array_tag_name);

    match timeout(Duration::from_secs(5), client.read_tag(array_tag_name)).await {
        Ok(Ok(value)) => {
            println!("✅ {:?}", value);
            println!("  ✅ Base array tag read successful - connectivity is good\n");
        }
        Ok(Err(e)) => {
            println!("❌ Error: {}", e);
            println!("  ⚠️  Base array tag read failed - this may indicate a different issue\n");
        }
        Err(_) => {
            println!("❌ Timeout");
            println!("  ⚠️  Base array tag read timeout\n");
        }
    }

    // Test 1: Read individual array elements
    println!("📊 Test 1: Reading Individual Array Elements");
    println!("----------------------------------------------");

    let test_indices = vec![0, 1, 2, 5, 10, 99];
    let mut success_count = 0;
    let mut fail_count = 0;

    for &index in &test_indices {
        let tag_path = format!("{}[{}]", array_tag_name, index);
        print!("  Reading {}[{}]... ", array_tag_name, index);

        match timeout(Duration::from_secs(5), client.read_tag(&tag_path)).await {
            Ok(Ok(value)) => {
                println!("✅ {:?}", value);
                success_count += 1;
            }
            Ok(Err(e)) => {
                println!("❌ Error: {}", e);
                fail_count += 1;

                // Check if it's the path segment error we're trying to fix
                if e.to_string().contains("Path segment error")
                    || e.to_string().contains("CIP Error 4")
                {
                    println!("     ⚠️  This is the error we're trying to fix!");
                }
            }
            Err(_) => {
                println!("❌ Timeout");
                fail_count += 1;
            }
        }
    }

    println!(
        "\n  Results: {} successful, {} failed\n",
        success_count, fail_count
    );

    // Test 2: Read a range of array elements
    println!("📊 Test 2: Reading Array Range");
    println!("-------------------------------");

    let start_index = 0;
    let count = 10;
    println!(
        "  Reading {}[{}..{}]...\n",
        array_tag_name,
        start_index,
        start_index + count - 1
    );

    let mut range_success = 0;
    let mut range_fail = 0;

    for i in start_index..start_index + count {
        let tag_path = format!("{}[{}]", array_tag_name, i);

        match timeout(Duration::from_secs(5), client.read_tag(&tag_path)).await {
            Ok(Ok(value)) => {
                println!("  ✅ {}[{}] = {:?}", array_tag_name, i, value);
                range_success += 1;
            }
            Ok(Err(e)) => {
                println!("  ❌ {}[{}] failed: {}", array_tag_name, i, e);
                range_fail += 1;
            }
            Err(_) => {
                println!("  ❌ {}[{}] timeout", array_tag_name, i);
                range_fail += 1;
            }
        }
    }

    println!(
        "\n  Range Results: {} successful, {} failed\n",
        range_success, range_fail
    );

    // Test 3: Test different array types (if you have them)
    println!("📊 Test 3: Testing Different Array Types");
    println!("-----------------------------------------");

    // First test base tags (no index)
    println!("  Testing base tags (no index):");
    let base_tags = vec![
        (array_tag_name.to_string(), "Controller-scoped base"),
        (
            format!("Program:MainProgram.{}", array_tag_name),
            "Program-scoped base",
        ),
    ];

    for (test_tag, description) in &base_tags {
        print!("    {} ({})... ", test_tag, description);

        match timeout(Duration::from_secs(5), client.read_tag(test_tag)).await {
            Ok(Ok(value)) => {
                println!("✅ {:?}", value);
            }
            Ok(Err(e)) => {
                println!("❌ {}", e);
            }
            Err(_) => {
                println!("❌ Timeout");
            }
        }
    }

    // Then test with index [0]
    println!("\n  Testing with index [0]:");
    let array_types = vec![
        (format!("{}[0]", array_tag_name), "Controller-scoped"),
        (
            format!("Program:MainProgram.{}[0]", array_tag_name),
            "Program-scoped",
        ),
    ];

    for (test_tag, description) in &array_types {
        print!("    {} ({})... ", test_tag, description);

        match timeout(Duration::from_secs(5), client.read_tag(test_tag)).await {
            Ok(Ok(value)) => {
                println!("✅ {:?}", value);
            }
            Ok(Err(e)) => {
                println!("❌ {}", e);
            }
            Err(_) => {
                println!("❌ Timeout");
            }
        }
    }

    // Test 4: Write and read back (if writable)
    println!("\n📊 Test 4: Write and Read Back");
    println!("-------------------------------");

    let test_index = 0;
    let test_value = PlcValue::Dint(12345);
    let tag_path = format!("{}[{}]", array_tag_name, test_index);

    println!("  Writing {} = {:?}...", tag_path, test_value);
    match timeout(
        Duration::from_secs(5),
        client.write_tag(&tag_path, test_value.clone()),
    )
    .await
    {
        Ok(Ok(())) => {
            println!("  ✅ Write successful");

            // Read back
            println!("  Reading back...");
            match timeout(Duration::from_secs(5), client.read_tag(&tag_path)).await {
                Ok(Ok(read_value)) => {
                    println!("  ✅ Read back: {:?}", read_value);
                    if read_value == test_value {
                        println!("  ✅ Values match!");
                    } else {
                        println!("  ⚠️  Values don't match (expected {:?})", test_value);
                    }
                }
                Ok(Err(e)) => {
                    println!("  ❌ Read back failed: {}", e);
                }
                Err(_) => {
                    println!("  ❌ Read back timeout");
                }
            }
        }
        Ok(Err(e)) => {
            println!("  ❌ Write failed: {} (tag may be read-only)", e);
        }
        Err(_) => {
            println!("  ❌ Write timeout");
        }
    }

    // Summary
    println!("\n📈 Test Summary");
    println!("===============");
    println!(
        "  Total individual reads: {} successful, {} failed",
        success_count, fail_count
    );
    println!(
        "  Range reads: {} successful, {} failed",
        range_success, range_fail
    );

    if fail_count == 0 && range_fail == 0 {
        println!("\n🎉 All tests passed! Array member reading is working correctly.");
    } else if fail_count > 0 || range_fail > 0 {
        println!("\n⚠️  Some tests failed. Check the errors above.");
        println!(
            "   If you see 'Path segment error' or 'CIP Error 4', the fix may need adjustment."
        );
    }

    Ok(())
}
