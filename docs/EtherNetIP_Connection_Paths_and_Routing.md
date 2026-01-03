# EtherNet/IP Connection Paths and CIP Routing Guide

**For ControlLogix, CompactLogix, and Third-Party EtherNet/IP Devices**

Based on Rockwell Automation Developer Guide and Logix 5000 Controller Messages Manual

---

## Table of Contents

1. [Overview: Why Connection Paths Matter](#overview-why-connection-paths-matter)
2. [ControlLogix vs CompactLogix Architecture](#controllogix-vs-compactlogix-architecture)
3. [CIP Routing Fundamentals](#cip-routing-fundamentals)
4. [Connection Path Format](#connection-path-format)
5. [Port Segment Encoding](#port-segment-encoding)
6. [Practical Examples](#practical-examples)
7. [Forward Open Request Structure](#forward-open-request-structure)
8. [MSG Instruction Path Configuration](#msg-instruction-path-configuration)
9. [Connection Types and Caching](#connection-types-and-caching)
10. [Troubleshooting Connection Issues](#troubleshooting-connection-issues)

---

## Overview: Why Connection Paths Matter

When communicating with Logix controllers over EtherNet/IP, the **connection path** tells the system how to route messages from the Ethernet interface to the target (usually the CPU).

**Key Concept:** In ControlLogix systems, the Ethernet module and the CPU are **separate modules** in different chassis slots. Messages arriving at the Ethernet module must be routed across the **backplane** to reach the CPU.

```
┌─────────────────────────────────────────────────────────────┐
│                   ControlLogix Chassis                       │
├─────────┬─────────┬─────────┬─────────┬─────────┬──────────┤
│ Slot 0  │ Slot 1  │ Slot 2  │ Slot 3  │ Slot 4  │ Slot 5   │
│  CPU    │  I/O    │ EN2T    │  I/O    │  I/O    │  I/O     │
│ L85E    │ Module  │ Enet    │ Module  │ Module  │ Module   │
└─────────┴─────────┴────┬────┴─────────┴─────────┴──────────┘
                         │
                         │ EtherNet/IP Connection arrives here
                         │ Must route to Slot 0 (CPU)
                         ▼
```

---

## ControlLogix vs CompactLogix Architecture

### ControlLogix (1756 Series)

- **Modular architecture** - CPU, Ethernet, and I/O are separate modules
- Ethernet module can be in **any slot** (0-16 depending on chassis size)
- CPU can be in **any slot**
- **Backplane routing required** - must specify CPU slot in connection path
- Multiple Ethernet modules possible in same chassis

```
ControlLogix Chassis Example:
┌────────────────────────────────────────────┐
│ Slot 0: 1756-L85E  (CPU)                   │
│ Slot 1: 1756-IB16  (Digital Input)         │
│ Slot 2: 1756-EN2T  (Ethernet) ← Connect    │
│ Slot 3: 1756-OB16  (Digital Output)        │
└────────────────────────────────────────────┘

Connection Path: Port 1 (backplane), Slot 0 (CPU location)
```

### CompactLogix (1769/5069 Series)

- **Integrated architecture** - CPU has **built-in Ethernet port**
- No separate Ethernet module required
- **No backplane routing needed** (or minimal routing)
- Single Ethernet interface (dual ports for Device Level Ring)

```
CompactLogix 5380 Example:
┌────────────────────────────────────────────┐
│ 5069-L330ER (CPU with built-in Ethernet)   │
│     ├── Ethernet Port 1 ← Connect here     │
│     └── Ethernet Port 2                    │
│ 5069-IB16  (Digital Input)                 │
│ 5069-OB16  (Digital Output)                │
└────────────────────────────────────────────┘

Connection Path: Empty or Port 1, Slot 0
```

### Summary: Path Differences

| Platform | Ethernet Location | Path to CPU |
|----------|------------------|-------------|
| **ControlLogix** | Separate module in slot X | `Port 1, Slot Y` (where Y = CPU slot) |
| **CompactLogix 5370** | Built into CPU | `Port 1, Slot 0` or empty |
| **CompactLogix 5380/5480** | Built into CPU | Empty path or `Port 1, Slot 0` |

---

## CIP Routing Fundamentals

### The Routing Principle

From the Rockwell Developer Guide:

> "Communication modules are not an integral part of any processors (at least not logically), meaning that all communication from the outside world into a processor, and vice versa, must use the routing principles built into CIP. These principles state that processors should message across routed connections or through the Unconnected Send service, which carries at least one port segment."

### Port Segment Requirement

> "To communicate with a Logix system, a routed explicit message (Unconnected Send or Forward Open) must contain a **port segment identifying the backplane port (1) and the slot number of the module to be reached**."

### Ports in Logix Systems

| Port Number | Description |
|-------------|-------------|
| **1** | Backplane |
| **2** | Ethernet port (for EtherNet/IP modules) |
| **3** | Serial port (if present) |

---

## Connection Path Format

### Path Components

A complete connection path may include:

1. **Electronic Key Segment** (optional) - Verify target device identity
2. **Port Segments** - Route through networks/backplanes
3. **Application Path** - Target object (Class/Instance/Attribute or Symbolic)
4. **Data Segment** (optional) - Configuration data

### Port Segment Encoding

| Segment Type | Value | Format |
|--------------|-------|--------|
| Port Segment (8-bit link) | `0x01` | `0x01, port, link_address` |
| Port Segment (16-bit link) | `0x11` | `0x11, port, 0x00, link_low, link_high` |
| Extended Port Segment | `0x1x` | Variable format for larger addresses |

### Backplane Port Segment Examples

**8-bit Slot Address (slots 0-255):**
```
01 XX
│  │
│  └── Slot number (link address)
└───── Port 1 (backplane) + 8-bit link format
```

**Examples:**
| CPU Slot | Port Segment Bytes |
|----------|-------------------|
| Slot 0 | `01 00` |
| Slot 1 | `01 01` |
| Slot 2 | `01 02` |
| Slot 3 | `01 03` |
| Slot 5 | `01 05` |

---

## Practical Examples

### Example 1: ControlLogix - CPU in Slot 0, Ethernet in Slot 2

```
Chassis Layout:
  Slot 0: 1756-L85E (CPU) ← Target
  Slot 1: 1756-IB16
  Slot 2: 1756-EN2T (Ethernet) ← Entry point
  Slot 3: 1756-OB16
```

**Connection Path (hex):** `01 00`
- `01` = Port 1 (backplane)
- `00` = Slot 0 (CPU location)

**In Python:**
```python
route_path = bytes([0x01, 0x00])  # Port 1, Slot 0
```

---

### Example 2: ControlLogix - CPU in Slot 3, Ethernet in Slot 1

```
Chassis Layout:
  Slot 0: Empty
  Slot 1: 1756-EN2T (Ethernet) ← Entry point
  Slot 2: 1756-IB16
  Slot 3: 1756-L83E (CPU) ← Target
```

**Connection Path (hex):** `01 03`
- `01` = Port 1 (backplane)
- `03` = Slot 3 (CPU location)

**In Python:**
```python
route_path = bytes([0x01, 0x03])  # Port 1, Slot 3
```

---

### Example 3: CompactLogix 5380 - Built-in Ethernet

```
Controller: 5069-L330ER with integrated Ethernet
```

**Connection Path:** Empty or `01 00`

**In Python:**
```python
# Option 1: Empty path
route_path = bytes([])

# Option 2: Explicit (for compatibility)
route_path = bytes([0x01, 0x00])  # Port 1, Slot 0
```

---

### Example 4: Multi-Hop Routing (Through Bridge)

```
Network Path:
  Your PC → EtherNet/IP → Remote EN2T (192.168.1.100) → Backplane → CPU (Slot 2)
```

**Connection Path:** Route through Ethernet to backplane to CPU

```python
# Extended path with IP address routing
# 2 = Ethernet port, then IP address, then backplane routing
path = "2,192.168.1.100,1,2"  # RSLogix format

# Or in CIP segment encoding:
# Port 2 (Ethernet) + IP address + Port 1 (backplane) + Slot 2
```

---

## Forward Open Request Structure

### Forward Open Service (0x54)

The Forward Open request establishes a CIP connection. The **connection path** is critical for routing.

### Request Format

```
Forward Open Request:
├── Service Code: 0x54
├── Request Path: Connection Manager (Class 0x06, Instance 1)
├── Priority/Time Tick
├── Timeout Ticks
├── O→T Network Connection ID
├── T→O Network Connection ID
├── Connection Serial Number
├── Originator Vendor ID
├── Originator Serial Number
├── Connection Timeout Multiplier
├── Reserved (3 bytes)
├── O→T RPI (Requested Packet Interval)
├── O→T Network Connection Parameters
├── T→O RPI
├── T→O Network Connection Parameters
├── Transport Type/Trigger
├── Connection Path Size (words)
└── Connection Path ← ROUTE PATH GOES HERE
    ├── Port Segment(s)
    └── Application Path (symbolic or logical)
```

### Connection Path in Forward Open

The path has two parts:
1. **Route Path** - How to get to the target device (port segments)
2. **Application Path** - What to connect to (tag name or assembly instance)

**Example - Read tag "MyTag" from CPU in Slot 0:**
```
Connection Path:
├── Route: 01 00 (Port 1, Slot 0)
└── Application: 91 05 4D 79 54 61 67 00 (Symbolic segment "MyTag")
```

---

## MSG Instruction Path Configuration

### RSLogix 5000 Path Format

In RSLogix 5000 MSG instruction configuration, paths use a comma-separated format:

```
Format: port, address, port, address, ...
```

### Path Examples for MSG Instructions

| Scenario | Path String | Description |
|----------|-------------|-------------|
| Local backplane, slot 0 | `1,0` | Port 1, Slot 0 |
| Local backplane, slot 3 | `1,3` | Port 1, Slot 3 |
| Through Ethernet to remote | `2,192.168.1.100,1,0` | Ethernet → IP → Backplane → Slot 0 |
| CompactLogix to CompactLogix | `2,192.168.1.101` | Just Ethernet + IP (no backplane) |
| ControlLogix to ControlLogix | `2,192.168.1.100,1,3` | Ethernet → IP → Backplane → Slot 3 |

### Path Breakdown

**Example: `2, 192.168.1.100, 1, 3`**

| Element | Meaning |
|---------|---------|
| `2` | Exit via Ethernet port (Port 2) |
| `192.168.1.100` | IP address of remote Ethernet module |
| `1` | Enter backplane (Port 1) on remote chassis |
| `3` | Target slot 3 (where CPU is located) |

---

## Connection Types and Caching

### Connected vs Unconnected Messaging

| Type | Description | Use Case |
|------|-------------|----------|
| **Unconnected** | Single request/response, no session | Infrequent access |
| **Connected** | Persistent session, Forward Open first | Frequent/cyclic access |
| **Connected + Cached** | Connection stays open indefinitely | High-frequency messaging |

### Connection Caching Guidelines

From the Rockwell documentation:

> "For messages that execute at a high frequency to the same device, configure the messages for connected and cached."

**Cache Limits:**
- Logix controllers: Up to **32 cached connections**
- If limit exceeded, controller closes least recently used connection

### Connection Sharing

Multiple MSG instructions to the same device can share a connection:

| Condition | Connections Used |
|-----------|-----------------|
| Different devices | 1 per MSG |
| Same device, enabled simultaneously | 1 per MSG |
| Same device, not simultaneous, cached | **1 shared** |

---

## Unconnected Buffers

### What Are Unconnected Buffers?

> "An allocation of memory that the controller uses to process unconnected communication. The controller performs unconnected communication when it:
> - Establishes a connection with a device, including an I/O module
> - Executes a MSG instruction that does not use a connection"

### Buffer Limits

| Setting | Value |
|---------|-------|
| Default buffers | 10 |
| Maximum buffers | 40 |
| Memory per buffer | 1.2 KB |

### Buffer Guidelines

- Keep unconnected/uncached MSGs **below** buffer count
- Increase buffers if needed via CIP Generic MSG instruction
- Each buffer uses 1.2 KB of memory

---

## Troubleshooting Connection Issues

### Common Connection Errors

| Error | Possible Cause | Solution |
|-------|---------------|----------|
| Path error (0x04) | Invalid route path | Verify slot numbers |
| Path destination unknown (0x05) | Wrong slot or CPU not present | Check chassis layout |
| Connection timeout | Network issue or wrong IP | Verify IP and network |
| Too many connections | Exceeded limit | Enable caching, reduce connections |

### Diagnostic Steps

1. **Verify Physical Layout**
   - Which slot is the CPU in?
   - Which slot is the Ethernet module in?

2. **Verify IP Configuration**
   - Can you ping the Ethernet module?
   - Is the IP address correct?

3. **Check Path Format**
   - ControlLogix: Need port segment to CPU slot
   - CompactLogix: Usually empty or minimal path

4. **Test with RSLinx**
   - Use RSLinx to browse to the controller
   - Note the path RSLinx uses

### Path Debugging Code (Python)

```python
def build_route_path(cpu_slot: int, is_controllogix: bool = True) -> bytes:
    """
    Build route path for Logix controllers.
    
    Args:
        cpu_slot: Slot number where CPU is located
        is_controllogix: True for ControlLogix, False for CompactLogix
    
    Returns:
        Route path bytes
    """
    if is_controllogix:
        # ControlLogix needs backplane routing
        return bytes([0x01, cpu_slot])  # Port 1, Slot X
    else:
        # CompactLogix with built-in Ethernet
        return bytes([])  # Empty path, or bytes([0x01, 0x00])


def build_connection_path(
    cpu_slot: int,
    tag_name: str,
    is_controllogix: bool = True
) -> bytes:
    """
    Build complete connection path including route and symbolic segment.
    """
    # Route path
    route = build_route_path(cpu_slot, is_controllogix)
    
    # Symbolic segment for tag
    tag_bytes = tag_name.encode('ascii')
    tag_len = len(tag_bytes)
    
    # Build symbolic segment: 0x91 + length + name + optional pad
    symbolic = bytes([0x91, tag_len]) + tag_bytes
    if tag_len % 2 == 1:
        symbolic += b'\x00'  # Pad to word boundary
    
    return route + symbolic
```

---

## Quick Reference

### ControlLogix Connection Checklist

- [ ] Identify CPU slot number
- [ ] Identify Ethernet module slot number (not needed for path)
- [ ] Build route path: `01 XX` where XX = CPU slot
- [ ] Add application path (symbolic segment for tag name)
- [ ] Use Forward Open for connected messaging

### CompactLogix Connection Checklist

- [ ] Verify CompactLogix model (5370, 5380, 5480)
- [ ] Use empty route path or `01 00`
- [ ] Add application path (symbolic segment for tag name)
- [ ] Use Forward Open for connected messaging

### Port Reference

| Port | Network/Interface |
|------|------------------|
| 1 | Backplane |
| 2 | Ethernet (EtherNet/IP modules) |
| 3 | Serial/RS-232 |

### Slot Encoding

| Slot | Hex Value |
|------|-----------|
| 0 | 0x00 |
| 1 | 0x01 |
| 2 | 0x02 |
| 3 | 0x03 |
| ... | ... |
| 16 | 0x10 |

---

## References

- Rockwell Automation Developer Guide: "Integration with ControlLogix Programmable Automation Controllers (PACs) Using EtherNet/IP"
- Logix 5000 Controller Messages Programming Manual (1756-PM012)
- Logix 5000 Controllers Data Access Programming Manual (1756-PM020)
- CIP Networks Library, Volume 1 (ODVA)
- CIP Networks Library, Volume 2 - EtherNet/IP Adaptation (ODVA)

---

*Document compiled from Rockwell Automation technical publications*
