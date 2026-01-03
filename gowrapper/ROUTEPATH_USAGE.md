# RoutePath Usage Guide for Go Wrapper

## Overview

The Go wrapper now supports ControlLogix routing through the `RoutePath` type. This allows you to connect to ControlLogix PLCs where the CPU is in a specific slot.

## Basic Usage

### Connecting to ControlLogix

```go
package main

import (
    "fmt"
    "log"
    "github.com/your-repo/rust-ethernet-ip/gowrapper/ethernetip"
)

func main() {
    // ControlLogix: CPU in Slot 0
    route := ethernetip.NewRoutePath().AddSlot(0)
    client, err := ethernetip.NewClientWithRoute("192.168.0.1:44818", route)
    if err != nil {
        log.Fatal(err)
    }
    defer client.Close()

    // Read tags
    value, err := client.ReadDint("gTestArray_DINT[5]")
    if err != nil {
        log.Fatal(err)
    }
    fmt.Printf("Value: %d\n", value)
}
```

### Different CPU Slots

```go
// CPU in Slot 0
route0 := ethernetip.NewRoutePath().AddSlot(0)
client0, _ := ethernetip.NewClientWithRoute("192.168.0.1:44818", route0)

// CPU in Slot 3
route3 := ethernetip.NewRoutePath().AddSlot(3)
client3, _ := ethernetip.NewClientWithRoute("192.168.0.2:44818", route3)

// CPU in Slot 5
route5 := ethernetip.NewRoutePath().AddSlot(5)
client5, _ := ethernetip.NewClientWithRoute("192.168.0.3:44818", route5)
```

### Setting Route Path After Connection

```go
client, err := ethernetip.NewClient("192.168.0.1:44818")
if err != nil {
    log.Fatal(err)
}

// Set route path later
route := ethernetip.NewRoutePath().AddSlot(0)
err = client.SetRoutePath(route)
if err != nil {
    log.Fatal(err)
}
```

### Network Routing (Multi-Hop)

```go
// Route through Ethernet to remote PLC
route := ethernetip.NewRoutePath().
    AddPort(2).                    // Port 2 = Ethernet
    AddAddress("192.168.1.100").   // Remote Ethernet module IP
    AddSlot(0)                      // CPU slot on remote PLC

client, err := ethernetip.NewClientWithRoute("192.168.0.1:44818", route)
```

## UdtData Usage

### Reading UDTs with New Format

```go
// Read UDT using new generic format
udtData, err := client.ReadUdtData("gTestUDT")
if err != nil {
    log.Fatal(err)
}

fmt.Printf("Symbol ID: %d\n", udtData.SymbolID)
fmt.Printf("Data Length: %d bytes\n", len(udtData.Data))

// Access raw bytes
for i, b := range udtData.Data {
    fmt.Printf("Byte[%d]: 0x%02X\n", i, b)
}
```

### Writing UDTs with New Format

```go
// Read existing UDT to get symbol_id
existingUdt, err := client.ReadUdtData("gTestUDT")
if err != nil {
    log.Fatal(err)
}

// Modify raw bytes (example)
newData := make([]byte, len(existingUdt.Data))
copy(newData, existingUdt.Data)
newData[0] = 0xFF  // Modify first byte

// Create new UdtData
udtData := &ethernetip.UdtData{
    SymbolID: existingUdt.SymbolID,  // Keep same symbol_id
    Data:     newData,
}

// Write back
err = client.WriteUdtData("gTestUDT", udtData)
if err != nil {
    log.Fatal(err)
}
```

### Legacy UDT Format (Still Supported)

```go
// Read UDT using legacy format
udtValue, err := client.ReadUdt("gTestUDT")
if err != nil {
    log.Fatal(err)
}

// Access members
if member, ok := udtValue.Members["MemberName"]; ok {
    fmt.Printf("Member: %v\n", member)
}
```

## Examples

### Complete Example: ControlLogix with Route Path

```go
package main

import (
    "fmt"
    "log"
    "time"
    "github.com/your-repo/rust-ethernet-ip/gowrapper/ethernetip"
)

func main() {
    // Create route path for ControlLogix (CPU in Slot 0)
    route := ethernetip.NewRoutePath().AddSlot(0)
    
    // Connect with route path
    client, err := ethernetip.NewClientWithRoute("192.168.0.1:44818", route)
    if err != nil {
        log.Fatalf("Failed to connect: %v", err)
    }
    defer client.Close()

    fmt.Println("✅ Connected to ControlLogix PLC")

    // Read array element
    value, err := client.ReadDint("gTestArray_DINT[5]")
    if err != nil {
        log.Fatalf("Failed to read: %v", err)
    }
    fmt.Printf("Array[5] = %d\n", value)

    // Read UDT
    udtData, err := client.ReadUdtData("gTestUDT")
    if err != nil {
        log.Fatalf("Failed to read UDT: %v", err)
    }
    fmt.Printf("UDT Symbol ID: %d, Data Length: %d bytes\n", 
        udtData.SymbolID, len(udtData.Data))
}
```

### Example: Multiple PLCs with Different Slots

```go
func connectToMultiplePLCs() {
    // PLC 1: CPU in Slot 0
    route1 := ethernetip.NewRoutePath().AddSlot(0)
    client1, _ := ethernetip.NewClientWithRoute("192.168.1.10:44818", route1)
    defer client1.Close()

    // PLC 2: CPU in Slot 2
    route2 := ethernetip.NewRoutePath().AddSlot(2)
    client2, _ := ethernetip.NewClientWithRoute("192.168.1.11:44818", route2)
    defer client2.Close()

    // PLC 3: CPU in Slot 5
    route3 := ethernetip.NewRoutePath().AddSlot(5)
    client3, _ := ethernetip.NewClientWithRoute("192.168.1.12:44818", route3)
    defer client3.Close()

    // Read from different PLCs
    val1, _ := client1.ReadDint("MyTag")
    val2, _ := client2.ReadDint("MyTag")
    val3, _ := client3.ReadDint("MyTag")

    fmt.Printf("PLC1: %d, PLC2: %d, PLC3: %d\n", val1, val2, val3)
}
```

## Migration Guide

### From CompactLogix to ControlLogix

**Before (CompactLogix):**
```go
client, err := ethernetip.NewClient("192.168.0.1:44818")
```

**After (ControlLogix):**
```go
route := ethernetip.NewRoutePath().AddSlot(0)  // CPU slot
client, err := ethernetip.NewClientWithRoute("192.168.0.1:44818", route)
```

### From Legacy UDT to UdtData

**Before (Legacy):**
```go
udtValue, err := client.ReadUdt("MyUDT")
members := udtValue.Members
```

**After (New Format):**
```go
udtData, err := client.ReadUdtData("MyUDT")
// Access symbol_id and raw bytes
symbolID := udtData.SymbolID
rawBytes := udtData.Data
```

## References

- `docs/CONTROLLOGIX_ROUTING_IMPLEMENTATION.md` - ControlLogix routing details
- `docs/UDT_IMPLEMENTATION_REVIEW.md` - UDT implementation details
- `gowrapper/ethernetip/ethernet_ip.go` - Go wrapper implementation

