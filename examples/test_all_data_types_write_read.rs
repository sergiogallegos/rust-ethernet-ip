use rust_ethernet_ip::{EipClient, PlcValue};
use std::time::Duration;
use tokio::time::timeout;

const PLC_IP: &str = "192.168.0.1:44818";
const PROGRAM_NAME: &str = "API_Web";
const REAL_TEST_VALUE: f32 = 88.88;
const DINT_TEST_VALUE: i32 = 12345;
const BOOL_TEST_VALUE: bool = true;
const FLOAT_TOLERANCE: f32 = 0.001;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Testing ALL Data Types Write and Read Operations");
    println!("====================================================");
    println!("PLC IP: {}", PLC_IP);
    println!(
        "Testing REAL, DINT, and BOOL tags in {} program",
        PROGRAM_NAME
    );
    println!("Writing test values, then reading back to verify");
    println!();

    let mut client = EipClient::connect(PLC_IP).await?;
    println!("✅ Connected to PLC successfully!\n");

    // Test tags with their expected values
    let test_tags = vec![
        // REAL tags (already tested, but let's verify they still work)
        (
            format!("Program:{}.out_FuseWeight2", PROGRAM_NAME),
            PlcValue::Real(REAL_TEST_VALUE),
            "REAL",
        ),
        (
            format!("Program:{}.out_FuseWeight1", PROGRAM_NAME),
            PlcValue::Real(REAL_TEST_VALUE),
            "REAL",
        ),
        (
            format!("Program:{}.out_FuseSandFillTime", PROGRAM_NAME),
            PlcValue::Real(REAL_TEST_VALUE),
            "REAL",
        ),
        (
            format!("Program:{}.out_FuseResistance1", PROGRAM_NAME),
            PlcValue::Real(REAL_TEST_VALUE),
            "REAL",
        ),
        // DINT tags
        (
            format!("Program:{}.out_MachineStatus", PROGRAM_NAME),
            PlcValue::Dint(DINT_TEST_VALUE),
            "DINT",
        ),
        (
            format!("Program:{}.out_FusePartStatus", PROGRAM_NAME),
            PlcValue::Dint(DINT_TEST_VALUE),
            "DINT",
        ),
        (
            format!("Program:{}.out_FuseLastStationDone", PROGRAM_NAME),
            PlcValue::Dint(DINT_TEST_VALUE),
            "DINT",
        ),
        (
            format!("Program:{}.cmd_SaveFuseData", PROGRAM_NAME),
            PlcValue::Dint(DINT_TEST_VALUE),
            "DINT",
        ),
        // BOOL tags
        (
            format!("Program:{}.sts_PLCHandshake", PROGRAM_NAME),
            PlcValue::Bool(BOOL_TEST_VALUE),
            "BOOL",
        ),
        (
            format!("Program:{}.out_MachineReady", PROGRAM_NAME),
            PlcValue::Bool(BOOL_TEST_VALUE),
            "BOOL",
        ),
        (
            format!("Program:{}.out_MachineAlarm", PROGRAM_NAME),
            PlcValue::Bool(BOOL_TEST_VALUE),
            "BOOL",
        ),
        (
            format!("Program:{}.in_PCHandshake", PROGRAM_NAME),
            PlcValue::Bool(BOOL_TEST_VALUE),
            "BOOL",
        ),
        (
            format!("Program:{}.in_ClearAlarms", PROGRAM_NAME),
            PlcValue::Bool(BOOL_TEST_VALUE),
            "BOOL",
        ),
    ];

    println!("🎯 Test Plan:");
    println!("1. Read initial values for all tags");
    println!("2. Write test values to all tags");
    println!("3. Read back values to verify changes");
    println!();

    // Step 1: Read initial values
    println!("📖 Step 1: Reading initial values");
    println!("----------------------------------");
    let mut initial_values = Vec::new();
    for (tag_name, _expected_value, _data_type) in &test_tags {
        match timeout(Duration::from_secs(5), client.read_tag(tag_name)).await {
            Ok(Ok(value)) => {
                println!("✅ {}: {:?}", tag_name, value);
                initial_values.push((tag_name.clone(), value));
            }
            Ok(Err(e)) => {
                eprintln!("❌ {}: Read failed - {}", tag_name, e);
                return Err(e.into());
            }
            Err(_) => {
                eprintln!("❌ {}: Read timed out", tag_name);
                return Err(format!("Read operation for {tag_name} timed out").into());
            }
        }
    }
    println!();

    // Step 2: Write test values to all tags
    println!("✏️ Step 2: Writing test values to all tags");
    println!("--------------------------------------------");
    for (tag_name, test_value, data_type) in &test_tags {
        println!(
            "📝 Writing '{:?}' to {} tag '{}'",
            test_value, data_type, tag_name
        );
        match timeout(
            Duration::from_secs(5),
            client.write_tag(tag_name, test_value.clone()),
        )
        .await
        {
            Ok(Ok(())) => {
                println!("✅ {}: Write successful", tag_name);
            }
            Ok(Err(e)) => {
                eprintln!("❌ {}: Write failed - {}", tag_name, e);
            }
            Err(_) => {
                eprintln!("❌ {}: Write timed out", tag_name);
            }
        }
    }
    println!();

    // Small delay to ensure PLC processes the writes
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Step 3: Read back values to verify changes
    println!("📖 Step 3: Reading back values to verify changes");
    println!("------------------------------------------------");
    let mut read_verify_success_count = 0;
    let mut real_success_count = 0;
    let mut dint_success_count = 0;
    let mut bool_success_count = 0;

    for (tag_name, expected_value, _data_type) in &test_tags {
        match timeout(Duration::from_secs(5), client.read_tag(tag_name)).await {
            Ok(Ok(actual_value)) => {
                let success = match (expected_value, &actual_value) {
                    (PlcValue::Real(expected), PlcValue::Real(actual)) => {
                        if (actual - expected).abs() < FLOAT_TOLERANCE {
                            real_success_count += 1;
                            true
                        } else {
                            false
                        }
                    }
                    (PlcValue::Dint(expected), PlcValue::Dint(actual)) => {
                        if actual == expected {
                            dint_success_count += 1;
                            true
                        } else {
                            false
                        }
                    }
                    (PlcValue::Bool(expected), PlcValue::Bool(actual)) => {
                        if actual == expected {
                            bool_success_count += 1;
                            true
                        } else {
                            false
                        }
                    }
                    _ => false,
                };

                if success {
                    println!("✅ {}: {:?} ✓", tag_name, actual_value);
                    read_verify_success_count += 1;
                } else {
                    eprintln!(
                        "❌ {}: {:?} ✗ Expected: {:?}",
                        tag_name, actual_value, expected_value
                    );
                }
            }
            Ok(Err(e)) => {
                eprintln!("❌ {}: Read failed - {}", tag_name, e);
            }
            Err(_) => {
                eprintln!("❌ {}: Read timed out", tag_name);
            }
        }
    }
    println!();

    println!("📊 Test Results Summary");
    println!("======================");
    println!("Total tags tested: {}", test_tags.len());
    println!(
        "Successful write/read cycles: {}/{}",
        read_verify_success_count,
        test_tags.len()
    );
    println!();
    println!("📈 Data Type Breakdown:");
    println!("  REAL tags: {}/4 successful", real_success_count);
    println!("  DINT tags: {}/4 successful", dint_success_count);
    println!("  BOOL tags: {}/5 successful", bool_success_count);
    println!();

    if read_verify_success_count == test_tags.len() {
        println!("🎉 ALL TESTS PASSED: All data types written and read successfully!");
        println!("✅ REAL tags: Write/read working perfectly");
        println!("✅ DINT tags: Write/read working perfectly");
        println!("✅ BOOL tags: Write/read working perfectly");
    } else {
        eprintln!("❌ SOME TESTS FAILED: Not all tags could be written/read successfully");
        return Err("Multi-data-type write/read verification failed".into());
    }

    println!("\n⚡ Performance Notes:");
    println!("- Write operations should complete in <10ms");
    println!("- Read operations should complete in <5ms");
    println!("- Total test time should be <2 seconds for all operations");

    Ok(())
}
