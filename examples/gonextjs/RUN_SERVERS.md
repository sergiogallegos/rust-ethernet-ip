# Running the Go + Next.js Example

This guide explains how to run the backend and frontend servers for testing the EtherNet/IP library with array element access support.

## Prerequisites

1. **Rust library built**: The `rust_ethernet_ip.dll` must be compiled
   ```powershell
   cargo build --release
   ```

2. **Go installed**: Version 1.23.0 or later
   ```powershell
   go version
   ```

3. **Node.js installed**: Version 18 or later
   ```powershell
   node --version
   npm --version
   ```

## Step 1: Build the Rust Library

From the project root:

```powershell
cd C:\Users\Sergio Gallegos\projects\rust-ethernet-ip
cargo build --release
```

This creates `target\release\rust_ethernet_ip.dll`

## Step 2: Copy DLL to Backend

```powershell
Copy-Item -Path "target\release\rust_ethernet_ip.dll" -Destination "examples\gonextjs\backend\rust_ethernet_ip.dll" -Force
```

## Step 3: Start the Go Backend Server

Open a **new terminal/PowerShell window**:

```powershell
cd C:\Users\Sergio Gallegos\projects\rust-ethernet-ip\examples\gonextjs\backend
.\main.exe
```

Or if you need to build it first:

```powershell
cd C:\Users\Sergio Gallegos\projects\rust-ethernet-ip\examples\gonextjs\backend
go build -o main.exe .
.\main.exe
```

The backend will start on **http://localhost:8080**

You should see:
```
Starting server on :8080
```

## Step 4: Start the Next.js Frontend

Open a **second terminal/PowerShell window**:

```powershell
cd C:\Users\Sergio Gallegos\projects\rust-ethernet-ip\examples\gonextjs\frontend
npm install
npm run dev
```

The frontend will start on **http://localhost:3000**

You should see:
```
  ▲ Next.js 14.x.x
  - Local:        http://localhost:3000
```

## Step 5: Access the Application

1. Open your browser and navigate to: **http://localhost:3000**

2. **Connect to your PLC:**
   - Enter your PLC IP address (e.g., `192.168.0.1`)
   - Click the "Connect" button
   - Wait for connection confirmation

3. **Test Array Element Access:**
   - Navigate to the **"Advanced"** tab
   - Scroll down to **"Array Element Access Test (v0.5.5)"**
   - Select a test type:
     - **"All Tests"** - Tests all array types
     - **"Controller-Scoped DINT Array"** - Tests `gArrayTest[0-4]`
     - **"Program-Scoped DINT Array"** - Tests `Program:MainProgram.ArrayTest[0-4]`
     - **"Controller-Scoped BOOL Array"** - Tests `gArrayBoolTest[0-9]`
   - Click **"🚀 Run Array Element Tests"**
   - Review the results showing read/write/verify status

## Stopping the Servers

To stop the servers:

1. **Backend**: Press `Ctrl+C` in the backend terminal
2. **Frontend**: Press `Ctrl+C` in the frontend terminal

Or use PowerShell:

```powershell
# Stop backend
Get-Process | Where-Object { $_.ProcessName -eq "main" } | Stop-Process -Force

# Stop frontend (Node.js)
Get-Process | Where-Object { $_.ProcessName -eq "node" } | Stop-Process -Force
```

## Troubleshooting

### Backend won't start
- **Error: "rust_ethernet_ip.dll not found"**
  - Make sure you copied the DLL to `examples\gonextjs\backend\`
  - Verify the DLL exists: `Test-Path "examples\gonextjs\backend\rust_ethernet_ip.dll"`

- **Error: "go.mod not found"**
  - Make sure you're in the correct directory: `examples\gonextjs\backend`
  - Run `go mod tidy` to fix dependencies

### Frontend won't start
- **Error: "node_modules not found"**
  - Run `npm install` in the frontend directory

- **Error: "Port 3000 already in use"**
  - Stop any other Node.js processes
  - Or change the port in `package.json`

### Connection issues
- **Backend not responding**
  - Check if backend is running on port 8080: `netstat -ano | findstr ":8080"`
  - Check backend terminal for error messages

- **Frontend can't connect to backend**
  - Verify backend is running on `http://localhost:8080`
  - Check browser console for CORS errors
  - Verify API endpoint: `http://localhost:8080/api/health`

## Quick Start Script

You can create a PowerShell script to automate the setup:

**`start-servers.ps1`**:
```powershell
# Build Rust library
Write-Host "Building Rust library..."
cargo build --release

# Copy DLL
Write-Host "Copying DLL..."
Copy-Item -Path "target\release\rust_ethernet_ip.dll" -Destination "examples\gonextjs\backend\rust_ethernet_ip.dll" -Force

# Start backend in new window
Write-Host "Starting backend..."
Start-Process powershell -ArgumentList "-NoExit", "-Command", "cd 'examples\gonextjs\backend'; .\main.exe"

# Wait a moment
Start-Sleep -Seconds 2

# Start frontend in new window
Write-Host "Starting frontend..."
Start-Process powershell -ArgumentList "-NoExit", "-Command", "cd 'examples\gonextjs\frontend'; npm run dev"

Write-Host "✅ Servers starting in separate windows"
Write-Host "Backend: http://localhost:8080"
Write-Host "Frontend: http://localhost:3000"
```

Run it with:
```powershell
.\start-servers.ps1
```

## Testing Array Element Access

The array element access feature (v0.5.5) uses a workaround that:
1. Reads the entire base array
2. Extracts the specific element from the response
3. For writes: Modifies the element in the array and writes the entire array back

This workaround is necessary because direct CIP path element access is not supported by the PLC firmware.

### Expected Test Results

- **Controller DINT Array** (`gArrayTest[0-4]`):
  - Read: ✅ Should read values 1-10 (or current PLC values)
  - Write: ✅ Should write values 100-104
  - Verify: ✅ Should match written values

- **Program DINT Array** (`Program:MainProgram.ArrayTest[0-4]`):
  - Read: ✅ Should read values 1-10 (or current PLC values)
  - Write: ✅ Should write values 200-204
  - Verify: ✅ Should match written values

- **BOOL Array** (`gArrayBoolTest[0-9]`):
  - Read: ✅ Should read boolean values
  - Write: ✅ Should toggle boolean values
  - Verify: ✅ Should match written values

## API Endpoints

The backend provides these endpoints:

- `POST /api/connect` - Connect to PLC
- `POST /api/disconnect` - Disconnect from PLC
- `GET /api/tag?name=<tag_name>&type=<type>` - Read tag
- `POST /api/tag` - Write tag
- `POST /api/test-arrays` - Test array element access
- `GET /api/health` - Health check
- `GET /api/status` - Connection status

## Notes

- Keep both terminal windows open while testing
- The backend must be running before the frontend can connect
- Array element access uses the workaround automatically - no special syntax needed
- All array operations (read/write) work with the standard tag syntax: `ArrayName[index]`

