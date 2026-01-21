# Quick Start Guide

Get the Rust backend + React frontend example running in 5 minutes!

## Prerequisites Check

- ✅ Rust installed (`rustc --version`)
- ✅ Node.js 16+ installed (`node --version`)
- ✅ PLC accessible on your network

## Step 1: Start the Backend

```bash
cd examples/web_app/backend
cargo run
```

You should see:
```
🚀 PLC Web Backend server running on http://0.0.0.0:3000
```

## Step 2: Start the Frontend (New Terminal)

```bash
cd examples/web_app/frontend
npm install  # First time only
npm start
```

The browser should open automatically to `http://localhost:3000` (or another port).

## Step 3: Connect to Your PLC

1. Enter your PLC address (e.g., `192.168.1.120:44818`)
2. Click "Connect"
3. Once connected, you can read/write tags!

## Example Tag Operations

### Read a Tag
- Tag Name: `TestDINT`
- Click "Read Tag"

### Write a Tag
- Tag Name: `TestDINT`
- Data Type: `DINT`
- Value: `42`
- Click "Write Tag"

## Troubleshooting

**Backend won't start?**
- Check if port 3000 is already in use
- Ensure Rust is properly installed

**Frontend can't connect?**
- Verify backend is running on port 3000
- Check browser console for errors

**Can't connect to PLC?**
- Verify PLC is on the same network
- Check firewall settings (port 44818)
- Ensure PLC address format is correct: `IP:PORT`

## Next Steps

- Read the full [README.md](README.md) for detailed documentation
- Explore the API endpoints
- Customize the UI in `frontend/src/components/`
- Add new endpoints in `backend/src/main.rs`

