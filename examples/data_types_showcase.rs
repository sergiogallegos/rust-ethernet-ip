// examples/data_types_showcase.rs
// =========================================================================
//
// Data Types Showcase for Allen-Bradley PLCs
//
// This example demonstrates all the data types supported by the
// rust-ethernet-ip library for CompactLogix and ControlLogix PLCs:
//
// - All integer types (signed and unsigned)
// - Floating point types (REAL and LREAL)
// - Boolean and string types
// - User Defined Types (UDTs)
//
// =========================================================================

use rust_ethernet_ip::{EipClient, PlcValue};
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("🎯 Allen-Bradley Data Types Showcase");
    println!("====================================");

    // Note: This example requires a CompactLogix or ControlLogix PLC
    // Update the IP address to match your PLC
    let plc_address = "192.168.1.100:44818";

    println!("📡 Connecting to PLC at {}", plc_address);

    // For demonstration purposes, we'll show data type operations without connecting
    // Uncomment the following line to actually connect to a PLC:
    // let mut client = EipClient::connect(plc_address).await?;

    demonstrate_data_types().await?;
    demonstrate_data_type_encoding().await?;

    // Uncomment to test with real PLC:
    // demonstrate_real_plc_data_types(&mut client).await?;

    Ok(())
}

/// Demonstrates all supported data types and their characteristics
async fn demonstrate_data_types() -> Result<(), Box<dyn Error>> {
    println!("\n📊 Supported Data Types");
    println!("=======================");

    // 1. Boolean Type
    println!("\n1. BOOL - Boolean Values:");
    let bool_true = PlcValue::Bool(true);
    let bool_false = PlcValue::Bool(false);
    println!(
        "   True:  {:?} -> CIP Type: 0x{:04X}",
        bool_true,
        bool_true.get_data_type()
    );
    println!(
        "   False: {:?} -> CIP Type: 0x{:04X}",
        bool_false,
        bool_false.get_data_type()
    );

    // 2. Signed Integer Types
    println!("\n2. Signed Integer Types:");

    let sint_val = PlcValue::Sint(-128);
    println!(
        "   SINT:  {:?} -> CIP Type: 0x{:04X} (Range: -128 to 127)",
        sint_val,
        sint_val.get_data_type()
    );

    let int_val = PlcValue::Int(-32768);
    println!(
        "   INT:   {:?} -> CIP Type: 0x{:04X} (Range: -32,768 to 32,767)",
        int_val,
        int_val.get_data_type()
    );

    let dint_val = PlcValue::Dint(-2147483648);
    println!(
        "   DINT:  {:?} -> CIP Type: 0x{:04X} (Range: -2,147,483,648 to 2,147,483,647)",
        dint_val,
        dint_val.get_data_type()
    );

    let lint_val = PlcValue::Lint(-9223372036854775808);
    println!("   LINT:  {:?} -> CIP Type: 0x{:04X} (Range: -9,223,372,036,854,775,808 to 9,223,372,036,854,775,807)", 
             lint_val, lint_val.get_data_type());

    // 3. Unsigned Integer Types
    println!("\n3. Unsigned Integer Types:");

    let usint_val = PlcValue::Usint(255);
    println!(
        "   USINT: {:?} -> CIP Type: 0x{:04X} (Range: 0 to 255)",
        usint_val,
        usint_val.get_data_type()
    );

    let uint_val = PlcValue::Uint(65535);
    println!(
        "   UINT:  {:?} -> CIP Type: 0x{:04X} (Range: 0 to 65,535)",
        uint_val,
        uint_val.get_data_type()
    );

    let udint_val = PlcValue::Udint(4294967295);
    println!(
        "   UDINT: {:?} -> CIP Type: 0x{:04X} (Range: 0 to 4,294,967,295)",
        udint_val,
        udint_val.get_data_type()
    );

    let ulint_val = PlcValue::Ulint(18446744073709551615);
    println!(
        "   ULINT: {:?} -> CIP Type: 0x{:04X} (Range: 0 to 18,446,744,073,709,551,615)",
        ulint_val,
        ulint_val.get_data_type()
    );

    // 4. Floating Point Types
    println!("\n4. Floating Point Types:");

    let real_val = PlcValue::Real(123.456);
    println!(
        "   REAL:  {:?} -> CIP Type: 0x{:04X} (32-bit IEEE 754)",
        real_val,
        real_val.get_data_type()
    );

    let lreal_val = PlcValue::Lreal(123.456789012345);
    println!(
        "   LREAL: {:?} -> CIP Type: 0x{:04X} (64-bit IEEE 754)",
        lreal_val,
        lreal_val.get_data_type()
    );

    // 5. String Type
    println!("\n5. String Type:");
    let string_val = PlcValue::String("Hello, PLC!".to_string());
    println!(
        "   STRING: {:?} -> CIP Type: 0x{:04X}",
        string_val,
        string_val.get_data_type()
    );

    // 6. User Defined Type (UDT)
    println!("\n6. User Defined Type (UDT):");
    println!("   Note: UDTs are now represented as UdtData with symbol_id and raw bytes");
    println!("   To create a UDT, you need to read from PLC or use UdtData::from_hash_map()");
    println!("   with a UDT definition. For demonstration, showing UDT type:");
    println!("   UDT CIP Type: 0x{:04X}", 0x00A0);

    Ok(())
}

/// Demonstrates byte encoding for network transmission
async fn demonstrate_data_type_encoding() -> Result<(), Box<dyn Error>> {
    println!("\n🔧 Data Type Encoding (Little-Endian)");
    println!("=====================================");

    // Show how each data type is encoded for network transmission
    let test_values = vec![
        ("BOOL(true)", PlcValue::Bool(true)),
        ("BOOL(false)", PlcValue::Bool(false)),
        ("SINT(-1)", PlcValue::Sint(-1)),
        ("INT(0x1234)", PlcValue::Int(0x1234)),
        ("DINT(0x12345678)", PlcValue::Dint(0x12345678)),
        ("USINT(255)", PlcValue::Usint(255)),
        ("UINT(0xABCD)", PlcValue::Uint(0xABCD)),
        ("UDINT(0xDEADBEEF)", PlcValue::Udint(0xDEADBEEF)),
        ("REAL(123.45)", PlcValue::Real(123.45)),
        ("STRING(\"Hi\")", PlcValue::String("Hi".to_string())),
    ];

    for (name, value) in test_values {
        let bytes = value.to_bytes();
        println!("   {:<20} -> {:02X?}", name, bytes);
    }

    // Show precision differences
    println!("\n🎯 Precision Comparison:");
    let real_pi = PlcValue::Real(std::f32::consts::PI);
    let lreal_pi = PlcValue::Lreal(std::f64::consts::PI);

    println!("   REAL π:  {:?}", real_pi);
    println!("   LREAL π: {:?}", lreal_pi);

    Ok(())
}

/// Demonstrates real PLC operations with all data types (uncomment when connected to PLC)
#[allow(dead_code)]
async fn demonstrate_real_plc_data_types(client: &mut EipClient) -> Result<(), Box<dyn Error>> {
    println!("\n🏭 Real PLC Data Type Operations");
    println!("===============================");

    // Test writing different data types
    println!("\n✏️ Writing Different Data Types:");

    // Boolean operations
    match client.write_tag("TestBool", PlcValue::Bool(true)).await {
        Ok(_) => println!("   ✅ BOOL written successfully"),
        Err(e) => println!("   ❌ BOOL write failed: {}", e),
    }

    // Integer operations
    match client.write_tag("TestSint", PlcValue::Sint(-100)).await {
        Ok(_) => println!("   ✅ SINT written successfully"),
        Err(e) => println!("   ❌ SINT write failed: {}", e),
    }

    match client.write_tag("TestInt", PlcValue::Int(-30000)).await {
        Ok(_) => println!("   ✅ INT written successfully"),
        Err(e) => println!("   ❌ INT write failed: {}", e),
    }

    match client.write_tag("TestDint", PlcValue::Dint(-2000000)).await {
        Ok(_) => println!("   ✅ DINT written successfully"),
        Err(e) => println!("   ❌ DINT write failed: {}", e),
    }

    match client
        .write_tag("TestLint", PlcValue::Lint(-9000000000000000000))
        .await
    {
        Ok(_) => println!("   ✅ LINT written successfully"),
        Err(e) => println!("   ❌ LINT write failed: {}", e),
    }

    // Unsigned integer operations
    match client.write_tag("TestUsint", PlcValue::Usint(200)).await {
        Ok(_) => println!("   ✅ USINT written successfully"),
        Err(e) => println!("   ❌ USINT write failed: {}", e),
    }

    match client.write_tag("TestUint", PlcValue::Uint(50000)).await {
        Ok(_) => println!("   ✅ UINT written successfully"),
        Err(e) => println!("   ❌ UINT write failed: {}", e),
    }

    match client
        .write_tag("TestUdint", PlcValue::Udint(3000000000))
        .await
    {
        Ok(_) => println!("   ✅ UDINT written successfully"),
        Err(e) => println!("   ❌ UDINT write failed: {}", e),
    }

    match client
        .write_tag("TestUlint", PlcValue::Ulint(15000000000000000000))
        .await
    {
        Ok(_) => println!("   ✅ ULINT written successfully"),
        Err(e) => println!("   ❌ ULINT write failed: {}", e),
    }

    // Floating point operations
    match client.write_tag("TestReal", PlcValue::Real(123.456)).await {
        Ok(_) => println!("   ✅ REAL written successfully"),
        Err(e) => println!("   ❌ REAL write failed: {}", e),
    }

    match client
        .write_tag("TestLreal", PlcValue::Lreal(123.456789012345))
        .await
    {
        Ok(_) => println!("   ✅ LREAL written successfully"),
        Err(e) => println!("   ❌ LREAL write failed: {}", e),
    }

    // String operations
    match client
        .write_tag(
            "TestString",
            PlcValue::String("Hello from Rust!".to_string()),
        )
        .await
    {
        Ok(_) => println!("   ✅ STRING written successfully"),
        Err(e) => println!("   ❌ STRING write failed: {}", e),
    }

    // UDT operations
    // To write a UDT, first read it to get the symbol_id, then modify and write back
    match client.read_tag("TestUdt").await {
        Ok(PlcValue::Udt(udt_data)) => {
            // Modify the UDT data if needed, then write back
            match client.write_tag("TestUdt", PlcValue::Udt(udt_data)).await {
                Ok(_) => println!("   ✅ UDT written successfully"),
                Err(e) => println!("   ❌ UDT write failed: {}", e),
            }
        }
        Ok(_) => println!("   ⚠️  TestUdt is not a UDT type"),
        Err(e) => println!("   ❌ UDT read failed: {}", e),
    }

    // Test reading back the values
    println!("\n📖 Reading Back Values:");

    let tags_to_read = vec![
        "TestBool",
        "TestSint",
        "TestInt",
        "TestDint",
        "TestLint",
        "TestUsint",
        "TestUint",
        "TestUdint",
        "TestUlint",
        "TestReal",
        "TestLreal",
        "TestString",
    ];

    for tag in tags_to_read {
        match client.read_tag(tag).await {
            Ok(value) => println!("   {} = {:?}", tag, value),
            Err(e) => println!("   {} read failed: {}", tag, e),
        }
    }

    Ok(())
}

/// Demonstrates data type conversion and validation
#[allow(dead_code)]
async fn demonstrate_data_type_validation() -> Result<(), Box<dyn Error>> {
    println!("\n🔍 Data Type Validation");
    println!("=======================");

    // Test boundary values
    println!("\n📏 Boundary Value Testing:");

    // SINT boundaries
    let sint_min = PlcValue::Sint(i8::MIN);
    let sint_max = PlcValue::Sint(i8::MAX);
    println!("   SINT min: {:?}, max: {:?}", sint_min, sint_max);

    // INT boundaries
    let int_min = PlcValue::Int(i16::MIN);
    let int_max = PlcValue::Int(i16::MAX);
    println!("   INT min: {:?}, max: {:?}", int_min, int_max);

    // DINT boundaries
    let dint_min = PlcValue::Dint(i32::MIN);
    let dint_max = PlcValue::Dint(i32::MAX);
    println!("   DINT min: {:?}, max: {:?}", dint_min, dint_max);

    // REAL precision
    let real_small = PlcValue::Real(1.0e-38);
    let real_large = PlcValue::Real(3.4e38);
    println!("   REAL small: {:?}, large: {:?}", real_small, real_large);

    // LREAL precision
    let lreal_small = PlcValue::Lreal(2.2e-308);
    let lreal_large = PlcValue::Lreal(1.7e308);
    println!(
        "   LREAL small: {:?}, large: {:?}",
        lreal_small, lreal_large
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_data_type_demonstrations() {
        // Test that all demonstrations work without errors
        assert!(demonstrate_data_types().await.is_ok());
        assert!(demonstrate_data_type_encoding().await.is_ok());
    }

    #[test]
    fn test_all_data_types_have_correct_cip_codes() {
        // Verify that all data types have the correct CIP type codes
        assert_eq!(PlcValue::Bool(true).get_data_type(), 0x00C1);
        assert_eq!(PlcValue::Sint(0).get_data_type(), 0x00C2);
        assert_eq!(PlcValue::Int(0).get_data_type(), 0x00C3);
        assert_eq!(PlcValue::Dint(0).get_data_type(), 0x00C4);
        assert_eq!(PlcValue::Lint(0).get_data_type(), 0x00C5);
        assert_eq!(PlcValue::Usint(0).get_data_type(), 0x00C6);
        assert_eq!(PlcValue::Uint(0).get_data_type(), 0x00C7);
        assert_eq!(PlcValue::Udint(0).get_data_type(), 0x00C8);
        assert_eq!(PlcValue::Ulint(0).get_data_type(), 0x00C9);
        assert_eq!(PlcValue::Real(0.0).get_data_type(), 0x00CA);
        assert_eq!(PlcValue::Lreal(0.0).get_data_type(), 0x00CB);
        assert_eq!(PlcValue::String("".to_string()).get_data_type(), 0x00DA);
        use rust_ethernet_ip::UdtData;
        assert_eq!(PlcValue::Udt(UdtData { symbol_id: 0, data: vec![] }).get_data_type(), 0x00A0);
    }

    #[test]
    fn test_data_type_encoding_consistency() {
        // Test that encoding is consistent and correct

        // Test little-endian encoding for multi-byte types
        let int_val = PlcValue::Int(0x1234);
        assert_eq!(int_val.to_bytes(), vec![0x34, 0x12]);

        let dint_val = PlcValue::Dint(0x12345678);
        assert_eq!(dint_val.to_bytes(), vec![0x78, 0x56, 0x34, 0x12]);

        let uint_val = PlcValue::Uint(0xABCD);
        assert_eq!(uint_val.to_bytes(), vec![0xCD, 0xAB]);

        let udint_val = PlcValue::Udint(0xDEADBEEF);
        assert_eq!(udint_val.to_bytes(), vec![0xEF, 0xBE, 0xAD, 0xDE]);

        // Test string encoding
        let string_val = PlcValue::String("Hi".to_string());
        assert_eq!(string_val.to_bytes(), vec![2, b'H', b'i']);
    }
}
