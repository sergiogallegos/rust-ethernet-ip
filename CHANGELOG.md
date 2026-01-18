# Changelog

All notable changes to the rust-ethernet-ip project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.6.1] - 2026-01-17

### 🧹 Removed
- **Go Wrapper**: Removed `gowrapper/` directory to focus on Rust library and C# integration
- **Python Wrapper**: Removed `pywrapper/` directory to focus on Rust library and C# integration
- **Go Examples**: Removed `GoWrapperTest` and `gonextjs` examples
- **Python Examples**: Removed `PythonWrapperTest` and `PLC_Monitor_Dashboard` examples
- **TypeScript/Vue Examples**: Removed `TypeScriptExample`, `VueExample` to streamline examples

### ✨ Changed
- **Repository Focus**: Streamlined to focus on Rust library, Rust native examples, C# wrapper, and C# examples (WinForms, WPF, ASP.NET)
- **Documentation**: Updated all documentation to reflect current focus on Microsoft stack
- **Cargo.toml**: Removed Python dependencies and workspace members

### 📚 Documentation
- **README.md**: Updated to remove Go/Python references and focus on Microsoft stack
- **Version References**: Updated all version references from 0.6.0 to 0.6.1

## [0.6.0] - 2025-01-XX

### ✨ Added - C# Wrapper Enhancements
- **Batch Operations**: `ReadTagsBatch()` and `WriteTagsBatch()` for high-performance multi-tag operations
- **TagGroup**: Periodic polling with event-driven updates (`TagGroup` class with `DataChanged` event)
- **Performance Statistics**: `ClientStatistics` class tracking read/write counts, errors, and average response times
- **Data Quality & Timestamp**: `TagReadResult` with `Quality`, `TimeStamp`, and detailed error information
- **Value Scaling**: `ValueScaling` utility class with `ScaleLinear()` and `ScaleSquareRoot()` methods
- **Enhanced Error Handling**: Detailed error messages with quality indicators and timestamps

### 🔧 Fixed - Connection & RoutePath
- **WinForms Application**: Fixed connection to use `ConnectWithRoute()` when RoutePath is enabled
- **WPF Application**: Fixed connection to use `ConnectWithRoute()` when RoutePath is enabled
- **ASP.NET Application**: Updated `PlcService.Connect()` to accept and use RoutePath parameters
- **Connection Verification**: Added automatic connection tests after successful connection
- **Error Handling**: Improved error messages and exception handling across all example applications

### 🐛 Fixed - Type System
- **TagReadResult Duplicate**: Renamed internal `TagReadResult` to `TagReadResultBatch` to resolve naming conflicts
- **Nullability Warnings**: Fixed nullable reference type warnings in `PlcValue`, `TagSubscription`, `UdtData`, and `EthernetNetIpClient`
- **DLL Deployment**: Fixed DLL path in `RustEtherNetIp.csproj` to ensure `rust_ethernet_ip.dll` is correctly copied

### 📚 Documentation
- **Known Limitations**: Added comprehensive documentation for STRING and UDT array write limitations
- **AB_String_UDT_Write_Limitations.md**: Detailed technical document explaining PLC firmware restrictions
- **Updated Examples**: All example applications (WinForms, WPF, ASP.NET) updated with new features and proper error handling

### 🎯 Current Development Focus
- **.NET Stack**: Actively polishing C# wrapper and example applications to production quality

## [0.5.3] - 2025-01-15

### Fixed
- **Tag Discovery**: Fixed `discover_tags()` function to properly discover and parse tag lists from PLCs
- **Program Tag Reading**: Fixed reading of program tags like `Program:ProgramName.TagName` that were failing with "Path segment error"
- **CIP Request Format**: Updated tag list requests to use correct `GET_INSTANCE_ATTRIBUTE_LIST` service
- **Response Parsing**: Fixed tag list response parsing to handle proper attribute list format
- **Tag Path Building**: Improved tag path building to correctly handle program prefixes

### Technical Details
- Updated CIP request building to match working Node.js implementation
- Fixed response parsing format from `[name_len][name][type]` to `[InstanceID(4)][NameLength(2)][Name][Type(2)]`
- Added proper program tag path splitting and segment building
- Enhanced error handling and debugging output for tag operations

## [0.5.2] - 2025-01-15

### 🔧 Code Quality & Documentation Improvements
- **Enhanced FFI safety documentation**: Added comprehensive `# Safety` sections to all unsafe functions
- **Clippy optimizations**: Fixed needless range loops, vec initialization patterns, and pointer arithmetic
- **PyO3 integration**: Resolved non-local impl definition warnings with proper allow attributes
- **Memory safety**: Enhanced pointer validation and buffer overflow protection
- **Build system**: Added criterion dependency for benchmarks and improved build scripts
- **Code formatting**: Consistent formatting across all files with proper doc comment structure
- **Test infrastructure**: All 47 tests pass with enhanced coverage and reliability

### 🛠️ Development Experience
- **Benchmark compatibility**: Fixed criterion version compatibility issues
- **Error handling**: Improved error handling in FFI layer and connection management
- **Documentation**: Enhanced API documentation with better examples and safety guidelines
- **Wrapper updates**: Synchronized all wrapper versions (Python, C#, JavaScript/TypeScript, Go)

## [0.5.1] - 2025-01-15

### ⚡ Performance Improvements
- **Memory allocation optimizations**: 20-30% reduction in allocation overhead for network operations
- **Vec::with_capacity() implementation**: Pre-allocated buffers for CIP requests and packet building
- **Code quality enhancements**: Fixed clippy lints with more idiomatic Rust patterns
- **Network efficiency**: Optimized packet building with reduced memory fragmentation
- **Throughput improvements**: 20% increase in single tag operations (2,500+ → 3,000+ ops/sec)
- **Memory usage reduction**: 20% reduction in memory footprint per operation

## [0.5.0] - 2025-01-15

### 🎯 Production-Ready Release
- **Professional HMI/SCADA Demo** with real-time production monitoring
- **Production Monitoring System** with comprehensive metrics and health checks
- **Configuration Management** for production deployment
- **Production API Endpoints** for system management and monitoring
- **Performance Benchmarking Framework** for optimization and testing
- **Enhanced Real-time Monitoring** with stable continuous updates

### ✨ Added - Professional HMI/SCADA Demo
- **Real-time Production Dashboard** with live monitoring capabilities
- **OEE Analysis** (Overall Equipment Effectiveness) with availability, performance, and quality metrics
- **Process Parameter Monitoring** with color-coded alerts for temperature, pressure, vibration, and cycle time
- **Machine Status Tracking** with shift information and operator identification
- **Maintenance Management** with scheduled maintenance tracking
- **Responsive Design** that works seamlessly on desktop, tablet, and mobile devices
- **Professional UI/UX** with modern industrial aesthetics

### ✨ Added - Production Monitoring System
- **Comprehensive Metrics Collection** for connections, operations, performance, and errors
- **Health Status Monitoring** with configurable thresholds and alerting
- **Real-time Performance Tracking** with latency and throughput metrics
- **Error Categorization** with detailed error analysis and reporting
- **System Uptime Tracking** with automatic health status calculation
- **Memory and CPU Usage Monitoring** for resource management

### ✨ Added - Configuration Management
- **Production-Ready Config System** with validation and environment-specific settings
- **PLC-Specific Configuration** for different Allen-Bradley models
- **Security and Performance Tuning** options for production deployment
- **Configuration Validation** with comprehensive error checking
- **Development vs Production** configuration presets

### ✨ Added - Production API Endpoints
- **Health Check Endpoint** (`/api/health`) for system status monitoring
- **Metrics Endpoint** (`/api/metrics`) for performance and operational data
- **Configuration Management** (`/api/config`) for runtime configuration updates
- **System Status** (`/api/status`) for comprehensive system information
- **RESTful API Design** following industry best practices

### ✨ Added - Performance Benchmarking Framework
- **Criterion-Based Benchmarking** for Rust operations
- **Comparative Analysis** capabilities for performance optimization
- **Stress Testing Framework** for long-term stability validation
- **Automated Performance Regression Testing**

### 🐛 Fixed - Real-time Monitoring Stability
- **Fixed Monitoring Flashing Issue** - Resolved the problem where monitoring status was flashing and buttons became unresponsive
- **Stable Continuous Updates** - Monitoring now works continuously without stopping after the first read
- **Proper State Management** - Fixed React closure issues that were causing monitoring to stop unexpectedly
- **Improved Error Handling** - Better error recovery and user feedback for monitoring operations

### 🚀 Performance Improvements
- **Optimized Batch Operations** with improved packet packing
- **Better Connection Pooling** for concurrent operations
- **Reduced Memory Footprint** with more efficient data structures
- **Faster Tag Path Parsing** with optimized algorithms
- **Enhanced Network Resilience** with improved connection handling

### 📚 Documentation Updates
- **Production Deployment Guide** with step-by-step instructions
- **Configuration Reference** with all available options and examples
- **Troubleshooting Guide** for common issues and solutions
- **Performance Tuning Guide** for optimal system configuration
- **Updated All Examples** with the latest features and best practices

## [0.4.0] - 2025-01-15

### 🎯 Major Production Release
- **Real-time tag subscriptions** with millisecond-level updates
- **High-performance batch operations** for enterprise applications
- **Critical stability fixes** resolving all hanging and timeout issues
- **Enhanced Allen-Bradley STRING support** with complete CIP protocol compliance
- **Industrial-grade reliability** with comprehensive error handling and recovery
- **Python wrapper** with full API coverage and type-safe bindings

### ✨ Added - Real-Time Subscriptions
- **Real-time tag monitoring** with configurable update intervals (1ms - 10s)
- **Event-driven notifications** for tag value changes
- **Subscription management** with automatic reconnection and error recovery
- **Multi-tag subscriptions** supporting hundreds of concurrent tag monitors
- **Callback-based architecture** for responsive industrial applications
- **Memory-efficient subscription engine** with minimal CPU overhead

### ✨ Added - High-Performance Batch Operations
- **Batch read operations** - read up to 100+ tags in a single request
- **Batch write operations** - write multiple tags atomically
- **Configurable batch sizes** with automatic optimization for PLC capabilities
- **Parallel processing** with concurrent batch execution
- **Transaction support** with rollback capabilities for critical operations
- **Performance monitoring** with detailed timing metrics (2,000+ ops/sec throughput)
- **Intelligent packet packing** to maximize network efficiency

### 🔧 Fixed - Critical Stability Issues
- **RESOLVED: Complete hanging in send_cip_request method**
  - Fixed EtherNet/IP command codes (0x6F,0x00 for SendRRData)
  - Added proper session handle management
  - Implemented 10-second timeout protection with tokio::time::timeout
  - Enhanced debug logging for troubleshooting
- **RESOLVED: String read parsing failures**
  - Fixed CPF (Common Packet Format) extraction algorithm
  - Added proper handling for Unconnected Data Item type (0x00B2)
  - Implemented correct CIP data extraction before response parsing
- **RESOLVED: Connection timeout and recovery issues**
  - Enhanced session management with automatic keep-alive
  - Improved error detection and graceful recovery
  - Added connection health monitoring and diagnostics

### 🔧 Enhanced - Allen-Bradley STRING Support
- **Complete STRING format compliance** with Allen-Bradley specifications
- **Proper CIP type 0x02A0 handling** matching PLC read/write expectations
- **Optimized string serialization** with length + data format (no padding)
- **Support for all string operations** including empty strings and special characters
- **String length validation** with proper 82-character limit enforcement
- **Enhanced debug output** for STRING operation troubleshooting

### 🔧 Enhanced - Error Handling & Diagnostics
- **Comprehensive CIP error mapping** with detailed extended status codes
- **Enhanced debug logging** throughout the protocol stack
- **Connection health monitoring** with automatic diagnostics
- **Graceful error recovery** for network interruptions and PLC restarts
- **Detailed error messages** with actionable troubleshooting information
- **Protocol-level validation** to prevent malformed requests

### 🚀 Performance Improvements
- **50% faster tag operations** due to protocol optimizations
- **2x improved throughput** for batch operations (2,000+ ops/sec)
- **Reduced memory footprint** with optimized buffer management
- **Lower latency** with streamlined packet processing (sub-millisecond improvements)
- **Enhanced connection pooling** for multi-client scenarios
- **Optimized network utilization** with intelligent request batching

### 📚 Enhanced - Documentation & Examples
- **Updated README** with v0.4.0 capabilities and performance metrics
- **Comprehensive subscription examples** showing real-time monitoring patterns
- **Batch operation tutorials** with enterprise application patterns
- **Troubleshooting guides** for common industrial networking scenarios
- **Performance tuning documentation** for high-throughput applications
- **Updated API documentation** with all new subscription and batch methods

### 🧪 Enhanced - Testing & Validation
- **Production validation** with extensive PLC testing on CompactLogix and ControlLogix
- **Stress testing** with thousands of concurrent operations
- **Network resilience testing** with connection interruption scenarios
- **Memory leak detection** and long-running stability validation
- **Performance benchmarking** with detailed metrics collection
- **Integration testing** with real industrial environments

### 🔗 Enhanced - Integration Capabilities
- **Improved C# wrapper** with subscription and batch operation support
- **Enhanced FFI exports** for better C/C++ integration
- **Thread-safe operations** with proper synchronization
- **Async/await support** throughout the API surface
- **Cross-platform validation** on Windows, Linux, and macOS
- **Docker compatibility** for containerized industrial applications
- **Python wrapper** with PyO3 integration:
  - Full API coverage with type-safe bindings
  - Synchronous and asynchronous APIs
  - Comprehensive error handling with Python exceptions
  - Easy installation via pip or maturin
  - Cross-platform support (Windows, Linux, macOS)
  - Example scripts and documentation

### 📊 Updated Performance Metrics
- **Single Tag Read**: 2,500+ ops/sec, <1ms latency (67% improvement)
- **Single Tag Write**: 1,200+ ops/sec, <2ms latency (50% improvement)
- **Batch Operations**: 2,000+ ops/sec, 5-20ms latency (NEW)
- **Real-time Subscriptions**: 1000+ tags/sec, 1-10ms update intervals (NEW)
- **Memory Usage**: ~1KB per operation, ~4KB per connection (50% reduction)
- **Connection Setup**: 50-200ms typical (60% improvement)

### 🏭 Production Readiness
- **Enterprise deployment ready** with comprehensive testing and validation
- **24/7 operation capable** with automatic error recovery
- **Scalable architecture** supporting hundreds of concurrent connections
- **Industrial network compatibility** with common plant floor configurations
- **Comprehensive logging** for production monitoring and diagnostics
- **Support for critical applications** with millisecond-level responsiveness

## [0.3.0] - 2025-06-01

### 🎯 Major Focus Shift
- **Specialized for Allen-Bradley CompactLogix and ControlLogix PLCs**
- **Optimized for PC applications** (Windows, Linux, macOS)
- **Enhanced for industrial automation** and SCADA systems
- **Production-ready Phase 1 completion** with comprehensive feature set

### ✨ Added - Enhanced Tag Addressing
- **Advanced tag path parsing** with comprehensive support for:
  - Program-scoped tags: `Program:MainProgram.Tag1`
  - Array element access: `MyArray[5]`, `MyArray[1,2,3]`
  - Bit-level operations: `MyDINT.15` (access individual bits)
  - UDT member access: `MyUDT.Member1.SubMember`
  - String operations: `MyString.LEN`, `MyString.DATA[5]`
  - Complex nested paths: `Program:Production.Lines[2].Stations[5].Motor.Status.15`

### ✨ Added - Complete Data Type Support
- **All Allen-Bradley native data types** with proper CIP encoding:
  - **SINT**: 8-bit signed integer (-128 to 127) - CIP type 0x00C2
  - **INT**: 16-bit signed integer (-32,768 to 32,767) - CIP type 0x00C3
  - **LINT**: 64-bit signed integer - CIP type 0x00C5
  - **USINT**: 8-bit unsigned integer (0 to 255) - CIP type 0x00C6
  - **UINT**: 16-bit unsigned integer (0 to 65,535) - CIP type 0x00C7
  - **UDINT**: 32-bit unsigned integer (0 to 4,294,967,295) - CIP type 0x00C8
  - **ULINT**: 64-bit unsigned integer - CIP type 0x00C9
  - **LREAL**: 64-bit IEEE 754 double precision float - CIP type 0x00CB
  - Enhanced **BOOL** (CIP type 0x00C1), **DINT** (CIP type 0x00C4), **REAL** (CIP type 0x00CA)
  - Enhanced **STRING** (CIP type 0x00DA) and **UDT** (CIP type 0x00A0) support

### ✨ Added - C# Wrapper Integration
- **Complete C# wrapper** with full .NET integration
- **22 FFI functions** covering all data types and operations:
  - Connection management: `eip_connect`, `eip_disconnect`
  - Boolean operations: `eip_read_bool`, `eip_write_bool`
  - Signed integers: `eip_read_sint`, `eip_read_int`, `eip_read_dint`, `eip_read_lint`
  - Unsigned integers: `eip_read_usint`, `eip_read_uint`, `eip_read_udint`, `eip_read_ulint`
  - Floating point: `eip_read_real`, `eip_read_lreal`
  - String and UDT operations: `eip_read_string`, `eip_read_udt`
  - Tag management: `eip_discover_tags`, `eip_get_tag_metadata`
  - Configuration: `eip_set_max_packet_size`, `eip_check_health`
- **Type-safe C# API** with comprehensive error handling
- **Cross-platform support** (Windows .dll, Linux .so, macOS .dylib)
- **NuGet package ready** for easy distribution

### ✨ Added - Build System and Automation
- **Automated build scripts** for all platforms:
  - `build.bat` for Windows with error checking and progress reporting
  - `build.sh` for Linux/macOS with cross-platform library handling
- **4-step build process**: Rust compilation → Library copy → C# build → Testing
- **Comprehensive BUILD.md guide** with:
  - Prerequisites and setup instructions
  - Cross-platform build procedures
  - Troubleshooting section
  - CI/CD pipeline examples
  - Distribution packaging instructions

### ✨ Added - Comprehensive Examples
- **Advanced Tag Addressing Example** (`examples/advanced_tag_addressing.rs`):
  - Demonstrates all tag addressing capabilities with real-world scenarios
  - Production line monitoring, motor control, recipe management
  - Complex nested UDT access and array operations
- **Data Types Showcase Example** (`examples/data_types_showcase.rs`):
  - Shows all supported data types with encoding details
  - Precision comparisons and boundary value testing
  - Performance demonstrations and validation

### 🔧 Enhanced - Core Infrastructure
- **TagPath module** (`src/tag_path.rs`):
  - Complete tag path parsing with error handling
  - CIP path generation for network transmission
  - Support for all addressing patterns
- **Enhanced error handling** with detailed CIP error mapping (40+ error codes)
- **Improved session management** with proper registration/unregistration
- **Memory safety** with proper resource cleanup and FFI safety documentation

### 🔧 Enhanced - Protocol Implementation
- **Proper CIP type codes** for all data types with correct 16-bit identifiers
- **Little-endian byte encoding** for network transmission consistency
- **Robust response parsing** for all data types with comprehensive validation
- **Enhanced EtherNet/IP encapsulation** with proper packet structure
- **Improved timeout handling** and network resilience

### 📚 Enhanced - Documentation
- **Comprehensive README** updates:
  - Focus on CompactLogix/ControlLogix PLCs
  - Production-ready status with Phase 1 completion
  - C# wrapper integration information
  - Updated performance characteristics and roadmap
- **Detailed API documentation** with examples for each function
- **C# wrapper documentation** (`csharp/RustEtherNetIp/README.md`):
  - Complete usage guide with all data types
  - Advanced tag addressing examples
  - Performance characteristics and thread safety guidance
  - Real-time monitoring examples
- **Build documentation** with comprehensive instructions
- **Updated lib.rs header** with current capabilities and architecture diagrams

### 🧪 Enhanced - Testing
- **30+ comprehensive unit tests** covering:
  - All data types with encoding/decoding validation
  - Tag path parsing for complex addressing scenarios
  - Boundary value testing for all numeric types
  - CIP type code verification
  - Little-endian encoding consistency
- **C# wrapper tests** with integration validation
- **Documentation tests** for all public APIs (marked as `no_run` for PLC examples)
- **Build verification** with automated testing in build scripts

### 🚀 Performance Improvements
- **Optimized tag path parsing** with efficient CIP path generation (10,000+ ops/sec)
- **Zero-copy operations** where possible for memory efficiency
- **Enhanced memory management** for large data operations (~8KB per connection)
- **Improved error handling** with minimal overhead
- **Network optimization** with configurable packet sizes

### 🔧 Code Quality Improvements
- **Fixed all linter warnings** and compilation issues
- **Resolved rust-analyzer warnings** about unsafe FFI operations
- **Added proper safety documentation** for all FFI functions
- **Fixed redundant closures** and error handling patterns
- **Added `#[allow(dead_code)]` attributes** for future API methods
- **Consistent error handling** using `EtherNetIpError` throughout

### 📋 Roadmap Updates
- **Phase 1**: Enhanced tag addressing ✅ **COMPLETED**
- **Phase 1**: Complete data type support ✅ **COMPLETED**
- **Phase 1**: C# wrapper integration ✅ **COMPLETED**
- **Phase 1**: Build automation ✅ **COMPLETED**
- **Phase 1**: Comprehensive testing ✅ **COMPLETED**
- **Phase 2**: Batch operations (planned Q3 2025)
- **Phase 2**: Real-time subscriptions (planned Q3-Q4 2025)
- **Phase 3**: Production v1.0 release (planned Q4 2025)

### 🏗️ Build and Distribution
- **Cross-platform library generation**:
  - Windows: `rust_ethernet_ip.dll` (783KB optimized)
  - Linux: `librust_ethernet_ip.so`
  - macOS: `librust_ethernet_ip.dylib`
- **C# NuGet package structure** ready for distribution
- **Automated build verification** with success/failure reporting
- **CI/CD ready** with GitHub Actions examples

### 📊 Performance Metrics
- **Single Tag Read**: 1,500+ ops/sec, 1-3ms latency
- **Single Tag Write**: 800+ ops/sec, 2-5ms latency
- **Tag Path Parsing**: 10,000+ ops/sec, <0.1ms latency
- **Memory Usage**: ~2KB per operation, ~8KB per connection
- **Connection Setup**: 100-500ms typical

### 🔗 Integration Capabilities
- **Native Rust API** with full async support
- **C FFI exports** for C/C++ integration
- **C# wrapper** with comprehensive .NET integration
- **Cross-language compatibility** with proper marshaling
- **Thread safety guidance** and best practices

## [0.2.0] - Previous Release

### Added
- Basic EtherNet/IP communication
- BOOL, DINT, REAL data types
- C FFI exports
- Session management

## [0.1.0] - Initial Release

### Added
- Initial project structure
- Basic PLC connection
- Simple tag operations

### Fixed
- Fixed Python wrapper's write_tag method to correctly return a boolean indicating success or failure.