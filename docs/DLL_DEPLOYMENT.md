# DLL Deployment Guide

> Historical reference: this guide is Windows- and legacy-wrapper-centric and still references removed `gowrapper/` paths. Use the current packaging and wrapper docs for the active deployment story.

## Overview

The `rust_ethernet_ip.dll` is the native Rust library compiled for Windows. This DLL must be present in the same directory as the executable or in a location where the .NET runtime can find it.

## DLL Location

**Source:** `target/release/rust_ethernet_ip.dll` (after running `cargo build --release`)

## Deployment Locations

The DLL has been copied to the following locations for examples and wrappers:

### C# Examples
- `examples/WinFormsExample/rust_ethernet_ip.dll`
- `examples/WpfExample/rust_ethernet_ip.dll`
- `examples/AspNetExample/rust_ethernet_ip.dll`
- `examples/CSharpFFITest/rust_ethernet_ip.dll`
- `examples/csharp_examples/rust_ethernet_ip.dll`

### C# Wrapper Library
- `csharp/RustEtherNetIp/rust_ethernet_ip.dll`

### Other Examples
- `examples/web_app/frontend/rust_ethernet_ip.dll`
- `examples/VueExample/rust_ethernet_ip.dll`
- `examples/gonextjs/backend/rust_ethernet_ip.dll`

### Go Wrapper
- `gowrapper/rust_ethernet_ip.dll`

### Root
- `rust_ethernet_ip.dll` (root directory)
- `examples/rust_ethernet_ip.dll`

## Building the DLL

To rebuild the DLL:

```bash
cargo build --release
```

The DLL will be generated at: `target/release/rust_ethernet_ip.dll`

## Updating DLLs

To update all DLLs after rebuilding:

### Windows PowerShell
```powershell
# Copy to all locations
Copy-Item -Path "target\release\rust_ethernet_ip.dll" -Destination "examples\WinFormsExample\rust_ethernet_ip.dll" -Force
Copy-Item -Path "target\release\rust_ethernet_ip.dll" -Destination "examples\WpfExample\rust_ethernet_ip.dll" -Force
Copy-Item -Path "target\release\rust_ethernet_ip.dll" -Destination "examples\AspNetExample\rust_ethernet_ip.dll" -Force
Copy-Item -Path "target\release\rust_ethernet_ip.dll" -Destination "csharp\RustEtherNetIp\rust_ethernet_ip.dll" -Force
Copy-Item -Path "target\release\rust_ethernet_ip.dll" -Destination "examples\CSharpFFITest\rust_ethernet_ip.dll" -Force
Copy-Item -Path "target\release\rust_ethernet_ip.dll" -Destination "examples\csharp_examples\rust_ethernet_ip.dll" -Force
Copy-Item -Path "target\release\rust_ethernet_ip.dll" -Destination "examples\web_app\frontend\rust_ethernet_ip.dll" -Force
Copy-Item -Path "target\release\rust_ethernet_ip.dll" -Destination "examples\VueExample\rust_ethernet_ip.dll" -Force
Copy-Item -Path "target\release\rust_ethernet_ip.dll" -Destination "examples\gonextjs\backend\rust_ethernet_ip.dll" -Force
Copy-Item -Path "target\release\rust_ethernet_ip.dll" -Destination "gowrapper\rust_ethernet_ip.dll" -Force
Copy-Item -Path "target\release\rust_ethernet_ip.dll" -Destination "examples\rust_ethernet_ip.dll" -Force
Copy-Item -Path "target\release\rust_ethernet_ip.dll" -Destination "rust_ethernet_ip.dll" -Force
```

### Automated Script

You can create a PowerShell script `update-dlls.ps1`:

```powershell
# update-dlls.ps1
$dllPath = "target\release\rust_ethernet_ip.dll"
$destinations = @(
    "examples\WinFormsExample",
    "examples\WpfExample",
    "examples\AspNetExample",
    "csharp\RustEtherNetIp",
    "examples\CSharpFFITest",
    "examples\csharp_examples",
    "examples\web_app\frontend",
    "examples\VueExample",
    "examples\gonextjs\backend",
    "gowrapper",
    "examples",
    "."
)

foreach ($dest in $destinations) {
    $fullDest = Join-Path $dest "rust_ethernet_ip.dll"
    Copy-Item -Path $dllPath -Destination $fullDest -Force
    Write-Host "Copied to: $fullDest"
}
```

## Runtime Requirements

The DLL requires:
- Windows x64 architecture
- Visual C++ Redistributable (usually already installed)
- .NET runtime (for C# examples)

## Troubleshooting

### DLL Not Found Error

If you get a "DLL not found" error:

1. **Check DLL location**: Ensure `rust_ethernet_ip.dll` is in the same directory as your executable
2. **Check architecture**: Ensure you're using x64 build (not x86)
3. **Check dependencies**: Ensure Visual C++ Redistributable is installed
4. **Check PATH**: The DLL should be in the same directory or in PATH

### For .NET Projects

The DLL should be:
- In the same directory as the `.exe` or `.dll`
- Or configured in the `.csproj` file to copy during build:

```xml
<ItemGroup>
  <None Include="..\..\target\release\rust_ethernet_ip.dll">
    <CopyToOutputDirectory>PreserveNewest</CopyToOutputDirectory>
  </None>
</ItemGroup>
```

## Verification

To verify the DLL is in the correct location:

```powershell
# Check if DLL exists in example directories
Get-ChildItem -Recurse -Filter "rust_ethernet_ip.dll" | Select-Object FullName, Length, LastWriteTime
```

## Last Updated

All DLLs were last updated after building with `cargo build --release` on the latest codebase with RoutePath and UdtData support.
