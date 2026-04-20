# Building the Rust EtherNet/IP Library

This guide covers building the enhanced Rust EtherNet/IP library with comprehensive data type support and C# wrapper integration.

## 📋 **Prerequisites**

### Rust Development Environment
- **Rust 1.95+** (current stable baseline)
- **Cargo** (included with Rust)
- **Git** for version control

### C# Development Environment (Optional)
- **.NET 10.0 SDK** or later
- **Visual Studio 2022** or **Visual Studio Code** with C# extension

### System Requirements
- **Windows**: Windows 10/11 (x64)
- **Linux**: Ubuntu 20.04+ or equivalent
- **macOS**: macOS 11+ (Intel/Apple Silicon)

## 🔧 **Building the Rust Library**

### 1. Clone and Setup
```bash
git clone https://github.com/your-repo/rust-ethernet-ip.git
cd rust-ethernet-ip
```

### 2. Build for Development
```bash
# Debug build (faster compilation, includes debug symbols)
cargo build

# Run tests
cargo test

# Run examples
cargo run --example advanced_tag_addressing
cargo run --example data_types_showcase
```

### 3. Build for Production
```bash
# Release build (optimized, smaller binary)
cargo build --release

# Build with specific target
cargo build --release --target x86_64-pc-windows-msvc
```

### 4. Build Dynamic Library (for C# integration)
```bash
# Build as dynamic library (.dll on Windows, .so on Linux, .dylib on macOS)
cargo build --release --lib

# The output will be in:
# Windows: target/release/rust_ethernet_ip.dll
# Linux:   target/release/librust_ethernet_ip.so
# macOS:   target/release/librust_ethernet_ip.dylib
```

## 🏗️ **Building the C# Wrapper**

### 1. Copy the Rust Library
```bash
# Windows
copy target\release\rust_ethernet_ip.dll csharp\RustEtherNetIp\

# Linux/macOS
cp target/release/librust_ethernet_ip.so csharp/RustEtherNetIp/
# or
cp target/release/librust_ethernet_ip.dylib csharp/RustEtherNetIp/
```

### 2. Build the C# Project
```bash
cd csharp/RustEtherNetIp
dotnet build
```

### 3. Run C# Tests
```bash
cd ../RustEtherNetIp.Tests
dotnet test
```

### 4. Create NuGet Package
```bash
cd ../RustEtherNetIp
dotnet pack --configuration Release
```

## 🚀 **Quick Build Script**

Create a build script for your platform:

### Windows (build.bat)
```batch
@echo off
echo Building Rust EtherNet/IP Library...

echo.
echo [1/4] Building Rust library (release)...
cargo build --release --lib
if %errorlevel% neq 0 exit /b %errorlevel%

echo.
echo [2/4] Copying DLL to C# project...
copy target\release\rust_ethernet_ip.dll csharp\RustEtherNetIp\
if %errorlevel% neq 0 exit /b %errorlevel%

echo.
echo [3/4] Building C# wrapper...
cd csharp\RustEtherNetIp
dotnet build --configuration Release
if %errorlevel% neq 0 exit /b %errorlevel%

echo.
echo [4/4] Running tests...
cd ..\RustEtherNetIp.Tests
dotnet test --configuration Release
if %errorlevel% neq 0 exit /b %errorlevel%

echo.
echo ✅ Build completed successfully!
echo.
echo Outputs:
echo   Rust DLL: target\release\rust_ethernet_ip.dll
echo   C# DLL:   csharp\RustEtherNetIp\bin\Release\net10.0\RustEtherNetIp.dll
```

### Linux/macOS (build.sh)
```bash
#!/bin/bash
set -e

echo "Building Rust EtherNet/IP Library..."

echo
echo "[1/4] Building Rust library (release)..."
cargo build --release --lib

echo
echo "[2/4] Copying library to C# project..."
if [[ "$OSTYPE" == "linux-gnu"* ]]; then
    cp target/release/librust_ethernet_ip.so csharp/RustEtherNetIp/
elif [[ "$OSTYPE" == "darwin"* ]]; then
    cp target/release/librust_ethernet_ip.dylib csharp/RustEtherNetIp/
fi

echo
echo "[3/4] Building C# wrapper..."
cd csharp/RustEtherNetIp
dotnet build --configuration Release

echo
echo "[4/4] Running tests..."
cd ../RustEtherNetIp.Tests
dotnet test --configuration Release

echo
echo "✅ Build completed successfully!"
echo
echo "Outputs:"
if [[ "$OSTYPE" == "linux-gnu"* ]]; then
    echo "  Rust Library: target/release/librust_ethernet_ip.so"
elif [[ "$OSTYPE" == "darwin"* ]]; then
    echo "  Rust Library: target/release/librust_ethernet_ip.dylib"
fi
echo "  C# DLL: csharp/RustEtherNetIp/bin/Release/net10.0/RustEtherNetIp.dll"
```

Make the script executable:
```bash
chmod +x build.sh
./build.sh
```

## 🔍 **Verification**

### 1. Verify Rust Build
```bash
# Check that all tests pass
cargo test

# Verify examples work
cargo run --example advanced_tag_addressing
cargo run --example data_types_showcase

# Check library exports (Windows)
dumpbin /exports target\release\rust_ethernet_ip.dll

# Check library exports (Linux)
nm -D target/release/librust_ethernet_ip.so

# Check library exports (macOS)
nm -D target/release/librust_ethernet_ip.dylib
```

### 2. Verify C# Integration
```bash
cd csharp/RustEtherNetIp
dotnet build --verbosity normal

# Check that the native library is found
dotnet run --project Program.cs
```

### 3. Test FFI Functions
The C# wrapper should be able to call all these Rust FFI functions:
- `eip_connect`
- `eip_disconnect`
- `eip_read_bool`, `eip_write_bool`
- `eip_read_sint`, `eip_write_sint`
- `eip_read_int`, `eip_write_int`
- `eip_read_dint`, `eip_write_dint`
- `eip_read_lint`, `eip_write_lint`
- `eip_read_usint`, `eip_write_usint`
- `eip_read_uint`, `eip_write_uint`
- `eip_read_udint`, `eip_write_udint`
- `eip_read_ulint`, `eip_write_ulint`
- `eip_read_real`, `eip_write_real`
- `eip_read_lreal`, `eip_write_lreal`
- `eip_read_string`, `eip_write_string`
- `eip_read_udt`, `eip_write_udt`
- `eip_discover_tags`
- `eip_get_tag_metadata`
- `eip_set_max_packet_size`
- `eip_check_health`

## 🐛 **Troubleshooting**

### Common Build Issues

#### Rust Compilation Errors
```bash
# Update Rust toolchain
rustup update

# Clean and rebuild
cargo clean
cargo build --release
```

#### Missing Dependencies
```bash
# Install required system packages (Ubuntu/Debian)
sudo apt-get update
sudo apt-get install build-essential pkg-config

# Install required system packages (CentOS/RHEL)
sudo yum groupinstall "Development Tools"
sudo yum install pkgconfig

# Install required system packages (macOS)
xcode-select --install
```

#### C# Build Issues
```bash
# Restore NuGet packages
dotnet restore

# Clean and rebuild
dotnet clean
dotnet build --configuration Release
```

#### Native Library Loading Issues

**Windows:**
- Ensure `rust_ethernet_ip.dll` is in the same directory as the C# executable
- Check that Visual C++ Redistributable is installed
- Verify the DLL architecture matches (x64 vs x86)

**Linux:**
- Ensure `librust_ethernet_ip.so` is in the library path
- Set `LD_LIBRARY_PATH` if needed:
  ```bash
  export LD_LIBRARY_PATH=$LD_LIBRARY_PATH:./csharp/RustEtherNetIp
  ```

**macOS:**
- Ensure `librust_ethernet_ip.dylib` is in the library path
- Set `DYLD_LIBRARY_PATH` if needed:
  ```bash
  export DYLD_LIBRARY_PATH=$DYLD_LIBRARY_PATH:./csharp/RustEtherNetIp
  ```

## 📦 **Distribution**

### Creating Release Packages

#### Rust Library Only
```bash
# Create source distribution
cargo package

# Publish to crates.io (if configured)
cargo publish
```

#### C# NuGet Package
```bash
cd csharp/RustEtherNetIp

# Create NuGet package
dotnet pack --configuration Release --output ./nupkg

# Publish to NuGet.org (if configured)
dotnet nuget push ./nupkg/*.nupkg --api-key YOUR_API_KEY --source https://api.nuget.org/v3/index.json
```

#### Complete Distribution
```bash
# Create a complete distribution with both Rust and C# components
mkdir -p dist/rust-ethernet-ip-v0.3.0

# Copy Rust artifacts
cp target/release/rust_ethernet_ip.dll dist/rust-ethernet-ip-v0.3.0/
cp target/release/rust_ethernet_ip.lib dist/rust-ethernet-ip-v0.3.0/

# Copy C# artifacts
cp csharp/RustEtherNetIp/bin/Release/net10.0/RustEtherNetIp.dll dist/rust-ethernet-ip-v0.3.0/
cp csharp/RustEtherNetIp/bin/Release/net10.0/RustEtherNetIp.xml dist/rust-ethernet-ip-v0.3.0/

# Copy documentation
cp README.md dist/rust-ethernet-ip-v0.3.0/
cp CHANGELOG.md dist/rust-ethernet-ip-v0.3.0/
cp csharp/RustEtherNetIp/README.md dist/rust-ethernet-ip-v0.3.0/CSharp-README.md

# Create archive
cd dist
tar -czf rust-ethernet-ip-v0.3.0.tar.gz rust-ethernet-ip-v0.3.0/
```

## 🎯 **Performance Optimization**

### Rust Build Optimizations
```toml
# Add to Cargo.toml for maximum performance
[profile.release]
opt-level = 3
lto = true
codegen-units = 1
panic = "abort"
```

### C# Build Optimizations
```xml
<!-- Add to .csproj for maximum performance -->
<PropertyGroup Condition="'$(Configuration)'=='Release'">
  <Optimize>true</Optimize>
  <DebugType>none</DebugType>
  <DebugSymbols>false</DebugSymbols>
  <TrimMode>link</TrimMode>
</PropertyGroup>
```

## 📊 **Build Metrics**

Expected build times on modern hardware:

| Component | Debug Build | Release Build | Notes |
|-----------|-------------|---------------|-------|
| Rust Library | 30-60s | 2-5 minutes | First build includes dependencies |
| C# Wrapper | 5-10s | 10-20s | Includes XML documentation |
| Full Test Suite | 10-30s | 30-60s | Includes integration tests |
| Complete Build | 1-2 minutes | 3-6 minutes | All components + tests |

## 🔄 **Continuous Integration**

### GitHub Actions Example
```yaml
name: Build and Test

on: [push, pull_request]

jobs:
  build:
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, windows-latest, macos-latest]
    
    steps:
    - uses: actions/checkout@v3
    
    - name: Install Rust
      uses: actions-rs/toolchain@v1
      with:
        toolchain: stable
        override: true
    
    - name: Install .NET
      uses: actions/setup-dotnet@v3
      with:
        dotnet-version: '8.0.x'
    
    - name: Build Rust Library
      run: cargo build --release --lib
    
    - name: Run Rust Tests
      run: cargo test
    
    - name: Copy Native Library
      shell: bash
      run: |
        if [[ "${{ runner.os }}" == "Windows" ]]; then
          cp target/release/rust_ethernet_ip.dll csharp/RustEtherNetIp/
        elif [[ "${{ runner.os }}" == "Linux" ]]; then
          cp target/release/librust_ethernet_ip.so csharp/RustEtherNetIp/
        else
          cp target/release/librust_ethernet_ip.dylib csharp/RustEtherNetIp/
        fi
    
    - name: Build C# Wrapper
      run: dotnet build csharp/RustEtherNetIp --configuration Release
    
    - name: Test C# Wrapper
      run: dotnet test csharp/RustEtherNetIp.Tests --configuration Release
```

This comprehensive build guide should help you successfully build and deploy the enhanced Rust EtherNet/IP library with full C# integration! 
