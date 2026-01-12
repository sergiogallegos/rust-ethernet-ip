# 🚀 Rust EtherNet/IP ASP.NET Core Example with Batch Operations (v0.6.0)

A comprehensive ASP.NET Core Web API demonstrating the power of **batch operations** and **STRING support** in the Rust EtherNet/IP library (v0.6.0). This example showcases how batch operations can provide **3-10x performance improvements** over individual tag operations through optimized REST API endpoints.

## 🎯 Features

### Core Functionality
- **RESTful API**: Complete HTTP endpoints for PLC communication
- **Individual Operations**: Traditional single-tag read/write endpoints
- **Batch Operations**: High-performance multi-tag API endpoints
- **STRING Support**: Full Allen-Bradley STRING read/write with proper AB format
- **Performance Benchmarking**: Built-in comparison tools
- **Configuration Management**: Flexible batch operation tuning
- **Statistics Tracking**: Real-time performance monitoring

### Batch Operations Highlights
- **🚀 Batch Read API**: `/api/plc/batch/read` - Read multiple tags in optimized packets
- **✏️ Batch Write API**: `/api/plc/batch/write` - Write multiple tags atomically 
- **🔄 Mixed Operations API**: `/api/plc/batch/execute` - Combine reads and writes
- **📊 Performance Testing**: `/api/plc/batch/benchmark` - Compare individual vs batch
- **⚙️ Configuration API**: `/api/plc/batch/config` - Tune for your PLC

### STRING Operations Highlights
- **📝 STRING Read API**: `/api/plc/string/{tagName}` - Read Allen-Bradley STRING tags
- **✏️ STRING Write API**: `/api/plc/string/{tagName}` - Write STRING with validation
- **🚀 Batch STRING Read**: `/api/plc/string/batch/read` - Read multiple STRINGs efficiently
- **📝 Batch STRING Write**: `/api/plc/string/batch/write` - Write multiple STRINGs atomically
- **✅ Full AB Support**: Proper Len, MaxLen, and Data[82] structure handling

## 📊 Performance Benefits

| Endpoint Type | Individual | Batch | Improvement | Network Efficiency |
|--------------|------------|-------|-------------|-------------------|
| 5 Tag Reads | ~15ms | ~3ms | **5x faster** | 5x fewer packets |
| 10 Tag Writes | ~30ms | ~5ms | **6x faster** | 10x fewer packets |
| 20 Mixed Ops | ~50ms | ~8ms | **6.25x faster** | 20x fewer packets |
| API Throughput | 200 ops/sec | 1000+ ops/sec | **5x higher** | Dramatically reduced |

## 🌐 API Endpoints

### Connection Management
- `POST /api/plc/connect` - Connect to PLC
- `POST /api/plc/disconnect` - Disconnect from PLC  
- `GET /api/plc/status` - Get connection status

### Individual Operations
- `GET /api/plc/tag/{tagName}` - Read single tag
- `POST /api/plc/tag/{tagName}` - Write single tag
- `POST /api/plc/benchmark` - Performance test individual operations

### 🚀 Batch Operations (High Performance)
- `POST /api/plc/batch/read` - Read multiple tags
- `POST /api/plc/batch/write` - Write multiple tags  
- `POST /api/plc/batch/execute` - Mixed read/write operations
- `POST /api/plc/batch/benchmark` - Compare performance
- `GET /api/plc/batch/config` - Get batch configuration
- `POST /api/plc/batch/config` - Update batch configuration
- `GET /api/plc/batch/stats` - Get performance statistics
- `DELETE /api/plc/batch/stats` - Reset statistics

### 📝 STRING Operations (Allen-Bradley Format)
- `GET /api/plc/string/{tagName}` - Read single STRING tag
- `POST /api/plc/string/{tagName}` - Write single STRING tag
- `POST /api/plc/string/batch/read` - Read multiple STRING tags
- `POST /api/plc/string/batch/write` - Write multiple STRING tags

## 🚀 Getting Started

### Prerequisites
- .NET 9.0 or later
- Allen-Bradley CompactLogix PLC (or compatible)
- Network connectivity to PLC

### Building and Running

```bash
# Clone the repository
git clone https://github.com/your-repo/rust-ethernet-ip
cd rust-ethernet-ip/examples/AspNetExample

# Build the API
dotnet build

# Run the API (development)
dotnet run

# API will be available at:
# - HTTP: http://localhost:5000
# - HTTPS: https://localhost:5001
# - Swagger UI: http://localhost:5000/swagger
```

### Docker Support (Optional)

```bash
# Build Docker image
docker build -t rust-ethernet-ip-api .

# Run container
docker run -p 5000:8080 rust-ethernet-ip-api
```

## 📖 API Usage Examples

### 1. Connect to PLC

```bash
curl -X POST http://localhost:5000/api/plc/connect \
  -H "Content-Type: application/json" \
  -d '{"address": "192.168.0.1:44818"}'
```

Response:
```json
{
  "success": true,
  "message": "Connected successfully"
}
```

### 2. Batch Read Operations

```bash
curl -X POST http://localhost:5000/api/plc/batch/read \
  -H "Content-Type: application/json" \
  -d '{
    "tagNames": [
      "ProductionCount",
      "Temperature_1", 
      "Temperature_2",
      "Pressure_1",
      "FlowRate"
    ]
  }'
```

Response:
```json
{
  "success": true,
  "results": {
    "ProductionCount": {
      "success": true,
      "value": 1542,
      "dataType": "DINT"
    },
    "Temperature_1": {
      "success": true,
      "value": 78.5,
      "dataType": "REAL"
    }
  },
  "performance": {
    "totalTimeMs": 8,
    "successCount": 5,
    "errorCount": 0,
    "averageTimePerTagMs": 1.6,
    "tagsPerSecond": 625
  },
  "message": "Batch read completed: 5/5 successful in 8ms"
}
```

### 3. Batch Write Operations

```bash
curl -X POST http://localhost:5000/api/plc/batch/write \
  -H "Content-Type: application/json" \
  -d '{
    "tagValues": {
      "SetPoint_1": 75.5,
      "SetPoint_2": 80.0,
      "EnableFlag": true,
      "ProductionMode": 2,
      "RecipeNumber": 42
    }
  }'
```

Response:
```json
{
  "success": true,
  "results": {
    "SetPoint_1": {
      "success": true
    },
    "SetPoint_2": {
      "success": true
    }
  },
  "performance": {
    "totalTimeMs": 5,
    "successCount": 5,
    "errorCount": 0,
    "averageTimePerTagMs": 1.0,
    "tagsPerSecond": 1000
  },
  "message": "Batch write completed: 5/5 successful in 5ms"
}
```

### 4. Mixed Batch Operations

```bash
curl -X POST http://localhost:5000/api/plc/batch/execute \
  -H "Content-Type: application/json" \
  -d '{
    "operations": [
      {
        "isWrite": false,
        "tagName": "CurrentTemp"
      },
      {
        "isWrite": false,
        "tagName": "CurrentPressure"
      },
      {
        "isWrite": true,
        "tagName": "TempSetpoint",
        "value": 78.5
      },
      {
        "isWrite": true,
        "tagName": "PressureSetpoint", 
        "value": 15.2
      }
    ]
  }'
```

### 5. STRING Operations

#### Read STRING Tag
```bash
curl http://localhost:5000/api/plc/string/StatusMessage
```

Response:
```json
{
  "success": true,
  "value": "Production Running",
  "type": "STRING",
  "length": 17,
  "maxLength": 82,
  "message": "Successfully read STRING tag 'StatusMessage'"
}
```

#### Write STRING Tag
```bash
curl -X POST http://localhost:5000/api/plc/string/StatusMessage \
  -H "Content-Type: application/json" \
  -d '{"value": "Maintenance Mode"}'
```

Response:
```json
{
  "success": true,
  "message": "Successfully wrote STRING 'Maintenance Mode' to tag 'StatusMessage'",
  "value": "Maintenance Mode",
  "length": 16,
  "maxLength": 82
}
```

#### Batch STRING Read
```bash
curl -X POST http://localhost:5000/api/plc/string/batch/read \
  -H "Content-Type: application/json" \
  -d '{
    "tagNames": [
      "StatusMessage",
      "ProductCode", 
      "OperatorName",
      "RecipeName"
    ]
  }'
```

Response:
```json
{
  "success": true,
  "results": [
    {
      "tagName": "StatusMessage",
      "success": true,
      "value": "Production Running",
      "length": 17,
      "type": "STRING"
    },
    {
      "tagName": "ProductCode",
      "success": true,
      "value": "ABC-123-XYZ",
      "length": 11,
      "type": "STRING"
    }
  ],
  "performance": {
    "totalTimeMs": 6,
    "successCount": 4,
    "errorCount": 0,
    "averageTimePerTagMs": 1.5,
    "tagsPerSecond": 666.7
  },
  "message": "Batch STRING read completed: 4/4 successful in 6ms"
}
```

#### Batch STRING Write
```bash
curl -X POST http://localhost:5000/api/plc/string/batch/write \
  -H "Content-Type: application/json" \
  -d '{
    "tagValues": {
      "StatusMessage": "Quality Check",
      "ProductCode": "DEF-456-ABC",
      "OperatorName": "John Smith",
      "RecipeName": "Recipe_v2.1"
    }
  }'
```

### 6. Performance Benchmark

```bash
curl -X POST http://localhost:5000/api/plc/batch/benchmark \
  -H "Content-Type: application/json" \
  -d '{
    "tagCount": 10,
    "testType": "Mixed",
    "compareWithIndividual": true
  }'
```

Response:
```json
{
  "success": true,
  "benchmark": {
    "success": true,
    "testType": "Mixed",
    "tagCount": 10,
    "individualTotalTimeMs": 45,
    "individualAverageTimeMs": 4.5,
    "batchTotalTimeMs": 8,
    "batchAverageTimeMs": 0.8,
    "speedupFactor": 5.625,
    "timeSavedMs": 37,
    "timeSavedPercentage": 82.2,
    "networkEfficiencyFactor": 10
  },
  "message": "Benchmark completed: 5.6x speedup with batch operations"
}
```

### 7. Configuration Management

```bash
# Get current configuration
curl http://localhost:5000/api/plc/batch/config

# Apply high-performance configuration
curl -X POST http://localhost:5000/api/plc/batch/config \
  -H "Content-Type: application/json" \
  -d '{
    "maxOperationsPerPacket": 50,
    "maxPacketSize": 4000,
    "packetTimeoutMs": 1000,
    "continueOnError": true,
    "optimizePacketPacking": true
  }'
```

## ⚙️ Configuration Options

### Batch Configuration Parameters

| Parameter | Description | Default | Recommended Range |
|-----------|-------------|---------|-------------------|
| `maxOperationsPerPacket` | Operations per CIP packet | 20 | 1-100 |
| `maxPacketSize` | Maximum packet size (bytes) | 504 | 200-8000 |
| `packetTimeoutMs` | Timeout per packet | 3000 | 500-30000 |
| `continueOnError` | Process remaining on failure | true | true/false |
| `optimizePacketPacking` | Group similar operations | true | true/false |

### PLC-Specific Presets

#### High Performance (Modern PLCs)
```json
{
  "maxOperationsPerPacket": 50,
  "maxPacketSize": 4000,
  "packetTimeoutMs": 1000,
  "continueOnError": true,
  "optimizePacketPacking": true
}
```

#### Conservative (Older PLCs)
```json
{
  "maxOperationsPerPacket": 10,
  "maxPacketSize": 504,
  "packetTimeoutMs": 5000,
  "continueOnError": false,
  "optimizePacketPacking": false
}
```

## 🏭 Industrial Use Cases

### 1. Manufacturing Execution Systems (MES)

```bash
# Production data collection (every second)
curl -X POST http://localhost:5000/api/plc/batch/read \
  -d '{
    "tagNames": [
      "ProductionCount", "QualityGrade", "CycleTime",
      "Energy_Consumption", "Temperature_Avg", "Pressure_Avg"
    ]
  }'

# Recipe download (atomic operation)
curl -X POST http://localhost:5000/api/plc/batch/write \
  -d '{
    "tagValues": {
      "Recipe_ID": 101,
      "Mix_Time": 45,
      "Temperature_SP": 180,
      "Pressure_SP": 25,
      "Speed_SP": 1200
    }
  }'
```

### 2. SCADA Data Acquisition

```bash
# Multi-zone monitoring
curl -X POST http://localhost:5000/api/plc/batch/read \
  -d '{
    "tagNames": [
      "Zone1_Temp", "Zone1_Humidity", "Zone1_Alarm",
      "Zone2_Temp", "Zone2_Humidity", "Zone2_Alarm",
      "Zone3_Temp", "Zone3_Humidity", "Zone3_Alarm"
    ]
  }'
```

### 3. Quality Control Systems

```bash
# Test results collection
curl -X POST http://localhost:5000/api/plc/batch/read \
  -d '{
    "tagNames": [
      "Test1_Result", "Test1_Value", "Test1_Timestamp",
      "Test2_Result", "Test2_Value", "Test2_Timestamp",
      "Overall_Status", "Part_Number", "Serial_Number"
    ]
  }'
```

## 🔧 Error Handling

### Common HTTP Status Codes

- **200 OK**: Operation successful
- **400 Bad Request**: Invalid request format or parameters
- **503 Service Unavailable**: Not connected to PLC
- **500 Internal Server Error**: PLC communication error

### Error Response Format

```json
{
  "success": false,
  "message": "Not connected to PLC",
  "details": {
    "errorCode": "PLC_NOT_CONNECTED",
    "timestamp": "2024-01-15T10:30:00Z"
  }
}
```

## 📈 Monitoring and Statistics

### Performance Statistics API

```bash
# Get comprehensive statistics
curl http://localhost:5000/api/plc/batch/stats
```

Response:
```json
{
  "success": true,
  "stats": {
    "Read": {
      "operationType": "Read",
      "totalOperations": 150,
      "totalTimeMs": 1200,
      "successfulOperations": 147,
      "executionCount": 30,
      "averageTimePerOperation": 8.0,
      "successRate": 98.0,
      "lastExecuted": "2024-01-15T10:29:45Z"
    }
  },
  "summary": {
    "totalOperationTypes": 3,
    "totalOperations": 450,
    "totalTimeMs": 3600,
    "overallSuccessRate": 97.8
  }
}
```

## 🐳 Docker Deployment

### Dockerfile
```dockerfile
FROM mcr.microsoft.com/dotnet/aspnet:9.0
WORKDIR /app
COPY bin/Release/net9.0/publish/ .
EXPOSE 8080
ENTRYPOINT ["dotnet", "AspNetExample.dll"]
```

### Docker Compose
```yaml
version: '3.8'
services:
  rust-ethernet-ip-api:
    build: .
    ports:
      - "5000:8080"
    environment:
      - ASPNETCORE_ENVIRONMENT=Production
      - ASPNETCORE_URLS=http://+:8080
    restart: unless-stopped
```

## 🔗 Integration Examples

### JavaScript/TypeScript Frontend
```typescript
// Batch read operation
const batchRead = async (tagNames: string[]) => {
  const response = await fetch('/api/plc/batch/read', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ tagNames })
  });
  return await response.json();
};

// Batch write operation  
const batchWrite = async (tagValues: Record<string, any>) => {
  const response = await fetch('/api/plc/batch/write', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ tagValues })
  });
  return await response.json();
};
```

### Python Client
```python
import requests

# Batch operations client
class RustEtherNetIPClient:
    def __init__(self, base_url="http://localhost:5000"):
        self.base_url = base_url
    
    def batch_read(self, tag_names):
        response = requests.post(
            f"{self.base_url}/api/plc/batch/read",
            json={"tagNames": tag_names}
        )
        return response.json()
    
    def batch_write(self, tag_values):
        response = requests.post(
            f"{self.base_url}/api/plc/batch/write", 
            json={"tagValues": tag_values}
        )
        return response.json()
```

## 🧪 Testing

### Unit Tests
```bash
dotnet test
```

### Load Testing with Apache Bench
```bash
# Test batch read performance
ab -n 1000 -c 10 -p batch_read.json -T application/json \
  http://localhost:5000/api/plc/batch/read

# Test individual read performance (comparison)
ab -n 1000 -c 10 http://localhost:5000/api/plc/tag/TestTag
```

## 🔗 Related Examples

- **[WinForms Example](../WinFormsExample/)**: Desktop application with batch operations
- **[WPF Example](../WpfExample/)**: MVVM pattern with batch operations  
- **[TypeScript Example](../TypeScriptExample/)**: React frontend consuming this API

## 📄 License

This example is part of the rust-ethernet-ip project and is licensed under the same terms.

## 🤝 Contributing

Contributions are welcome! Please see the main project repository for contribution guidelines.

---

**🚀 Experience the power of batch operations - 3-10x faster PLC communication through optimized REST APIs!** 