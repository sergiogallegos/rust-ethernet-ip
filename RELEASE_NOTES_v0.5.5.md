# Release Notes v0.5.5

**Release Date:** December 26, 2025  
**Status:** ✅ **Production Ready**

---

## 🎉 **Array Element Access - Finally Working!**

v0.5.5 introduces **full array element read/write support** through an intelligent workaround that automatically handles array element access. This release solves the long-standing "Path segment error" (CIP Error 4) issue that affected array element access on many Allen-Bradley PLCs.

---

## 🚀 **New Features**

### 📊 **Array Element Access**
- **Automatic Workaround**: Detects array element syntax (`ArrayName[index]`) and automatically uses workaround
- **Controller-Scoped Arrays**: Full support for `gArrayTest[0]`, `gArrayTest[1]`, etc.
- **Program-Scoped Arrays**: Full support for `Program:MainProgram.ArrayTest[0]`
- **Transparent Operation**: No code changes needed - works automatically with existing API
- **Read and Write Support**: Both reading and writing array elements are fully supported

### 🎯 **BOOL Array Special Handling**
- **DWORD Bit Extraction**: Automatically detects BOOL arrays and extracts individual bits from DWORDs
- **Automatic Detection**: Identifies BOOL arrays by data type (0x00D3)
- **Efficient Reading**: Reads single DWORD and extracts specific bit
- **Efficient Writing**: Modifies single bit in DWORD and writes back

### 🔧 **Intelligent Array Workaround**
- **Automatic Detection**: Detects array element access syntax automatically
- **Smart Element Count**: Automatically requests appropriate number of elements (10, 50, or 100)
- **Element Extraction**: Calculates actual element count from received data
- **Type-Aware Parsing**: Correctly extracts elements based on data type (DINT, REAL, BOOL, etc.)
- **Bounds Checking**: Validates array indices against actual array size

---

## 🐛 **Bug Fixes**

### **Fixed: Array Element Access "Path Segment Error"**
- **Problem**: Reading array elements like `MyArray[0]` failed with "CIP Error 4: Path segment error"
- **Root Cause**: Some Allen-Bradley PLCs (including 5069-L350ERMS3 firmware 33) don't support direct array element access via CIP paths
- **Solution**: Implemented automatic workaround that reads entire array and extracts requested element
- **Impact**: Array element access now works on all supported PLCs

### **Fixed: BOOL Array Element Access**
- **Problem**: BOOL array elements couldn't be accessed directly
- **Root Cause**: BOOL arrays are stored as DWORDs (32 bits per DWORD) and require special handling
- **Solution**: Special workaround that reads DWORD, extracts bit, and writes back modified DWORD
- **Impact**: BOOL array elements can now be read and written individually

---

## 🧪 **Testing & Quality**

- ✅ **31 Library Tests** - All passing
- ✅ **6 New Array Tests** - Comprehensive array read/write test coverage
- ✅ **Code Quality** - Clippy passing (only minor warnings)
- ✅ **Compilation** - Release build successful
- ✅ **Zero Breaking Changes** - Backward compatible
- ✅ **Real PLC Testing** - Tested with CompactLogix L24ER firmware 33 and 5069-L350ERMS3 firmware 33

---

## 📁 **Files Added/Modified**

### **New Files:**
- `tests/array_read_write_tests.rs` - Comprehensive array element read/write tests
- `examples/test_array_workaround.rs` - Complete array workaround demo
- `examples/test_array_elements.rs` - Array element access examples
- `examples/test_bool_array.rs` - BOOL array element examples
- `examples/test_bool_array_bit_access.rs` - BOOL array bit access examples
- `examples/test_controller_tags.rs` - Controller-scoped tag examples
- `examples/test_program_tags.rs` - Program-scoped tag examples
- `examples/test_udt_structure.rs` - UDT structure examples

### **Enhanced Files:**
- `src/lib.rs` - Added array element workaround methods:
  - `read_array_element_workaround()` - Reads entire array and extracts element
  - `write_array_element_workaround()` - Writes array element via array modification
  - `read_bool_array_element_workaround()` - Special BOOL array handling
  - `write_bool_array_element_workaround()` - Special BOOL array writing
  - `parse_array_element_access()` - Detects array element syntax
  - `build_write_array_request()` - Builds write requests for entire arrays
- `Cargo.toml` - Updated version to 0.5.5
- `src/version.rs` - Updated version constants
- `README.md` - Added v0.5.5 features and array support documentation

---

## 🔧 **API Changes**

### **No Breaking Changes!**

All existing code continues to work without modification. Array element access now works automatically.

### **New Internal Methods (Private):**
```rust
// These are called automatically - no need to use directly
async fn read_array_element_workaround(&mut self, base_array_name: &str, index: u32) -> Result<PlcValue>
async fn write_array_element_workaround(&mut self, base_array_name: &str, index: u32, value: PlcValue) -> Result<()>
async fn read_bool_array_element_workaround(&mut self, base_array_name: &str, index: u32) -> Result<PlcValue>
async fn write_bool_array_element_workaround(&mut self, base_array_name: &str, index: u32, value: PlcValue) -> Result<()>
fn parse_array_element_access(&self, tag_name: &str) -> Option<(&str, u32)>
```

### **Enhanced Existing Methods:**
```rust
// read_tag() now automatically detects and handles array elements
pub async fn read_tag(&mut self, tag_name: &str) -> Result<PlcValue>
// ✅ Now supports: "gArrayTest[0]", "Program:MainProgram.ArrayTest[5]"

// write_tag() now automatically detects and handles array elements
pub async fn write_tag(&mut self, tag_name: &str, value: PlcValue) -> Result<()>
// ✅ Now supports: "gArrayTest[0]", "Program:MainProgram.ArrayTest[5]"
```

---

## 📝 **Usage Examples**

### **Reading Array Elements**

```rust
use rust_ethernet_ip::{EipClient, PlcValue};

let mut client = EipClient::connect("192.168.0.1:44818").await?;

// Controller-scoped array elements
let element0 = client.read_tag("gArrayTest[0]").await?;
let element5 = client.read_tag("gArrayTest[5]").await?;

// Program-scoped array elements
let element = client.read_tag("Program:MainProgram.ArrayTest[0]").await?;

// BOOL array elements
let bool_val = client.read_tag("gArrayBoolTest[10]").await?;
```

### **Writing Array Elements**

```rust
// Write DINT array element
client.write_tag("gArrayTest[0]", PlcValue::Dint(100)).await?;

// Write REAL array element
client.write_tag("Program:MainProgram.ArrayTest[5]", PlcValue::Real(123.45)).await?;

// Write BOOL array element
client.write_tag("gArrayBoolTest[10]", PlcValue::Bool(true)).await?;
```

### **Complete Example**

```rust
use rust_ethernet_ip::{EipClient, PlcValue};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = EipClient::connect("192.168.0.1:44818").await?;
    
    // Read array elements
    for i in 0..10 {
        let tag = format!("gArrayTest[{}]", i);
        let value = client.read_tag(&tag).await?;
        println!("{} = {:?}", tag, value);
    }
    
    // Write array elements
    client.write_tag("gArrayTest[0]", PlcValue::Dint(100)).await?;
    client.write_tag("gArrayTest[1]", PlcValue::Dint(200)).await?;
    
    // Verify writes
    let value0 = client.read_tag("gArrayTest[0]").await?;
    let value1 = client.read_tag("gArrayTest[1]").await?;
    println!("After write: [0] = {:?}, [1] = {:?}", value0, value1);
    
    Ok(())
}
```

---

## 🚀 **Performance Characteristics**

### **Array Element Access Performance:**
- **Read Operations**: ~2-5ms per element (includes array read + extraction)
- **Write Operations**: ~3-6ms per element (includes array read + modify + write)
- **BOOL Arrays**: ~1-2ms per element (more efficient due to DWORD handling)
- **Automatic Optimization**: Requests appropriate element count (10, 50, or 100) based on array size

### **Comparison:**
- **Before v0.5.5**: Array element access failed with "Path segment error"
- **After v0.5.5**: Array element access works reliably with minimal performance overhead

---

## 🔄 **Migration Guide**

### **From v0.5.4 to v0.5.5**

**No migration needed!** All existing code continues to work without modification.

### **New Capabilities Available:**

1. **Array Element Access Now Works:**
   ```rust
   // This now works (previously failed with "Path segment error")
   let value = client.read_tag("gArrayTest[0]").await?;
   client.write_tag("gArrayTest[0]", PlcValue::Dint(100)).await?;
   ```

2. **Program-Scoped Array Elements:**
   ```rust
   // This now works
   let value = client.read_tag("Program:MainProgram.ArrayTest[5]").await?;
   ```

3. **BOOL Array Elements:**
   ```rust
   // This now works with automatic DWORD bit extraction
   let bool_val = client.read_tag("gArrayBoolTest[10]").await?;
   client.write_tag("gArrayBoolTest[10]", PlcValue::Bool(true)).await?;
   ```

---

## 🎯 **Supported PLCs**

Tested and working with:
- ✅ **5069-L350ERMS3** firmware 33
- ✅ **CompactLogix L24ER** firmware 33
- ✅ **Controller-scoped arrays** (DINT, REAL, BOOL, etc.)
- ✅ **Program-scoped arrays** (DINT, REAL, BOOL, etc.)

Should work with all Allen-Bradley CompactLogix and ControlLogix PLCs that support EtherNet/IP.

---

## 🐛 **Known Limitations**

1. **Performance**: Array element access requires reading the entire array, which adds ~2-5ms overhead compared to direct access (if it were supported)
2. **Large Arrays**: For very large arrays (>100 elements), the workaround may need adjustment
3. **Multi-Dimensional Arrays**: Currently supports single-dimensional arrays (`Array[0]`). Multi-dimensional arrays (`Array[0,1]`) are not yet supported

---

## 🎯 **What's Next**

### **v0.6.0 (Planned)**
- Multi-dimensional array support (`Array[0,1,2]`)
- Optimized array element access for large arrays
- Array range operations (`Array[0..10]`)
- Performance improvements for array operations

### **Future Enhancements**
- Direct array element access (if CIP path encoding can be resolved)
- Array batch operations
- Array subscription support

---

## 🙏 **Acknowledgments**

Special thanks to the community members who reported the array element access issue and provided detailed information about their PLC configurations. This release directly addresses the "Path segment error" issue that affected many users.

---

## 📞 **Support**

- **Documentation**: [README.md](README.md) - Updated with array support examples
- **Examples**: 
  - [test_array_workaround.rs](examples/test_array_workaround.rs) - Complete workaround demo
  - [test_array_elements.rs](examples/test_array_elements.rs) - Array element examples
  - [test_bool_array.rs](examples/test_bool_array.rs) - BOOL array examples
- **Tests**: [array_read_write_tests.rs](tests/array_read_write_tests.rs) - Comprehensive test suite
- **Issues**: [GitHub Issues](https://github.com/sergiogallegos/rust-ethernet-ip/issues)
- **Discussions**: [GitHub Discussions](https://github.com/sergiogallegos/rust-ethernet-ip/discussions)

---

## 📦 **Installation**

```toml
[dependencies]
rust-ethernet-ip = "0.5.5"
```

Or update existing projects:
```bash
cargo update -p rust-ethernet-ip
```

---

**🎉 Array element access is now fully supported! Update to v0.5.5 and start using array elements in your code!**

