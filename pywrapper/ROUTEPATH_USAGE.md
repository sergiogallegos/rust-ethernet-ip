# RoutePath Usage Guide for Python Wrapper

## Overview

The Python wrapper now supports ControlLogix routing through the `RoutePath` class. This allows you to connect to ControlLogix PLCs where the CPU is in a specific slot.

## Basic Usage

### Connecting to ControlLogix

```python
import asyncio
from rust_ethernet_ip import EipClient, RoutePath

async def main():
    # ControlLogix: CPU in Slot 0
    route = RoutePath().add_slot(0)
    client = await EipClient.connect_with_route("192.168.0.1:44818", route)
    
    # Read tags
    value = await client.read_tag("gTestArray_DINT[5]")
    print(f"Value: {value}")

asyncio.run(main())
```

### Different CPU Slots

```python
# CPU in Slot 0
route0 = RoutePath().add_slot(0)
client0 = await EipClient.connect_with_route("192.168.0.1:44818", route0)

# CPU in Slot 3
route3 = RoutePath().add_slot(3)
client3 = await EipClient.connect_with_route("192.168.0.2:44818", route3)

# CPU in Slot 5
route5 = RoutePath().add_slot(5)
client5 = await EipClient.connect_with_route("192.168.0.3:44818", route5)
```

### Setting Route Path After Connection

```python
client = await EipClient.connect("192.168.0.1:44818")

# Set route path later
route = RoutePath().add_slot(0)
await client.set_route_path(route)
```

### Network Routing (Multi-Hop)

```python
# Route through Ethernet to remote PLC
route = RoutePath()\
    .add_port(2)\                    # Port 2 = Ethernet
    .add_address("192.168.1.100")\   # Remote Ethernet module IP
    .add_slot(0)                      # CPU slot on remote PLC

client = await EipClient.connect_with_route("192.168.0.1:44818", route)
```

## UdtData Usage

### Reading UDTs with New Format

```python
# Read UDT using new generic format
udt_data = await client.read_udt_data("gTestUDT")
print(f"Symbol ID: {udt_data.symbol_id}")
print(f"Data Length: {len(udt_data.data)} bytes")

# Access raw bytes
for i, b in enumerate(udt_data.data):
    print(f"Byte[{i}]: 0x{b:02X}")
```

### Writing UDTs with New Format

```python
# Read existing UDT to get symbol_id
existing_udt = await client.read_udt_data("gTestUDT")

# Modify raw bytes (example)
new_data = bytearray(existing_udt.data)
new_data[0] = 0xFF  # Modify first byte

# Create new UdtData
udt_data = UdtData(
    symbol_id=existing_udt.symbol_id,  # Keep same symbol_id
    data=bytes(new_data)
)

# Write back
await client.write_udt_data("gTestUDT", udt_data)
```

## Examples

### Complete Example: ControlLogix with Route Path

```python
import asyncio
from rust_ethernet_ip import EipClient, RoutePath, UdtData

async def main():
    # Create route path for ControlLogix (CPU in Slot 0)
    route = RoutePath().add_slot(0)
    
    # Connect with route path
    client = await EipClient.connect_with_route("192.168.0.1:44818", route)
    print("✅ Connected to ControlLogix PLC")

    # Read array element
    value = await client.read_tag("gTestArray_DINT[5]")
    print(f"Array[5] = {value}")

    # Read UDT
    udt_data = await client.read_udt_data("gTestUDT")
    print(f"UDT Symbol ID: {udt_data.symbol_id}, Data Length: {len(udt_data.data)} bytes")

asyncio.run(main())
```

### Example: Multiple PLCs with Different Slots

```python
async def connect_to_multiple_plcs():
    # PLC 1: CPU in Slot 0
    route1 = RoutePath().add_slot(0)
    client1 = await EipClient.connect_with_route("192.168.1.10:44818", route1)

    # PLC 2: CPU in Slot 2
    route2 = RoutePath().add_slot(2)
    client2 = await EipClient.connect_with_route("192.168.1.11:44818", route2)

    # PLC 3: CPU in Slot 5
    route3 = RoutePath().add_slot(5)
    client3 = await EipClient.connect_with_route("192.168.1.12:44818", route3)

    # Read from different PLCs
    val1 = await client1.read_tag("MyTag")
    val2 = await client2.read_tag("MyTag")
    val3 = await client3.read_tag("MyTag")

    print(f"PLC1: {val1}, PLC2: {val2}, PLC3: {val3}")
```

## Migration Guide

### From CompactLogix to ControlLogix

**Before (CompactLogix):**
```python
client = await EipClient.connect("192.168.0.1:44818")
```

**After (ControlLogix):**
```python
route = RoutePath().add_slot(0)  # CPU slot
client = await EipClient.connect_with_route("192.168.0.1:44818", route)
```

### From Legacy UDT to UdtData

**Before (Legacy):**
```python
udt_value = await client.read_tag("MyUDT")
# Access as dictionary-like structure
```

**After (New Format):**
```python
udt_data = await client.read_udt_data("MyUDT")
# Access symbol_id and raw bytes
symbol_id = udt_data.symbol_id
raw_bytes = udt_data.data
```

## References

- `docs/CONTROLLOGIX_ROUTING_IMPLEMENTATION.md` - ControlLogix routing details
- `docs/UDT_IMPLEMENTATION_REVIEW.md` - UDT implementation details
- `pywrapper/src/lib.rs` - PyO3 bindings implementation
- `pywrapper/python/rust_ethernet_ip/client.py` - Python wrapper implementation

