# 🦀 Rust EtherNet/IP WinForms Example with Batch Operations (v0.6.1)

A comprehensive Windows Forms application demonstrating the power of **batch operations** in the Rust EtherNet/IP library (v0.6.1). This example showcases how batch operations can provide **3-10x performance improvements** over individual tag operations.

## 🚀 Features

### Core Functionality
- **Individual Operations**: Traditional single-tag read/write operations
- **Batch Operations**: High-performance multi-tag operations
- **Performance Comparison**: Side-by-side benchmarking
- **Batch Configuration**: Optimize settings for your PLC

### Batch Operations Highlights
- **🚀 Batch Read**: Read multiple tags in a single optimized operation
- **✏️ Batch Write**: Write multiple tags atomically 
- **🔄 Mixed Operations**: Combine reads and writes in coordinated batches
- **📊 Performance Testing**: Compare individual vs batch operation speeds
- **⚙️ Configuration**: Tune batch behavior for different PLC types

## 📊 Performance Benefits

| Operation Type | Individual | Batch | Improvement |
|---------------|------------|-------|-------------|
| 5 Tag Reads | ~15ms | ~3ms | **5x faster** |
| 10 Tag Writes | ~30ms | ~5ms | **6x faster** |
| 20 Mixed Ops | ~50ms | ~8ms | **6.25x faster** |
| Network Packets | N packets | 1-3 packets | **5-20x reduction** |

## 🎯 Use Cases Demonstrated

### 1. **Data Acquisition**
```
TestTag
TestBool
TestInt
TestReal
```
Read multiple sensor values simultaneously for real-time monitoring.

✅ **v0.4.0 Update**: STRING tags are now fully supported with complete Allen-Bradley format compliance!

### 2. **Recipe Management**
```
TestTag=true
TestBool=false
TestInt=999
TestReal=88.8
```
Update multiple setpoints atomically for consistent process control.

### 3. **Coordinated Control**
```
READ:TestTag
READ:TestBool
WRITE:TestInt=777
WRITE:TestReal=99.9
```
Read current values and write new setpoints in a single coordinated operation.

## 🏗️ Application Structure

### Tab-Based Interface

#### 1. **Individual Operations**
- Traditional single-tag operations
- Tag discovery and type detection
- Manual read/write operations
- Baseline for performance comparison

#### 2. **🚀 Batch Operations**
- **Batch Read**: Multi-tag reading with performance metrics
- **Batch Write**: Atomic multi-tag writing
- **Mixed Operations**: Combined read/write operations

#### 3. **📊 Performance Comparison**
- Configurable test parameters (tag count, operation type)
- Side-by-side timing comparison
- Visual performance charts
- Network efficiency analysis

#### 4. **⚙️ Batch Configuration**
- **Current Config**: View active batch settings
- **Preset Configs**: 
  - 📊 Default (20 ops/packet, 504 bytes)
  - 🚀 High Performance (50 ops/packet, 4000 bytes)
  - 🛡️ Conservative (10 ops/packet, 504 bytes)
- **Custom Config**: Fine-tune all parameters

## 🚀 Getting Started

### Prerequisites
- .NET 9.0 or later
- Windows OS (for WinForms)
- Allen-Bradley CompactLogix PLC (or compatible)

### Building and Running

```bash
# Clone the repository
git clone https://github.com/your-repo/rust-ethernet-ip
cd rust-ethernet-ip/examples/WinFormsExample

# Build the application
dotnet build

# Run the application
dotnet run
```

### Connecting to Your PLC

1. **Enter PLC Address**: Default is `192.168.0.1:44818`
2. **Click Connect**: Establishes EtherNet/IP session
3. **Verify Connection**: Status shows "Connected" with session ID

### Setting Up Test Tags

For optimal testing experience, create these test tags in your PLC:

| Tag Name | Data Type | Description |
|----------|-----------|-------------|
| `TestTag` | BOOL | Boolean test value |
| `TestBool` | BOOL | Another boolean for testing |
| `TestInt` | DINT | 32-bit integer test value |
| `TestReal` | REAL | Floating point test value |

**Automatic Setup**: Use the individual operations tab to write these tags, or they'll be created automatically when the application connects.

## 📖 Usage Examples

### Basic Batch Read
1. Navigate to **🚀 Batch Operations** → **Batch Read**
2. Enter tag names (one per line):
   ```
   TestTag
   TestBool
   TestInt
   TestReal
   ```
3. Click **🚀 Execute Batch Read**
4. View results and performance metrics

### Batch Write Operations
1. Go to **Batch Write** tab
2. Enter tag=value pairs:
   ```
   TestTag=true
   TestBool=false
   TestInt=999
   TestReal=88.8
   ```
3. Click **✏️ Execute Batch Write**
4. Monitor success/failure for each tag

### Performance Testing
1. Open **📊 Performance Comparison** tab
2. Configure test parameters:
   - Number of tags: 5-50
   - Test type: Read Only, Write Only, or Mixed
3. Click **🚀 Run Performance Test**
4. Compare individual vs batch performance

### Optimizing Configuration
1. Visit **⚙️ Batch Configuration** tab
2. Try different presets:
   - **High Performance**: For modern PLCs with high bandwidth
   - **Conservative**: For older PLCs or unreliable networks
3. Or create custom configuration for your specific needs

## ⚙️ Configuration Options

### Batch Configuration Parameters

| Parameter | Description | Default | Range |
|-----------|-------------|---------|-------|
| Max Operations per Packet | Number of operations in one CIP packet | 20 | 1-100 |
| Max Packet Size | Maximum packet size in bytes | 504 | 200-8000 |
| Packet Timeout | Timeout per packet in milliseconds | 3000 | 500-30000 |
| Continue on Error | Process remaining operations if one fails | true | true/false |
| Optimize Packing | Group similar operations for efficiency | true | true/false |

### PLC-Specific Recommendations

#### CompactLogix L3x/L4x/L5x (Modern)
```
Max Operations: 50
Max Packet Size: 4000 bytes
Timeout: 1000ms
```

#### CompactLogix L1x/L2x (Entry-Level)
```
Max Operations: 20
Max Packet Size: 504 bytes
Timeout: 3000ms
```

#### MicroLogix 1100/1400
```
Max Operations: 10
Max Packet Size: 504 bytes
Timeout: 5000ms
```

## 🔧 Troubleshooting

### Common Issues

#### Connection Problems
- **Verify PLC IP address and port**
- **Check network connectivity**
- **Ensure EtherNet/IP is enabled on PLC**
- **Verify firewall settings**

#### Batch Operation Errors
- **Tag not found**: Verify tag names exist in PLC
- **Data type mismatch**: Check value formats
- **Timeout errors**: Increase packet timeout or reduce operations per packet
- **Packet size errors**: Reduce max packet size for older PLCs

#### Performance Issues
- **Lower than expected speedup**: Try high-performance configuration
- **Network errors**: Use conservative configuration
- **Inconsistent results**: Check network stability

### Debug Logging
The application provides detailed logging in the Activity Log panel:
- Connection status and session information
- Batch operation execution details
- Performance metrics and timing
- Error messages with troubleshooting hints

## 🏭 Industrial Applications

### Manufacturing Execution Systems (MES)
- **Production Monitoring**: Batch read production counters, quality metrics
- **Recipe Downloads**: Batch write process parameters
- **Status Collection**: Gather equipment status from multiple machines

### SCADA Systems
- **Data Acquisition**: Efficient collection of sensor data
- **Alarm Management**: Batch read alarm status from multiple zones
- **Setpoint Management**: Coordinated updates to control parameters

### Quality Control
- **Test Data Collection**: Batch read measurement results
- **Calibration Updates**: Batch write calibration parameters
- **Audit Trails**: Coordinated logging of process changes

## 📚 Code Examples

### Batch Read Implementation
```csharp
var tagNames = new[] { "Tag1", "Tag2", "Tag3" };
var results = client.ReadTagsBatch(tagNames);

foreach (var result in results)
{
    if (result.Value.Success)
    {
        Console.WriteLine($"{result.Key}: {result.Value.Value}");
    }
    else
    {
        Console.WriteLine($"{result.Key}: Error - {result.Value.ErrorMessage}");
    }
}
```

### Batch Write Implementation
```csharp
var tagValues = new Dictionary<string, object>
{
    ["Setpoint1"] = 75.5f,
    ["Setpoint2"] = 80.0f,
    ["EnableFlag"] = true
};

var results = client.WriteTagsBatch(tagValues);
```

### Mixed Batch Operations
```csharp
var operations = new[]
{
    BatchOperation.Read("CurrentTemp"),
    BatchOperation.Read("CurrentPressure"),
    BatchOperation.Write("TempSetpoint", 78.5f),
    BatchOperation.Write("PressureSetpoint", 15.2f)
};

var results = client.ExecuteBatch(operations);
```

## 🔗 Related Examples

- **[ASP.NET Core Example](../AspNetExample/)**: Web API with batch operations
- **[WPF Example](../WpfExample/)**: MVVM pattern with batch operations  
- **[TypeScript Example](../TypeScriptExample/)**: React frontend with batch operations

## 📄 License

This example is part of the rust-ethernet-ip project and is licensed under the same terms.

## 🤝 Contributing

Contributions are welcome! Please see the main project repository for contribution guidelines.

---

**🚀 Experience the power of batch operations - 3-10x faster PLC communication!** 