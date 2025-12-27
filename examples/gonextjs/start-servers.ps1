# Quick Start Script for Go + Next.js Example
# This script builds the Rust library, copies the DLL, and starts both servers

Write-Host "🚀 Starting Go + Next.js Example Servers" -ForegroundColor Cyan
Write-Host ""

# Get the project root directory (assuming script is in examples/gonextjs/)
$scriptPath = Split-Path -Parent $MyInvocation.MyCommand.Path
$projectRoot = Join-Path $scriptPath "..\.."

# Step 1: Build Rust library
Write-Host "📦 Step 1: Building Rust library..." -ForegroundColor Yellow
Set-Location $projectRoot
cargo build --release
if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ Failed to build Rust library" -ForegroundColor Red
    exit 1
}
Write-Host "✅ Rust library built successfully" -ForegroundColor Green

# Step 2: Copy DLL to backend
Write-Host ""
Write-Host "📋 Step 2: Copying DLL to backend..." -ForegroundColor Yellow
$dllSource = Join-Path $projectRoot "target\release\rust_ethernet_ip.dll"
$dllDest = Join-Path $scriptPath "backend\rust_ethernet_ip.dll"

if (Test-Path $dllSource) {
    Copy-Item -Path $dllSource -Destination $dllDest -Force
    Write-Host "✅ DLL copied to backend" -ForegroundColor Green
} else {
    Write-Host "❌ DLL not found at: $dllSource" -ForegroundColor Red
    exit 1
}

# Step 3: Build Go backend (if needed)
Write-Host ""
Write-Host "🔨 Step 3: Building Go backend..." -ForegroundColor Yellow
$backendDir = Join-Path $scriptPath "backend"
Set-Location $backendDir

if (-not (Test-Path "main.exe")) {
    Write-Host "Building Go backend executable..." -ForegroundColor Gray
    go build -o main.exe .
    if ($LASTEXITCODE -ne 0) {
        Write-Host "❌ Failed to build Go backend" -ForegroundColor Red
        exit 1
    }
    Write-Host "✅ Go backend built successfully" -ForegroundColor Green
} else {
    Write-Host "✅ Go backend executable already exists" -ForegroundColor Green
}

# Step 4: Start backend in new window
Write-Host ""
Write-Host "🌐 Step 4: Starting Go backend server..." -ForegroundColor Yellow
Start-Process powershell -ArgumentList "-NoExit", "-Command", "cd '$backendDir'; Write-Host '🚀 Go Backend Server (Port 8080)' -ForegroundColor Cyan; Write-Host ''; .\main.exe"
Start-Sleep -Seconds 2
Write-Host "✅ Backend server started in new window" -ForegroundColor Green

# Step 5: Install frontend dependencies (if needed)
Write-Host ""
Write-Host "📦 Step 5: Checking frontend dependencies..." -ForegroundColor Yellow
$frontendDir = Join-Path $scriptPath "frontend"
Set-Location $frontendDir

if (-not (Test-Path "node_modules")) {
    Write-Host "Installing npm dependencies..." -ForegroundColor Gray
    npm install
    if ($LASTEXITCODE -ne 0) {
        Write-Host "❌ Failed to install npm dependencies" -ForegroundColor Red
        exit 1
    }
    Write-Host "✅ Dependencies installed" -ForegroundColor Green
} else {
    Write-Host "✅ Dependencies already installed" -ForegroundColor Green
}

# Step 6: Start frontend in new window
Write-Host ""
Write-Host "🎨 Step 6: Starting Next.js frontend..." -ForegroundColor Yellow
Start-Process powershell -ArgumentList "-NoExit", "-Command", "cd '$frontendDir'; Write-Host '🎨 Next.js Frontend (Port 3000)' -ForegroundColor Cyan; Write-Host ''; npm run dev"
Start-Sleep -Seconds 2
Write-Host "✅ Frontend server started in new window" -ForegroundColor Green

# Summary
Write-Host ""
Write-Host "═══════════════════════════════════════════════════════════" -ForegroundColor Cyan
Write-Host "✅ All servers started successfully!" -ForegroundColor Green
Write-Host ""
Write-Host "📍 Backend:  http://localhost:8080" -ForegroundColor Yellow
Write-Host "📍 Frontend: http://localhost:3000" -ForegroundColor Yellow
Write-Host ""
Write-Host "💡 Open your browser and navigate to:" -ForegroundColor Cyan
Write-Host "   http://localhost:3000" -ForegroundColor White
Write-Host ""
Write-Host "📝 To stop servers:" -ForegroundColor Cyan
Write-Host "   - Press Ctrl+C in each server window" -ForegroundColor Gray
Write-Host "   - Or close the PowerShell windows" -ForegroundColor Gray
Write-Host "═══════════════════════════════════════════════════════════" -ForegroundColor Cyan
Write-Host ""

# Return to project root
Set-Location $projectRoot

