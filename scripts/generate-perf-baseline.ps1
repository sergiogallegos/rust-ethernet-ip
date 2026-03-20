#!/usr/bin/env pwsh
<#
.SYNOPSIS
    Generate a simulator-based performance baseline report for HEAD vs a baseline git ref.
.DESCRIPTION
    Runs `examples/perf_baseline_sim.rs` on the current workspace and a detached worktree
    for the baseline ref, then writes JSON artifacts and a markdown comparison report.
.PARAMETER BaselineRef
    Git ref to compare against. Default: v0.6.3
.PARAMETER Iterations
    Number of benchmark loop iterations per scenario. Default: 500
.EXAMPLE
    .\scripts\generate-perf-baseline.ps1
.EXAMPLE
    .\scripts\generate-perf-baseline.ps1 -BaselineRef v0.6.3 -Iterations 1000
#>

param(
    [string]$BaselineRef = "v0.6.3",
    [int]$Iterations = 500
)

$ErrorActionPreference = "Stop"

function Invoke-PerfRun {
    param(
        [string]$WorkDir,
        [int]$RunIterations
    )

    $cmd = "cargo run --quiet --release --example perf_baseline_sim -- --iterations $RunIterations"
    $output = Invoke-Expression $cmd
    if (-not $output) {
        throw "No output from perf benchmark command in $WorkDir"
    }

    $jsonLine = $null
    foreach ($line in $output) {
        if ($line -match "^\{.*\}$") {
            $jsonLine = $line
        }
    }
    if (-not $jsonLine) {
        throw "Could not find JSON benchmark output in command output."
    }

    return $jsonLine | ConvertFrom-Json
}

function Get-Metric {
    param(
        [object]$Run,
        [string]$Name
    )
    return $Run.metrics | Where-Object { $_.name -eq $Name } | Select-Object -First 1
}

function Format-DeltaPercent {
    param(
        [double]$Current,
        [double]$Baseline
    )
    if ($Baseline -eq 0) {
        return "N/A"
    }
    $delta = (($Current - $Baseline) / $Baseline) * 100.0
    return ("{0:+0.00;-0.00;0.00}%" -f $delta)
}

$repoRoot = (Resolve-Path ".").Path
$outDir = Join-Path $repoRoot "docs\perf"
if (!(Test-Path $outDir)) {
    New-Item -ItemType Directory -Path $outDir | Out-Null
}

$worktreesDir = Join-Path $repoRoot ".perf_worktrees"
if (!(Test-Path $worktreesDir)) {
    New-Item -ItemType Directory -Path $worktreesDir | Out-Null
}

Write-Host "Running HEAD simulator baseline..." -ForegroundColor Cyan
Push-Location $repoRoot
$headRun = Invoke-PerfRun -WorkDir $repoRoot -RunIterations $Iterations
Pop-Location

$baselineWorktree = Join-Path $worktreesDir ("baseline-" + ($BaselineRef -replace "[^a-zA-Z0-9\.\-_]", "_"))
if (Test-Path $baselineWorktree) {
    git worktree remove --force $baselineWorktree | Out-Null
}

Write-Host "Preparing baseline worktree for $BaselineRef..." -ForegroundColor Cyan
git worktree add --detach $baselineWorktree $BaselineRef | Out-Null

try {
    Copy-Item (Join-Path $repoRoot "examples\perf_baseline_sim.rs") (Join-Path $baselineWorktree "examples\perf_baseline_sim.rs") -Force

    Write-Host "Running baseline simulator benchmark for $BaselineRef..." -ForegroundColor Cyan
    Push-Location $baselineWorktree
    $baselineRun = Invoke-PerfRun -WorkDir $baselineWorktree -RunIterations $Iterations
    Pop-Location
}
finally {
    git worktree remove --force $baselineWorktree | Out-Null
}

$headJsonPath = Join-Path $outDir "perf_head.json"
$baselineJsonPath = Join-Path $outDir ("perf_" + ($BaselineRef -replace "[^a-zA-Z0-9\.\-_]", "_") + ".json")
$headRun | ConvertTo-Json -Depth 8 | Set-Content $headJsonPath
$baselineRun | ConvertTo-Json -Depth 8 | Set-Content $baselineJsonPath

$scenarioNames = @("single_read", "single_write", "batch_read", "batch_write", "mixed_execute")

$reportPath = Join-Path $outDir "0.7.0_baseline_vs_0.6.3.md"
$timestamp = Get-Date -Format "yyyy-MM-dd HH:mm:ss zzz"
$rustVersion = (rustc --version)
$cpuName = (Get-CimInstance Win32_Processor | Select-Object -First 1 -ExpandProperty Name)
$osInfo = (Get-CimInstance Win32_OperatingSystem | Select-Object -First 1)
$osName = $osInfo.Caption
$osVersion = $osInfo.Version

$lines = @()
$lines += "# 0.7.0 Performance Baseline vs 0.6.3"
$lines += ""
$lines += "Generated: $timestamp"
$lines += ""
$lines += "## Environment"
$lines += ""
$lines += "- OS: $osName ($osVersion)"
$lines += "- CPU: $cpuName"
$lines += "- Rust: $rustVersion"
$lines += "- Iterations per scenario: $Iterations"
$lines += "- Current ref: HEAD"
$lines += "- Baseline ref: $BaselineRef"
$lines += "- Benchmark mode: simulator-based (deterministic, no hardware dependency)"
$lines += ""
$lines += "## Results"
$lines += ""
$lines += "| Scenario | HEAD ops/sec | $BaselineRef ops/sec | Delta vs $BaselineRef | HEAD avg call ms | $BaselineRef avg call ms |"
$lines += "|---|---:|---:|---:|---:|---:|"

foreach ($name in $scenarioNames) {
    $headMetric = Get-Metric -Run $headRun -Name $name
    $baselineMetric = Get-Metric -Run $baselineRun -Name $name
    if ($null -eq $headMetric -or $null -eq $baselineMetric) {
        continue
    }

    $delta = Format-DeltaPercent -Current ([double]$headMetric.ops_per_sec) -Baseline ([double]$baselineMetric.ops_per_sec)
    $lines += "| $name | {0:N2} | {1:N2} | {2} | {3:N4} | {4:N4} |" -f `
        [double]$headMetric.ops_per_sec, `
        [double]$baselineMetric.ops_per_sec, `
        $delta, `
        [double]$headMetric.avg_call_ms, `
        [double]$baselineMetric.avg_call_ms
}

$lines += ""
$lines += "## Artifacts"
$lines += ""
$lines += "- Raw HEAD metrics: docs/perf/perf_head.json"
$lines += "- Raw baseline metrics: docs/perf/" + (Split-Path $baselineJsonPath -Leaf)
$lines += "- This report: docs/perf/0.7.0_baseline_vs_0.6.3.md"
$lines += ""
$lines += "## Notes"
$lines += ""
$lines += "- This baseline uses the in-repo simulator to ensure deterministic execution in CI and offline environments."
$lines += "- Hardware PLC performance validation should be run separately for production-floor qualification."

$lines | Set-Content $reportPath

Write-Host "Baseline report generated:" -ForegroundColor Green
Write-Host "  $reportPath"
