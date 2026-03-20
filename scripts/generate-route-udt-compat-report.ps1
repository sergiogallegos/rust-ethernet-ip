#!/usr/bin/env pwsh
<#
.SYNOPSIS
    Generate route-path + UDT compatibility report for 0.7.0 hardening.
.DESCRIPTION
    Executes deterministic Rust/C# test suites focused on route-path behavior
    and UDT-heavy workloads, then writes a markdown report under docs/compat.
#>

$ErrorActionPreference = "Stop"

function Run-Step {
    param(
        [Parameter(Mandatory = $true)][string]$Name,
        [Parameter(Mandatory = $true)][scriptblock]$Action
    )

    Write-Host "Running: $Name" -ForegroundColor Cyan
    $start = Get-Date
    try {
        & $Action
        $end = Get-Date
        return [pscustomobject]@{
            Name = $Name
            Status = "PASS"
            DurationMs = [int](($end - $start).TotalMilliseconds)
            Notes = ""
        }
    }
    catch {
        $end = Get-Date
        return [pscustomobject]@{
            Name = $Name
            Status = "FAIL"
            DurationMs = [int](($end - $start).TotalMilliseconds)
            Notes = $_.Exception.Message
        }
    }
}

$repoRoot = (Resolve-Path ".").Path
$compatDir = Join-Path $repoRoot "docs\compat"
if (!(Test-Path $compatDir)) {
    New-Item -ItemType Directory -Path $compatDir | Out-Null
}

$results = @()
$results += Run-Step -Name "Rust route-path simulator compatibility" -Action {
    cargo test --test route_path_sim_compat_tests -- --nocapture | Out-Host
    if ($LASTEXITCODE -ne 0) { throw "cargo test route_path_sim_compat_tests failed ($LASTEXITCODE)" }
}
$results += Run-Step -Name "Rust UDT enhanced workload suite" -Action {
    cargo test --test udt_enhanced_tests -- --nocapture | Out-Host
    if ($LASTEXITCODE -ne 0) { throw "cargo test udt_enhanced_tests failed ($LASTEXITCODE)" }
}
$results += Run-Step -Name "Rust UDT data format suite" -Action {
    cargo test --test udt_data_tests -- --nocapture | Out-Host
    if ($LASTEXITCODE -ne 0) { throw "cargo test udt_data_tests failed ($LASTEXITCODE)" }
}
$results += Run-Step -Name "C# UDT batch wrapper compatibility" -Action {
    $env:DOTNET_CLI_HOME = Join-Path $repoRoot ".dotnet_home"
    if (!(Test-Path $env:DOTNET_CLI_HOME)) {
        New-Item -ItemType Directory -Path $env:DOTNET_CLI_HOME | Out-Null
    }
    dotnet test csharp\RustEtherNetIp.Tests\RustEtherNetIp.Tests.csproj --filter WriteTagsBatchTests --configuration Release --verbosity minimal | Out-Host
    if ($LASTEXITCODE -ne 0) { throw "dotnet test WriteTagsBatchTests failed ($LASTEXITCODE)" }
}

$timestamp = Get-Date -Format "yyyy-MM-dd HH:mm:ss zzz"
$rustVersion = (rustc --version)
$dotnetVersion = (dotnet --version)
$overall = if (($results | Where-Object { $_.Status -eq "FAIL" }).Count -eq 0) { "PASS" } else { "FAIL" }

$lines = @()
$lines += "# 0.7.0 Route-Path and UDT Compatibility Pass"
$lines += ""
$lines += "Generated: $timestamp"
$lines += ""
$lines += "## Environment"
$lines += ""
$lines += "- Rust: $rustVersion"
$lines += "- .NET SDK: $dotnetVersion"
$lines += "- Overall Status: **$overall**"
$lines += ""
$lines += "## Matrix"
$lines += ""
$lines += "| Suite | Status | Duration (ms) | Notes |"
$lines += "|---|---|---:|---|"
foreach ($r in $results) {
    $note = if ([string]::IsNullOrWhiteSpace($r.Notes)) { "-" } else { $r.Notes.Replace("|", "/") }
    $lines += "| $($r.Name) | $($r.Status) | $($r.DurationMs) | $note |"
}
$lines += ""
$lines += "## Coverage Summary"
$lines += ""
$lines += "- Route-path scenarios: connect with route, set/modify/clear route path, route-path batch/mixed execute."
$lines += "- UDT-heavy workloads: enhanced parsing/serialization/member access and raw UdtData conversion flows."
$lines += "- Cross-wrapper check: C# UDT batch behavior and payload compatibility tests."

$reportPath = Join-Path $compatDir "0.7.0_route_udt_compatibility.md"
$lines | Set-Content $reportPath

Write-Host "Compatibility report written to: $reportPath" -ForegroundColor Green
if ($overall -ne "PASS") {
    exit 1
}
