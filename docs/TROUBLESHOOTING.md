# Troubleshooting Guide: Common EtherNet/IP Issues

This guide helps you diagnose and resolve common issues when using the Rust EtherNet/IP library with Allen-Bradley PLCs.

## Table of Contents

1. [CIP Error 0x01: Connection Failure](#cip-error-0x01-connection-failure)
2. [CIP Error 0x04: Path Segment Error](#cip-error-0x04-path-segment-error)
3. [CIP Error 0x05: Path Destination Unknown](#cip-error-0x05-path-destination-unknown)
4. [CIP Error 0x16: Object Does Not Exist](#cip-error-0x16-object-does-not-exist)
5. [Connection Issues](#connection-issues)
6. [Tag Reading/Writing Issues](#tag-readingwriting-issues)
7. [Performance Issues](#performance-issues)

---

## CIP Error 0x01: Connection Failure

### What It Means

**CIP Error 0x01: Connection failure** is a generic error that occurs when the PLC cannot process your request. Unlike more specific errors, this one requires investigation to determine the root cause.

### Common Causes

#### 1. Tag Doesn't Exist or Wrong Name

**Symptoms:**
- Error occurs immediately when reading/writing
- Session registration and connection succeed
- Tag path encoding looks correct in debug output

**Solutions:**

```rust
// Step 1: Verify connectivity with a system tag
match client.read_tag("Controller").await {
    Ok(_) => println!("✅ Connectivity is good"),
    Err(e) => println!("❌ Connectivity issue: {}", e),
}

// Step 2: Discover available tags
match client.discover_tags().await {
    Ok(tags) => {
        println!("📋 Found {} tags:", tags.len());
        for tag in tags.iter().take(20) {
            println!("  - {}", tag.name);
        }
        // Check if your tag is in the list
        if tags.iter().any(|t| t.name == "YourTagName") {
            println!("✅ Tag found in tag list");
        } else {
            println!("❌ Tag NOT found - check spelling");
        }
    }
    Err(e) => println!("❌ Tag discovery failed: {}", e),
}

// Step 3: Try different tag name variations
let variations = vec![
    "YourTag",
    "yourTag",
    "YOURTAG",
    "Program:MainProgram.YourTag",
    "Program:Main.YourTag",
];

for tag_name in variations {
    match client.read_tag(tag_name).await {
        Ok(value) => println!("✅ '{}' works: {:?}", tag_name, value),
        Err(e) => println!("❌ '{}' failed: {}", tag_name, e),
    }
}
```

**Checklist:**
- ✅ Tag name is **exactly** correct (case-sensitive)
- ✅ Tag is **downloaded** to the PLC (not just saved in project)
- ✅ Tag exists in RSLogix/Studio 5000 tag list
- ✅ Tag has **External Access** enabled

#### 2. Tag Scope Issue (Program-Scoped vs Controller-Scoped)

**Symptoms:**
- Controller-scoped tags work fine
- Specific tag fails with Error 0x01
- Tag exists in RSLogix/Studio 5000

**Solutions:**

```rust
// If tag is in a program, use program-scoped syntax
// Format: "Program:ProgramName.TagName"

// Try controller-scoped first
match client.read_tag("MyTag").await {
    Ok(value) => println!("✅ Controller-scoped: {:?}", value),
    Err(_) => {
        // Try program-scoped
        match client.read_tag("Program:MainProgram.MyTag").await {
            Ok(value) => println!("✅ Program-scoped: {:?}", value),
            Err(e) => println!("❌ Both failed: {}", e),
        }
    }
}
```

**How to Identify Tag Scope:**
1. Open RSLogix/Studio 5000
2. Check the tag's "Scope" column
3. If it shows a program name, use `"Program:ProgramName.TagName"`
4. If it shows "Controller", use just `"TagName"`

#### 3. ControlLogix Routing Issue (Multiple Slots)

**Symptoms:**
- Works on CompactLogix but fails on ControlLogix
- CPU is in a slot other than 0
- Error occurs even with correct tag names

**Solutions:**

```rust
use rust_ethernet_ip::{EipClient, RoutePath};

// For CPU in slot 0 (default - no route path needed)
let mut client = EipClient::connect("192.168.1.100:44818").await?;

// For CPU in slot 3
let route = RoutePath::new().add_slot(3);
let mut client = EipClient::with_route_path("192.168.1.100:44818", route).await?;

// For CPU in slot 5
let route = RoutePath::new().add_slot(5);
let mut client = EipClient::with_route_path("192.168.1.100:44818", route).await?;
```

**How to Find CPU Slot:**
1. Check the physical slot number in the ControlLogix chassis
2. Check RSLogix/Studio 5000 controller properties
3. Default is usually slot 0, but can be 1-31

#### 4. Tag Access Permissions

**Symptoms:**
- Tag exists and name is correct
- Other tags work fine
- Error 0x01 occurs consistently

**Solutions:**

**Check in RSLogix/Studio 5000:**
1. Right-click the tag → Properties
2. Check "External Access" setting:
   - ✅ **Read/Write** - Full access
   - ✅ **Read Only** - Can read, cannot write
   - ❌ **None** - No external access (will cause Error 0x01)

3. Verify controller is in **RUN mode** (some tags only accessible in RUN)

**Code to Check:**
```rust
// Try reading first (read-only tags can still be read)
match client.read_tag("MyTag").await {
    Ok(value) => {
        println!("✅ Tag is readable: {:?}", value);
        // Now try writing
        match client.write_tag("MyTag", value).await {
            Ok(_) => println!("✅ Tag is writable"),
            Err(e) => println!("⚠️ Tag is read-only: {}", e),
        }
    }
    Err(e) => println!("❌ Cannot read tag: {}", e),
}
```

#### 5. Tag Type or Structure Issue

**Symptoms:**
- Simple tags work, but specific tag fails
- Tag might be an array or UDT

**Solutions:**

```rust
// If it's an array, try reading the base tag first
match client.read_tag("MyArray").await {
    Ok(value) => {
        println!("✅ Base array readable: {:?}", value);
        // Then try specific element
        match client.read_tag("MyArray[0]").await {
            Ok(value) => println!("✅ Element readable: {:?}", value),
            Err(e) => println!("❌ Element access failed: {}", e),
        }
    }
    Err(e) => println!("❌ Base array failed: {}", e),
}

// If it's a UDT, try reading the entire UDT
match client.read_tag("MyUDT").await {
    Ok(value) => println!("✅ UDT readable: {:?}", value),
    Err(e) => println!("❌ UDT failed: {}", e),
}

// Then try UDT members
match client.read_tag("MyUDT.Member1").await {
    Ok(value) => println!("✅ UDT member readable: {:?}", value),
    Err(e) => println!("❌ UDT member failed: {}", e),
}
```

### Debugging Steps

#### Step 1: Verify Basic Connectivity

```rust
// Always test with a known system tag first
match client.read_tag("Controller").await {
    Ok(_) => println!("✅ Basic connectivity works"),
    Err(e) => {
        println!("❌ Basic connectivity failed: {}", e);
        // Fix connectivity issues first before troubleshooting tag-specific errors
        return;
    }
}
```

#### Step 2: Discover Available Tags

```rust
match client.discover_tags().await {
    Ok(tags) => {
        println!("📋 Found {} tags:", tags.len());
        
        // Search for your tag
        let search_name = "YourTagName";
        let matching_tags: Vec<_> = tags.iter()
            .filter(|t| t.name.to_lowercase().contains(&search_name.to_lowercase()))
            .collect();
        
        if matching_tags.is_empty() {
            println!("❌ No tags found matching '{}'", search_name);
            println!("Available tags (first 20):");
            for tag in tags.iter().take(20) {
                println!("  - {}", tag.name);
            }
        } else {
            println!("✅ Found matching tags:");
            for tag in matching_tags {
                println!("  - {} (Type: {:?}, Scope: {:?})", 
                    tag.name, tag.data_type, tag.scope);
            }
        }
    }
    Err(e) => println!("❌ Tag discovery failed: {}", e),
}
```

#### Step 3: Try Tag Name Variations

```rust
fn try_tag_variations(client: &mut EipClient, base_name: &str) {
    let variations = vec![
        base_name.to_string(),
        base_name.to_lowercase(),
        base_name.to_uppercase(),
        format!("Program:MainProgram.{}", base_name),
        format!("Program:Main.{}", base_name),
        format!("Program:{}.{}", base_name, base_name), // If program name matches tag
    ];

    for tag_name in variations {
        print!("Trying '{}'... ", tag_name);
        match client.read_tag(&tag_name).await {
            Ok(value) => println!("✅ Success: {:?}", value),
            Err(e) => {
                // Extract just the error code
                let error_msg = e.to_string();
                if let Some(err_part) = error_msg.split("CIP Error").nth(1) {
                    println!("❌{}", err_part.trim());
                } else {
                    println!("❌ {}", e);
                }
            }
        }
    }
}
```

#### Step 4: Check Tag Metadata

```rust
// Get tag metadata if available
if let Some(metadata) = client.get_tag_metadata("YourTagName") {
    println!("📊 Tag metadata:");
    println!("  Name: {}", metadata.name);
    println!("  Type: {:?}", metadata.data_type);
    println!("  Scope: {:?}", metadata.scope);
    println!("  Array: {:?}", metadata.is_array);
} else {
    println!("⚠️ Tag metadata not available (tag might not exist)");
}
```

### What the Debug Output Tells You

When you see debug output like this:

```
🔧 [DEBUG] TagPath generated 6 bytes (3 words) for 'laki'
🔧 [DEBUG] Path bytes (6 bytes): [91, 04, 6C, 61, 6B, 69]
```

**This means:**
- ✅ Tag path encoding is **correct**: `[0x91, 0x04, 0x6C, 0x61, 0x6B, 0x69]`
  - `0x91` = ANSI Extended Symbol Segment
  - `0x04` = Length 4
  - `0x6C, 0x61, 0x6B, 0x69` = "laki" in ASCII
- ✅ Session registration works
- ✅ Connection is established
- ❌ Read fails with CIP Error 0x01

**This suggests the issue is likely:**
1. Tag doesn't exist with that exact name
2. Tag is in a different scope (program-scoped)
3. Tag has restricted access permissions
4. Routing issue (if ControlLogix with non-zero slot)

---

## CIP Error 0x04: Path Segment Error

### What It Means

**CIP Error 0x04: Path segment error** indicates that the CIP path encoding is incorrect or malformed.

### Common Causes

1. **Incorrect tag path format**
2. **Array element addressing issue**
3. **Program-scoped tag path format**

### Solutions

```rust
// Ensure proper tag path format
// Controller-scoped: "TagName"
// Program-scoped: "Program:ProgramName.TagName"
// Array element: "ArrayName[0]"
// UDT member: "UDTName.MemberName"

// If you get Error 0x04, try:
// 1. Verify tag name spelling
// 2. Check if it's program-scoped
// 3. Try reading base tag first (for arrays)
```

---

## CIP Error 0x05: Path Destination Unknown

### What It Means

**CIP Error 0x05: Path destination unknown** indicates a routing issue - the PLC cannot find the path to the tag.

### Common Causes

1. **ControlLogix routing issue** (CPU in non-zero slot)
2. **Network routing problem**
3. **Remote rack connection issue**

### Solutions

```rust
// For ControlLogix with CPU in slot 3:
let route = RoutePath::new().add_slot(3);
let mut client = EipClient::with_route_path("192.168.1.100:44818", route).await?;
```

---

## CIP Error 0x16: Object Does Not Exist

### What It Means

**CIP Error 0x16: Object does not exist** means the tag doesn't exist in the PLC.

### Solutions

1. Verify tag name spelling (case-sensitive)
2. Check if tag is downloaded to PLC
3. Use `discover_tags()` to find available tags
4. Verify tag scope (controller vs program)

---

## Connection Issues

### Cannot Connect to PLC

**Symptoms:**
- Connection timeout
- Network unreachable
- Session registration fails

**Solutions:**

```rust
// 1. Verify IP address and port
let address = "192.168.1.100:44818"; // Default EtherNet/IP port is 44818

// 2. Check network connectivity
// Ping the PLC first: ping 192.168.1.100

// 3. Try connection with timeout
match tokio::time::timeout(
    Duration::from_secs(5),
    EipClient::connect(address)
).await {
    Ok(Ok(client)) => println!("✅ Connected"),
    Ok(Err(e)) => println!("❌ Connection failed: {}", e),
    Err(_) => println!("❌ Connection timeout"),
}

// 4. Check firewall settings
// Ensure port 44818 is not blocked

// 5. Verify PLC is in RUN mode (some PLCs require RUN mode for connections)
```

### Session Registration Fails

**Symptoms:**
- Connection succeeds but session registration fails
- Error during Register Session

**Solutions:**

1. Check PLC firmware version (older firmware may have issues)
2. Verify no other connections are using all available sessions
3. Try disconnecting and reconnecting
4. Check PLC connection limits

---

## Tag Reading/Writing Issues

### Tag Read Returns Wrong Type

**Solutions:**

```rust
// Use type-specific read methods
let bool_val = client.read_bool("MyTag").await?;
let int_val = client.read_int("MyTag").await?;
let dint_val = client.read_dint("MyTag").await?;
let real_val = client.read_real("MyTag").await?;

// Or use generic read and check type
match client.read_tag("MyTag").await? {
    PlcValue::Bool(v) => println!("BOOL: {}", v),
    PlcValue::Int(v) => println!("INT: {}", v),
    PlcValue::Dint(v) => println!("DINT: {}", v),
    PlcValue::Real(v) => println!("REAL: {}", v),
    _ => println!("Other type"),
}
```

### Tag Write Fails

**Common Causes:**
1. Tag is read-only
2. Tag type mismatch
3. Value out of range
4. Tag doesn't exist

**Solutions:**

```rust
// 1. Verify tag is writable
// Check External Access in RSLogix/Studio 5000

// 2. Use correct type
client.write_bool("MyTag", true).await?;
client.write_int("MyTag", 100).await?;
client.write_dint("MyTag", 1000).await?;
client.write_real("MyTag", 3.14).await?;

// 3. Check for errors
match client.write_tag("MyTag", value).await {
    Ok(_) => println!("✅ Write successful"),
    Err(e) => println!("❌ Write failed: {}", e),
}
```

---

## Performance Issues

### Slow Tag Reads/Writes

**Solutions:**

```rust
// Use batch operations for multiple tags
let tags = vec!["Tag1", "Tag2", "Tag3", "Tag4", "Tag5"];
let results = client.read_tags_batch(&tags).await?;

// Batch operations are much faster than individual reads
// Individual: ~5ms per tag = 25ms for 5 tags
// Batch: ~10ms total for 5 tags (5x faster)
```

### High CPU Usage

**Solutions:**

1. Use batch operations instead of individual operations
2. Implement connection pooling
3. Cache tag metadata
4. Use async/await properly (don't block)

---

## Getting Help

If you're still experiencing issues after following this guide:

1. **Enable debug logging** to see detailed packet information
2. **Check the debug output** for path encoding and packet structure
3. **Verify tag configuration** in RSLogix/Studio 5000:
   - Tag name (exact spelling, case-sensitive)
   - Tag scope (Controller vs Program)
   - External Access permissions
   - Tag is downloaded to PLC
4. **Test with a known working tag** (like "Controller") to verify connectivity
5. **Check PLC model and firmware version** - some features require specific firmware
6. **Verify network configuration** - IP address, port, routing (for ControlLogix)

### Useful Debug Information to Provide

When asking for help, please provide:

1. **Error message** (full error text)
2. **CIP error code** (e.g., 0x01, 0x04, 0x05, 0x16)
3. **Tag name** you're trying to access
4. **PLC model** (e.g., CompactLogix L32E, ControlLogix L75)
5. **Firmware version**
6. **Tag scope** (Controller or Program)
7. **Debug output** (if available)
8. **Code snippet** showing how you're calling the library

### Example Issue Report

```
Error: CIP Error 0x01: Connection failure
Tag: "laki"
PLC: ControlLogix L75, Firmware 30.11
Scope: Controller-scoped
Debug output shows correct path encoding: [91, 04, 6C, 61, 6B, 69]
Session registration: ✅ Success
Connection: ✅ Success
Read operation: ❌ Fails with Error 0x01

Tried:
- Verified tag exists in RSLogix
- Checked External Access (enabled)
- Tried "Program:MainProgram.laki" (also fails)
- Tested with "Controller" tag (works fine)
```

---

## Quick Reference: Error Codes

| Error Code | Meaning | Common Cause | Solution |
|------------|---------|--------------|----------|
| 0x01 | Connection failure | Tag doesn't exist, wrong scope, or permissions | Check tag name, scope, and External Access |
| 0x04 | Path segment error | Incorrect path encoding | Verify tag path format |
| 0x05 | Path destination unknown | Routing issue | Check ControlLogix slot routing |
| 0x16 | Object does not exist | Tag doesn't exist | Verify tag name and download status |

---

## Additional Resources

- [Library Documentation](https://docs.rs/rust-ethernet-ip)
- [Examples](../examples/)
- [GitHub Issues](https://github.com/your-repo/issues) - Search for similar issues
- [Allen-Bradley Documentation](https://literature.rockwellautomation.com/) - 1756-PM020 (EtherNet/IP Communication)

---

**Last Updated:** 2026
**Library Version:** 0.6.0+
