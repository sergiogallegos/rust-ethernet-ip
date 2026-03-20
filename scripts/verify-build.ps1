#!/usr/bin/env pwsh
<#
.SYNOPSIS
    Verify Rust + C# build and test health for release hardening.
.DESCRIPTION
    Runs version consistency checks, Rust builds/tests, C# builds/tests,
    and validates expected native/managed artifacts.
.EXAMPLE
    .\scripts\verify-build.ps1
#>

$ErrorActionPreference = "Stop"
$success = $true

function Test-Command {
    param(
        [Parameter(Mandatory = $true)][string]$Command,
        [Parameter(Mandatory = $true)][string]$Description
    )

    Write-Host "Running: $Description" -ForegroundColor Yellow
    try {
        Invoke-Expression $Command
        Write-Host "PASS: $Description" -ForegroundColor Green
        return $true
    }
    catch {
        Write-Host "FAIL: $Description" -ForegroundColor Red
        Write-Host "Error: $($_.Exception.Message)" -ForegroundColor Red
        return $false
    }
}

Write-Host "Verifying Rust EtherNet/IP build health..." -ForegroundColor Green

# Version consistency
Write-Host "`nChecking version consistency..." -ForegroundColor Cyan
$cargoVersion = (Get-Content "Cargo.toml" | Select-String 'version = "(.+)"' | Select-Object -First 1).Matches[0].Groups[1].Value
$versionFile = (Get-Content "VERSION" -Raw).Trim()

Write-Host "Cargo.toml version: $cargoVersion" -ForegroundColor White
Write-Host "VERSION file:      $versionFile" -ForegroundColor White

if ($cargoVersion -ne $versionFile) {
    Write-Host "FAIL: VERSION does not match Cargo.toml" -ForegroundColor Red
    $success = $false
}
else {
    Write-Host "PASS: Version consistency" -ForegroundColor Green
}

# Rust verification
Write-Host "`nBuilding Rust artifacts..." -ForegroundColor Cyan
$success = $success -and (Test-Command "cargo check" "Rust syntax check")
$success = $success -and (Test-Command "cargo build --release --lib" "Rust release library build")
$success = $success -and (Test-Command "cargo test --lib" "Rust unit tests")

# C# verification
Write-Host "`nBuilding C# projects..." -ForegroundColor Cyan
$csharpProjects = @(
    @{ Path = "csharp/RustEtherNetIp/RustEtherNetIp.csproj"; Name = "C# wrapper" },
    @{ Path = "csharp/RustEtherNetIp.Tests/RustEtherNetIp.Tests.csproj"; Name = "C# test project" },
    @{ Path = "examples/WpfExample/WpfExample.csproj"; Name = "WPF example" },
    @{ Path = "examples/WinFormsExample/WinFormsExample.csproj"; Name = "WinForms example" },
    @{ Path = "examples/AspNetExample/AspNetExample.csproj"; Name = "ASP.NET example" }
)

foreach ($project in $csharpProjects) {
    if (Test-Path $project.Path) {
        $success = $success -and (Test-Command "dotnet build `"$($project.Path)`" --configuration Release" $project.Name)
    }
    else {
        Write-Host "WARN: Project not found: $($project.Path)" -ForegroundColor Yellow
    }
}

$testsProject = "csharp/RustEtherNetIp.Tests/RustEtherNetIp.Tests.csproj"
if (Test-Path $testsProject) {
    $success = $success -and (Test-Command "dotnet test `"$testsProject`" --configuration Release --verbosity minimal" "C# tests")
}

# Artifact verification
Write-Host "`nValidating build artifacts..." -ForegroundColor Cyan
$nativeLibPath = "target/release/rust_ethernet_ip.dll"
if (Test-Path $nativeLibPath) {
    Write-Host "PASS: Native library found at $nativeLibPath" -ForegroundColor Green
}
else {
    Write-Host "FAIL: Native library not found at $nativeLibPath" -ForegroundColor Red
    $success = $false
}

$managedDll = Get-ChildItem "csharp/RustEtherNetIp/bin/Release" -Recurse -Filter "RustEtherNetIp.dll" -ErrorAction SilentlyContinue |
    Where-Object { $_.FullName -notmatch "\\ref\\" } |
    Select-Object -First 1

if ($null -ne $managedDll) {
    Write-Host "PASS: Managed wrapper found at $($managedDll.FullName)" -ForegroundColor Green
}
else {
    Write-Host "FAIL: Managed wrapper not found under csharp/RustEtherNetIp/bin/Release" -ForegroundColor Red
    $success = $false
}

Write-Host "`nBuild verification summary" -ForegroundColor Cyan
if ($success) {
    Write-Host "ALL CHECKS PASSED" -ForegroundColor Green
    exit 0
}
else {
    Write-Host "SOME CHECKS FAILED" -ForegroundColor Red
    exit 1
}
