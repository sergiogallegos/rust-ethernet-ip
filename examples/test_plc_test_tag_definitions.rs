/// Comprehensive Test for All Tags from PLC_TEST_TAG_DEFINITIONS.md
///
/// This test verifies that the Rust library can correctly:
/// 1. Read all tags (controller and program-scoped)
/// 2. Write new values to all tags
/// 3. Read back and verify the writes were successful
///
/// Run with: cargo run --example test_plc_test_tag_definitions
///
/// Prerequisites:
/// - All tags from PLC_TEST_TAG_DEFINITIONS.md must exist in the PLC
/// - PLC must be accessible at 192.168.0.1:44818
/// - ControlLogix CPU in Slot 0 (or adjust CPU_SLOT constant)
use rust_ethernet_ip::{EipClient, PlcValue, RoutePath};
use std::collections::HashMap;
use std::env;

fn get_plc_address() -> String {
    env::var("TEST_PLC_ADDRESS").unwrap_or_else(|_| "192.168.0.1:44818".to_string())
}

fn get_cpu_slot() -> u8 {
    env::var("TEST_PLC_SLOT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0)
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
struct TestTag {
    name: String,
    initial_value: PlcValue,
    test_value: PlcValue,
    description: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let plc_address = get_plc_address();
    let cpu_slot = get_cpu_slot();
    println!("═══════════════════════════════════════════════════════════════════════════════");
    println!("🔬 Comprehensive Test: All Tags from PLC_TEST_TAG_DEFINITIONS.md");
    println!("═══════════════════════════════════════════════════════════════════════════════");
    println!();

    println!("🔌 Connecting to ControlLogix PLC at {}...", plc_address);
    println!("   CPU Slot: {}", cpu_slot);

    // Create route path for ControlLogix
    let route_path = RoutePath::new().add_slot(cpu_slot);

    // Connect with route path
    let mut client = EipClient::with_route_path(&plc_address, route_path).await?;
    println!("✅ Connected successfully!\n");

    // Define all test tags from PLC_TEST_TAG_DEFINITIONS.md
    let test_tags = create_test_tags();

    let mut total_tests = 0;
    let mut passed_tests = 0;
    let mut failed_tests = 0;
    let mut skipped_tests = 0;

    // Track failures with error messages
    let mut read_failures: Vec<(String, String)> = Vec::new();
    let mut write_failures: Vec<(String, String)> = Vec::new();
    let mut verify_failures: Vec<(String, String, PlcValue, PlcValue)> = Vec::new();

    // Step 1: Read initial values
    println!("═══════════════════════════════════════════════════════════════════════════════");
    println!("📖 STEP 1: Reading Initial Values");
    println!("═══════════════════════════════════════════════════════════════════════════════");
    println!();

    let mut initial_values = HashMap::new();

    for tag in &test_tags {
        total_tests += 1;
        print!("   Reading {}... ", tag.name);
        match client.read_tag(&tag.name).await {
            Ok(value) => {
                println!("✅ {:?}", value);
                initial_values.insert(tag.name.clone(), value);
            }
            Err(e) => {
                let error_msg = format!("{}", e);
                println!("❌ FAILED: {}", error_msg);
                println!("      ⚠️  Tag may not exist in PLC - skipping write/verify for this tag");
                read_failures.push((tag.name.clone(), error_msg));
                failed_tests += 1;
                skipped_tests += 1;
            }
        }
    }

    println!();
    println!(
        "📊 Step 1 Summary: {} read, {} failed",
        initial_values.len(),
        test_tags.len() - initial_values.len()
    );
    println!();

    // Step 2: Write test values
    println!("═══════════════════════════════════════════════════════════════════════════════");
    println!("✏️  STEP 2: Writing Test Values");
    println!("═══════════════════════════════════════════════════════════════════════════════");
    println!();

    let mut written_tags = Vec::new();

    for tag in &test_tags {
        // Skip if we couldn't read it initially
        if !initial_values.contains_key(&tag.name) {
            continue;
        }

        print!("   Writing {} = {:?}... ", tag.name, tag.test_value);
        match client.write_tag(&tag.name, tag.test_value.clone()).await {
            Ok(_) => {
                println!("✅");
                written_tags.push(tag.name.clone());

                // For STRING types, immediately read back to verify write
                if matches!(tag.test_value, PlcValue::String(_)) {
                    print!("      Reading back {}... ", tag.name);
                    match client.read_tag(&tag.name).await {
                        Ok(read_value) => {
                            if values_match(&read_value, &tag.test_value) {
                                println!("✅ VERIFIED: {:?}", read_value);
                            } else {
                                println!("⚠️  MISMATCH after write!");
                                println!("         Expected: {:?}", tag.test_value);
                                println!("         Got:      {:?}", read_value);
                            }
                        }
                        Err(e) => {
                            println!("❌ FAILED TO READ BACK: {}", e);
                        }
                    }
                }
            }
            Err(e) => {
                let error_msg = format!("{}", e);
                println!("❌ FAILED: {}", error_msg);
                write_failures.push((tag.name.clone(), error_msg));
                failed_tests += 1;
            }
        }
    }

    println!();
    println!(
        "📊 Step 2 Summary: {} written successfully",
        written_tags.len()
    );
    println!();

    // Small delay to ensure writes are processed
    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

    // Step 3: Read back and verify
    println!("═══════════════════════════════════════════════════════════════════════════════");
    println!("🔍 STEP 3: Reading Back and Verifying Writes");
    println!("═══════════════════════════════════════════════════════════════════════════════");
    println!();

    for tag in &test_tags {
        if !written_tags.contains(&tag.name) {
            continue;
        }

        print!("   Verifying {}... ", tag.name);
        match client.read_tag(&tag.name).await {
            Ok(value) => {
                if values_match(&value, &tag.test_value) {
                    println!("✅ VERIFIED: {:?}", value);
                    passed_tests += 1;
                } else {
                    println!("❌ MISMATCH!");
                    println!("      Expected: {:?}", tag.test_value);
                    println!("      Got:      {:?}", value);

                    // For STRING types, show detailed comparison
                    if let (PlcValue::String(expected_str), PlcValue::String(actual_str)) =
                        (&tag.test_value, &value)
                    {
                        println!("      Expected string length: {}", expected_str.len());
                        println!("      Actual string length:   {}", actual_str.len());
                        println!("      Expected bytes: {:?}", expected_str.as_bytes());
                        println!("      Actual bytes:   {:?}", actual_str.as_bytes());
                        println!("      Expected text: \"{}\"", expected_str);
                        println!("      Actual text:   \"{}\"", actual_str);
                    }

                    verify_failures.push((
                        tag.name.clone(),
                        format!(
                            "Value mismatch: expected {:?}, got {:?}",
                            tag.test_value, value
                        ),
                        tag.test_value.clone(),
                        value,
                    ));
                    failed_tests += 1;
                }
            }
            Err(e) => {
                let error_msg = format!("{}", e);
                println!("❌ FAILED TO READ: {}", error_msg);
                verify_failures.push((
                    tag.name.clone(),
                    format!("Read error: {}", error_msg),
                    tag.test_value.clone(),
                    PlcValue::Dint(0), // Placeholder
                ));
                failed_tests += 1;
            }
        }
    }

    println!();
    println!("═══════════════════════════════════════════════════════════════════════════════");
    println!("♻️  STEP 4: Restoring Initial Values");
    println!("═══════════════════════════════════════════════════════════════════════════════");
    println!();

    let mut restore_failures: Vec<(String, String)> = Vec::new();
    let mut restored_tags = 0usize;
    for tag_name in &written_tags {
        let Some(original_value) = initial_values.get(tag_name) else {
            continue;
        };

        print!("   Restoring {}... ", tag_name);
        match client.write_tag(tag_name, original_value.clone()).await {
            Ok(_) => {
                println!("✅");
                restored_tags += 1;
            }
            Err(e) => {
                let error_msg = format!("{}", e);
                println!("❌ FAILED: {}", error_msg);
                restore_failures.push((tag_name.clone(), error_msg));
            }
        }
    }
    println!();
    println!(
        "📊 Step 4 Summary: {} restored, {} failed",
        restored_tags,
        restore_failures.len()
    );

    println!();
    println!("═══════════════════════════════════════════════════════════════════════════════");
    println!("📊 FINAL RESULTS");
    println!("═══════════════════════════════════════════════════════════════════════════════");
    println!("   Total Tests:     {}", total_tests);
    println!("   ✅ Passed:        {}", passed_tests);
    println!("   ❌ Failed:        {}", failed_tests);
    println!("   ⏭️  Skipped:       {}", skipped_tests);
    println!(
        "   Success Rate:    {:.1}%",
        (passed_tests as f64 / (total_tests - skipped_tests) as f64) * 100.0
    );
    println!();

    // Display failure summary
    if !read_failures.is_empty()
        || !write_failures.is_empty()
        || !verify_failures.is_empty()
        || !restore_failures.is_empty()
    {
        println!("═══════════════════════════════════════════════════════════════════════════════");
        println!("❌ FAILED TAGS SUMMARY");
        println!("═══════════════════════════════════════════════════════════════════════════════");
        println!();

        if !read_failures.is_empty() {
            println!("📖 READ FAILURES ({} tags):", read_failures.len());
            println!("   These tags could not be read initially (may not exist in PLC):");

            // Group by error pattern
            let mut error_groups: std::collections::HashMap<String, Vec<String>> =
                std::collections::HashMap::new();
            for (tag_name, error) in &read_failures {
                let error_key = error.clone();
                error_groups
                    .entry(error_key)
                    .or_default()
                    .push(tag_name.clone());
            }

            for (error, tags) in &error_groups {
                if tags.len() <= 5 {
                    // Show all tags if 5 or fewer
                    for tag in tags {
                        println!("   • {}: {}", tag, error);
                    }
                } else {
                    // Show first 3, then summary
                    for tag in tags.iter().take(3) {
                        println!("   • {}: {}", tag, error);
                    }
                    println!(
                        "   • ... and {} more tags with the same error",
                        tags.len() - 3
                    );
                }
            }
            println!();
        }

        if !write_failures.is_empty() {
            println!("✏️  WRITE FAILURES ({} tags):", write_failures.len());

            // Group by error pattern
            let mut error_groups: std::collections::HashMap<String, Vec<String>> =
                std::collections::HashMap::new();
            for (tag_name, error) in &write_failures {
                // Extract error pattern (simplify long error messages)
                let error_key = if error.contains("0x2107") || error.contains("2107") {
                    // Check if it's a UDT array element member (e.g., gTestUDT_Array[0].Member1_DINT)
                    if tag_name.contains("_Array[") && tag_name.contains('.') {
                        "PLC does not support writing to UDT array element members directly (Error 0x2107)".to_string()
                    }
                    // Check if it's a STRING member in a UDT (e.g., gTestUDT.Member5_String)
                    else if tag_name.contains("Member5_String")
                        || tag_name.ends_with(".Member5_String")
                    {
                        "PLC does not support writing to STRING members in UDTs directly (Error 0x2107)".to_string()
                    }
                    // Check if it's a simple STRING tag (e.g., gTest_STRING)
                    else if tag_name == "gTest_STRING"
                        || tag_name == "Program:TestProgram.gTest_STRING"
                    {
                        "PLC does not support writing to STRING tags directly (Error 0x2107)"
                            .to_string()
                    }
                    // Other 0x2107 errors
                    else {
                        format!("Error 0x2107: {}", error)
                    }
                } else if error.contains("UDT array element members directly") {
                    "PLC does not support writing to UDT array element members directly (Error 0x2107)".to_string()
                } else {
                    error.clone()
                };
                error_groups
                    .entry(error_key)
                    .or_default()
                    .push(tag_name.clone());
            }

            for (error, tags) in &error_groups {
                println!("   Error: {}", error);
                println!("   Affected tags ({}):", tags.len());

                // Group tags by pattern
                let mut pattern_groups: std::collections::HashMap<String, Vec<String>> =
                    std::collections::HashMap::new();
                for tag in tags {
                    // Extract pattern (e.g., "gTestUDT_Array[*].Member1_DINT")
                    if let Some(bracket_pos) = tag.find('[') {
                        if let Some(dot_pos) = tag[bracket_pos..].find('.') {
                            let base = &tag[..bracket_pos];
                            let member = &tag[bracket_pos + dot_pos..];
                            let pattern = format!("{}[*]{}", base, member);
                            pattern_groups
                                .entry(pattern)
                                .or_default()
                                .push(tag.clone());
                        } else {
                            pattern_groups
                                .entry(tag.clone())
                                .or_default()
                                .push(tag.clone());
                        }
                    } else {
                        pattern_groups
                            .entry(tag.clone())
                            .or_default()
                            .push(tag.clone());
                    }
                }

                // Sort patterns: controller-scoped first, then program-scoped, then by member name
                let mut sorted_patterns: Vec<_> = pattern_groups.iter().collect();
                sorted_patterns.sort_by(|a, b| {
                    let a_is_program = a.0.contains("Program:");
                    let b_is_program = b.0.contains("Program:");
                    match (a_is_program, b_is_program) {
                        (true, false) => std::cmp::Ordering::Greater,
                        (false, true) => std::cmp::Ordering::Less,
                        _ => a.0.cmp(b.0), // Same scope, sort alphabetically
                    }
                });

                for (pattern, pattern_tags) in &sorted_patterns {
                    if pattern.contains("[*]") {
                        // Show pattern with count
                        println!("     • {} ({} tags)", pattern, pattern_tags.len());
                        // Show first few examples
                        for tag in pattern_tags.iter().take(3) {
                            println!("       - {}", tag);
                        }
                        if pattern_tags.len() > 3 {
                            println!("       - ... and {} more", pattern_tags.len() - 3);
                        }
                    } else {
                        // Individual tags
                        for tag in pattern_tags.iter().take(5) {
                            println!("     • {}", tag);
                        }
                        if pattern_tags.len() > 5 {
                            println!("     • ... and {} more", pattern_tags.len() - 5);
                        }
                    }
                }
            }
            println!();
        }

        if !verify_failures.is_empty() {
            println!("🔍 VERIFICATION FAILURES ({} tags):", verify_failures.len());
            println!("   These tags were written but verification failed:");

            // Group by error pattern
            let mut error_groups: std::collections::HashMap<
                String,
                Vec<(String, PlcValue, PlcValue)>,
            > = std::collections::HashMap::new();
            for (tag_name, error, expected, actual) in &verify_failures {
                let error_key = error.clone();
                error_groups
                    .entry(error_key)
                    .or_default()
                    .push((tag_name.clone(), expected.clone(), actual.clone()));
            }

            for (error, tags) in &error_groups {
                println!("   Error: {}", error);
                for (tag_name, expected, actual) in tags.iter().take(10) {
                    println!("     • {}", tag_name);
                    if !error.contains("Read error") {
                        println!("       Expected: {:?}", expected);
                        println!("       Actual:   {:?}", actual);
                    }
                }
                if tags.len() > 10 {
                    println!(
                        "     • ... and {} more tags with similar errors",
                        tags.len() - 10
                    );
                }
            }
            println!();
        }

        if !restore_failures.is_empty() {
            println!("♻️  RESTORE FAILURES ({} tags):", restore_failures.len());
            for (tag_name, error) in &restore_failures {
                println!("   • {}: {}", tag_name, error);
            }
            println!();
        }

        println!("═══════════════════════════════════════════════════════════════════════════════");
        println!();
    }

    if failed_tests == 0 && skipped_tests == 0 {
        println!("🎉 ALL TESTS PASSED! The Rust library is working correctly.");
    } else if failed_tests > 0 {
        println!("⚠️  Some tests failed. See the FAILED TAGS SUMMARY above for details.");
    } else {
        println!("ℹ️  Some tags were skipped (may not exist in PLC).");
    }

    if restore_failures.is_empty() {
        println!("ℹ️  Successfully restored all tags written during this run.");
    } else {
        println!("⚠️  Some written tags could not be restored. See RESTORE FAILURES above.");
    }

    Ok(())
}

fn create_test_tags() -> Vec<TestTag> {
    let mut tags = Vec::new();

    // ============================================================================
    // Controller-Scoped Array Elements
    // ============================================================================

    // DINT Array - elements 0-9
    for i in 0..10 {
        tags.push(TestTag {
            name: format!("gTestArray_DINT[{}]", i),
            initial_value: PlcValue::Dint((i + 1) * 10),
            test_value: PlcValue::Dint(1000 + (i * 111)),
            description: format!("Controller DINT array element {}", i),
        });
    }

    // REAL Array - elements 0-9
    for i in 0..10 {
        tags.push(TestTag {
            name: format!("gTestArray_REAL[{}]", i),
            initial_value: PlcValue::Real((i as f32 + 1.0) * 1.1),
            test_value: PlcValue::Real(10.0 + (i as f32 * 1.11)),
            description: format!("Controller REAL array element {}", i),
        });
    }

    // BOOL Array - elements 0-9 (alternating)
    for i in 0..10 {
        tags.push(TestTag {
            name: format!("gTestArray_BOOL[{}]", i),
            initial_value: PlcValue::Bool(i % 2 == 0),
            test_value: PlcValue::Bool(i % 2 == 1),
            description: format!("Controller BOOL array element {}", i),
        });
    }

    // INT Array - elements 0-9
    for i in 0..10 {
        tags.push(TestTag {
            name: format!("gTestArray_INT[{}]", i),
            initial_value: PlcValue::Int((i + 1) * 100),
            test_value: PlcValue::Int(1000 + (i * 111)),
            description: format!("Controller INT array element {}", i),
        });
    }

    // Large DINT Array - test 16-bit indices
    for &idx in &[100, 200, 300, 500, 999] {
        tags.push(TestTag {
            name: format!("gTestArray_Large[{}]", idx),
            initial_value: PlcValue::Dint(0), // Unknown initial value
            test_value: PlcValue::Dint(10000 + idx),
            description: format!("Controller large DINT array element {} (16-bit index)", idx),
        });
    }

    // Simple STRING tag (controller-scoped)
    tags.push(TestTag {
        name: "gTest_STRING".to_string(),
        initial_value: PlcValue::String("Initial String Value".to_string()),
        test_value: PlcValue::String("Test String Write 789".to_string()),
        description: "Controller simple STRING tag (not UDT member)".to_string(),
    });

    // ============================================================================
    // Controller-Scoped UDT Members
    // ============================================================================

    tags.push(TestTag {
        name: "gTestUDT.Member1_DINT".to_string(),
        initial_value: PlcValue::Dint(100),
        test_value: PlcValue::Dint(7777),
        description: "Controller UDT member: Member1_DINT".to_string(),
    });

    tags.push(TestTag {
        name: "gTestUDT.Member2_REAL".to_string(),
        initial_value: PlcValue::Real(std::f32::consts::PI),
        test_value: PlcValue::Real(77.77),
        description: "Controller UDT member: Member2_REAL".to_string(),
    });

    tags.push(TestTag {
        name: "gTestUDT.Member3_BOOL".to_string(),
        initial_value: PlcValue::Bool(true),
        test_value: PlcValue::Bool(false),
        description: "Controller UDT member: Member3_BOOL".to_string(),
    });

    tags.push(TestTag {
        name: "gTestUDT.Member4_INT".to_string(),
        initial_value: PlcValue::Int(42),
        test_value: PlcValue::Int(8888),
        description: "Controller UDT member: Member4_INT".to_string(),
    });

    tags.push(TestTag {
        name: "gTestUDT.Member5_String".to_string(),
        initial_value: PlcValue::String("Hello PLC".to_string()),
        test_value: PlcValue::String("Test String 123".to_string()),
        description: "Controller UDT member: Member5_String".to_string(),
    });

    // UDT Array_DINT - elements 0-9
    for i in 0..10 {
        tags.push(TestTag {
            name: format!("gTestUDT.Array_DINT[{}]", i),
            initial_value: PlcValue::Dint(i + 1),
            test_value: PlcValue::Dint(1000 + (i * 111)),
            description: format!("Controller UDT array member: Array_DINT[{}]", i),
        });
    }

    // UDT Array_REAL - elements 0-4
    for i in 0..5 {
        tags.push(TestTag {
            name: format!("gTestUDT.Array_REAL[{}]", i),
            initial_value: PlcValue::Real((i as f32 + 1.0) * 1.1),
            test_value: PlcValue::Real(10.0 + (i as f32 * 1.11)),
            description: format!("Controller UDT array member: Array_REAL[{}]", i),
        });
    }

    // UDT Array_BOOL - elements 0-19
    for i in 0..20 {
        tags.push(TestTag {
            name: format!("gTestUDT.Array_BOOL[{}]", i),
            initial_value: PlcValue::Bool(i % 2 == 0),
            test_value: PlcValue::Bool(i % 2 == 1),
            description: format!("Controller UDT array member: Array_BOOL[{}]", i),
        });
    }

    // ============================================================================
    // Controller-Scoped UDT Array Elements
    // ============================================================================

    // UDT Array elements 0-9 - Member1_DINT
    for i in 0..10 {
        tags.push(TestTag {
            name: format!("gTestUDT_Array[{}].Member1_DINT", i),
            initial_value: PlcValue::Dint((i + 1) * 100),
            test_value: PlcValue::Dint(1000 + (i * 444)),
            description: format!("Controller UDT array element {}, member Member1_DINT", i),
        });
    }

    // UDT Array elements 0-9 - Member2_REAL
    for i in 0..10 {
        tags.push(TestTag {
            name: format!("gTestUDT_Array[{}].Member2_REAL", i),
            initial_value: PlcValue::Real((i as f32 + 1.0) * 1.1),
            test_value: PlcValue::Real(10.0 + (i as f32 * 7.77)),
            description: format!("Controller UDT array element {}, member Member2_REAL", i),
        });
    }

    // UDT Array elements 0-9 - Member3_BOOL
    for i in 0..10 {
        tags.push(TestTag {
            name: format!("gTestUDT_Array[{}].Member3_BOOL", i),
            initial_value: PlcValue::Bool(i % 2 == 0),
            test_value: PlcValue::Bool(i % 2 == 1),
            description: format!("Controller UDT array element {}, member Member3_BOOL", i),
        });
    }

    // UDT Array elements 0-9 - Member4_INT
    for i in 0..10 {
        tags.push(TestTag {
            name: format!("gTestUDT_Array[{}].Member4_INT", i),
            initial_value: PlcValue::Int((i + 1) * 10),
            test_value: PlcValue::Int(1000 + (i * 111)),
            description: format!("Controller UDT array element {}, member Member4_INT", i),
        });
    }

    // UDT Array elements 0-9 - Array_DINT[0-9] (sample a few)
    for i in 0..10 {
        for j in 0..10 {
            tags.push(TestTag {
                name: format!("gTestUDT_Array[{}].Array_DINT[{}]", i, j),
                initial_value: PlcValue::Dint(j + 1),
                test_value: PlcValue::Dint(1000 + (i * 100) + (j * 11)),
                description: format!(
                    "Controller UDT array element {}, nested array member Array_DINT[{}]",
                    i, j
                ),
            });
        }
    }

    // UDT Array elements 0-9 - Array_REAL[0-4] (sample a few)
    for i in 0..10 {
        for j in 0..5 {
            tags.push(TestTag {
                name: format!("gTestUDT_Array[{}].Array_REAL[{}]", i, j),
                initial_value: PlcValue::Real((j as f32 + 1.0) * 1.1),
                test_value: PlcValue::Real(10.0 + (i as f32 * 10.0) + (j as f32 * 1.11)),
                description: format!(
                    "Controller UDT array element {}, nested array member Array_REAL[{}]",
                    i, j
                ),
            });
        }
    }

    // ============================================================================
    // Program-Scoped Array Elements
    // ============================================================================

    // Program DINT Array - elements 0-9
    for i in 0..10 {
        tags.push(TestTag {
            name: format!("Program:TestProgram.gTestArray_DINT[{}]", i),
            initial_value: PlcValue::Dint((i + 1) * 1000),
            test_value: PlcValue::Dint(10000 + (i * 1111)),
            description: format!("Program-scoped DINT array element {}", i),
        });
    }

    // Program REAL Array - elements 0-9
    for i in 0..10 {
        tags.push(TestTag {
            name: format!("Program:TestProgram.gTestArray_REAL[{}]", i),
            initial_value: PlcValue::Real(10.1 + (i as f32 * 10.1)),
            test_value: PlcValue::Real(100.0 + (i as f32 * 11.11)),
            description: format!("Program-scoped REAL array element {}", i),
        });
    }

    // Program BOOL Array - elements 0-9
    for i in 0..10 {
        tags.push(TestTag {
            name: format!("Program:TestProgram.gTestArray_BOOL[{}]", i),
            initial_value: PlcValue::Bool(i % 2 == 1),
            test_value: PlcValue::Bool(i % 2 == 0),
            description: format!("Program-scoped BOOL array element {}", i),
        });
    }

    // Simple STRING tag (program-scoped)
    tags.push(TestTag {
        name: "Program:TestProgram.gTest_STRING".to_string(),
        initial_value: PlcValue::String("Program Initial String".to_string()),
        test_value: PlcValue::String("Program Test String Write 999".to_string()),
        description: "Program-scoped simple STRING tag (not UDT member)".to_string(),
    });

    // ============================================================================
    // Program-Scoped UDT Members
    // ============================================================================

    tags.push(TestTag {
        name: "Program:TestProgram.gTestUDT.Member1_DINT".to_string(),
        initial_value: PlcValue::Dint(500),
        test_value: PlcValue::Dint(55555),
        description: "Program-scoped UDT member: Member1_DINT".to_string(),
    });

    tags.push(TestTag {
        name: "Program:TestProgram.gTestUDT.Member2_REAL".to_string(),
        initial_value: PlcValue::Real(std::f32::consts::E),
        test_value: PlcValue::Real(88.88),
        description: "Program-scoped UDT member: Member2_REAL".to_string(),
    });

    tags.push(TestTag {
        name: "Program:TestProgram.gTestUDT.Member3_BOOL".to_string(),
        initial_value: PlcValue::Bool(false),
        test_value: PlcValue::Bool(true),
        description: "Program-scoped UDT member: Member3_BOOL".to_string(),
    });

    tags.push(TestTag {
        name: "Program:TestProgram.gTestUDT.Member4_INT".to_string(),
        initial_value: PlcValue::Int(24),
        test_value: PlcValue::Int(9999),
        description: "Program-scoped UDT member: Member4_INT".to_string(),
    });

    tags.push(TestTag {
        name: "Program:TestProgram.gTestUDT.Member5_String".to_string(),
        initial_value: PlcValue::String("Program UDT".to_string()),
        test_value: PlcValue::String("Program Test String 456".to_string()),
        description: "Program-scoped UDT member: Member5_String".to_string(),
    });

    // Program UDT Array_DINT - elements 0-9
    for i in 0..10 {
        tags.push(TestTag {
            name: format!("Program:TestProgram.gTestUDT.Array_DINT[{}]", i),
            initial_value: PlcValue::Dint(i + 1),
            test_value: PlcValue::Dint(2000 + (i * 111)),
            description: format!("Program-scoped UDT array member: Array_DINT[{}]", i),
        });
    }

    // Program UDT Array_REAL - elements 0-4
    for i in 0..5 {
        tags.push(TestTag {
            name: format!("Program:TestProgram.gTestUDT.Array_REAL[{}]", i),
            initial_value: PlcValue::Real((i as f32 + 1.0) * 1.1),
            test_value: PlcValue::Real(20.0 + (i as f32 * 1.11)),
            description: format!("Program-scoped UDT array member: Array_REAL[{}]", i),
        });
    }

    // ============================================================================
    // Program-Scoped UDT Array Elements
    // ============================================================================

    // Program UDT Array elements 0-4 - Member1_DINT
    for i in 0..5 {
        tags.push(TestTag {
            name: format!("Program:TestProgram.gTestUDT_Array[{}].Member1_DINT", i),
            initial_value: PlcValue::Dint((i + 1) * 100),
            test_value: PlcValue::Dint(2000 + (i * 444)),
            description: format!(
                "Program-scoped UDT array element {}, member Member1_DINT",
                i
            ),
        });
    }

    // Program UDT Array elements 0-4 - Member2_REAL
    for i in 0..5 {
        tags.push(TestTag {
            name: format!("Program:TestProgram.gTestUDT_Array[{}].Member2_REAL", i),
            initial_value: PlcValue::Real((i as f32 + 1.0) * 1.1),
            test_value: PlcValue::Real(20.0 + (i as f32 * 7.77)),
            description: format!(
                "Program-scoped UDT array element {}, member Member2_REAL",
                i
            ),
        });
    }

    // Program UDT Array elements 0-4 - Member3_BOOL
    for i in 0..5 {
        tags.push(TestTag {
            name: format!("Program:TestProgram.gTestUDT_Array[{}].Member3_BOOL", i),
            initial_value: PlcValue::Bool(i % 2 == 0),
            test_value: PlcValue::Bool(i % 2 == 1),
            description: format!(
                "Program-scoped UDT array element {}, member Member3_BOOL",
                i
            ),
        });
    }

    // Program UDT Array elements 0-4 - Array_DINT[0-9] (sample a few)
    for i in 0..5 {
        for j in 0..10 {
            tags.push(TestTag {
                name: format!(
                    "Program:TestProgram.gTestUDT_Array[{}].Array_DINT[{}]",
                    i, j
                ),
                initial_value: PlcValue::Dint(j + 1),
                test_value: PlcValue::Dint(2000 + (i * 100) + (j * 11)),
                description: format!(
                    "Program-scoped UDT array element {}, nested array member Array_DINT[{}]",
                    i, j
                ),
            });
        }
    }

    tags
}

fn values_match(actual: &PlcValue, expected: &PlcValue) -> bool {
    match (actual, expected) {
        (PlcValue::Dint(a), PlcValue::Dint(e)) => a == e,
        (PlcValue::Int(a), PlcValue::Int(e)) => a == e,
        (PlcValue::Real(a), PlcValue::Real(e)) => (a - e).abs() < 0.001,
        (PlcValue::Bool(a), PlcValue::Bool(e)) => a == e,
        (PlcValue::String(a), PlcValue::String(e)) => a == e,
        (PlcValue::Udt(_), PlcValue::Udt(_)) => {
            // For UDT, we just check if both are UDT type
            // Full comparison would require parsing the UDT structure
            true
        }
        _ => false,
    }
}
