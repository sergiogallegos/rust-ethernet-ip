# PLC Web Application - Rust Backend + React/TypeScript Frontend

This example is now a MacBook-oriented manufacturing dashboard demo built on a native Rust `axum` backend and a React/TypeScript frontend for Allen-Bradley PLC communication over EtherNet/IP.

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│              React + TypeScript Frontend                │
│  • Modern UI with React components                     │
│  • TypeScript for type safety                           │
│  • HTTP/REST API communication                         │
└────────────────────┬────────────────────────────────────┘
                     │ HTTP/REST
                     │
┌────────────────────┴────────────────────────────────────┐
│              Rust Backend (Axum)                        │
│  • High-performance async web server                    │
│  • RESTful API endpoints                                │
│  • Direct integration with rust-ethernet-ip library    │
└────────────────────┬────────────────────────────────────┘
                     │ EtherNet/IP Protocol
                     │
┌────────────────────┴────────────────────────────────────┐
│              Allen-Bradley PLC                          │
│  • CompactLogix / ControlLogix                         │
└─────────────────────────────────────────────────────────┘
```

## Features

- **Pure Rust PLC Backend**: Direct use of `rust-ethernet-ip` without wrapper dependencies
- **MacBook-Ready Web UI**: Manager-facing dashboard layout optimized for browser demos
- **Route-Path Support**: Direct or routed ControlLogix connection from the connection panel
- **Controller Identity**: Reads controller identity information for model, firmware, and vendor display
- **Live Dashboard Snapshot**: Batch-read KPI cards and monitored tag tiles refreshed from the backend
- **Trend Charts**: Time-series panels for throughput, quality proxy, cycle proxy, and availability proxy
- **Read/Write Workstation**: Manual single-tag read and write panel for supported primitive paths
- **Speed Benchmarking**: Runs a live single-vs-batch benchmark using the validated `gTest*` tag set
- **Traceability Demo**: Stores part-tracking events locally for product-flow storytelling
- **CORS Enabled**: Ready for local frontend/backend split during development

## Project Structure

```
web_app/
├── backend/              # Rust backend server
│   ├── Cargo.toml       # Rust dependencies
│   └── src/
│       └── main.rs      # Axum web server and API handlers
├── frontend/            # React/TypeScript frontend
│   ├── package.json     # Node.js dependencies
│   ├── tsconfig.json    # TypeScript configuration
│   └── src/
│       ├── App.tsx      # Main application component
│       ├── types.ts     # TypeScript type definitions
│       └── components/  # React components
│           ├── ConnectionPanel.tsx
│           ├── TagOperations.tsx
│           └── StatusBar.tsx
└── README.md            # This file
```

## Prerequisites

- **Rust**: Install from [rustup.rs](https://rustup.rs/)
- **Node.js**: Version 16 or higher (for React frontend)
- **PLC Access**: A CompactLogix or ControlLogix PLC on your network

## Setup Instructions

### 1. Backend Setup

Navigate to the backend directory and build the server:

```bash
cd examples/web_app/backend
cargo build --release
```

Or run directly in development mode:

```bash
cargo run
```

The backend server will start on `http://localhost:3000`.

### 2. Frontend Setup

Navigate to the frontend directory and install dependencies:

```bash
cd examples/web_app/frontend
npm install
```

Start the development server:

```bash
npm start
```

The frontend will start on `http://localhost:3000` (or another port if 3000 is taken).

## API Endpoints

### `GET /api/health`
Health check endpoint.

**Response:**
```json
{
  "status": "ok",
  "service": "PLC Web Backend"
}
```

### `POST /api/connect`
Connect to a PLC, with optional route-path support.

**Request:**
```json
{
  "address": "192.168.0.101:44818",
  "use_route_path": true,
  "slot": 0
}
```

**Response:**
```json
{
  "success": true,
  "message": "Connected to 192.168.0.101:44818 using route path slot 0",
  "status": {
    "connected": true,
    "address": "192.168.0.101:44818",
    "use_route_path": true,
    "slot": 0,
    "plc_identity": {
      "product_name": "1756-L81ES",
      "firmware": "37.0"
    }
  }
}
```

### `POST /api/disconnect`
Disconnect from the PLC.

**Response:**
```json
{
  "success": true,
  "message": "Disconnected from PLC"
}
```

### `GET /api/status`
Get current connection status.

The response includes connection mode, optional slot, connection timestamp, and PLC identity when available.

### `GET /api/overview`
Return dashboard-ready context:

- supported feature list
- validated notes
- known controller limitations
- default monitored tag configuration

### `GET /api/demo/snapshot`
Read the demo KPI tags in one batch and return:

- refresh latency
- KPI cards
- monitored signal tiles

### `POST /api/demo/benchmark`
Run the live speed demo against the validated benchmark tag set.

**Request:**
```json
{
  "iterations": 25
}
```

### `GET /api/traceability`
Return saved traceability / part-tracking demo events.

### `POST /api/traceability`
Persist a traceability event locally.

**Request:**
```json
{
  "part_id": "PART-10027",
  "product_code": "SKU-AXUM-01",
  "lot_code": "LOT-2026-04A",
  "station": "Station-3",
  "status": "Completed",
  "notes": "MacBook demo event captured from the dashboard."
}
```

### `POST /api/read`
Read a tag value from the PLC.

**Request:**
```json
{
  "tag_name": "TestDINT"
}
```

**Response:**
```json
{
  "success": true,
  "tag_name": "TestDINT",
  "value": {
    "type": "DINT",
    "value": 42
  },
  "data_type": "DINT",
  "error": null
}
```

### `POST /api/write`
Write a value to a PLC tag.

**Request:**
```json
{
  "tag_name": "TestDINT",
  "value": {
    "type": "DINT",
    "value": 100
  }
}
```

**Response:**
```json
{
  "success": true,
  "message": "Successfully wrote to tag 'TestDINT'",
  "error": null
}
```

## Supported Data Types

The API supports all Allen-Bradley data types:

- **BOOL**: Boolean (true/false)
- **SINT**: 8-bit signed integer
- **INT**: 16-bit signed integer
- **DINT**: 32-bit signed integer
- **LINT**: 64-bit signed integer
- **USINT**: 8-bit unsigned integer
- **UINT**: 16-bit unsigned integer
- **UDINT**: 32-bit unsigned integer
- **ULINT**: 64-bit unsigned integer
- **REAL**: 32-bit floating point
- **LREAL**: 64-bit floating point
- **STRING**: Variable-length string

## Demo Story

The current dashboard is designed to support a live demo flow like this:

1. Connect from the MacBook to a CompactLogix or routed ControlLogix target.
2. Show the controller model, firmware, and route-path context in the UI.
3. Show live KPIs, production summary, and monitored tags based on the validated `gTest*` controller and `Program:TestProgram.*` tags.
4. Show time-series trends for throughput, quality, cycle, and availability proxies.
5. Run the benchmark panel to compare single operations with batch operations in real time.
6. Save a traceability event to simulate a part-tracking / production-history workflow.

Traceability persistence is currently local JSON storage under `backend/data/traceability.json`. This keeps the demo self-contained. If the example later needs multi-user durability or query-heavy history, SQLite is the next reasonable upgrade.

## Usage Example

1. **Start the backend server:**
   ```bash
   cd examples/web_app/backend
   cargo run
   ```

2. **Start the frontend (in another terminal):**
   ```bash
   cd examples/web_app/frontend
   npm start
   ```

3. **Open your browser** to `http://localhost:3000` (or the port shown)

4. **Connect to your PLC:**
   - Enter the PLC address
   - Enable route path and slot `0` for the validated ControlLogix path if needed
   - Click `Connect`

5. **Show controller identity:**
   - Confirm the dashboard displays the controller model and firmware

6. **Refresh live data:**
   - Review KPI cards and monitored tags from the batch snapshot panel

7. **Run the speed demo:**
   - Click the benchmark button to compare single and batch operations

8. **Capture a traceability event:**
   - Enter part metadata and save a record

## Development

### Backend Development

The backend uses:
- **Axum**: Modern async web framework
- **Tokio**: Async runtime
- **Serde**: JSON serialization/deserialization
- **Tower**: Middleware and utilities

To add new backend endpoints, edit `backend/src/main.rs`.

### Frontend Development

The frontend uses:
- **React 18**: UI library
- **TypeScript**: Type safety
- **Create React App**: Build tooling

To modify the UI, edit the components in `frontend/src/components/`.

## Production Deployment

### Backend

Build the release binary:

```bash
cd examples/web_app/backend
cargo build --release
```

The binary will be at `target/release/plc-web-backend`.

### Frontend

Build the production bundle:

```bash
cd examples/web_app/frontend
npm run build
```

The static files will be in `frontend/build/`. You can serve them with any static file server or integrate with the Rust backend.

### CORS Configuration

For production, update the CORS configuration in `backend/src/main.rs` to only allow your frontend domain:

```rust
.layer(
    CorsLayer::new()
        .allow_origin("https://yourdomain.com".parse::<HeaderValue>().unwrap())
        .allow_methods([Method::GET, Method::POST])
        .allow_headers([CONTENT_TYPE])
)
```

## Troubleshooting

### Connection Issues

- Ensure the PLC is on the same network
- Check firewall settings (port 44818)
- Verify the PLC address format: `IP:PORT` (e.g., `192.168.1.120:44818`)

### Frontend Can't Connect to Backend

- Ensure the backend is running on port 3000
- Check CORS settings if accessing from a different origin
- Verify the `API_BASE` environment variable in the frontend

### Tag Read/Write Errors

- Verify the tag exists in the PLC
- Check the data type matches the tag's actual type
- Ensure the tag is not protected or read-only

## Comparison with C# Examples

This example differs from the C# examples in that:

1. **No C# Wrapper**: Direct use of the Rust library via native Rust code
2. **Web-Based**: Browser-based UI instead of desktop applications
3. **REST API**: HTTP-based communication instead of direct library calls
4. **Cross-Platform**: Works on any platform that supports Rust and Node.js

## License

Same license as the main `rust-ethernet-ip` library.
