# 🦀 Rust EtherNet/IP Driver

[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org)
[![Version](https://img.shields.io/badge/version-0.6.0-blue.svg)](https://github.com/sergiogallegos/rust-ethernet-ip/releases)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Performance](https://img.shields.io/badge/performance-3000%2B%20ops%2Fsec-green.svg)]()
[![Status](https://img.shields.io/badge/status-production--ready-brightgreen.svg)]()
[![C# Wrapper](https://img.shields.io/badge/C%23%20wrapper-available-blue.svg)]()
[![Python Wrapper](https://img.shields.io/badge/Python%20wrapper-available-blue.svg)]()
[![Crates.io](https://img.shields.io/crates/v/rust-ethernet-ip.svg)](https://crates.io/crates/rust-ethernet-ip)
[![Documentation](https://docs.rs/rust-ethernet-ip/badge.svg)](https://docs.rs/rust-ethernet-ip)
[![Downloads](https://img.shields.io/crates/d/rust-ethernet-ip.svg)](https://crates.io/crates/rust-ethernet-ip)
[![Sponsor](https://img.shields.io/badge/sponsor-💖-ff69b4.svg)](https://github.com/sponsors/sergiogallegos)

A high-performance, production-ready EtherNet/IP communication library specifically designed for **Allen-Bradley CompactLogix and ControlLogix PLCs**. Built in pure Rust with focus on **PC applications**, offering exceptional performance, memory safety, and comprehensive industrial features.
 
 **📦 Available on [crates.io](https://crates.io/crates/rust-ethernet-ip)**

## 🎯 **Current Development Focus**

**We are currently focusing on polishing the .NET stack (C# wrappers and examples) to production quality.**

- 🎯 **Active Development**: 
  - C# wrapper library (`RustEtherNetIp.dll`)
  - WinForms example application
  - WPF example application  
  - ASP.NET example application
  - Advanced features: TagGroup, Statistics, Batch Operations, STRING support, UDT arrays
- ⏸️ **On Hold**: 
  - Go wrapper and applications (will resume after .NET stack is complete)
  - Python wrapper and applications (will resume after .NET stack is complete)

This focused approach ensures we deliver a robust, well-tested, production-ready .NET integration before expanding to other language ecosystems. The .NET stack serves as the reference implementation for future language bindings.

## 🎯 **Project Focus**

This library is specifically designed for:
- **Allen-Bradley CompactLogix** (L1x, L2x, L3x, L4x, L5x series)
- **Allen-Bradley ControlLogix** (L6x, L7x, L8x series)
- **PC Applications** (Windows, Linux, macOS)
- **Industrial Automation** software and SCADA systems
- **High-performance** data acquisition and control

### ✅ **v0.6.0 New Features**
- **🔧 Generic UDT Format**: New `UdtData` struct with `symbol_id` and raw bytes
  - Works with any UDT without requiring prior knowledge of member structure
  - Supports reading and writing UDTs generically
  - Enables parsing UDT members using UDT definitions when needed
- **📊 Array Element Access**: Full read/write support for array elements using intelligent workaround
  - Controller-scoped arrays: `gArrayTest[0]`, `gArrayTest[1]`, etc.
  - Program-scoped arrays: `Program:MainProgram.ArrayTest[0]`
  - BOOL array support: Automatic DWORD bit extraction for BOOL arrays
  - Automatic detection and workaround for array element access
- **✍️ Array Element Writing**: Write individual array elements with automatic array modification
  - Reads entire array, modifies element, writes back seamlessly
  - Supports all data types (DINT, REAL, BOOL, etc.)
  - Works with both controller-scoped and program-scoped arrays
- **✅ Library Health**: All 31 unit tests passing, production-ready core library

### ✅ **v0.5.4 Features**
- **🔍 UDT Definition Discovery**: Automatic UDT structure detection from PLC
- **🏷️ Enhanced Tag Discovery**: Full attribute support with permissions and scope
- **📦 Packet Size Negotiation**: Dynamic optimization for firmware 20+
- **🛣️ Route Path Support**: Slot configuration and multi-hop routing
- **💾 Cache Management**: Smart caching for UDT definitions and tag attributes
- **🔧 CIP Services**: Full implementation of Services 0x03 and 0x4C
- **🎯 Program Tag Support**: Fixed program-scoped tag access with correct CIP path format
- **🧠 Enhanced UDT Parsing**: Intelligent multi-member UDT parsing with byte alignment detection
- **⚡ Advanced Chunked Reading**: Multiple strategies for large UDTs with intelligent error recovery
- **🎯 .NET Stack Focus**: Comprehensive C# wrapper with WinForms, WPF, and ASP.NET examples

> **🎉 Major Milestone Achieved!**  
> v0.6.0 introduces a new generic UDT format (`UdtData`) that works with any UDT without requiring prior knowledge of member structure. The library core is production-ready with all 31 unit tests passing.

## ✨ **Key Features**

### 🔍 **UDT Discovery & Management (v0.5.4)**
- **Automatic UDT Structure Detection**: No more manual offset/size/type specifications
- **CIP Service 0x03**: Get Attribute List for comprehensive tag metadata
- **CIP Service 0x4C**: Read Tag Fragmented for large data structures
- **Smart Caching**: UDT definitions and tag attributes cached for performance
- **Template Management**: Full UDT template parsing and member discovery
- **Program-Scoped Discovery**: Find tags within specific program scopes

### 🛣️ **Route Path Support (v0.5.4, Enhanced in v0.6.0)**
- **Slot Configuration**: Support for slots 0-31 (✅ Fully implemented and tested)
- **Backplane Routing**: Direct communication with CPUs in different slots (✅ Fully implemented)
- **Network Routing**: Multi-hop routing through network addresses and ports (✅ Implemented)
- **CIP Path Building**: Automatic CIP route path byte generation (✅ Implemented)
- **ControlLogix Support**: Tested with ControlLogix systems with CPUs in different slots
- **Remote Rack Connections**: Basic support exists, needs additional testing
- **Dynamic Path Building**: Automatic CIP route path generation from slots, ports, and addresses

### 📦 **Packet Size Optimization (v0.5.4)**
- **Dynamic Negotiation**: Automatically negotiates optimal packet size with PLC
- **Firmware 20+ Support**: Enhanced performance for modern PLCs
- **Adaptive Sizing**: Adjusts packet size based on PLC capabilities
- **Performance Boost**: 20-30% improvement for large data transfers

### 🧠 **Enhanced UDT Processing** ✅ **NEW in v0.5.4**
- **Intelligent Multi-Member Parsing**: Automatically detects and parses multiple UDT members (DINT, DINT, REAL)
- **Byte Alignment Detection**: Smart alignment detection with reasonableness checks
- **Advanced Chunked Reading**: Multiple strategies for large UDTs with intelligent error recovery
- **Cross-Language Support**: All improvements work seamlessly across Rust, Go, and C# wrappers
- **Performance Optimized**: Sub-5ms response times for complex UDT operations

### 🔧 **Connection Robustness**
- **Automatic session management** with proper registration/unregistration
- **Connection health monitoring** with configurable timeouts
- **Network resilience** handling for industrial environments
- **Comprehensive error handling** with detailed CIP error mapping

### ⚠️ **Known Limitations**

The following operations are **not supported** due to PLC firmware restrictions. These limitations are inherent to the Allen-Bradley PLC firmware and cannot be bypassed at the library level.

#### STRING Tag Writing

**Cannot write directly to STRING tags** (e.g., `gTest_STRING`, `Program:TestProgram.gTest_STRING`).

**Root Cause:** PLC firmware limitation (CIP Error 0x2107). The PLC rejects direct write operations to STRING tags, regardless of the communication method used.

**What Works:**
- ✅ Reading STRING tags: `gTest_STRING` (read successfully)
- ✅ Reading STRING members in UDTs: `gTestUDT.Member5_String` (read successfully)

**What Doesn't Work:**
- ❌ Writing simple STRING tags: `gTest_STRING` (write fails - PLC limitation)
- ❌ Writing program-scoped STRING tags: `Program:TestProgram.gTest_STRING` (write fails - PLC limitation)
- ❌ Writing STRING members in UDTs directly: `gTestUDT.Member5_String` (write fails - must write entire UDT)

**Workaround for STRING Members in UDTs:**
If the STRING is part of a UDT structure, you can write it by reading the entire UDT, modifying the STRING member in memory, then writing the entire UDT back:

```rust
// Read entire UDT
let mut udt = client.read_tag("gTestUDT").await?;

// Modify STRING member in memory (if UDT structure is known)
// ... modify UDT structure ...

// Write entire UDT back
client.write_tag("gTestUDT", udt).await?;
```

**Note:** For standalone STRING tags (not part of a UDT), there is no workaround at the communication library level. Alternative approaches may include using PLC ladder logic or other PLC-side mechanisms to update STRING values.

#### UDT Array Element Member Writing

**Cannot write directly to members of UDT array elements** (e.g., `gTestUDT_Array[0].Member1_DINT`).

**Root Cause:** PLC firmware limitation (CIP Error 0x2107). The PLC does not support direct write operations to individual members within UDT array elements.

**What Works:**
- ✅ Reading UDT array element members: `gTestUDT_Array[0].Member1_DINT` (read successfully)
- ✅ Writing entire UDT array elements: `gTestUDT_Array[0]` (write full UDT structure)
- ✅ Writing UDT members (non-array): `gTestUDT.Member1_DINT` (write individual members of non-array UDTs)
- ✅ Writing simple array elements: `gArray[5]` (write elements of simple arrays like DINT[], REAL[], etc.)

**What Doesn't Work:**
- ❌ Writing UDT array element members: `gTestUDT_Array[0].Member1_DINT` (write fails - PLC limitation)
- ❌ Writing program-scoped UDT array element members: `Program:TestProgram.gTestUDT_Array[0].Member1_DINT` (write fails - PLC limitation)

**Workaround:**
Use a read-modify-write pattern:

```rust
// Read entire UDT array element
let mut element = client.read_tag("gTestUDT_Array[0]").await?;

// Modify member in memory (if UDT structure is known)
// ... modify UDT structure ...

// Write entire UDT array element back
client.write_tag("gTestUDT_Array[0]", element).await?;
```

#### Summary of Limitations

**Test Results (392 tags tested):**
- ✅ **333/392 tags** (84.9%) successfully read and written
- ❌ **59/392 tags** failed due to PLC firmware limitations:
  - 55 tags: UDT array element member writes (e.g., `gTestUDT_Array[0].Member1_DINT`)
  - 2 tags: Simple STRING tag writes (e.g., `gTest_STRING`)
  - 2 tags: STRING member writes in UDTs (e.g., `gTestUDT.Member5_String`)

**Important Notes:**
- These limitations are **PLC firmware restrictions**, not library bugs
- The library correctly implements the EtherNet/IP and CIP protocols
- All read operations work correctly for all tag types
- Workarounds are available for UDT array element members and STRING members in UDTs
- Standalone STRING tag writes have no workaround at the communication library level

**📚 For detailed technical information about these limitations, including official Rockwell documentation references and technical background, see [AB_String_UDT_Write_Limitations.md](docs/AB_String_UDT_Write_Limitations.md).**

### 📍 **Advanced Tag Addressing** ✅ **COMPLETED**
- **Program-scoped tags**: `Program:MainProgram.Tag1` ✅ **FIXED in v0.5.4**
- **Array element access**: `MyArray[5]`, `MyArray[1,2,3]` ✅ **WORKING in v0.5.5**
  - Automatic workaround: Reads entire array and extracts element
  - Supports read and write operations
  - Works with controller-scoped and program-scoped arrays
  - Special handling for BOOL arrays (DWORD bit extraction)
- **Bit-level operations**: `MyDINT.15` (access individual bits)
- **UDT member access**: `MyUDT.Member1.SubMember`
- **String operations**: `MyString.LEN`, `MyString.DATA[5]`
- **Complex nested paths**: `Program:Production.Lines[2].Stations[5].Motor.Status.15`

### 📊 **Complete Data Type Support** ✅ **COMPLETED**
All Allen-Bradley native data types with proper CIP encoding:
- **BOOL** - Boolean values (CIP type 0x00C1)
- **SINT** - 8-bit signed integer (-128 to 127, CIP type 0x00C2)
- **INT** - 16-bit signed integer (-32,768 to 32,767, CIP type 0x00C3)
- **DINT** - 32-bit signed integer (-2.1B to 2.1B, CIP type 0x00C4)
- **LINT** - 64-bit signed integer (CIP type 0x00C5)
- **USINT** - 8-bit unsigned integer (0 to 255, CIP type 0x00C6)
- **UINT** - 16-bit unsigned integer (0 to 65,535, CIP type 0x00C7)
- **UDINT** - 32-bit unsigned integer (0 to 4.3B, CIP type 0x00C8)
- **ULINT** - 64-bit unsigned integer (CIP type 0x00C9)
- **REAL** - 32-bit IEEE 754 float (CIP type 0x00CA)
- **LREAL** - 64-bit IEEE 754 double (CIP type 0x00CB)
- **STRING** - Variable-length strings (CIP type 0x00DA)
- **UDT** - User Defined Types with full nesting support (CIP type 0x00A0)

### 🔗 **Language Bindings**

#### **C# Integration** 🎯 **CURRENT FOCUS - Active Development**
- **Complete C# wrapper** with all data types
- **22 FFI functions** for seamless integration
- **Type-safe API** with comprehensive error handling
- **Cross-platform support** (Windows, Linux, macOS)
- **Production-ready examples**: WinForms, WPF, ASP.NET
- **Advanced features**: TagGroup, Statistics, Batch Operations, STRING support
- **Status**: Actively being polished to production quality

#### **Go Integration** ⏸️ **ON HOLD**
- **CGO wrapper** with comprehensive API coverage
- **Type-safe Go bindings** for all PLC data types
- **Connection management** and health monitoring
- **Error handling** with Go-idiomatic patterns
- **Full-stack example** with Go backend + Next.js frontend ([see example](examples/gonextjs/README.md))
- **Status**: Development paused until .NET stack is complete

#### **Python Integration** ⏸️ **ON HOLD**
- **PyO3-based Python wrapper** with full API coverage
- **Type-safe Python bindings** for all PLC data types
- **Synchronous and asynchronous APIs** for flexible usage
- **Comprehensive error handling** with Python exceptions
- **Easy installation** via pip or maturin
- **Cross-platform support** (Windows, Linux, macOS)
- **Status**: Development paused until .NET stack is complete

### ⚠️ **Comprehensive Error Handling** ✅ **COMPLETED**
- **Detailed CIP error mapping** with 40+ error codes
- **Network-level diagnostics** and troubleshooting
- **Granular error types** for precise error handling
- **Automatic error recovery** for transient issues

### 🏗️ **Build System** ✅ **COMPLETED**
- **Automated build scripts** for Windows and Linux/macOS
- **Cross-platform compilation** with proper library generation
- **Comprehensive testing** with 30+ unit tests
- **CI/CD ready** with GitHub Actions examples

### ⚡ **Real-Time Subscriptions** ✅ **NEW in v0.4.0**
- **Real-time tag monitoring** with configurable update intervals (1ms - 10s)
- **Event-driven notifications** for tag value changes
- **Multi-tag subscriptions** supporting hundreds of concurrent monitors
- **Automatic reconnection** and error recovery
- **Memory-efficient engine** with minimal CPU overhead

### 🚀 **High-Performance Batch Operations** ✅ **NEW in v0.4.0**
- **Batch read operations** - read up to 100+ tags in a single request
- **Batch write operations** - write multiple tags atomically
- **Parallel processing** with concurrent execution
- **Transaction support** with rollback capabilities
- **2,000+ ops/sec throughput** with intelligent packet packing

## 🏭 **HMI/SCADA Production Demo**

Experience a **professional-grade HMI dashboard** showcasing real-world industrial data tracking and monitoring capabilities. This demo demonstrates the library's potential for building production-ready SCADA systems and industrial dashboards.

![HMI/SCADA Production Demo](images/SampleHMI.png)

### 🎯 **Demo Features**
- **📊 Real-time Production Monitoring** - Live production counts, targets, and progress tracking
- **📈 OEE Analysis** - Overall Equipment Effectiveness with availability, performance, and quality metrics
- **🌡️ Process Parameters** - Temperature, pressure, vibration monitoring with color-coded alerts
- **⚙️ Machine Status** - Real-time machine state, shift information, and operator tracking
- **🔧 Maintenance Management** - Scheduled maintenance tracking and history
- **📱 Responsive Design** - Works seamlessly on desktop, tablet, and mobile devices

### 🚀 **Perfect for Demonstrations**
This demo showcases:
- **High-frequency data collection** (1-second intervals)
- **Professional HMI aesthetics** with industrial-grade visualizations
- **Real-world metrics** that matter to production managers
- **Scalable architecture** for larger SCADA systems
- **Modern web technologies** for cross-platform deployment

### 🏷️ **Required PLC Tags**
The demo reads 13 industrial tags including machine status, production metrics, process parameters, and OEE data. See the [Go + Next.js example](examples/gonextjs/README.md) for complete tag specifications and setup instructions.

**Try it now:** Navigate to the "HMI Demo" tab in the [Go + Next.js fullstack example](examples/gonextjs/README.md)!

## 🚀 **Performance Characteristics**

Optimized for PC applications with excellent performance:

> **🆕 Latest Performance Improvements (v0.6.0)**
> 
> Recent optimizations and improvements:
> - **Generic UDT Format**: New `UdtData` struct enables universal UDT handling
> - **Memory allocation improvements**: 20-30% reduction in allocation overhead for network operations
> - **Batch operations**: 3-10x faster than individual operations
> - **Code quality**: Enhanced with idiomatic Rust patterns and clippy optimizations
> - **Network efficiency**: Optimized packet building with pre-allocated buffers
> - **Library Health**: All 31 unit tests passing, production-ready core

| Operation | Throughput | Latency | Memory Usage |
|-----------|------------|---------|--------------|
| Single Tag Read | 3,000+ ops/sec | <1ms | ~800B |
| Single Tag Write | 1,500+ ops/sec | <2ms | ~800B |
| Batch Operations | 2,000+ ops/sec | 5-20ms | ~2KB |
| Real-time Subscriptions | 1,000+ tags/sec | 1-10ms | ~1KB |
| Tag Path Parsing | 10,000+ ops/sec | <0.1ms | ~1KB |
| Connection Setup | N/A | 50-200ms | ~4KB |
| Memory per Connection | N/A | N/A | ~4KB base |

## 📋 **Development Roadmap**

### 🔥 **Phase 1: Core Enhancements** ✅ **COMPLETED - January 2026**
- [x] Basic tag read/write operations
- [x] Connection management and session handling
- [x] **Enhanced tag path parsing** (Program-scoped, arrays, bit access)
- [x] **Complete data type support** (All Allen-Bradley types)
- [x] **C# wrapper integration** (22 FFI functions)
- [x] **Comprehensive testing** (30+ unit tests)
- [x] **Build automation** (Cross-platform build scripts)
- [x] **Documentation** (Examples, API docs, guides)

### ⚡ **Phase 2: Advanced Features** ✅ **COMPLETED - January 2026**
- [x] **Batch operations** (multi-tag read/write) ✅ **COMPLETED**
- [x] **Real-time subscriptions** (tag change notifications) ✅ **COMPLETED**
- [x] **Performance optimizations** (20% faster operations + memory improvements) ✅ **COMPLETED**
- [x] **Enhanced error handling & recovery** ✅ **COMPLETED**

### 🎯 **Phase 3: Production Ready** ✅ **COMPLETED - January 2026**
- [x] **Production monitoring** - Comprehensive metrics and health checks
- [x] **Configuration management** - Production-ready config system
- [x] **Error handling** - Detailed CIP error mapping and recovery
- [x] **Performance optimization** - Batch operations and connection pooling
- [x] **Generic UDT Format** - Universal UDT handling with `UdtData` struct (v0.6.0)
- [x] **Library Testing** - All 31 unit tests passing
- [x] **Code Quality** - Comprehensive examples and tests updated for new API
- [x] **Community features** - Discord server, GitHub discussions, sponsorship program

### 🚀 **Phase 4: Industrial Routing Support** ✅ **PARTIALLY IMPLEMENTED**
- [x] **Basic slot configuration** - Support for CPUs in slots 0-31 (✅ Implemented in v0.6.0)
- [x] **Simple backplane routing** - Direct backplane communication (✅ Implemented in v0.6.0)
- [x] **Route path building** - CIP route path construction (✅ Implemented in v0.6.0)
- [x] **Network routing** - Multi-hop network routing via addresses and ports (✅ Implemented in v0.6.0)
- [x] **Path validation** - Route path verification and CIP byte construction (✅ Implemented in v0.6.0)
- [ ] **Remote rack support** - Connect to remote racks via network (⚠️ Basic support exists, needs testing)
- [ ] **Advanced routing** - Complex network topologies (⚠️ Basic support exists, needs validation)
- [ ] **Route discovery** - Automatic path detection (❌ Not implemented)

**Note:** ControlLogix systems with CPUs in different slots can be tested using the `RoutePath` API. 
Use `EipClient::with_route_path()` or `set_route_path()` with `RoutePath::new().add_slot(slot_number)` 
to connect to ControlLogix CPUs in slots 0-31.

## 🛠️ **Installation**

### 📦 **Rust Library (Crates.io)**

The easiest way to get started is by adding the crate to your `Cargo.toml`:

```toml
[dependencies]
rust-ethernet-ip = "0.5.3"
tokio = { version = "1.0", features = ["full"] }

### Rust Library
Add to your `Cargo.toml`:

```toml
[dependencies]
rust-ethernet-ip = "0.6.0"
tokio = { version = "1.0", features = ["full"] }
```

### C# Wrapper
Install via NuGet:

```xml
<PackageReference Include="RustEtherNetIp" Version="0.6.0" />
```

Or via Package Manager Console:
```powershell
Install-Package RustEtherNetIp
```

### Python Wrapper
Install via pip:
```bash
pip install rust-ethernet-ip
```

Or build from source using maturin:
```bash
cd pywrapper
maturin develop
```

## 📖 **Quick Start**

### UDT Discovery (v0.5.4)
```rust
use rust_ethernet_ip::{EipClient, RoutePath};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Connect to PLC
    let mut client = EipClient::connect("192.168.0.1:44818").await?;
    
    // Discover UDT structure automatically
    let definition = client.get_udt_definition("Part_Data").await?;
    println!("UDT: {}", definition.name);
    
    for member in &definition.members {
        println!("  {}: {} (offset: {}, size: {} bytes)", 
            member.name, 
            get_data_type_name(member.data_type),
            member.offset, 
            member.size
        );
    }
    
    // Read UDT data using discovered structure
    let udt_data = client.read_udt_chunked("Part_Data").await?;
    
    // Read individual members using discovered offsets
    for member in &definition.members {
        let value = client.read_udt_member_by_offset(
            "Part_Data",
            member.offset as usize,
            member.size as usize,
            member.data_type
        ).await?;
        
        println!("{}: {:?}", member.name, value);
    }
    
    Ok(())
}
```

### Route Path Support (v0.5.4)
```rust
// Create route path for slot 2
let route = RoutePath::new()
    .add_slot(0)  // Backplane slot 0
    .add_slot(2); // Target slot 2

// Connect with route path
let mut client = EipClient::with_route_path("192.168.0.1:44818", route).await?;

// Read tags through the route
let value = client.read_tag("TestTag").await?;
```

### Basic Usage

```rust
use rust_ethernet_ip::{EipClient, PlcValue};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Connect to CompactLogix PLC
    let mut client = EipClient::connect("192.168.1.100:44818").await?;
    
    // Read different data types
    let motor_running = client.read_tag("Program:Main.MotorRunning").await?;
    let production_count = client.read_tag("Program:Main.ProductionCount").await?;
    let temperature = client.read_tag("Program:Main.Temperature").await?;
    
    // Write values
    client.write_tag("Program:Main.SetPoint", PlcValue::Dint(1500)).await?;
    client.write_tag("Program:Main.StartButton", PlcValue::Bool(true)).await?;
    
    println!("Motor running: {:?}", motor_running);
    println!("Production count: {:?}", production_count);
    println!("Temperature: {:?}", temperature);
    
    Ok(())
}
```

### C# Usage

```csharp
using RustEtherNetIp;

using var client = new EtherNetIpClient();
if (client.Connect("192.168.1.100:44818"))
{
    // Read different data types
    bool motorRunning = client.ReadBool("Program:Main.MotorRunning");
    int productionCount = client.ReadDint("Program:Main.ProductionCount");
    float temperature = client.ReadReal("Program:Main.Temperature");
    
    // Write values
    client.WriteDint("Program:Main.SetPoint", 1500);
    client.WriteBool("Program:Main.StartButton", true);
    
    Console.WriteLine($"Motor running: {motorRunning}");
    Console.WriteLine($"Production count: {productionCount}");
    Console.WriteLine($"Temperature: {temperature:F1}°C");
}
```

### Python Usage

```python
from rust_ethernet_ip import PyEipClient

def main():
    # Create a client and connect to the PLC
    client = PyEipClient(addr="192.168.0.1")
    if not client.connect():
        print("Failed to connect to PLC")
        return

    try:
        # Read a DINT value
        value = client.read_dint("TestDINT")
        print("Read TestDINT:", value)
        
        # Write a new value
        client.write_dint("TestDINT", 42)
        print("Wrote 42 to TestDINT")
        
    except Exception as e:
        print("Error:", e)

if __name__ == "__main__":
    main()
```

### Advanced Tag Addressing

```rust
// Program-scoped tags
let value = client.read_tag("Program:MainProgram.Tag1").await?;

// Array elements (v0.5.5 - automatic workaround)
let array_element = client.read_tag("Program:Main.MyArray[5]").await?;
// Writing array elements
client.write_tag("gArrayTest[0]", PlcValue::Dint(100)).await?;
// BOOL arrays work too
let bool_value = client.read_tag("gArrayBoolTest[5]").await?;
client.write_tag("gArrayBoolTest[5]", PlcValue::Bool(true)).await?;
let multi_dim = client.read_tag("Program:Main.Matrix[1,2,3]").await?;

// Bit access
let bit_value = client.read_tag("Program:Main.StatusWord.15").await?;

// UDT members
let udt_member = client.read_tag("Program:Main.MotorData.Speed").await?;
let nested_udt = client.read_tag("Program:Main.Recipe.Step1.Temperature").await?;

// String operations
let string_length = client.read_tag("Program:Main.ProductName.LEN").await?;
let string_char = client.read_tag("Program:Main.ProductName.DATA[0]").await?;
```

### Complete Data Type Examples

```rust
// All supported data types
let bool_val = client.read_tag("BoolTag").await?;           // BOOL
let sint_val = client.read_tag("SintTag").await?;           // SINT (-128 to 127)
let int_val = client.read_tag("IntTag").await?;             // INT (-32,768 to 32,767)
let dint_val = client.read_tag("DintTag").await?;           // DINT (-2.1B to 2.1B)
let lint_val = client.read_tag("LintTag").await?;           // LINT (64-bit signed)
let usint_val = client.read_tag("UsintTag").await?;         // USINT (0 to 255)
let uint_val = client.read_tag("UintTag").await?;           // UINT (0 to 65,535)
let udint_val = client.read_tag("UdintTag").await?;         // UDINT (0 to 4.3B)
let ulint_val = client.read_tag("UlintTag").await?;         // ULINT (64-bit unsigned)
let real_val = client.read_tag("RealTag").await?;           // REAL (32-bit float)
let lreal_val = client.read_tag("LrealTag").await?;         // LREAL (64-bit double)
let string_val = client.read_tag("StringTag").await?;       // STRING
let udt_val = client.read_tag("UdtTag").await?;             // UDT
```

## ⚡ **Batch Operations** ✅ **COMPLETED**

**Dramatically improve performance** with batch operations that execute multiple read/write operations in a single network packet. Perfect for data acquisition, recipe management, and coordinated control scenarios.

### 🚀 **Performance Benefits**
- **3-10x faster** than individual operations
- **Reduced network traffic** (1-5 packets instead of N packets for N operations)
- **Lower PLC CPU usage** due to fewer connection handling overheads
- **Better throughput** for data collection and control applications

### 📊 **Use Cases**
- **Data acquisition**: Reading multiple sensor values simultaneously
- **Recipe management**: Writing multiple setpoints at once
- **Status monitoring**: Reading multiple status flags efficiently
- **Coordinated control**: Atomic operations across multiple tags

### 🔧 **Basic Batch Reading**

```rust
use rust_ethernet_ip::{EipClient, BatchOperation, PlcValue};

// Read multiple tags in a single operation
let tags_to_read = vec![
    "ProductionCount",
    "Temperature_1", 
    "Temperature_2",
    "Pressure_1",
    "FlowRate",
];

let results = client.read_tags_batch(&tags_to_read).await?;
for (tag_name, result) in results {
    match result {
        Ok(value) => println!("📊 {}: {:?}", tag_name, value),
        Err(error) => println!("❌ {}: {}", tag_name, error),
    }
}
```

### ✏️ **Basic Batch Writing**

```rust
// Write multiple tags in a single operation
let tags_to_write = vec![
    ("SetPoint_1", PlcValue::Real(75.5)),
    ("SetPoint_2", PlcValue::Real(80.0)),
    ("EnableFlag", PlcValue::Bool(true)),
    ("ProductionMode", PlcValue::Dint(2)),
    ("RecipeNumber", PlcValue::Dint(42)),
];

let results = client.write_tags_batch(&tags_to_write).await?;
for (tag_name, result) in results {
    match result {
        Ok(()) => println!("✅ {}: Write successful", tag_name),
        Err(error) => println!("❌ {}: {}", tag_name, error),
    }
}
```

### 🔄 **Mixed Operations (Reads + Writes)**

```rust
use rust_ethernet_ip::BatchOperation;

let operations = vec![
    // Read current values
    BatchOperation::Read { tag_name: "CurrentTemp".to_string() },
    BatchOperation::Read { tag_name: "CurrentPressure".to_string() },
    
    // Write new setpoints
    BatchOperation::Write { 
        tag_name: "TempSetpoint".to_string(), 
        value: PlcValue::Real(78.5) 
    },
    BatchOperation::Write { 
        tag_name: "PressureSetpoint".to_string(), 
        value: PlcValue::Real(15.2) 
    },
    
    // Update control flags
    BatchOperation::Write { 
        tag_name: "AutoModeEnabled".to_string(), 
        value: PlcValue::Bool(true) 
    },
];

let results = client.execute_batch(&operations).await?;
for result in results {
    match result.operation {
        BatchOperation::Read { tag_name } => {
            match result.result {
                Ok(Some(value)) => println!("📊 Read {}: {:?} ({}μs)", 
                    tag_name, value, result.execution_time_us),
                Err(error) => println!("❌ Read {}: {}", tag_name, error),
            }
        }
        BatchOperation::Write { tag_name, .. } => {
            match result.result {
                Ok(_) => println!("✅ Write {}: Success ({}μs)", 
                    tag_name, result.execution_time_us),
                Err(error) => println!("❌ Write {}: {}", tag_name, error),
            }
        }
    }
}
```

### ⚙️ **Advanced Configuration**

```rust
use rust_ethernet_ip::BatchConfig;

// High-performance configuration
let high_perf_config = BatchConfig {
    max_operations_per_packet: 50,      // More operations per packet
    max_packet_size: 4000,              // Larger packets for modern PLCs
    packet_timeout_ms: 1000,            // Faster timeout
    continue_on_error: true,            // Don't stop on single failures
    optimize_packet_packing: true,      // Optimize packet efficiency
};

client.configure_batch_operations(high_perf_config);

// Conservative/reliable configuration
let conservative_config = BatchConfig {
    max_operations_per_packet: 10,      // Fewer operations per packet
    max_packet_size: 504,               // Smaller packets for compatibility
    packet_timeout_ms: 5000,            // Longer timeout
    continue_on_error: false,           // Stop on first error
    optimize_packet_packing: false,     // Preserve exact operation order
};

client.configure_batch_operations(conservative_config);
```

### 📈 **Performance Comparison Example**

```rust
use std::time::Instant;

let tags = vec!["Tag1", "Tag2", "Tag3", "Tag4", "Tag5"];

// Individual operations (traditional approach)
let individual_start = Instant::now();
for tag in &tags {
    let _ = client.read_tag(tag).await?;
}
let individual_duration = individual_start.elapsed();

// Batch operations (optimized approach)  
let batch_start = Instant::now();
let _ = client.read_tags_batch(&tags).await?;
let batch_duration = batch_start.elapsed();

let speedup = individual_duration.as_nanos() as f64 / batch_duration.as_nanos() as f64;
println!("📈 Performance improvement: {:.1}x faster with batch operations!", speedup);
```

### 🚨 **Error Handling**

```rust
// Batch operations provide detailed error information per operation
match client.execute_batch(&operations).await {
    Ok(results) => {
        let mut success_count = 0;
        let mut error_count = 0;
        
        for result in results {
            match result.result {
                Ok(_) => success_count += 1,
                Err(_) => error_count += 1,
            }
        }
        
        println!("📊 Results: {} successful, {} failed", success_count, error_count);
        println!("📈 Success rate: {:.1}%", 
            (success_count as f32 / (success_count + error_count) as f32) * 100.0);
    }
    Err(e) => println!("❌ Entire batch failed: {}", e),
}
```

### 🎯 **Best Practices**

- **Use batch operations for 3+ operations** to see significant performance benefits
- **Group similar operations** (reads together, writes together) for optimal packet packing
- **Adjust max_operations_per_packet** based on your PLC's capabilities (10-50 typical)
- **Use higher packet sizes** (up to 4000 bytes) for modern CompactLogix/ControlLogix PLCs
- **Enable continue_on_error** for data collection scenarios where partial results are acceptable
- **Disable optimize_packet_packing** if precise operation order is critical for your application

## 🏗️ **Building**

### Quick Build
```bash
# Windows
build.bat

# Linux/macOS
./build.sh
```

### Manual Build
```bash
# Build Rust library
cargo build --release --lib

# Copy to C# project (Windows)
copy target\release\rust_ethernet_ip.dll csharp\RustEtherNetIp\

# Build C# wrapper
cd csharp/RustEtherNetIp
dotnet build --configuration Release
```

See [BUILD.md](BUILD.md) for comprehensive build instructions.

## 🧪 **Testing**

Run the comprehensive test suite:

```bash
# Rust unit tests (30+ tests)
cargo test

# C# wrapper tests
cd csharp/RustEtherNetIp.Tests
dotnet test

# Run examples
cargo run --example advanced_tag_addressing
cargo run --example data_types_showcase
```

## 🎯 **Examples**

Explore comprehensive examples demonstrating all library capabilities across different platforms:

### **🌐 TypeScript + React Dashboard** *(Recommended)*
Modern web-based PLC dashboard with real-time monitoring and advanced features.

```bash
# Start backend API
cd examples/AspNetExample
dotnet run

# Start frontend (new terminal)
cd examples/TypeScriptExample/frontend
npm install && npm run dev
```

**Features:**
- ✅ **Modern UI/UX** with glassmorphism design and responsive layout
- ✅ **Real-time monitoring** with live tag updates and performance metrics
- ✅ **Complete data type support** for all 13 Allen-Bradley types
- ✅ **Advanced tag addressing** with interactive examples
- ✅ **Type-safe API** with comprehensive TypeScript interfaces
- ✅ **Professional features** including benchmarking and activity logging

**Perfect for:** Web applications, dashboards, remote monitoring, modern industrial HMIs

### **🖥️ WPF Desktop Application**
Rich desktop application with MVVM architecture and modern UI.

```bash
cd examples/WpfExample
dotnet run
```

**Features:**
- ✅ **MVVM architecture** with CommunityToolkit.Mvvm
- ✅ **Real-time tag monitoring** with automatic refresh
- ✅ **Advanced tag discovery** with type detection
- ✅ **Performance benchmarking** with visual metrics
- ✅ **Comprehensive logging** with timestamped activity

**Perfect for:** Desktop HMIs, engineering tools, maintenance applications

### **🪟 WinForms Application**
Traditional Windows Forms application with familiar UI patterns.

```bash
cd examples/WinFormsExample
dotnet run
```

**Features:**
- ✅ **Classic Windows UI** with familiar controls
- ✅ **Connection monitoring** with automatic reconnection
- ✅ **Tag operations** with validation and error handling
- ✅ **Performance testing** with real-time metrics
- ✅ **Industrial styling** with professional appearance

**Perfect for:** Legacy system integration, simple HMIs, maintenance tools

### **🌐 ASP.NET Core Web API**
RESTful API backend providing HTTP access to PLC functionality.

```bash
cd examples/AspNetExample
dotnet run
```

**Features:**
- ✅ **RESTful endpoints** for all PLC operations
- ✅ **Swagger documentation** with interactive API explorer
- ✅ **Type-safe operations** with comprehensive validation
- ✅ **Performance monitoring** with built-in benchmarking
- ✅ **Production-ready** with proper error handling and logging

**Perfect for:** Web services, microservices, system integration, mobile backends

### **🦀 Rust Examples**
Native Rust examples demonstrating core library functionality.

```bash
# Advanced tag addressing showcase
cargo run --example advanced_tag_addressing

# Complete data types demonstration
cargo run --example data_types_showcase

# Batch operations performance demo
cargo run --example batch_operations_demo
```

**Features:**
- ✅ **Advanced tag parsing** with complex path examples
- ✅ **All data types** with encoding demonstrations
- ✅ **Performance examples** with async/await patterns
- ✅ **Error handling** with comprehensive error types
- ✅ **Batch operations** with performance comparisons and configuration examples

**Perfect for:** Rust applications, embedded systems, high-performance scenarios

### **🐹 Go + Next.js Fullstack Example** *(NEW in v0.4.0!)*
Modern fullstack demo with a Go backend (using the Rust Go wrapper) and a Next.js (TypeScript) frontend for real-time, batch, and performance operations.

```bash
# Start backend
cd examples/gonextjs/backend
go run .

# Start frontend (new terminal)
cd ../frontend
npm install && npm run dev
```

**Features:**
- ✅ **Go backend** using the Rust EtherNet/IP Go wrapper (FFI)
- ✅ **Next.js frontend** (TypeScript, Tailwind, App Router)
- ✅ **Batch read/write** and individual tag operations
- ✅ **Performance benchmarking** (ops/sec, latency)
- ✅ **Real-time tag updates** via WebSocket
- ✅ **Comprehensive PLC data type support**
- ✅ **Modern, responsive UI**
- ✅ **🏭 Professional HMI/SCADA Demo** - Production-ready dashboard with OEE analysis, process monitoring, and industrial data visualization

**Perfect for:** Modern web dashboards, Go/TypeScript fullstack apps, real-time industrial monitoring, HMI/SCADA systems

### **⚡ Vue.js 3 + TypeScript Frontend** *(NEW in v0.4.0!)*
Modern Vue.js 3 frontend with TypeScript, Tailwind CSS, and Pinia state management, designed to integrate with ASP.NET Core backends.

```bash
# Start backend API
cd examples/AspNetExample
dotnet run

# Start Vue.js frontend (new terminal)
cd examples/VueExample
npm install && npm run dev
```

**Features:**
- ✅ **Vue.js 3** with Composition API and TypeScript
- ✅ **Tailwind CSS** for modern, responsive design
- ✅ **Pinia state management** for application state
- ✅ **Backend detection system** for automatic ASP.NET Core port discovery
- ✅ **Component-based architecture** with reusable UI components
- ✅ **Real-time connection monitoring** with PLC status display
- ✅ **Tag operations interface** for read/write operations
- ✅ **Professional dashboard** with metrics and activity logging
- ✅ **Development tools** including BackendDetector for debugging

**Perfect for:** Modern web applications, Vue.js-based HMIs, industrial dashboards, ASP.NET Core integration

### **🚀 Quick Start Guide**

1. **Choose your platform:**
   - **Web/Modern UI** → TypeScript + React Dashboard or Vue.js 3 + TypeScript
   - **Desktop/Windows** → WPF or WinForms Application  
   - **Web API/Services** → ASP.NET Core Web API
   - **Native/Performance** → Rust Examples

2. **Start the backend** (for web examples):
   ```bash
   cd examples/AspNetExample
   dotnet run
   ```

3. **Run your chosen example** and connect to your PLC at `192.168.0.1:44818`

4. **Explore features:**
   - Tag discovery with advanced addressing
   - Real-time monitoring and benchmarking
   - All 13 Allen-Bradley data types
   - Professional error handling and logging

### **📁 Example Structure**
```
examples/
├── TypeScriptExample/          # React + TypeScript dashboard
│   ├── frontend/              # Modern web UI
│   ├── start-backend.bat      # Backend startup script
│   └── start-frontend.bat     # Frontend startup script
├── VueExample/                 # Vue.js 3 + TypeScript frontend
│   ├── src/                   # Vue.js source code
│   ├── start-frontend.bat     # Frontend startup script
│   └── README.md              # Comprehensive documentation
├── WpfExample/                # WPF desktop application
├── WinFormsExample/           # WinForms desktop application
├── AspNetExample/             # ASP.NET Core Web API
├── gonextjs/                  # Go + Next.js fullstack example
└── rust-examples/             # Native Rust examples
    ├── advanced_tag_addressing.rs
    ├── data_types_showcase.rs
    └── batch_operations_demo.rs
```

Each example includes comprehensive documentation, setup instructions, and demonstrates different aspects of the library's capabilities.

## 📚 **Documentation**

- **[API Documentation](https://docs.rs/rust-ethernet-ip)** - Complete API reference
- **[Examples](examples/)** - Practical usage examples
- **[Build Guide](BUILD.md)** - Comprehensive build instructions
- **[C# Wrapper Guide](csharp/RustEtherNetIp/README.md)** - C# integration documentation
- **[Changelog](CHANGELOG.md)** - Version history and changes
- **[Crates.io Page](https://crates.io/crates/rust-ethernet-ip)** - Official crate registry page

## 💖 **Sponsor This Project**

This project is developed for the industrial automation community. If you find this library valuable for your projects, please consider sponsoring its development!

### 🎯 **Why Sponsor?**
- **🚀 Accelerate Development** - Help fund new features, performance improvements, and platform support
- **🐛 Priority Bug Fixes** - Get faster resolution of issues affecting your production systems
- **💡 Feature Requests** - Influence the roadmap with your specific industrial automation needs
- **📚 Enhanced Documentation** - Support creation of comprehensive guides and tutorials
- **🔧 Professional Support** - Access to direct developer support for complex implementations

### 💰 **Sponsorship Tiers**
- **☕ Coffee** - Show appreciation for the project
- **🚀 Feature Sponsor** - Fund specific feature development
- **🏭 Enterprise Sponsor** - Priority support and custom development
- **🌟 Platinum Sponsor** - Direct collaboration and roadmap input

### 🎁 **Sponsor Benefits**
- **Priority issue resolution** for production-critical bugs
- **Direct access** to development team for technical questions
- **Early access** to new features and beta releases
- **Custom feature development** for your specific use cases
- **Recognition** in project documentation and releases

**[Sponsor on GitHub →](https://github.com/sponsors/sergiogallegos)**

### 💬 **Feedback & Feature Requests**

We value your input! Help us improve the library by sharing:

#### 🐛 **Bug Reports**
- **Production Issues** - Critical bugs affecting your systems
- **Performance Problems** - Slow operations or memory issues
- **Compatibility Issues** - Problems with specific PLC models or configurations
- **Error Handling** - Unexpected errors or unclear error messages

#### 💡 **Feature Requests**
- **New Data Types** - Additional Allen-Bradley data type support
- **Platform Support** - New operating systems or architectures
- **Performance Features** - Batch operations, caching, or optimization requests
- **Integration Features** - New language bindings or framework integrations
- **Industrial Features** - SCADA-specific functionality, alarms, or trending

#### 📊 **Use Case Sharing**
- **Success Stories** - How you're using the library in production
- **Performance Metrics** - Real-world performance data from your applications
- **Integration Examples** - Custom implementations or workarounds
- **Best Practices** - Tips for other users in similar industries

**[Submit Feedback →](https://github.com/sergiogallegos/rust-ethernet-ip/issues/new/choose)**

## 🔧 **Troubleshooting**

Experiencing issues? Check out our comprehensive troubleshooting guide:

📖 **[Complete Troubleshooting Guide](docs/TROUBLESHOOTING.md)**

### Quick Reference: Common Errors

| Error Code | Meaning | Quick Fix |
|------------|---------|-----------|
| **0x01** | Connection failure | Check tag name, scope, and External Access permissions |
| **0x04** | Path segment error | Verify tag path format (controller vs program-scoped) |
| **0x05** | Path destination unknown | Check ControlLogix slot routing |
| **0x16** | Object does not exist | Verify tag exists and is downloaded to PLC |

### Most Common Issues

**1. CIP Error 0x01: Connection Failure**
- ✅ Verify tag name is exactly correct (case-sensitive)
- ✅ Check if tag is program-scoped: use `"Program:ProgramName.TagName"`
- ✅ Verify tag has External Access enabled in RSLogix/Studio 5000
- ✅ Ensure tag is downloaded to PLC (not just saved)
- ✅ For ControlLogix, check CPU slot routing

**2. Tag Not Found**
- Use `discover_tags()` to find available tags
- Check tag scope (Controller vs Program)
- Verify tag spelling (case-sensitive)

**3. ControlLogix Routing Issues**
- If CPU is in slot other than 0, specify route path:
  ```rust
  let route = RoutePath::new().add_slot(3); // CPU in slot 3
  let mut client = EipClient::with_route_path("192.168.1.100:44818", route).await?;
  ```

**4. Connection Timeout**
- Verify IP address and port (default: 44818)
- Check network connectivity (ping the PLC)
- Ensure firewall allows port 44818
- Verify PLC is in RUN mode

For detailed troubleshooting steps, code examples, and debugging procedures, see the [Complete Troubleshooting Guide](docs/TROUBLESHOOTING.md).

## 🤝 **Community & Support**

- **[Discord Server](https://discord.gg/uzaM3tua)** - Community discussions, support, and development updates
- **[GitHub Issues](https://github.com/sergiogallegos/rust-ethernet-ip/issues)** - Bug reports and feature requests
- **[GitHub Discussions](https://github.com/sergiogallegos/rust-ethernet-ip/discussions)** - General questions and ideas 
- **[Crates.io](https://crates.io/crates/rust-ethernet-ip)** - Official Rust package registry

## 🙏 **Inspiration**

This project draws inspiration from excellent libraries in the industrial automation space:
- **[pylogix](https://github.com/dmroeder/pylogix)** - Python library for Allen-Bradley PLCs
- **[pycomm3](https://github.com/ottowayi/pycomm3)** - Python library for Allen-Bradley PLCs
- **[gologix](https://github.com/danomagnum/gologix)** - Go library for Allen-Bradley PLCs
- **[libplctag](https://github.com/libplctag/libplctag)** - Cross-platform PLC communication library

## 🚀 **Contributing**

We welcome contributions! Please see our [Contributing Guide](CONTRIBUTING.md) for details on:
- Code style and standards
- Testing requirements
- Pull request process
- Development setup

## ⚠️ **Disclaimer and Liability**

### **Use at Your Own Risk**
This library is provided "AS IS" without warranty of any kind. Users assume full responsibility for its use in their applications and systems.

### **No Warranties**
The developers and contributors make **NO WARRANTIES, EXPRESS OR IMPLIED**, including but not limited to:
- **Merchantability** or fitness for a particular purpose
- **Reliability** or availability of the software
- **Accuracy** of data transmission or processing
- **Safety** for use in critical or production systems

### **Industrial Safety Responsibility**
- **🏭 Industrial Use:** Users are solely responsible for ensuring this library meets their industrial safety requirements
- **🔒 Safety Systems:** This library should NOT be used for safety-critical applications without proper validation
- **⚙️ Production Systems:** Thoroughly test in non-production environments before deploying to production systems
- **📋 Compliance:** Users must ensure compliance with all applicable industrial standards and regulations

### **Limitation of Liability**
Under no circumstances shall the developers, contributors, or associated parties be liable for:
- **Equipment damage** or malfunction
- **Production downtime** or operational disruptions  
- **Data loss** or corruption
- **Personal injury** or property damage
- **Financial losses** of any kind
- **Consequential or indirect damages**

### **User Responsibilities**
By using this library, you acknowledge and agree that:
- You have the technical expertise to properly implement and test the library
- You will perform adequate testing before production deployment
- You will implement appropriate safety measures and fail-safes
- You understand the risks associated with industrial automation systems
- You accept full responsibility for any consequences of using this library

### **Indemnification**
Users agree to indemnify and hold harmless the developers and contributors from any claims, damages, or liabilities arising from the use of this library.

---

**⚠️ IMPORTANT: This disclaimer is an integral part of the license terms. Use of this library constitutes acceptance of these terms.**

## 📄 **License**

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

---

**Built for the industrial automation community**

## 📦 Examples

- **C# + React**: Modern web and desktop examples using the C# wrapper
- **Go + Next.js**: [Fullstack Go backend + Next.js frontend example](examples/gonextjs/README.md) (**NEW in v0.4.0!**)
- **Vue.js 3 + TypeScript**: [Modern Vue.js frontend with ASP.NET Core integration](examples/VueExample/README.md) (**NEW in v0.4.0!**)
- **TypeScript + ASP.NET**: Classic React + ASP.NET example
- ...and more in the `examples/` directory

## 🏗️ Build All

To build all wrappers, libraries, and examples (including Go + Next.js and Vue.js):

```bash
./build-all.bat
```

This script builds:
- Rust library (DLL/SO/DYLIB)
- C# wrapper and tests
- Go wrapper and tests
- All example backends and frontends (C#, Go, TypeScript, Next.js, Vue.js)

See [BUILD.md](BUILD.md) for details.

## 🆕 Version

**Current Release:** v0.6.0 ([CHANGELOG.md](CHANGELOG.md))

## 📝 Changelog

### v0.6.0 (January 2026) - **CURRENT** 🎉
- **🔧 Generic UDT Format**: New `UdtData` struct with `symbol_id` and raw bytes
- **✅ Library Health**: All 31 unit tests passing, production-ready core
- **📊 Array Element Access**: Full read/write support for array elements
- **✍️ Array Element Writing**: Write individual array elements with automatic array modification
- **🚀 C# Wrapper Enhancements**: Batch operations, TagGroup, Statistics, Data Quality & Timestamp
- **🔧 Connection Fixes**: Fixed RoutePath handling in WinForms, WPF, and ASP.NET applications
- **📚 Documentation**: Comprehensive documentation for STRING and UDT array write limitations

### v0.5.5 (December 2025)
- **📊 Array Element Access**: Full read/write support for array elements using intelligent workaround
- **✍️ Array Element Writing**: Write individual array elements with automatic array modification
- **🔧 BOOL Array Support**: Automatic DWORD bit extraction for BOOL arrays

### v0.5.4 (October 2025)
- **🔍 UDT Definition Discovery**: Automatic UDT structure detection from PLC
- **🏷️ Enhanced Tag Discovery**: Full attribute support with permissions and scope
- **📦 Packet Size Negotiation**: Dynamic optimization for firmware 20+
- **🛣️ Route Path Support**: Slot configuration and multi-hop routing
- **💾 Cache Management**: Smart caching for UDT definitions and tag attributes
- **🔧 CIP Services**: Full implementation of Services 0x03 and 0x4C
- **🧪 Comprehensive Testing**: 14 new unit tests for UDT discovery features
- **📚 Documentation**: Complete API documentation and examples

See [CHANGELOG.md](CHANGELOG.md) for a full list of changes.

## 🚀 Release Notes

See [RELEASE_NOTES_v0.5.0.md](RELEASE_NOTES_v0.5.0.md) for detailed release notes and migration info.

## 🚀 Quick Start: Go + Next.js Fullstack Example

- See [examples/gonextjs/README.md](examples/gonextjs/README.md) for step-by-step instructions.
- Features: Go backend (using Rust FFI), Next.js frontend, batch ops, real-time, performance, and more.