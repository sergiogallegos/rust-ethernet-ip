param(
    [string]$OutputDir = (Join-Path $PSScriptRoot ".." "artifacts" "nuget"),
    [string]$Configuration = "Release",
    [string]$Runtime = "win-x64",
    [string]$NativeLibPath = (Join-Path $PSScriptRoot ".." "target" "release" "rust_ethernet_ip.dll")
)

$ErrorActionPreference = "Stop"

$root = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
$project = Join-Path $root "csharp" "RustEtherNetIp" "RustEtherNetIp.csproj"
$resolvedOutputDir = [System.IO.Path]::GetFullPath($OutputDir)
$resolvedNativeLib = [System.IO.Path]::GetFullPath($NativeLibPath)

if (-not (Test-Path $resolvedNativeLib)) {
    throw "Missing native library: $resolvedNativeLib. Build it first with 'cargo build --release'."
}

New-Item -ItemType Directory -Force -Path $resolvedOutputDir | Out-Null

dotnet pack $project -c $Configuration -o $resolvedOutputDir

$package = Get-ChildItem -Path $resolvedOutputDir -Filter "RustEtherNetIp.*.nupkg" |
    Where-Object { $_.Name -notlike "*.symbols.nupkg" } |
    Sort-Object LastWriteTimeUtc -Descending |
    Select-Object -First 1

if ($null -eq $package) {
    throw "Could not find generated NuGet package in $resolvedOutputDir"
}

Add-Type -AssemblyName System.IO.Compression.FileSystem

$tempDir = Join-Path ([System.IO.Path]::GetTempPath()) ("rust-ethernet-ip-nuget-" + [System.Guid]::NewGuid().ToString("N"))
New-Item -ItemType Directory -Force -Path $tempDir | Out-Null

try {
    [System.IO.Compression.ZipFile]::ExtractToDirectory($package.FullName, $tempDir)

    $runtimeDir = Join-Path $tempDir "runtimes" $Runtime "native"
    New-Item -ItemType Directory -Force -Path $runtimeDir | Out-Null
    Copy-Item -Path $resolvedNativeLib -Destination (Join-Path $runtimeDir ([System.IO.Path]::GetFileName($resolvedNativeLib))) -Force

    Remove-Item $package.FullName -Force
    [System.IO.Compression.ZipFile]::CreateFromDirectory($tempDir, $package.FullName)
}
finally {
    Remove-Item -Path $tempDir -Recurse -Force -ErrorAction SilentlyContinue
}

Write-Host "NuGet package ready: $($package.FullName)"
