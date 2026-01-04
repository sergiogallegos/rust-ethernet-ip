# Updating the DLL for C# Wrapper Test

## Quick Update

If the Rust library has been rebuilt, update the DLL:

```powershell
# From the project root
Copy-Item -Path "target\release\rust_ethernet_ip.dll" -Destination "examples\CSharpWrapperTest\bin\Debug\net9.0\rust_ethernet_ip.dll" -Force
Copy-Item -Path "target\release\rust_ethernet_ip.dll" -Destination "csharp\RustEtherNetIp\rust_ethernet_ip.dll" -Force
```

## Rebuild Process

1. **Rebuild Rust library:**
   ```bash
   cargo build --release
   ```

2. **Copy DLL to C# wrapper:**
   ```powershell
   Copy-Item -Path "target\release\rust_ethernet_ip.dll" -Destination "csharp\RustEtherNetIp\rust_ethernet_ip.dll" -Force
   ```

3. **Rebuild C# test:**
   ```bash
   cd examples/CSharpWrapperTest
   dotnet clean
   dotnet build
   ```

The `.csproj` file is configured to automatically copy the DLL from `target/release/rust_ethernet_ip.dll` to the output directory during build.

## Verification

Check DLL timestamps to ensure you have the latest:

```powershell
Get-Item "target\release\rust_ethernet_ip.dll" | Select-Object LastWriteTime
Get-Item "examples\CSharpWrapperTest\bin\Debug\net9.0\rust_ethernet_ip.dll" | Select-Object LastWriteTime
```

Both should have the same timestamp (or the output DLL should be newer).

