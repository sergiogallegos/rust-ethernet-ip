@echo off
echo Building Complete Rust EtherNet/IP Solution (0.7.0 hardening line)
echo ====================================================
echo.
echo ✨ This build includes:
echo   • 🦀 Rust Library - High-performance EtherNet/IP communication
echo   • 🔷 C# Wrapper - Complete .NET integration
echo   • 🖥️ WinForms Example - Desktop application
echo   • 🖥️ WPF Example - Modern desktop application
echo   • 🌐 ASP.NET Example - Web API backend
echo.

echo [1/6] 🦀 Building Rust library (release)...
echo ============================================
cargo build --release --lib --features ffi
if %errorlevel% neq 0 (
    echo ❌ Rust build failed!
    exit /b %errorlevel%
)
echo ✅ Rust library built successfully

echo.
echo [2/6] 📦 Copying DLL to projects...
echo =================================
copy target\release\rust_ethernet_ip.dll csharp\RustEtherNetIp\ >nul
copy target\release\rust_ethernet_ip.dll examples\WinFormsExample\ >nul
copy target\release\rust_ethernet_ip.dll examples\WpfExample\ >nul
copy target\release\rust_ethernet_ip.dll examples\AspNetExample\ >nul
if %errorlevel% neq 0 (
    echo ❌ Failed to copy DLL!
    exit /b %errorlevel%
)
echo ✅ DLL copied to all projects

echo.
echo [3/6] 🔷 Building C# wrapper...
echo ==============================
cd csharp\RustEtherNetIp
dotnet build --configuration Release --verbosity minimal
if %errorlevel% neq 0 (
    echo ❌ C# build failed!
    exit /b %errorlevel%
)
echo ✅ C# wrapper built successfully
cd ..\..

echo.
echo [4/6] 🖥️ Building WinForms Example...
echo ===================================
cd examples\WinFormsExample
dotnet build --configuration Release --verbosity minimal
if %errorlevel% neq 0 (
    echo ❌ WinForms build failed!
    exit /b %errorlevel%
)
echo ✅ WinForms example built successfully
cd ..\..

echo.
echo [5/6] 🖥️ Building WPF Example...
echo ===============================
cd examples\WpfExample
dotnet build --configuration Release --verbosity minimal
if %errorlevel% neq 0 (
    echo ❌ WPF build failed!
    exit /b %errorlevel%
)
echo ✅ WPF example built successfully
cd ..\..

echo.
echo [6/6] 🌐 Building ASP.NET Example...
echo =================================
cd examples\AspNetExample
dotnet build --configuration Release --verbosity minimal
if %errorlevel% neq 0 (
    echo ❌ ASP.NET build failed!
    exit /b %errorlevel%
)
echo ✅ ASP.NET example built successfully
cd ..\..

echo.
echo 🎉 COMPLETE BUILD SUCCESS!
echo =========================
echo.
echo 📦 Built Components:
echo   ✅ Rust Library (stable published version currently 0.6.3)
echo   ✅ C# Wrapper - Complete .NET integration
echo   ✅ WinForms Example - Desktop application
echo   ✅ WPF Example - Modern desktop application
echo   ✅ ASP.NET Example - Web API backend
echo.
echo 🚀 Ready for deployment!
echo.
echo 📋 Key Outputs:
echo   Rust DLL:     target\release\rust_ethernet_ip.dll
echo   C# Wrapper:    csharp\RustEtherNetIp\bin\Release\net10.0\RustEtherNetIp.dll
echo   WinForms App:  examples\WinFormsExample\bin\Release\net10.0-windows\WinFormsExample.exe
echo   WPF App:       examples\WpfExample\bin\Release\net10.0-windows\WpfExample.exe
echo   ASP.NET Web:   examples\AspNetExample\bin\Release\net10.0\AspNetExample.dll
echo.
echo 💡 Next Steps:
echo   1. Run WinForms: examples\WinFormsExample\bin\Release\net10.0-windows\WinFormsExample.exe
echo   2. Run WPF: examples\WpfExample\bin\Release\net10.0-windows\WpfExample.exe
echo   3. Run ASP.NET: dotnet run --project examples\AspNetExample
echo   4. Test Rust: cargo test
echo.
