$ErrorActionPreference = "Stop"

$ExampleDirectory = Split-Path -Parent $MyInvocation.MyCommand.Path
$RepositoryRoot = Resolve-Path (Join-Path $ExampleDirectory "../..")

Push-Location $RepositoryRoot
try {
    cargo build --release --features ffi --locked
}
finally {
    Pop-Location
}

Push-Location (Join-Path $ExampleDirectory "frontend")
try {
    npm ci
    npm run build
}
finally {
    Pop-Location
}

Set-Location $ExampleDirectory
dotnet run -c Release
