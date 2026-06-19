// examples/advanced_tag_addressing.rs
// =========================================================================
//
// Advanced Tag Addressing Example for Allen-Bradley PLCs
//
// This example demonstrates the comprehensive tag addressing capabilities
// of the rust-ethernet-ip library, including:
//
// - Program-scoped tags
// - Array element access
// - Bit-level operations
// - UDT member access
// - String operations
// - Complex nested paths
//
// =========================================================================

use rust_ethernet_ip::{EipClient, PlcValue, TagPath};
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("🚀 Advanced Tag Addressing Example");
    println!("==================================");

    // Note: This example requires a CompactLogix or ControlLogix PLC
    // Update the IP address to match your PLC
    let plc_address = "192.168.1.100:44818";

    println!("📡 Connecting to PLC at {}", plc_address);

    // For demonstration purposes, we'll show tag path parsing without connecting
    // Uncomment the following line to actually connect to a PLC:
    // let mut client = EipClient::connect(plc_address).await?;

    demonstrate_tag_path_parsing().await?;

    // Uncomment to test with real PLC:
    // demonstrate_real_plc_operations(&mut client).await?;

    Ok(())
}

/// Demonstrates tag path parsing capabilities
async fn demonstrate_tag_path_parsing() -> Result<(), Box<dyn Error>> {
    println!("\n🔍 Tag Path Parsing Demonstration");
    println!("=================================");

    // 1. Controller-scoped tags (traditional)
    println!("\n1. Controller-Scoped Tags:");
    let simple_tag = TagPath::parse("MotorRunning")?;
    println!("   Simple tag: {} -> {}", simple_tag, simple_tag);

    // 2. Program-scoped tags (CompactLogix/ControlLogix)
    println!("\n2. Program-Scoped Tags:");
    let program_tag = TagPath::parse("Program:MainProgram.ConveyorSpeed")?;
    println!(
        "   Program tag: {} -> Program: {:?}, Base: {}",
        program_tag,
        program_tag.program_name(),
        program_tag.base_tag_name()
    );

    let safety_tag = TagPath::parse("Program:Safety.EmergencyStop")?;
    println!(
        "   Safety tag: {} -> Program: {:?}",
        safety_tag,
        safety_tag.program_name()
    );

    // 3. Array element access
    println!("\n3. Array Element Access:");
    let array_element = TagPath::parse("SensorReadings[5]")?;
    println!("   Single dimension: {}", array_element);

    let matrix_element = TagPath::parse("Program:Vision.ImageData[10,20,3]")?;
    println!("   Multi-dimensional: {}", matrix_element);

    // 4. Bit-level operations
    println!("\n4. Bit-Level Operations:");
    let status_bit = TagPath::parse("StatusWord.15")?;
    println!("   Status bit: {}", status_bit);

    let program_bit = TagPath::parse("Program:IO.InputBank.7")?;
    println!("   Program bit: {}", program_bit);

    // 5. UDT member access
    println!("\n5. UDT Member Access:");
    let motor_speed = TagPath::parse("Motor1.Speed")?;
    println!("   Motor speed: {}", motor_speed);

    let nested_udt = TagPath::parse("Recipe.Step1.Temperature.Setpoint")?;
    println!("   Nested UDT: {}", nested_udt);

    let program_udt = TagPath::parse("Program:Process.Tank1.Level.Current")?;
    println!("   Program UDT: {}", program_udt);

    // 6. String operations
    println!("\n6. String Operations:");
    let string_length = TagPath::parse("ProductName.LEN")?;
    println!("   String length: {}", string_length);

    let string_char = TagPath::parse("ProductName.DATA[5]")?;
    println!("   String character: {}", string_char);

    // 7. Complex nested paths
    println!("\n7. Complex Nested Paths:");
    let complex_path = TagPath::parse("Program:Production.Lines[2].Stations[5].Motor.Status.15")?;
    println!("   Complex path: {}", complex_path);
    println!("   Base tag: {}", complex_path.base_tag_name());
    println!("   Is program-scoped: {}", complex_path.is_program_scoped());

    // 8. CIP path generation
    println!("\n8. CIP Path Generation:");
    let cip_path = complex_path.to_cip_path()?;
    println!("   CIP bytes: {:02X?}", cip_path);

    Ok(())
}

/// Demonstrates real PLC operations (uncomment when connected to PLC)
#[allow(dead_code)]
async fn demonstrate_real_plc_operations(client: &mut EipClient) -> Result<(), Box<dyn Error>> {
    println!("\n🏭 Real PLC Operations");
    println!("=====================");

    // Read program-scoped tags
    println!("\n📖 Reading Program-Scoped Tags:");

    // Example: Read a boolean from main program
    match client.read_tag("Program:MainProgram.MotorRunning").await {
        Ok(PlcValue::Bool(value)) => {
            println!("   Motor running: {}", value);
        }
        Ok(other) => println!("   Unexpected type: {:?}", other),
        Err(e) => println!("   Error reading motor status: {}", e),
    }

    // Example: Read an integer from a program
    match client.read_tag("Program:MainProgram.ProductionCount").await {
        Ok(PlcValue::Dint(value)) => {
            println!("   Production count: {}", value);
        }
        Ok(other) => println!("   Unexpected type: {:?}", other),
        Err(e) => println!("   Error reading production count: {}", e),
    }

    // Array operations
    println!("\n📊 Array Operations:");

    // Read array element
    match client
        .read_tag("Program:MainProgram.SensorReadings[0]")
        .await
    {
        Ok(value) => println!("   Sensor 0: {:?}", value),
        Err(e) => println!("   Error reading sensor: {}", e),
    }

    // Bit operations
    println!("\n🔢 Bit Operations:");

    // Read specific bit
    match client.read_tag("Program:MainProgram.StatusWord.5").await {
        Ok(PlcValue::Bool(value)) => {
            println!("   Status bit 5: {}", value);
        }
        Ok(other) => println!("   Unexpected type: {:?}", other),
        Err(e) => println!("   Error reading bit: {}", e),
    }

    // UDT operations
    println!("\n🏗️ UDT Operations:");

    // Read UDT member
    match client.read_tag("Program:MainProgram.Motor1.Speed").await {
        Ok(value) => println!("   Motor speed: {:?}", value),
        Err(e) => println!("   Error reading motor speed: {}", e),
    }

    // Write operations
    println!("\n✏️ Write Operations:");

    // Write to program-scoped tag
    match client
        .write_tag("Program:MainProgram.SetPoint", PlcValue::Dint(1500))
        .await
    {
        Ok(_) => println!("   ✅ Set point updated to 1500"),
        Err(e) => println!("   ❌ Error writing set point: {}", e),
    }

    // Write to array element
    match client
        .write_tag("Program:MainProgram.Setpoints[0]", PlcValue::Real(72.5))
        .await
    {
        Ok(_) => println!("   ✅ Setpoint[0] updated to 72.5"),
        Err(e) => println!("   ❌ Error writing setpoint: {}", e),
    }

    // Write to bit
    match client
        .write_tag("Program:MainProgram.ControlWord.3", PlcValue::Bool(true))
        .await
    {
        Ok(_) => println!("   ✅ Control bit 3 set to true"),
        Err(e) => println!("   ❌ Error writing bit: {}", e),
    }

    Ok(())
}

/// Demonstrates tag path validation and error handling
#[allow(dead_code)]
async fn demonstrate_error_handling() -> Result<(), Box<dyn Error>> {
    println!("\n⚠️ Error Handling Examples");
    println!("==========================");

    // Invalid tag paths
    let invalid_paths = vec![
        "",                       // Empty path
        "Program:",               // Incomplete program path
        "MyArray[",               // Unclosed bracket
        "MyArray]",               // Missing opening bracket
        "MyTag.",                 // Trailing dot
        "Program:Main.Tag[1,2,]", // Trailing comma
        "MyTag.99",               // Invalid bit index (>31)
    ];

    for invalid_path in invalid_paths {
        match TagPath::parse(invalid_path) {
            Ok(_) => println!("   ❌ Unexpectedly parsed: '{}'", invalid_path),
            Err(e) => println!("   ✅ Correctly rejected '{}': {}", invalid_path, e),
        }
    }

    Ok(())
}

/// Demonstrates performance characteristics
#[allow(dead_code)]
async fn demonstrate_performance() -> Result<(), Box<dyn Error>> {
    println!("\n⚡ Performance Demonstration");
    println!("============================");

    let test_paths = vec![
        "SimpleTag",
        "Program:MainProgram.ComplexTag",
        "Program:Production.Lines[5].Stations[10].Motors[2].Status.15",
        "Recipe.Steps[1].Parameters.Temperature.Setpoint",
        "Program:Safety.Zones[3].Devices[7].Inputs.Emergency.LEN",
    ];

    let start = std::time::Instant::now();

    for _ in 0..1000 {
        for path_str in &test_paths {
            let _path = TagPath::parse(path_str)?;
        }
    }

    let duration = start.elapsed();
    let ops_per_sec = (1000 * test_paths.len()) as f64 / duration.as_secs_f64();

    println!(
        "   Parsed {} paths in {:?}",
        1000 * test_paths.len(),
        duration
    );
    println!("   Performance: {:.0} parses/second", ops_per_sec);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_tag_path_examples() {
        // Test that all examples in the demonstration work
        assert!(demonstrate_tag_path_parsing().await.is_ok());
    }

    #[test]
    fn test_specific_tag_patterns() {
        // Test specific patterns that are common in industrial applications

        // HMI tags
        assert!(TagPath::parse("HMI_StartButton").is_ok());
        assert!(TagPath::parse("Program:HMI.Screens[1].Buttons[5].Pressed").is_ok());

        // Motion control
        assert!(TagPath::parse("Program:Motion.Axis1.Position").is_ok());
        assert!(TagPath::parse("Program:Motion.Axis1.Status.15").is_ok());

        // Safety systems
        assert!(TagPath::parse("Program:Safety.Zone1.LightCurtain.Broken").is_ok());
        assert!(TagPath::parse("Program:Safety.EmergencyStops[3].Pressed").is_ok());

        // Process control
        assert!(TagPath::parse("Program:Process.Tank1.Level.PV").is_ok());
        assert!(TagPath::parse("Program:Process.Pumps[2].Speed.SP").is_ok());

        // Recipe management
        assert!(TagPath::parse("Program:Recipe.Active.Steps[5].Time").is_ok());
        assert!(TagPath::parse("Program:Recipe.Library[10].Name.LEN").is_ok());
    }
}
