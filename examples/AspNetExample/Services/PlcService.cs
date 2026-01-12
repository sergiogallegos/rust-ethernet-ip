using System.Collections.Concurrent;
using RustEtherNetIp;
using Microsoft.Extensions.Logging;
using System;
using System.Threading.Tasks;
using System.Threading;
using System.Diagnostics;
using System.Linq;

namespace AspNetExample.Services;

public class PlcService : IDisposable
{
    private EtherNetIpClient _plcClient;
    private bool _isConnected;
    private string _currentAddress;
    private readonly ConcurrentDictionary<string, DateTime> _lastReadTimes = new();
    private readonly ILogger<PlcService> _logger;
    private readonly IConfiguration _configuration;
    private const int MAX_RETRIES = 3;
    private const int RETRY_DELAY = 1000;
    private readonly SemaphoreSlim _tagOperationLock = new(1, 1);
    
    // Connection health monitoring
    private Timer? _healthCheckTimer;
    private DateTime _lastHealthCheck = DateTime.UtcNow;
    private bool _isHealthy = true;

    // Batch operation statistics
    private readonly ConcurrentDictionary<string, BatchPerformanceStats> _batchStats = new();

    public PlcService(ILogger<PlcService> logger, IConfiguration configuration)
    {
        _logger = logger;
        _configuration = configuration;
        _plcClient = new EtherNetIpClient();
        _isConnected = false;
        _currentAddress = string.Empty;
    }

    public bool Connect(string address, bool useRoutePath = false, int cpuSlot = 0)
    {
        if (_isConnected && _currentAddress == address)
            return true;

        if (_plcClient != null)
            _plcClient.Dispose();

        _plcClient = new EtherNetIpClient();
        
        if (useRoutePath)
        {
            var routePath = new RoutePath().AddSlot((byte)cpuSlot);
            _logger.LogInformation("Connecting with RoutePath: CPU Slot {CpuSlot}", cpuSlot);
            _isConnected = _plcClient.ConnectWithRoute(address, routePath);
        }
        else
        {
            _isConnected = _plcClient.Connect(address);
        }
        
        _currentAddress = address;
        
        if (_isConnected)
        {
            _logger.LogInformation("Connected to PLC at {Address}. Starting health monitoring.", address);
            StartHealthMonitoring();
        }
        else
        {
            _logger.LogWarning("Failed to connect to PLC at {Address}", address);
        }
        
        return _isConnected;
    }

    public void Disconnect()
    {
        _logger.LogInformation("Disconnecting from PLC. Stopping health monitoring.");
        StopHealthMonitoring();
        
        if (_plcClient != null)
            _plcClient.Dispose();
        _plcClient = new EtherNetIpClient();
        _isConnected = false;
        _currentAddress = string.Empty;
        _lastReadTimes.Clear();
        _batchStats.Clear();
        _isHealthy = true;
    }

    public bool IsConnected => _isConnected;
    public string CurrentAddress => _currentAddress;
    public EtherNetIpClient Client => _plcClient;
    public bool IsHealthy => _isHealthy;
    public DateTime LastHealthCheck => _lastHealthCheck;

    public bool ReadBool(string tagName)
    {
        var value = _plcClient.ReadBool(tagName);
        UpdateLastReadTime(tagName);
        return value;
    }
    
    public sbyte ReadSint(string tagName)
    {
        var value = _plcClient.ReadSint(tagName);
        UpdateLastReadTime(tagName);
        return value;
    }
    
    public short ReadInt(string tagName)
    {
        var value = _plcClient.ReadInt(tagName);
        UpdateLastReadTime(tagName);
        return value;
    }
    
    public int ReadDint(string tagName)
    {
        var value = _plcClient.ReadDint(tagName);
        UpdateLastReadTime(tagName);
        return value;
    }
    
    public long ReadLint(string tagName)
    {
        var value = _plcClient.ReadLint(tagName);
        UpdateLastReadTime(tagName);
        return value;
    }
    
    public byte ReadUsint(string tagName)
    {
        var value = _plcClient.ReadUsint(tagName);
        UpdateLastReadTime(tagName);
        return value;
    }
    
    public ushort ReadUint(string tagName)
    {
        var value = _plcClient.ReadUint(tagName);
        UpdateLastReadTime(tagName);
        return value;
    }
    
    public uint ReadUdint(string tagName)
    {
        var value = _plcClient.ReadUdint(tagName);
        UpdateLastReadTime(tagName);
        return value;
    }
    
    public ulong ReadUlint(string tagName)
    {
        var value = _plcClient.ReadUlint(tagName);
        UpdateLastReadTime(tagName);
        return value;
    }
    
    public float ReadReal(string tagName)
    {
        var value = _plcClient.ReadReal(tagName);
        UpdateLastReadTime(tagName);
        return value;
    }
    
    public double ReadLreal(string tagName)
    {
        var value = _plcClient.ReadLreal(tagName);
        UpdateLastReadTime(tagName);
        return value;
    }
    
    public string ReadString(string tagName)
    {
        var value = _plcClient.ReadString(tagName);
        UpdateLastReadTime(tagName);
        return value;
    }
    
    public Dictionary<string, object> ReadUdt(string tagName)
    {
        try
        {
            // Use chunked reading for UDTs to handle large structures
            var value = _plcClient.ReadUdt(tagName);
            UpdateLastReadTime(tagName);
            
            // Convert PlcValue to Dictionary<string, object>
            if (value.IsUdt)
            {
                var result = value.UdtMembers.ToDictionary(
                    kvp => kvp.Key, 
                    kvp => ConvertPlcValueToObject(kvp.Value)
                );
                
                // Demo-specific: Apply Part_Data template parsing if this is a Part_Data UDT
                if (tagName == "Part_Data" && value.UdtMembers.ContainsKey("_raw_data"))
                {
                    try
                    {
                        // Parse raw data using demo-specific template
                        var rawDataString = value.UdtMembers["_raw_data"].As<string>();
                        if (rawDataString != null && rawDataString.StartsWith("[") && rawDataString.EndsWith("]"))
                        {
                            // Parse hex string like "[04, 00]" to bytes
                            var hexParts = rawDataString.Trim('[', ']').Split(',');
                            var rawBytes = hexParts.Select(h => Convert.ToByte(h.Trim(), 16)).ToArray();
                            
                            // Use demo-specific template to parse Part_Data
                            var parsedMembers = PartDataUdtTemplate.ParsePartData(rawBytes);
                            
                            // Add parsed members to result
                            foreach (var member in parsedMembers)
                            {
                                result[member.Key] = ConvertPlcValueToObject(member.Value);
                            }
                            
                            result["_demo_parsing"] = "Applied Part_Data template for demo purposes";
                        }
                    }
                    catch (Exception ex)
                    {
                        _logger.LogWarning(ex, "Failed to apply Part_Data template parsing: {Message}", ex.Message);
                    }
                }
                
                return result;
            }
            else
            {
                // If it's not a UDT, return as single value
                return new Dictionary<string, object> { { "value", ConvertPlcValueToObject(value) } };
            }
        }
        catch (Exception ex)
        {
            _logger.LogError(ex, "Error reading UDT: {TagName}", tagName);
            throw;
        }
    }
    
    public void WriteBool(string tagName, bool value) => _plcClient.WriteBool(tagName, value);
    public void WriteSint(string tagName, sbyte value) => _plcClient.WriteSint(tagName, value);
    public void WriteInt(string tagName, short value) => _plcClient.WriteInt(tagName, value);
    public void WriteDint(string tagName, int value) => _plcClient.WriteDint(tagName, value);
    public void WriteLint(string tagName, long value) => _plcClient.WriteLint(tagName, value);
    public void WriteUsint(string tagName, byte value) => _plcClient.WriteUsint(tagName, value);
    public void WriteUint(string tagName, ushort value) => _plcClient.WriteUint(tagName, value);
    public void WriteUdint(string tagName, uint value) => _plcClient.WriteUdint(tagName, value);
    public void WriteUlint(string tagName, ulong value) => _plcClient.WriteUlint(tagName, value);
    public void WriteReal(string tagName, float value) => _plcClient.WriteReal(tagName, value);
    public void WriteLreal(string tagName, double value) => _plcClient.WriteLreal(tagName, value);
    public void WriteString(string tagName, string value) => _plcClient.WriteString(tagName, value);
    public void WriteUdt(string tagName, Dictionary<string, object> value) => _plcClient.WriteUdt(tagName, value);

    public PlcValue GetUdtMember(string tagName, string memberPath)
    {
        try
        {
            return _plcClient.GetUdtMember(tagName, memberPath);
        }
        catch (Exception ex)
        {
            _logger.LogError(ex, "Error getting UDT member {TagName}.{MemberPath}", tagName, memberPath);
            throw;
        }
    }

    public void SetUdtMember(string tagName, string memberPath, PlcValue value)
    {
        try
        {
            _plcClient.SetUdtMember(tagName, memberPath, value);
        }
        catch (Exception ex)
        {
            _logger.LogError(ex, "Error setting UDT member {TagName}.{MemberPath}", tagName, memberPath);
            throw;
        }
    }

    private void UpdateLastReadTime(string tagName)
    {
        _lastReadTimes.AddOrUpdate(tagName, DateTime.Now, (_, _) => DateTime.Now);
    }

    public ConcurrentDictionary<string, DateTime> LastReadTimes => _lastReadTimes;

    private object ConvertPlcValueToObject(PlcValue value)
    {
        switch (value.Type)
        {
            case PlcValueType.Bool:
                return value.As<bool>();
            case PlcValueType.Sint:
                return value.As<sbyte>();
            case PlcValueType.Int:
                return value.As<short>();
            case PlcValueType.Dint:
                return value.As<int>();
            case PlcValueType.Real:
                return value.As<float>();
            case PlcValueType.String:
                return value.As<string>();
            case PlcValueType.Udt:
                return value.UdtMembers.ToDictionary(
                    kvp => kvp.Key,
                    kvp => ConvertPlcValueToObject(kvp.Value)
                );
            default:
                return value.ToString();
        }
    }

    public object GetTagMetadata(string plcAddress, string tagName)
    {
        if (!_isConnected || _currentAddress != plcAddress)
        {
            throw new Exception($"Not connected to PLC at {plcAddress}");
        }

        try
        {
            // First try to read the tag to determine its type
            try
            {
                var boolValue = _plcClient.ReadBool(tagName);
                return new { type = "BOOL", value = boolValue };
            }
            catch { }

            try
            {
                var dintValue = _plcClient.ReadDint(tagName);
                return new { type = "DINT", value = dintValue };
            }
            catch { }

            try
            {
                var realValue = _plcClient.ReadReal(tagName);
                return new { type = "REAL", value = realValue };
            }
            catch { }

            try
            {
                var stringValue = _plcClient.ReadString(tagName);
                return new { type = "STRING", value = stringValue };
            }
            catch { }

            throw new Exception($"Could not determine type for tag {tagName}");
        }
        catch (Exception ex)
        {
            _logger.LogError(ex, "Error getting metadata for tag {TagName} from PLC at {PlcAddress}", tagName, plcAddress);
            throw;
        }
    }

    public Dictionary<string, object> ReadUdt(string plcAddress, string tagName)
    {
        if (!_isConnected || _currentAddress != plcAddress)
        {
            throw new Exception($"Not connected to PLC at {plcAddress}");
        }

        try
        {
            // Use chunked reading for UDTs to handle large structures
            var value = _plcClient.ReadUdt(tagName);
            
            // Convert PlcValue to Dictionary<string, object>
            if (value.IsUdt)
            {
                return value.UdtMembers.ToDictionary(
                    kvp => kvp.Key, 
                    kvp => ConvertPlcValueToObject(kvp.Value)
                );
            }
            else
            {
                // If it's not a UDT, return as single value
                return new Dictionary<string, object> { { "value", ConvertPlcValueToObject(value) } };
            }
        }
        catch (Exception ex)
        {
            _logger.LogError(ex, "Error reading UDT tag {Tag} from PLC at {Address}", tagName, plcAddress);
            throw;
        }
    }

    public void WriteUdt(string plcAddress, string tagName, Dictionary<string, object> value)
    {
        if (!_isConnected || _currentAddress != plcAddress)
        {
            throw new Exception($"Not connected to PLC at {plcAddress}");
        }

        try
        {
            _plcClient.WriteUdt(tagName, value);
        }
        catch (Exception ex)
        {
            _logger.LogError(ex, "Error writing UDT tag {Tag} to PLC at {Address}", tagName, plcAddress);
            throw;
        }
    }

    public void Dispose()
    {
        StopHealthMonitoring();
        
        if (_plcClient != null)
        {
            try
            {
                _plcClient.Dispose();
            }
            catch (Exception ex)
            {
                _logger.LogError(ex, "Error disposing PLC client");
            }
        }
        
        _tagOperationLock?.Dispose();
        GC.SuppressFinalize(this);
    }

    public class TagNotFoundException : Exception { public TagNotFoundException(string tag) : base($"Tag not found: {tag}") { } }
    public class TagTypeMismatchException : Exception { public TagTypeMismatchException(string tag, string expected, string actual) : base($"Type mismatch for tag '{tag}': expected {expected}, got {actual}") { } }
    public class TagReadOnlyException : Exception { public TagReadOnlyException(string tag) : base($"Tag is read-only: {tag}") { } }
    public class PlcNotConnectedException : Exception { public PlcNotConnectedException(string address) : base($"Not connected to PLC at {address}") { } }
    public class PlcProtocolException : Exception { public PlcProtocolException(string msg) : base(msg) { } }

    public (bool Success, int? Value, string? Error) TryReadDint(string plcAddress, string tagName)
    {
        if (!_isConnected || _currentAddress != plcAddress)
        {
            var err = $"Not connected to PLC at {plcAddress}";
            _logger.LogError(err);
            return (false, null, err);
        }
        try
        {
            var value = _plcClient.ReadDint(tagName);
            _logger.LogInformation("Read DINT tag {TagName} from PLC at {PlcAddress}: {Value}", tagName, plcAddress, value);
            return (true, value, null);
        }
        catch (TagNotFoundException ex)
        {
            _logger.LogError(ex, "Tag not found: {TagName}", tagName);
            return (false, null, ex.Message);
        }
        catch (TagTypeMismatchException ex)
        {
            _logger.LogError(ex, "Type mismatch for tag: {TagName}", tagName);
            return (false, null, ex.Message);
        }
        catch (Exception ex)
        {
            _logger.LogError(ex, "Error reading DINT tag {TagName} from PLC at {PlcAddress}", tagName, plcAddress);
            return (false, null, ex.Message);
        }
    }

    public (bool Success, bool Value, string? Error) TryReadBool(string plcAddress, string tagName)
    {
        if (!_isConnected || _currentAddress != plcAddress)
        {
            return (false, false, $"Not connected to PLC at {plcAddress}");
        }

        try
        {
            var value = _plcClient.ReadBool(tagName);
            _logger.LogInformation("Read bool tag {TagName} from PLC at {PlcAddress}: {Value}", tagName, plcAddress, value);
            return (true, value, null);
        }
        catch (Exception ex)
        {
            _logger.LogError(ex, "Error reading bool tag {TagName} from PLC at {PlcAddress}", tagName, plcAddress);
            return (false, false, ex.Message);
        }
    }

    public (bool Success, float Value, string? Error) TryReadReal(string plcAddress, string tagName)
    {
        if (!_isConnected || _currentAddress != plcAddress)
        {
            return (false, 0, $"Not connected to PLC at {plcAddress}");
        }

        try
        {
            var value = _plcClient.ReadReal(tagName);
            _logger.LogInformation("Read real tag {TagName} from PLC at {PlcAddress}: {Value}", tagName, plcAddress, value);
            return (true, value, null);
        }
        catch (Exception ex)
        {
            _logger.LogError(ex, "Error reading real tag {TagName} from PLC at {PlcAddress}", tagName, plcAddress);
            return (false, 0, ex.Message);
        }
    }

    public (bool Success, string Value, string? Error) TryReadString(string plcAddress, string tagName)
    {
        if (!_isConnected || _currentAddress != plcAddress)
        {
            return (false, string.Empty, $"Not connected to PLC at {plcAddress}");
        }

        try
        {
            var value = _plcClient.ReadString(tagName);
            _logger.LogInformation("Read string tag {TagName} from PLC at {PlcAddress}: {Value}", tagName, plcAddress, value);
            return (true, value, null);
        }
        catch (Exception ex)
        {
            _logger.LogError(ex, "Error reading string tag {TagName} from PLC at {PlcAddress}", tagName, plcAddress);
            return (false, string.Empty, ex.Message);
        }
    }

    private async Task<T> RetryOperation<T>(Func<Task<T>> operation, string operationName)
    {
        for (int attempt = 0; attempt < MAX_RETRIES; attempt++)
        {
            try
            {
                await _tagOperationLock.WaitAsync();
                try
                {
                    return await operation();
                }
                finally
                {
                    _tagOperationLock.Release();
                }
            }
            catch (Exception ex)
            {
                if (attempt == MAX_RETRIES - 1)
                {
                    _logger.LogError(ex, "{Operation} failed after {MaxRetries} attempts", operationName, MAX_RETRIES);
                    throw;
                }
                _logger.LogWarning(ex, "{Operation} attempt {Attempt} failed", operationName, attempt + 1);
                await Task.Delay(RETRY_DELAY * (int)Math.Pow(2, attempt));
            }
        }
        throw new Exception($"{operationName} failed after {MAX_RETRIES} attempts");
    }

    public async Task<(bool success, string type, string value)> ReadTag(string plcAddress, string tagName)
    {
        if (!_isConnected || _currentAddress != plcAddress)
        {
            throw new PlcNotConnectedException(plcAddress);
        }

        return await RetryOperation(async () =>
        {
            try
            {
                // Try to read as different types
                try
                {
                    var value = _plcClient.ReadBool(tagName);
                    return (true, "BOOL", value.ToString());
                }
                catch { }

                try
                {
                    var value = _plcClient.ReadInt(tagName);
                    return (true, "INT", value.ToString());
                }
                catch { }

                try
                {
                    var value = _plcClient.ReadReal(tagName);
                    return (true, "REAL", value.ToString());
                }
                catch { }

                try
                {
                    var value = _plcClient.ReadString(tagName);
                    return (true, "STRING", value);
                }
                catch { }

                return (false, "UNKNOWN", string.Empty);
            }
            catch (Exception ex)
            {
                _logger.LogError(ex, "Error reading tag {Tag} from PLC at {Address}", tagName, plcAddress);
                throw;
            }
        }, $"Read tag {tagName}");
    }

    public async Task WriteTag(string plcAddress, string tagName, string value, string dataType)
    {
        if (!_isConnected || _currentAddress != plcAddress)
        {
            throw new PlcNotConnectedException(plcAddress);
        }

        await RetryOperation(async () =>
        {
            try
            {
                switch (dataType.ToUpper())
                {
                    case "BOOL":
                        _plcClient.WriteBool(tagName, bool.Parse(value));
                        break;
                    case "INT":
                        _plcClient.WriteInt(tagName, short.Parse(value));
                        break;
                    case "DINT":
                        _plcClient.WriteDint(tagName, int.Parse(value));
                        break;
                    case "REAL":
                        _plcClient.WriteReal(tagName, float.Parse(value));
                        break;
                    case "STRING":
                        _plcClient.WriteString(tagName, value);
                        break;
                    default:
                        throw new TagTypeMismatchException(tagName, dataType, "Unsupported type");
                }
                return true;
            }
            catch (Exception ex)
            {
                _logger.LogError(ex, "Error writing tag {Tag} to PLC at {Address}", tagName, plcAddress);
                throw;
            }
        }, $"Write tag {tagName}");
    }

    private void StartHealthMonitoring()
    {
        _healthCheckTimer?.Dispose();
        _lastHealthCheck = DateTime.UtcNow;
        _isHealthy = true;
        
        // Run health checks every 60 seconds (reduced frequency)
        _healthCheckTimer = new Timer(async _ => await CheckConnectionHealth(), null, 
            TimeSpan.FromSeconds(60), TimeSpan.FromSeconds(60));
        
        _logger.LogInformation("Health monitoring started - checks every 60 seconds");
    }
    
    private void StopHealthMonitoring()
    {
        _healthCheckTimer?.Dispose();
        _healthCheckTimer = null;
        _logger.LogInformation("Health monitoring stopped");
    }
    
    private async Task CheckConnectionHealth()
    {
        try
        {
            _logger.LogDebug("Performing connection health check...");
            
            // Use the simple health check to reduce load
            var isHealthy = _plcClient.CheckHealth();
            
            _lastHealthCheck = DateTime.UtcNow;
            var wasHealthy = _isHealthy;
            _isHealthy = isHealthy;
            
            if (!wasHealthy && isHealthy)
            {
                _logger.LogInformation("Connection health restored");
            }
            else if (wasHealthy && !isHealthy)
            {
                _logger.LogWarning("Connection health degraded - session may have timed out");
                _logger.LogInformation("Automatic reconnection disabled - manual reconnection required");
                // Note: Automatic reconnection can cause conflicts with active sessions
                // Users should manually disconnect and reconnect when needed
            }
            else if (isHealthy)
            {
                _logger.LogDebug("Connection health check passed");
            }
            else
            {
                _logger.LogDebug("Connection health check failed");
            }
        }
        catch (Exception ex)
        {
            _logger.LogError(ex, "Error during connection health check");
            _isHealthy = false;
        }
    }

    // ================================================================================
    // BATCH OPERATIONS - High Performance Multi-Tag Operations
    // ================================================================================

    /// <summary>
    /// Read multiple tags in a single optimized batch operation.
    /// Provides 3-10x performance improvement over individual reads.
    /// </summary>
    public async Task<BatchReadResult> ReadTagsBatch(string[] tagNames)
    {
        if (!_isConnected)
            throw new PlcNotConnectedException(_currentAddress);

        var stopwatch = Stopwatch.StartNew();
        _logger.LogInformation("Starting batch read operation for {TagCount} tags", tagNames.Length);

        try
        {
            var batchResults = _plcClient.ReadTagsBatch(tagNames);
            stopwatch.Stop();

            // Update read times for all tags
            foreach (var tagName in tagNames)
            {
                UpdateLastReadTime(tagName);
            }

            // Convert TagReadResultBatch to TagReadResult
            var results = new Dictionary<string, TagReadResult>();
            foreach (var kvp in batchResults)
            {
                var batchResult = kvp.Value;
                PlcValue? plcValue = null;
                
                if (batchResult.Success && batchResult.Value != null)
                {
                    // Convert object to PlcValue based on DataType
                    plcValue = batchResult.DataType switch
                    {
                        "BOOL" => PlcValue.Bool((bool)batchResult.Value),
                        "SINT" => PlcValue.Sint((sbyte)batchResult.Value),
                        "INT" => PlcValue.Int((short)batchResult.Value),
                        "DINT" => PlcValue.Dint((int)batchResult.Value),
                        "LINT" => PlcValue.Lint((long)batchResult.Value),
                        "USINT" => PlcValue.Usint((byte)batchResult.Value),
                        "UINT" => PlcValue.Uint((ushort)batchResult.Value),
                        "UDINT" => PlcValue.Udint((uint)batchResult.Value),
                        "ULINT" => PlcValue.Ulint((ulong)batchResult.Value),
                        "REAL" => PlcValue.Real((float)batchResult.Value),
                        "LREAL" => PlcValue.Lreal((double)batchResult.Value),
                        "STRING" => PlcValue.String((string)batchResult.Value),
                        _ => null
                    };
                }
                
                var tagResult = new TagReadResult
                {
                    TagName = batchResult.TagName,
                    Success = batchResult.Success,
                    Value = plcValue,
                    Quality = batchResult.Success ? DataQuality.Good : DataQuality.Bad,
                    TimeStamp = DateTime.Now,
                    ErrorMessage = batchResult.ErrorMessage
                };
                results[kvp.Key] = tagResult;
            }

            var successCount = results.Count(r => r.Value.Success);
            var result = new BatchReadResult
            {
                Success = true,
                Results = results,
                TotalTimeMs = stopwatch.ElapsedMilliseconds,
                SuccessCount = successCount,
                ErrorCount = results.Count - successCount,
                AverageTimePerTagMs = (double)stopwatch.ElapsedMilliseconds / tagNames.Length
            };

            // Update statistics
            UpdateBatchStats("Read", tagNames.Length, stopwatch.ElapsedMilliseconds, successCount);

            _logger.LogInformation("Batch read completed: {SuccessCount}/{TotalCount} successful in {TimeMs}ms", 
                successCount, tagNames.Length, stopwatch.ElapsedMilliseconds);

            return result;
        }
        catch (Exception ex)
        {
            stopwatch.Stop();
            _logger.LogError(ex, "Batch read operation failed");
            
            return new BatchReadResult
            {
                Success = false,
                ErrorMessage = ex.Message,
                TotalTimeMs = stopwatch.ElapsedMilliseconds
            };
        }
    }

    /// <summary>
    /// Write multiple tags in a single optimized batch operation.
    /// Provides 3-10x performance improvement over individual writes.
    /// </summary>
    public async Task<BatchWriteResult> WriteTagsBatch(Dictionary<string, object> tagValues)
    {
        if (!_isConnected)
            throw new PlcNotConnectedException(_currentAddress);

        var stopwatch = Stopwatch.StartNew();
        _logger.LogInformation("Starting batch write operation for {TagCount} tags", tagValues.Count);

        try
        {
            var results = _plcClient.WriteTagsBatch(tagValues);
            stopwatch.Stop();

            var successCount = results.Count(r => r.Value.Success);
            var result = new BatchWriteResult
            {
                Success = true,
                Results = results,
                TotalTimeMs = stopwatch.ElapsedMilliseconds,
                SuccessCount = successCount,
                ErrorCount = results.Count - successCount,
                AverageTimePerTagMs = (double)stopwatch.ElapsedMilliseconds / tagValues.Count
            };

            // Update statistics
            UpdateBatchStats("Write", tagValues.Count, stopwatch.ElapsedMilliseconds, successCount);

            _logger.LogInformation("Batch write completed: {SuccessCount}/{TotalCount} successful in {TimeMs}ms", 
                successCount, tagValues.Count, stopwatch.ElapsedMilliseconds);

            return result;
        }
        catch (Exception ex)
        {
            stopwatch.Stop();
            _logger.LogError(ex, "Batch write operation failed");
            
            return new BatchWriteResult
            {
                Success = false,
                ErrorMessage = ex.Message,
                TotalTimeMs = stopwatch.ElapsedMilliseconds
            };
        }
    }

    /// <summary>
    /// Execute a mixed batch of read and write operations in optimized packets.
    /// Ideal for coordinated control operations and data collection.
    /// </summary>
    public async Task<BatchMixedResult> ExecuteBatch(BatchOperation[] operations)
    {
        if (!_isConnected)
            throw new PlcNotConnectedException(_currentAddress);

        var stopwatch = Stopwatch.StartNew();
        _logger.LogInformation("Starting mixed batch operation with {OperationCount} operations", operations.Length);

        try
        {
            var results = _plcClient.ExecuteBatch(operations);
            stopwatch.Stop();

            // Update read times for read operations
            foreach (var op in operations.Where(o => !o.IsWrite))
            {
                UpdateLastReadTime(op.TagName);
            }

            var successCount = results.Count(r => r.Success);
            var result = new BatchMixedResult
            {
                Success = true,
                Results = results.Select((r, index) => new ApiMixedOperationResult
                {
                    TagName = GetTagNameFromResult(r, operations, index),
                    IsWrite = IsWriteOperation(r, operations, index),
                    Success = r.Success,
                    Value = r.Value,
                    ExecutionTimeMs = r.ExecutionTimeMs,
                    ErrorCode = r.ErrorCode,
                    ErrorMessage = r.ErrorMessage
                }).ToArray(),
                TotalTimeMs = stopwatch.ElapsedMilliseconds,
                SuccessCount = successCount,
                ErrorCount = results.Length - successCount,
                AverageTimePerOperationMs = (double)stopwatch.ElapsedMilliseconds / operations.Length
            };

            // Update statistics
            UpdateBatchStats("Mixed", operations.Length, stopwatch.ElapsedMilliseconds, successCount);

            _logger.LogInformation("Mixed batch completed: {SuccessCount}/{TotalCount} successful in {TimeMs}ms", 
                successCount, operations.Length, stopwatch.ElapsedMilliseconds);

            return result;
        }
        catch (Exception ex)
        {
            stopwatch.Stop();
            _logger.LogError(ex, "Mixed batch operation failed");
            
            return new BatchMixedResult
            {
                Success = false,
                ErrorMessage = ex.Message,
                TotalTimeMs = stopwatch.ElapsedMilliseconds
            };
        }
    }

    /// <summary>
    /// Configure batch operation behavior for performance optimization.
    /// </summary>
    public void ConfigureBatchOperations(BatchConfig config)
    {
        if (!_isConnected)
            throw new PlcNotConnectedException(_currentAddress);

        _plcClient.ConfigureBatchOperations(config);
        _logger.LogInformation("Batch configuration updated: {MaxOps} ops/packet, {MaxSize} bytes, {Timeout}ms timeout", 
            config.MaxOperationsPerPacket, config.MaxPacketSize, config.PacketTimeoutMs);
    }

    /// <summary>
    /// Get current batch operation configuration.
    /// </summary>
    public BatchConfig GetBatchConfig()
    {
        if (!_isConnected)
            throw new PlcNotConnectedException(_currentAddress);

        return _plcClient.GetBatchConfig();
    }

    /// <summary>
    /// Get batch operation performance statistics.
    /// </summary>
    public Dictionary<string, BatchPerformanceStats> GetBatchStats()
    {
        return new Dictionary<string, BatchPerformanceStats>(_batchStats);
    }

    /// <summary>
    /// Reset batch operation performance statistics.
    /// </summary>
    public void ResetBatchStats()
    {
        _batchStats.Clear();
        _logger.LogInformation("Batch performance statistics reset");
    }

    // Helper methods for batch operations
    private void UpdateBatchStats(string operationType, int operationCount, long totalTimeMs, int successCount)
    {
        _batchStats.AddOrUpdate(operationType, 
            new BatchPerformanceStats 
            { 
                OperationType = operationType,
                TotalOperations = operationCount,
                TotalTimeMs = totalTimeMs,
                SuccessfulOperations = successCount,
                ExecutionCount = 1,
                LastExecuted = DateTime.UtcNow
            },
            (key, existing) => new BatchPerformanceStats
            {
                OperationType = operationType,
                TotalOperations = existing.TotalOperations + operationCount,
                TotalTimeMs = existing.TotalTimeMs + totalTimeMs,
                SuccessfulOperations = existing.SuccessfulOperations + successCount,
                ExecutionCount = existing.ExecutionCount + 1,
                LastExecuted = DateTime.UtcNow
            });
    }

    private string GetTagNameFromResult(BatchOperationResult result, BatchOperation[] operations, int index)
    {
        // Match result to operation by index (assuming results are returned in same order as operations)
        return index < operations.Length ? operations[index].TagName : "Unknown";
    }

    private bool IsWriteOperation(BatchOperationResult result, BatchOperation[] operations, int index)
    {
        // Match result to operation by index (assuming results are returned in same order as operations)
        return index < operations.Length ? operations[index].IsWrite : false;
    }

}

// ================================================================================
// BATCH OPERATION DATA MODELS
// ================================================================================

/// <summary>
/// Result of a batch read operation
/// </summary>
public class BatchReadResult
{
    public bool Success { get; set; }
    public Dictionary<string, TagReadResult>? Results { get; set; }
    public long TotalTimeMs { get; set; }
    public int SuccessCount { get; set; }
    public int ErrorCount { get; set; }
    public double AverageTimePerTagMs { get; set; }
    public string? ErrorMessage { get; set; }
}

/// <summary>
/// Result of a batch write operation
/// </summary>
public class BatchWriteResult
{
    public bool Success { get; set; }
    public Dictionary<string, TagWriteResult>? Results { get; set; }
    public long TotalTimeMs { get; set; }
    public int SuccessCount { get; set; }
    public int ErrorCount { get; set; }
    public double AverageTimePerTagMs { get; set; }
    public string? ErrorMessage { get; set; }
}

/// <summary>
/// Result of a mixed batch operation
/// </summary>
public class BatchMixedResult
{
    public bool Success { get; set; }
    public ApiMixedOperationResult[]? Results { get; set; }
    public long TotalTimeMs { get; set; }
    public int SuccessCount { get; set; }
    public int ErrorCount { get; set; }
    public double AverageTimePerOperationMs { get; set; }
    public string? ErrorMessage { get; set; }
}

/// <summary>
/// Result of a single mixed operation for API responses
/// </summary>
public class ApiMixedOperationResult
{
    public string TagName { get; set; } = string.Empty;
    public bool IsWrite { get; set; }
    public bool Success { get; set; }
    public object? Value { get; set; }
    public double ExecutionTimeMs { get; set; }
    public int ErrorCode { get; set; }
    public string? ErrorMessage { get; set; }
}

/// <summary>
/// Performance statistics for batch operations
/// </summary>
public class BatchPerformanceStats
{
    public string OperationType { get; set; } = string.Empty;
    public int TotalOperations { get; set; }
    public long TotalTimeMs { get; set; }
    public int SuccessfulOperations { get; set; }
    public int ExecutionCount { get; set; }
    public DateTime LastExecuted { get; set; }
    
    public double AverageTimePerOperation => TotalOperations > 0 ? (double)TotalTimeMs / TotalOperations : 0;
    public double SuccessRate => TotalOperations > 0 ? (double)SuccessfulOperations / TotalOperations * 100 : 0;
    public double AverageTimePerExecution => ExecutionCount > 0 ? (double)TotalTimeMs / ExecutionCount : 0;
}

/// <summary>
/// Batch benchmark configuration and results
/// </summary>
public class BatchBenchmarkRequest
{
    public int TagCount { get; set; } = 5;
    public string TestType { get; set; } = "Read"; // Read, Write, Mixed
    public int DurationSeconds { get; set; } = 5;
    public bool CompareWithIndividual { get; set; } = true;
}

/// <summary>
/// Result of a performance benchmark comparing individual vs batch operations
/// </summary>
public class BatchBenchmarkResult
{
    public bool Success { get; set; }
    public string TestType { get; set; } = string.Empty;
    public int TagCount { get; set; }
    
    // Individual operation results
    public long IndividualTotalTimeMs { get; set; }
    public int IndividualSuccessCount { get; set; }
    public double IndividualAverageTimeMs { get; set; }
    
    // Batch operation results  
    public long BatchTotalTimeMs { get; set; }
    public int BatchSuccessCount { get; set; }
    public double BatchAverageTimeMs { get; set; }
    
    // Performance comparison
    public double SpeedupFactor { get; set; }
    public double TimeSavedMs { get; set; }
    public double TimeSavedPercentage { get; set; }
    public int NetworkEfficiencyFactor { get; set; }
    
    public string? ErrorMessage { get; set; }
} 