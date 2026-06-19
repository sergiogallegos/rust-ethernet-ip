param(
    [string]$OutputDir = (Join-Path $PSScriptRoot ".." "artifacts" "nuget"),
    [string]$Configuration = "Release",
    [string]$Runtime = "win-x64",
    [string]$NativeLibPath = (Join-Path $PSScriptRoot ".." "target" "release" "rust_ethernet_ip.dll"),
    # Optional: a directory containing one subdirectory per RID, each holding the
    # platform native library (e.g. <RuntimesDir>/linux-x64/librust_ethernet_ip.so,
    # <RuntimesDir>/win-x64/rust_ethernet_ip.dll). When provided, ALL of these are
    # injected so the package supports every platform; -Runtime/-NativeLibPath are
    # ignored.
    [string]$RuntimesDir = ""
)

$ErrorActionPreference = "Stop"

$root = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
$project = Join-Path $root "csharp" "RustEtherNetIp" "RustEtherNetIp.csproj"
$resolvedOutputDir = [System.IO.Path]::GetFullPath($OutputDir)

# Resolve the set of (RID, native file) pairs to inject.
$runtimePairs = @()
if ($RuntimesDir -ne "") {
    $resolvedRuntimesDir = [System.IO.Path]::GetFullPath($RuntimesDir)
    if (-not (Test-Path $resolvedRuntimesDir)) {
        throw "Missing runtimes directory: $resolvedRuntimesDir"
    }
    foreach ($ridDir in Get-ChildItem -Path $resolvedRuntimesDir -Directory) {
        $nativeFile = Get-ChildItem -Path $ridDir.FullName -File |
            Where-Object { $_.Extension -in @(".dll", ".so", ".dylib") } |
            Select-Object -First 1
        if ($null -ne $nativeFile) {
            $runtimePairs += [pscustomobject]@{ Rid = $ridDir.Name; Path = $nativeFile.FullName }
        }
    }
    if ($runtimePairs.Count -eq 0) {
        throw "No native libraries found under $resolvedRuntimesDir"
    }
}
else {
    $resolvedNativeLib = [System.IO.Path]::GetFullPath($NativeLibPath)
    if (-not (Test-Path $resolvedNativeLib)) {
        throw "Missing native library: $resolvedNativeLib. Build it first with 'cargo build --release'."
    }
    $runtimePairs += [pscustomobject]@{ Rid = $Runtime; Path = $resolvedNativeLib }
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

    foreach ($pair in $runtimePairs) {
        $runtimeDir = Join-Path $tempDir "runtimes" $pair.Rid "native"
        New-Item -ItemType Directory -Force -Path $runtimeDir | Out-Null
        Copy-Item -Path $pair.Path -Destination (Join-Path $runtimeDir ([System.IO.Path]::GetFileName($pair.Path))) -Force
        Write-Host "  injected $($pair.Rid)/native/$([System.IO.Path]::GetFileName($pair.Path))"
    }

    Remove-Item $package.FullName -Force
    [System.IO.Compression.ZipFile]::CreateFromDirectory($tempDir, $package.FullName)
}
finally {
    Remove-Item -Path $tempDir -Recurse -Force -ErrorAction SilentlyContinue
}

Write-Host "NuGet package ready: $($package.FullName)"
