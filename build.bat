@echo off
echo Building Rust EtherNet/IP Library (0.7.0 hardening line)...
echo =============================================

echo.
echo [1/4] Building Rust library (release)...
cargo build --release --lib --features ffi
if %errorlevel% neq 0 (
    echo ❌ Rust build failed!
    exit /b %errorlevel%
)
echo ✅ Rust library built successfully

echo.
echo [2/4] Copying DLL to C# project...
copy target\release\rust_ethernet_ip.dll csharp\RustEtherNetIp\
if %errorlevel% neq 0 (
    echo ❌ Failed to copy DLL!
    exit /b %errorlevel%
)
echo ✅ DLL copied successfully

echo.
echo [3/4] Building C# wrapper...
cd csharp\RustEtherNetIp
dotnet build --configuration Release
if %errorlevel% neq 0 (
    echo ❌ C# build failed!
    exit /b %errorlevel%
)
echo ✅ C# wrapper built successfully

echo.
echo [4/4] Running tests...
cd ..\RustEtherNetIp.Tests
dotnet test --configuration Release --verbosity minimal
if %errorlevel% neq 0 (
    echo ❌ Tests failed!
    exit /b %errorlevel%
)
echo ✅ All tests passed

cd ..\..

echo.
echo 🎉 Build completed successfully!
echo.
echo 📦 Outputs:
echo   Rust DLL: target\release\rust_ethernet_ip.dll
echo   C# DLL:   csharp\RustEtherNetIp\bin\Release\net10.0\RustEtherNetIp.dll
echo.
echo 🚀 Ready for deployment! 
