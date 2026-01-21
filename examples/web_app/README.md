# PLC Web Application - Rust Backend + React/TypeScript Frontend

This example demonstrates how to build a web application with a native Rust backend and a React/TypeScript frontend for communicating with Allen-Bradley PLCs via EtherNet/IP.

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

- **Native Rust Backend**: Direct use of the `rust-ethernet-ip` library without C# wrappers
- **Modern Frontend**: React with TypeScript for type-safe UI development
- **RESTful API**: Clean HTTP endpoints for PLC operations
- **Real-time Status**: Connection status monitoring
- **Tag Operations**: Read and write PLC tags with full data type support
- **CORS Support**: Configured for development and production

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
Connect to a PLC.

**Request:**
```json
{
  "address": "192.168.1.120:44818"
}
```

**Response:**
```json
{
  "success": true,
  "message": "Successfully connected to 192.168.1.120:44818"
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

**Response:**
```json
{
  "connected": true,
  "address": "192.168.1.120:44818"
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
   - Enter the PLC address (e.g., `192.168.1.120:44818`)
   - Click "Connect"

5. **Read a tag:**
   - Enter a tag name (e.g., `TestDINT`)
   - Click "Read Tag"

6. **Write a tag:**
   - Enter a tag name
   - Select the data type
   - Enter the value
   - Click "Write Tag"

## Development

### Backend Development

The backend uses:
- **Axum**: Modern async web framework
- **Tokio**: Async runtime
- **Serde**: JSON serialization/deserialization
- **Tower**: Middleware and utilities

To add new endpoints, edit `backend/src/main.rs` and add new route handlers.

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

