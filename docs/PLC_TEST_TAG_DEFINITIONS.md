# PLC Test Tag Definitions for Array and UDT Testing

**Target PLC:** ControlLogix 1756-L75  
**Ethernet Module:** 1756-EN2T (Slot 1)  
**IP Address:** 192.168.0.1

## Test Structure Overview

This document provides tag definitions to create in your PLC for comprehensive testing of:
- Array element addressing (8-bit, 16-bit, 32-bit indices)
- UDT structures with various data types
- UDT with array members
- Controller-scoped and Program-scoped tags

The full-coverage hardware exercisers use [`../examples/full_coverage_tags.json`](../examples/full_coverage_tags.json) as the machine-readable mirror of this layout. Keep this document human-first; update the JSON manifest when adding or reclassifying exercised tags.

---

## 1. Controller-Scoped Test Tags

### 1.1 Simple Arrays (Controller Scope)

Create these tags directly in the Controller Tags:

#### `gTestArray_DINT` (DINT Array)
- **Type:** DINT[100]
- **Purpose:** Test DINT array element addressing
- **Initial Values:** Set elements 0-9 to: 10, 20, 30, 40, 50, 60, 70, 80, 90, 100

#### `gTestArray_REAL` (REAL Array)
- **Type:** REAL[50]
- **Purpose:** Test REAL array element addressing
- **Initial Values:** Set elements 0-9 to: 1.1, 2.2, 3.3, 4.4, 5.5, 6.6, 7.7, 8.8, 9.9, 10.0

#### `gTestArray_BOOL` (BOOL Array)
- **Type:** BOOL[100]
- **Purpose:** Test BOOL array element addressing
- **Initial Values:** Set elements 0-9 to alternating: TRUE, FALSE, TRUE, FALSE, etc.

#### `gTestArray_INT` (INT Array)
- **Type:** INT[200]
- **Purpose:** Test 16-bit array element addressing (indices > 255)
- **Initial Values:** Set elements 0-9 to: 100, 200, 300, 400, 500, 600, 700, 800, 900, 1000

#### `gTestArray_Large` (DINT Array)
- **Type:** DINT[1000]
- **Purpose:** Test large array and 16-bit element addressing
- **Initial Values:** Set elements 0-9 to: 1, 2, 3, 4, 5, 6, 7, 8, 9, 10

---

### 1.2 UDT Definition: `TEST_UDT`

Create this User-Defined Type in the Controller:

```
TEST_UDT:
├── Member1_DINT      : DINT        (Offset: 0)
├── Member2_REAL     : REAL        (Offset: 4)
├── Member3_BOOL      : BOOL        (Offset: 8)
├── Member4_INT       : INT         (Offset: 10)
├── Member5_String    : STRING[82]  (Offset: 12)
├── Array_DINT        : DINT[10]    (Offset: 96)
├── Array_REAL        : REAL[5]     (Offset: 136)
└── Array_BOOL        : BOOL[20]    (Offset: 156)
```

**Total Size:** ~176 bytes (with padding)

**Initial Values:**
- Member1_DINT: 100
- Member2_REAL: 3.14159
- Member3_BOOL: TRUE
- Member4_INT: 42
- Member5_String: "Hello PLC"
- Array_DINT[0-9]: 1, 2, 3, 4, 5, 6, 7, 8, 9, 10
- Array_REAL[0-4]: 1.1, 2.2, 3.3, 4.4, 5.5
- Array_BOOL[0-19]: Alternating TRUE/FALSE

---

### 1.3 UDT Instance Tags (Controller Scope)

#### `gTestUDT` (TEST_UDT)
- **Type:** TEST_UDT
- **Purpose:** Test complete UDT read/write
- **Initial Values:** As defined in TEST_UDT above

#### `gTestUDT_Array` (TEST_UDT Array)
- **Type:** TEST_UDT[10]
- **Purpose:** Test array of UDTs
- **Initial Values:** 
  - Element 0: Member1_DINT = 100, Member2_REAL = 1.1
  - Element 1: Member1_DINT = 200, Member2_REAL = 2.2
  - Element 2: Member1_DINT = 300, Member2_REAL = 3.3
  - (Continue pattern for elements 3-9)

---

## 2. Program-Scoped Test Tags

Create a program named `TestProgram` and add these tags:

### 2.1 Simple Arrays (Program Scope)

#### `Program:TestProgram.gTestArray_DINT` (DINT Array)
- **Type:** DINT[100]
- **Initial Values:** Set elements 0-9 to: 1000, 2000, 3000, 4000, 5000, 6000, 7000, 8000, 9000, 10000

#### `Program:TestProgram.gTestArray_REAL` (REAL Array)
- **Type:** REAL[50]
- **Initial Values:** Set elements 0-9 to: 10.1, 20.2, 30.3, 40.4, 50.5, 60.6, 70.7, 80.8, 90.9, 100.0

#### `Program:TestProgram.gTestArray_BOOL` (BOOL Array)
- **Type:** BOOL[100]
- **Initial Values:** Set elements 0-9 to: FALSE, TRUE, FALSE, TRUE, etc.

---

### 2.2 UDT Instance Tags (Program Scope)

#### `Program:TestProgram.gTestUDT` (TEST_UDT)
- **Type:** TEST_UDT
- **Purpose:** Test program-scoped UDT
- **Initial Values:** 
  - Member1_DINT: 500
  - Member2_REAL: 2.71828
  - Member3_BOOL: FALSE
  - Member4_INT: 24
  - Member5_String: "Program UDT"
  - Arrays: Similar pattern as controller-scoped

#### `Program:TestProgram.gTestUDT_Array` (TEST_UDT Array)
- **Type:** TEST_UDT[5]
- **Purpose:** Test program-scoped array of UDTs
- **Initial Values:** Similar pattern as controller-scoped

---

## 3. Test Scenarios

### 3.1 Array Element Addressing Tests

#### Test 1: Single Element Read (8-bit index)
```
Read: gTestArray_DINT[5]
Expected: 60 (or current value)
```

#### Test 2: Single Element Write (8-bit index)
```
Write: gTestArray_DINT[5] = 999
Read back: gTestArray_DINT[5]
Expected: 999
```

#### Test 3: Single Element Read (16-bit index)
```
Read: gTestArray_Large[300]
Expected: Current value at index 300
```

#### Test 4: Single Element Write (16-bit index)
```
Write: gTestArray_Large[300] = 12345
Read back: gTestArray_Large[300]
Expected: 12345
```

#### Test 5: Range Read
```
Read: gTestArray_DINT[10] through gTestArray_DINT[14]
Expected: 5 consecutive elements
```

#### Test 6: BOOL Array Element
```
Read: gTestArray_BOOL[15]
Write: gTestArray_BOOL[15] = TRUE
Read back: gTestArray_BOOL[15]
Expected: TRUE
```

---

### 3.2 UDT Tests

#### Test 7: Complete UDT Read
```
Read: gTestUDT
Expected: UdtData with symbol_id and all member bytes
```

#### Test 8: Complete UDT Write
```
Read: gTestUDT (to get symbol_id)
Modify: Change Member1_DINT in raw bytes
Write: gTestUDT with modified bytes
Read back: Verify change
```

#### Test 9: UDT Member Access
```
Read: gTestUDT.Member1_DINT
Expected: 100 (or current value)
```

#### Test 10: UDT Array Member Access
```
Read: gTestUDT.Array_DINT[5]
Expected: 6 (or current value)
Write: gTestUDT.Array_DINT[5] = 99
Read back: Verify change
```

#### Test 11: Array of UDTs - Single Element
```
Read: gTestUDT_Array[3]
Expected: Complete UDT structure at index 3
```

#### Test 12: Array of UDTs - Member Access
```
Read: gTestUDT_Array[3].Member1_DINT
Expected: Value from element 3
Write: gTestUDT_Array[3].Member1_DINT = 777
Read back: Verify change
```

#### Test 13: Array of UDTs - Array Member Access
```
Read: gTestUDT_Array[2].Array_DINT[4]
Expected: Value from nested array
Write: gTestUDT_Array[2].Array_DINT[4] = 888
Read back: Verify change
```

---

### 3.3 Program-Scoped Tests

#### Test 14: Program-Scoped Array
```
Read: Program:TestProgram.gTestArray_DINT[5]
Write: Program:TestProgram.gTestArray_DINT[5] = 5555
Read back: Verify change
```

#### Test 15: Program-Scoped UDT
```
Read: Program:TestProgram.gTestUDT
Write: Program:TestProgram.gTestUDT (with modifications)
Read back: Verify changes
```

#### Test 16: Program-Scoped UDT Array
```
Read: Program:TestProgram.gTestUDT_Array[2].Member2_REAL
Write: Program:TestProgram.gTestUDT_Array[2].Member2_REAL = 99.99
Read back: Verify change
```

---

## 4. Creating Tags in Studio 5000

### Step-by-Step Instructions

1. **Open Studio 5000** and connect to your PLC (192.168.0.1)

2. **Create UDT Definition:**
   - Right-click on "User-Defined" → "New Data Type"
   - Name: `TEST_UDT`
   - Add members as specified above
   - Click "OK"

3. **Create Controller Tags:**
   - Go to "Controller Tags"
   - Right-click → "New Tag"
   - Create each tag with the specified type and name
   - Set initial values as specified

4. **Create Program:**
   - Right-click on "Programs" → "New Program"
   - Name: `TestProgram`
   - Click "OK"

5. **Create Program Tags:**
   - Open "TestProgram" → "Program Tags"
   - Right-click → "New Tag"
   - Create each tag with the specified type and name
   - Set initial values as specified

6. **Download to PLC:**
   - Click "Download" button
   - Wait for download to complete

---

## 5. Test Script Template

Once tags are created, you can use this Rust code to test:

```rust
use rust_ethernet_ip::{EipClient, PlcValue};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = EipClient::connect("192.168.0.1:44818").await?;
    
    // Test 1: Read single array element
    let value = client.read_tag("gTestArray_DINT[5]").await?;
    println!("Read gTestArray_DINT[5]: {:?}", value);
    
    // Test 2: Write single array element
    client.write_tag("gTestArray_DINT[5]", PlcValue::Dint(999)).await?;
    let value = client.read_tag("gTestArray_DINT[5]").await?;
    println!("Read back gTestArray_DINT[5]: {:?}", value);
    
    // Test 3: Read UDT
    let udt = client.read_tag("gTestUDT").await?;
    if let PlcValue::Udt(udt_data) = udt {
        println!("UDT symbol_id: {}", udt_data.symbol_id);
        println!("UDT data length: {} bytes", udt_data.data.len());
    }
    
    // Test 4: Read UDT member
    let member = client.read_tag("gTestUDT.Member1_DINT").await?;
    println!("gTestUDT.Member1_DINT: {:?}", member);
    
    // Test 5: Read UDT array member
    let array_member = client.read_tag("gTestUDT.Array_DINT[5]").await?;
    println!("gTestUDT.Array_DINT[5]: {:?}", array_member);
    
    // Test 6: Read array of UDTs
    let udt_array_elem = client.read_tag("gTestUDT_Array[3]").await?;
    println!("gTestUDT_Array[3]: {:?}", udt_array_elem);
    
    // Test 7: Program-scoped tag
    let prog_value = client.read_tag("Program:TestProgram.gTestArray_DINT[5]").await?;
    println!("Program tag: {:?}", prog_value);
    
    Ok(())
}
```

---

## 6. Expected Results

### Array Tests
- ✅ Single element read/write should work without reading entire array
- ✅ 16-bit index (300) should use 0x29 element segment
- ✅ 8-bit index (5) should use 0x28 element segment
- ✅ Range reads should use element addressing with start_index + count

### UDT Tests
- ✅ Complete UDT read should return `UdtData` with valid `symbol_id`
- ✅ UDT write should use `0x02A0 + symbol_id` as data type
- ✅ Member access should work correctly
- ✅ Array members within UDT should work
- ✅ Arrays of UDTs should work

### Program-Scoped Tests
- ✅ Program tags should be accessible with `Program:ProgramName.TagName` format
- ✅ All array and UDT operations should work for program-scoped tags

---

## 7. Troubleshooting

### If tags don't exist:
- Verify tag names match exactly (case-sensitive)
- Check that tags are downloaded to PLC
- Verify program name is correct for program-scoped tags

### If reads fail:
- Check network connectivity (ping 192.168.0.1)
- Verify EtherNet/IP module is in Slot 1
- Check CIP path if using route path

### If writes fail:
- Verify tag is not protected/read-only
- Check that data type matches
- For UDTs, ensure symbol_id is correct

---

**Ready for Testing!** 🚀

Create these tags in your PLC and we can run comprehensive tests to verify the array and UDT implementations work correctly with your ControlLogix 1756-L75.
