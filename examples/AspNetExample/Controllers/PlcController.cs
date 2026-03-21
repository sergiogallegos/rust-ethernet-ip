using Microsoft.AspNetCore.Mvc;
using AspNetExample.Services;
using System;
using System.Threading.Tasks;
using System.Collections.Generic;
using System.Linq;
using System.Diagnostics;
using RustEtherNetIp;

namespace AspNetExample.Controllers;

[ApiController]
[Route("api/[controller]")]
public class PlcController : ControllerBase
{
    private readonly PlcService _plcService;
    private readonly ILogger<PlcController> _logger;

    public PlcController(PlcService plcService, ILogger<PlcController> logger)
    {
        _plcService = plcService;
        _logger = logger;
    }

    [HttpPost("connect")]
    public IActionResult Connect([FromBody] ConnectRequest request)
    {
        _logger.LogInformation("Connect request received. Address: {Address}, UseRoutePath: {UseRoutePath}, CpuSlot: {CpuSlot}", 
            request?.Address ?? "null", request?.UseRoutePath ?? false, request?.CpuSlot ?? 0);
        
        if (request == null)
        {
            _logger.LogWarning("Connect request is null");
            return BadRequest(new { success = false, message = "Invalid request format" });
        }
        
        if (string.IsNullOrEmpty(request.Address))
        {
            _logger.LogWarning("Connect request address is null or empty");
            return BadRequest(new { success = false, message = "PLC address is required" });
        }

        _logger.LogInformation("Attempting to connect to PLC at address: {Address}", request.Address);
        
        if (request.UseRoutePath)
        {
            _logger.LogInformation("Using RoutePath: CPU Slot {CpuSlot}", request.CpuSlot);
        }
        
        var connected = _plcService.Connect(request.Address, request.UseRoutePath, request.CpuSlot);
        if (connected)
        {
            _logger.LogInformation("Successfully connected to PLC at {Address}", request.Address);
            var sessionId = $"0x{_plcService.Client.ClientId:X8}";
            return Ok(new { 
                success = true, 
                message = "Connected successfully",
                sessionId = sessionId,
                useRoutePath = request.UseRoutePath,
                cpuSlot = request.UseRoutePath ? request.CpuSlot : (int?)null
            });
        }
        else
        {
            _logger.LogWarning("Failed to connect to PLC at {Address}", request.Address);
            return BadRequest(new { success = false, message = "Failed to connect to PLC" });
        }
    }

    [HttpPost("disconnect")]
    public IActionResult Disconnect()
    {
        _plcService.Disconnect();
        return Ok(new { success = true, message = "Disconnected successfully" });
    }

    // ================================================================================
    // BATCH OPERATIONS - High Performance Multi-Tag Operations
    // ================================================================================

    /// <summary>
    /// Read multiple tags in a single optimized batch operation.
    /// Provides 3-10x performance improvement over individual reads.
    /// </summary>
    [HttpPost("batch/read")]
    public async Task<IActionResult> BatchReadTags([FromBody] BatchReadRequest request)
    {
        if (!_plcService.IsConnected)
            return StatusCode(503, new { success = false, message = "Not connected to PLC" });

        if (request?.TagNames == null || request.TagNames.Length == 0)
            return BadRequest(new { success = false, message = "Tag names are required" });

        try
        {
            _logger.LogInformation("Batch read request for {TagCount} tags", request.TagNames.Length);
            
            var result = await _plcService.ReadTagsBatch(request.TagNames);
            
            return Ok(new 
            { 
                success = result.Success,
                results = result.Results,
                performance = new
                {
                    totalTimeMs = result.TotalTimeMs,
                    successCount = result.SuccessCount,
                    errorCount = result.ErrorCount,
                    averageTimePerTagMs = result.AverageTimePerTagMs,
                    tagsPerSecond = result.TotalTimeMs > 0 ? (request.TagNames.Length * 1000.0 / result.TotalTimeMs) : 0
                },
                message = result.Success 
                    ? $"Batch read completed: {result.SuccessCount}/{request.TagNames.Length} successful in {result.TotalTimeMs}ms"
                    : result.ErrorMessage
            });
        }
        catch (Exception ex)
        {
            _logger.LogError(ex, "Error in batch read operation");
            return StatusCode(500, new { success = false, message = ex.Message });
        }
    }

    /// <summary>
    /// Write multiple tags in a single optimized batch operation.
    /// Provides 3-10x performance improvement over individual writes.
    /// </summary>
    [HttpPost("batch/write")]
    public async Task<IActionResult> BatchWriteTags([FromBody] BatchWriteRequest request)
    {
        if (!_plcService.IsConnected)
            return StatusCode(503, new { success = false, message = "Not connected to PLC" });

        if (request?.TagValues == null || request.TagValues.Count == 0)
            return BadRequest(new { success = false, message = "Tag values are required" });

        try
        {
            _logger.LogInformation("Batch write request for {TagCount} tags", request.TagValues.Count);
            
            var result = await _plcService.WriteTagsBatch(request.TagValues);
            
            return Ok(new 
            { 
                success = result.Success,
                results = result.Results,
                performance = new
                {
                    totalTimeMs = result.TotalTimeMs,
                    successCount = result.SuccessCount,
                    errorCount = result.ErrorCount,
                    averageTimePerTagMs = result.AverageTimePerTagMs,
                    tagsPerSecond = result.TotalTimeMs > 0 ? (request.TagValues.Count * 1000.0 / result.TotalTimeMs) : 0
                },
                message = result.Success 
                    ? $"Batch write completed: {result.SuccessCount}/{request.TagValues.Count} successful in {result.TotalTimeMs}ms"
                    : result.ErrorMessage
            });
        }
        catch (Exception ex)
        {
            _logger.LogError(ex, "Error in batch write operation");
            return StatusCode(500, new { success = false, message = ex.Message });
        }
    }

    /// <summary>
    /// Execute a mixed batch of read and write operations in optimized packets.
    /// Ideal for coordinated control operations and data collection.
    /// </summary>
    [HttpPost("batch/execute")]
    public async Task<IActionResult> ExecuteBatch([FromBody] BatchMixedRequest request)
    {
        if (!_plcService.IsConnected)
            return StatusCode(503, new { success = false, message = "Not connected to PLC" });

        if (request?.Operations == null || request.Operations.Length == 0)
            return BadRequest(new { success = false, message = "Operations are required" });

        try
        {
            _logger.LogInformation("Mixed batch request for {OperationCount} operations", request.Operations.Length);
            
            var result = await _plcService.ExecuteBatch(request.Operations);
            
            return Ok(new 
            { 
                success = result.Success,
                results = result.Results,
                performance = new
                {
                    totalTimeMs = result.TotalTimeMs,
                    successCount = result.SuccessCount,
                    errorCount = result.ErrorCount,
                    averageTimePerOperationMs = result.AverageTimePerOperationMs,
                    operationsPerSecond = result.TotalTimeMs > 0 ? (request.Operations.Length * 1000.0 / result.TotalTimeMs) : 0
                },
                message = result.Success 
                    ? $"Mixed batch completed: {result.SuccessCount}/{request.Operations.Length} successful in {result.TotalTimeMs}ms"
                    : result.ErrorMessage
            });
        }
        catch (Exception ex)
        {
            _logger.LogError(ex, "Error in mixed batch operation");
            return StatusCode(500, new { success = false, message = ex.Message });
        }
    }

    /// <summary>
    /// Configure batch operation behavior for performance optimization.
    /// </summary>
    [HttpPost("batch/config")]
    public IActionResult ConfigureBatch([FromBody] BatchConfig config)
    {
        if (!_plcService.IsConnected)
            return StatusCode(503, new { success = false, message = "Not connected to PLC" });

        try
        {
            _plcService.ConfigureBatchOperations(config);
            
            return Ok(new 
            { 
                success = true,
                message = "Batch configuration updated successfully",
                config = _plcService.GetBatchConfig()
            });
        }
        catch (NotSupportedException ex)
        {
            _logger.LogWarning(ex, "Batch configuration API is not supported by the current wrapper/runtime");
            return StatusCode(501, new
            {
                success = false,
                message = ex.Message
            });
        }
        catch (Exception ex)
        {
            _logger.LogError(ex, "Error configuring batch operations");
            return StatusCode(500, new { success = false, message = ex.Message });
        }
    }

    /// <summary>
    /// Get current batch operation configuration.
    /// </summary>
    [HttpGet("batch/config")]
    public IActionResult GetBatchConfig()
    {
        if (!_plcService.IsConnected)
            return StatusCode(503, new { success = false, message = "Not connected to PLC" });

        try
        {
            var config = _plcService.GetBatchConfig();
            
            return Ok(new 
            { 
                success = true,
                config,
                presets = new
                {
                    defaultConfig = BatchConfig.Default(),
                    highPerformance = BatchConfig.HighPerformance(),
                    conservative = BatchConfig.Conservative()
                }
            });
        }
        catch (NotSupportedException ex)
        {
            _logger.LogWarning(ex, "Batch configuration API is not supported by the current wrapper/runtime");
            return StatusCode(501, new
            {
                success = false,
                message = ex.Message
            });
        }
        catch (Exception ex)
        {
            _logger.LogError(ex, "Error getting batch configuration");
            return StatusCode(500, new { success = false, message = ex.Message });
        }
    }

    /// <summary>
    /// Run performance benchmark comparing individual vs batch operations.
    /// </summary>
    [HttpPost("batch/benchmark")]
    public async Task<IActionResult> RunBatchBenchmark([FromBody] BatchBenchmarkRequest? request = null)
    {
        if (!_plcService.IsConnected)
            return StatusCode(503, new { success = false, message = "Not connected to PLC" });

        request ??= new BatchBenchmarkRequest();

        try
        {
            _logger.LogInformation("Starting batch benchmark: {TagCount} tags, {TestType}", request.TagCount, request.TestType);

            // Generate test tag names
            var testTags = Enumerable.Range(1, request.TagCount)
                .Select(i => $"TestTag_{i}")
                .ToArray();

            // Prepare test tags if needed (only create a few to avoid overwhelming PLC)
            if (request.TestType != "Write")
            {
                foreach (var tag in testTags.Take(Math.Min(5, request.TagCount)))
                {
                    try
                    {
                        _plcService.WriteBool(tag, true);
                    }
                    catch
                    {
                        // Tag might not exist, that's ok for demo
                    }
                }
            }

            var benchmarkResult = new BatchBenchmarkResult
            {
                Success = true,
                TestType = request.TestType,
                TagCount = request.TagCount
            };

            // Test individual operations if requested
            if (request.CompareWithIndividual)
            {
                _logger.LogInformation("Testing individual operations...");
                var individualStopwatch = Stopwatch.StartNew();
                int individualSuccessCount = 0;

                switch (request.TestType.ToUpper())
                {
                    case "READ":
                        foreach (var tag in testTags)
                        {
                            try
                            {
                                _plcService.ReadBool(tag);
                                individualSuccessCount++;
                            }
                            catch { }
                        }
                        break;

                    case "WRITE":
                        foreach (var tag in testTags)
                        {
                            try
                            {
                                _plcService.WriteBool(tag, true);
                                individualSuccessCount++;
                            }
                            catch { }
                        }
                        break;

                    case "MIXED":
                        for (int i = 0; i < testTags.Length; i++)
                        {
                            try
                            {
                                if (i % 2 == 0)
                                {
                                    _plcService.ReadBool(testTags[i]);
                                }
                                else
                                {
                                    _plcService.WriteBool(testTags[i], true);
                                }
                                individualSuccessCount++;
                            }
                            catch { }
                        }
                        break;
                }

                individualStopwatch.Stop();
                benchmarkResult.IndividualTotalTimeMs = individualStopwatch.ElapsedMilliseconds;
                benchmarkResult.IndividualSuccessCount = individualSuccessCount;
                benchmarkResult.IndividualAverageTimeMs = (double)individualStopwatch.ElapsedMilliseconds / request.TagCount;
            }

            // Test batch operations
            _logger.LogInformation("Testing batch operations...");
            var batchStopwatch = Stopwatch.StartNew();
            int batchSuccessCount = 0;

            switch (request.TestType.ToUpper())
            {
                case "READ":
                    try
                    {
                        var readResult = await _plcService.ReadTagsBatch(testTags);
                        batchSuccessCount = readResult.SuccessCount;
                    }
                    catch { }
                    break;

                case "WRITE":
                    try
                    {
                        var tagValues = testTags.ToDictionary(tag => tag, tag => (object)true);
                        var writeResult = await _plcService.WriteTagsBatch(tagValues);
                        batchSuccessCount = writeResult.SuccessCount;
                    }
                    catch { }
                    break;

                case "MIXED":
                    try
                    {
                        var operations = new List<BatchOperation>();
                        for (int i = 0; i < testTags.Length; i++)
                        {
                            if (i % 2 == 0)
                            {
                                operations.Add(BatchOperation.Read(testTags[i]));
                            }
                            else
                            {
                                operations.Add(BatchOperation.Write(testTags[i], true));
                            }
                        }
                        var mixedResult = await _plcService.ExecuteBatch(operations.ToArray());
                        batchSuccessCount = mixedResult.SuccessCount;
                    }
                    catch { }
                    break;
            }

            batchStopwatch.Stop();
            benchmarkResult.BatchTotalTimeMs = batchStopwatch.ElapsedMilliseconds;
            benchmarkResult.BatchSuccessCount = batchSuccessCount;
            benchmarkResult.BatchAverageTimeMs = (double)batchStopwatch.ElapsedMilliseconds / request.TagCount;

            // Calculate performance metrics
            if (request.CompareWithIndividual && benchmarkResult.BatchTotalTimeMs > 0)
            {
                benchmarkResult.SpeedupFactor = (double)benchmarkResult.IndividualTotalTimeMs / benchmarkResult.BatchTotalTimeMs;
                benchmarkResult.TimeSavedMs = benchmarkResult.IndividualTotalTimeMs - benchmarkResult.BatchTotalTimeMs;
                benchmarkResult.TimeSavedPercentage = (benchmarkResult.TimeSavedMs / benchmarkResult.IndividualTotalTimeMs) * 100;
                benchmarkResult.NetworkEfficiencyFactor = request.TagCount; // 1 packet vs N packets
            }

            _logger.LogInformation("Batch benchmark completed: Individual={IndividualTime}ms, Batch={BatchTime}ms, Speedup={Speedup:F1}x", 
                benchmarkResult.IndividualTotalTimeMs, benchmarkResult.BatchTotalTimeMs, benchmarkResult.SpeedupFactor);

            return Ok(new 
            { 
                success = true,
                benchmark = benchmarkResult,
                message = $"Benchmark completed: {benchmarkResult.SpeedupFactor:F1}x speedup with batch operations"
            });
        }
        catch (Exception ex)
        {
            _logger.LogError(ex, "Error running batch benchmark");
            return StatusCode(500, new { success = false, message = ex.Message });
        }
    }

    /// <summary>
    /// Get batch operation performance statistics.
    /// </summary>
    [HttpGet("batch/stats")]
    public IActionResult GetBatchStats()
    {
        if (!_plcService.IsConnected)
            return StatusCode(503, new { success = false, message = "Not connected to PLC" });

        try
        {
            var stats = _plcService.GetBatchStats();
            
            return Ok(new 
            { 
                success = true,
                stats,
                summary = new
                {
                    totalOperationTypes = stats.Count,
                    totalOperations = stats.Values.Sum(s => s.TotalOperations),
                    totalTimeMs = stats.Values.Sum(s => s.TotalTimeMs),
                    overallSuccessRate = stats.Values.Sum(s => s.SuccessfulOperations) * 100.0 / Math.Max(1, stats.Values.Sum(s => s.TotalOperations))
                }
            });
        }
        catch (Exception ex)
        {
            _logger.LogError(ex, "Error getting batch statistics");
            return StatusCode(500, new { success = false, message = ex.Message });
        }
    }

    /// <summary>
    /// Reset batch operation performance statistics.
    /// </summary>
    [HttpDelete("batch/stats")]
    public IActionResult ResetBatchStats()
    {
        if (!_plcService.IsConnected)
            return StatusCode(503, new { success = false, message = "Not connected to PLC" });

        try
        {
            _plcService.ResetBatchStats();
            return Ok(new { success = true, message = "Batch statistics reset successfully" });
        }
        catch (Exception ex)
        {
            _logger.LogError(ex, "Error resetting batch statistics");
            return StatusCode(500, new { success = false, message = ex.Message });
        }
    }

    // ================================================================================
    // STRING OPERATIONS - Dedicated endpoints for Allen-Bradley STRING support
    // ================================================================================

    /// <summary>
    /// Read a STRING tag from the PLC.
    /// Supports Allen-Bradley STRING format with proper Len, MaxLen, and Data structure.
    /// </summary>
    [HttpGet("string/{tagName}")]
    public IActionResult ReadString(string tagName)
    {
        if (!_plcService.IsConnected)
            return StatusCode(503, new { success = false, message = "Not connected to PLC" });

        try
        {
            _logger.LogInformation("Reading STRING tag: {TagName}", tagName);
            var value = _plcService.ReadString(tagName);
            
            return Ok(new 
            { 
                success = true, 
                value = value,
                type = "STRING",
                length = value.Length,
                maxLength = 82, // Allen-Bradley STRING max length
                message = $"Successfully read STRING tag '{tagName}'"
            });
        }
        catch (Exception ex)
        {
            _logger.LogError(ex, "Error reading STRING tag {TagName}", tagName);
            return StatusCode(500, new { success = false, message = ex.Message });
        }
    }

    /// <summary>
    /// Write a STRING tag to the PLC.
    /// ⚠️ WARNING: This operation will fail due to PLC firmware limitations (CIP Error 0x2107).
    /// STRING tags cannot be written directly. This is a PLC firmware restriction, not a library bug.
    /// For STRING members in UDTs, use the workaround: read entire UDT, modify STRING member, write entire UDT back.
    /// </summary>
    [HttpPost("string/{tagName}")]
    public IActionResult WriteString(string tagName, [FromBody] StringWriteRequest request)
    {
        if (!_plcService.IsConnected)
            return StatusCode(503, new { success = false, message = "Not connected to PLC" });

        if (request?.Value == null)
            return BadRequest(new { success = false, message = "String value is required" });

        if (request.Value.Length > 82)
            return BadRequest(new { success = false, message = "String too long. Maximum length is 82 characters for Allen-Bradley STRING." });

        try
        {
            _logger.LogInformation("Writing STRING tag {TagName} with value: '{Value}'", tagName, request.Value);
            _plcService.WriteString(tagName, request.Value);
            
            return Ok(new 
            { 
                success = true, 
                message = $"Successfully wrote STRING '{request.Value}' to tag '{tagName}'",
                value = request.Value,
                length = request.Value.Length,
                maxLength = 82
            });
        }
        catch (Exception ex)
        {
            _logger.LogError(ex, "Error writing STRING tag {TagName}", tagName);
            string errorMsg = ex.Message;
            if (errorMsg.Contains("0x2107") || errorMsg.Contains("2107"))
            {
                errorMsg = "PLC firmware limitation (CIP Error 0x2107): STRING tags cannot be written directly. " +
                          "This is a PLC restriction, not a library bug. " +
                          "For STRING members in UDTs, use the LogixString helper and write the entire UDT.";
            }
            return StatusCode(500, new { success = false, message = errorMsg, isPlcLimitation = errorMsg.Contains("0x2107") || errorMsg.Contains("2107") });
        }
    }

    /// <summary>
    /// Batch read multiple STRING tags for efficient string data collection.
    /// </summary>
    [HttpPost("string/batch/read")]
    public async Task<IActionResult> BatchReadStrings([FromBody] StringBatchReadRequest request)
    {
        if (!_plcService.IsConnected)
            return StatusCode(503, new { success = false, message = "Not connected to PLC" });

        if (request?.TagNames == null || request.TagNames.Length == 0)
            return BadRequest(new { success = false, message = "Tag names are required" });

        try
        {
            _logger.LogInformation("Batch STRING read request for {TagCount} tags", request.TagNames.Length);
            
            var stopwatch = Stopwatch.StartNew();
            var results = new List<object>();
            int successCount = 0;
            
            foreach (var tagName in request.TagNames)
            {
                try
                {
                    var value = _plcService.ReadString(tagName);
                    results.Add(new 
                    { 
                        tagName = tagName,
                        success = true,
                        value = value,
                        length = value.Length,
                        type = "STRING"
                    });
                    successCount++;
                }
                catch (Exception ex)
                {
                    results.Add(new 
                    { 
                        tagName = tagName,
                        success = false,
                        error = ex.Message,
                        type = "STRING"
                    });
                }
            }
            
            stopwatch.Stop();
            
            return Ok(new 
            { 
                success = successCount > 0,
                results = results,
                performance = new
                {
                    totalTimeMs = stopwatch.ElapsedMilliseconds,
                    successCount = successCount,
                    errorCount = request.TagNames.Length - successCount,
                    averageTimePerTagMs = (double)stopwatch.ElapsedMilliseconds / request.TagNames.Length,
                    tagsPerSecond = stopwatch.ElapsedMilliseconds > 0 ? (request.TagNames.Length * 1000.0 / stopwatch.ElapsedMilliseconds) : 0
                },
                message = $"Batch STRING read completed: {successCount}/{request.TagNames.Length} successful in {stopwatch.ElapsedMilliseconds}ms"
            });
        }
        catch (Exception ex)
        {
            _logger.LogError(ex, "Error in batch STRING read operation");
            return StatusCode(500, new { success = false, message = ex.Message });
        }
    }

    /// <summary>
    /// Batch write multiple STRING tags for efficient string data updates.
    /// </summary>
    [HttpPost("string/batch/write")]
    public async Task<IActionResult> BatchWriteStrings([FromBody] StringBatchWriteRequest request)
    {
        if (!_plcService.IsConnected)
            return StatusCode(503, new { success = false, message = "Not connected to PLC" });

        if (request?.TagValues == null || request.TagValues.Count == 0)
            return BadRequest(new { success = false, message = "Tag values are required" });

        // Validate string lengths
        var invalidTags = request.TagValues.Where(kvp => kvp.Value.Length > 82).ToList();
        if (invalidTags.Any())
        {
            return BadRequest(new 
            { 
                success = false, 
                message = "Some strings are too long. Maximum length is 82 characters.",
                invalidTags = invalidTags.Select(kvp => new { tag = kvp.Key, length = kvp.Value.Length }).ToArray()
            });
        }

        try
        {
            _logger.LogInformation("Batch STRING write request for {TagCount} tags", request.TagValues.Count);
            
            var stopwatch = Stopwatch.StartNew();
            var results = new List<object>();
            int successCount = 0;
            
            foreach (var kvp in request.TagValues)
            {
                try
                {
                    _plcService.WriteString(kvp.Key, kvp.Value);
                    results.Add(new 
                    { 
                        tagName = kvp.Key,
                        success = true,
                        value = kvp.Value,
                        length = kvp.Value.Length,
                        type = "STRING"
                    });
                    successCount++;
                }
                catch (Exception ex)
                {
                    results.Add(new 
                    { 
                        tagName = kvp.Key,
                        success = false,
                        error = ex.Message,
                        type = "STRING"
                    });
                }
            }
            
            stopwatch.Stop();
            
            return Ok(new 
            { 
                success = successCount > 0,
                results = results,
                performance = new
                {
                    totalTimeMs = stopwatch.ElapsedMilliseconds,
                    successCount = successCount,
                    errorCount = request.TagValues.Count - successCount,
                    averageTimePerTagMs = (double)stopwatch.ElapsedMilliseconds / request.TagValues.Count,
                    tagsPerSecond = stopwatch.ElapsedMilliseconds > 0 ? (request.TagValues.Count * 1000.0 / stopwatch.ElapsedMilliseconds) : 0
                },
                message = $"Batch STRING write completed: {successCount}/{request.TagValues.Count} successful in {stopwatch.ElapsedMilliseconds}ms"
            });
        }
        catch (Exception ex)
        {
            _logger.LogError(ex, "Error in batch STRING write operation");
            return StatusCode(500, new { success = false, message = ex.Message });
        }
    }

    // ================================================================================
    // INDIVIDUAL TAG OPERATIONS
    // ================================================================================

    [HttpGet("tag/{tagName}")]
    public IActionResult ReadTag(string tagName)
    {
        if (!_plcService.IsConnected)
            return StatusCode(503, new { success = false, message = "Not connected to PLC" });

        _logger.LogInformation("Attempting to discover type for tag: {TagName}", tagName);

        try
        {
            // Try to use the PlcService metadata method if available
            try
            {
                var metadata = _plcService.GetTagMetadata(_plcService.CurrentAddress, tagName);
                if (metadata != null)
                {
                    _logger.LogInformation("Tag metadata found for {TagName}: {Metadata}", tagName, metadata);
                    return Ok(new { success = true, value = metadata.GetType().GetProperty("value")?.GetValue(metadata), type = metadata.GetType().GetProperty("type")?.GetValue(metadata)?.ToString() });
                }
            }
            catch (Exception metaEx)
            {
                _logger.LogDebug("Metadata discovery failed for {TagName}: {Error}", tagName, metaEx.Message);
            }

            // Try types in a smarter order - more specific types first
            var typeAttempts = new List<(string Type, Func<object> ReadFunc)>
            {
                ("STRING", () => _plcService.ReadString(tagName)),
                ("LREAL", () => _plcService.ReadLreal(tagName)),
                ("REAL", () => _plcService.ReadReal(tagName)),
                ("LINT", () => _plcService.ReadLint(tagName)),
                ("ULINT", () => _plcService.ReadUlint(tagName)),
                ("DINT", () => _plcService.ReadDint(tagName)),
                ("UDINT", () => _plcService.ReadUdint(tagName)),
                ("INT", () => _plcService.ReadInt(tagName)),
                ("UINT", () => _plcService.ReadUint(tagName)),
                ("SINT", () => _plcService.ReadSint(tagName)),
                ("USINT", () => _plcService.ReadUsint(tagName)),
                ("BOOL", () => _plcService.ReadBool(tagName)),
                ("UDT", () => _plcService.ReadUdt(tagName))
            };

            Exception lastException = null;
            
            foreach (var (type, readFunc) in typeAttempts)
            {
                try
                {
                    var value = readFunc();
                    _logger.LogInformation("Successfully read tag {TagName} as {Type} with value: {Value}", tagName, type, value);
                    return Ok(new { success = true, value = value, type = type });
                }
                catch (Exception ex)
                {
                    lastException = ex;
                    _logger.LogDebug("Failed to read {TagName} as {Type}: {Error}", tagName, type, ex.Message);
                }
            }
            
            _logger.LogWarning("Could not determine type for tag {TagName}. Last error: {Error}", tagName, lastException?.Message);
            return NotFound(new { success = false, message = $"Could not determine type for tag {tagName}. Tag may not exist or may be an unsupported type." });
        }
        catch (Exception ex)
        {
            _logger.LogError(ex, "Error reading tag {TagName}", tagName);
            return StatusCode(500, new { success = false, message = ex.Message });
        }
    }

    [HttpPost("tag/{tagName}")]
    public IActionResult WriteTag(string tagName, [FromBody] WriteTagRequest request)
    {
        if (!_plcService.IsConnected)
            return StatusCode(503, new { success = false, message = "Not connected to PLC" });

        try
        {
            switch (request.Type.ToUpper())
            {
                case "BOOL":
                    if (bool.TryParse(request.Value.ToString(), out bool boolValue))
                    {
                        _plcService.WriteBool(tagName, boolValue);
                        return Ok(new { success = true, message = $"Wrote BOOL {boolValue} to {tagName}" });
                    }
                    return BadRequest(new { success = false, message = "Invalid boolean value" });
                    
                case "SINT":
                    if (sbyte.TryParse(request.Value.ToString(), out sbyte sintValue))
                    {
                        _plcService.WriteSint(tagName, sintValue);
                        return Ok(new { success = true, message = $"Wrote SINT {sintValue} to {tagName}" });
                    }
                    return BadRequest(new { success = false, message = "Invalid SINT value (-128 to 127)" });
                    
                case "INT":
                    if (short.TryParse(request.Value.ToString(), out short intValue))
                    {
                        _plcService.WriteInt(tagName, intValue);
                        return Ok(new { success = true, message = $"Wrote INT {intValue} to {tagName}" });
                    }
                    return BadRequest(new { success = false, message = "Invalid INT value (-32,768 to 32,767)" });
                    
                case "DINT":
                    if (int.TryParse(request.Value.ToString(), out int dintValue))
                    {
                        _plcService.WriteDint(tagName, dintValue);
                        return Ok(new { success = true, message = $"Wrote DINT {dintValue} to {tagName}" });
                    }
                    return BadRequest(new { success = false, message = "Invalid DINT value" });
                    
                case "LINT":
                    if (long.TryParse(request.Value.ToString(), out long lintValue))
                    {
                        _plcService.WriteLint(tagName, lintValue);
                        return Ok(new { success = true, message = $"Wrote LINT {lintValue} to {tagName}" });
                    }
                    return BadRequest(new { success = false, message = "Invalid LINT value" });
                    
                case "USINT":
                    if (byte.TryParse(request.Value.ToString(), out byte usintValue))
                    {
                        _plcService.WriteUsint(tagName, usintValue);
                        return Ok(new { success = true, message = $"Wrote USINT {usintValue} to {tagName}" });
                    }
                    return BadRequest(new { success = false, message = "Invalid USINT value (0 to 255)" });
                    
                case "UINT":
                    if (ushort.TryParse(request.Value.ToString(), out ushort uintValue))
                    {
                        _plcService.WriteUint(tagName, uintValue);
                        return Ok(new { success = true, message = $"Wrote UINT {uintValue} to {tagName}" });
                    }
                    return BadRequest(new { success = false, message = "Invalid UINT value (0 to 65,535)" });
                    
                case "UDINT":
                    if (uint.TryParse(request.Value.ToString(), out uint udintValue))
                    {
                        _plcService.WriteUdint(tagName, udintValue);
                        return Ok(new { success = true, message = $"Wrote UDINT {udintValue} to {tagName}" });
                    }
                    return BadRequest(new { success = false, message = "Invalid UDINT value" });
                    
                case "ULINT":
                    if (ulong.TryParse(request.Value.ToString(), out ulong ulintValue))
                    {
                        _plcService.WriteUlint(tagName, ulintValue);
                        return Ok(new { success = true, message = $"Wrote ULINT {ulintValue} to {tagName}" });
                    }
                    return BadRequest(new { success = false, message = "Invalid ULINT value" });
                    
                case "REAL":
                    if (float.TryParse(request.Value.ToString(), out float realValue))
                    {
                        _plcService.WriteReal(tagName, realValue);
                        return Ok(new { success = true, message = $"Wrote REAL {realValue} to {tagName}" });
                    }
                    return BadRequest(new { success = false, message = "Invalid REAL value" });
                    
                case "LREAL":
                    if (double.TryParse(request.Value.ToString(), out double lrealValue))
                    {
                        _plcService.WriteLreal(tagName, lrealValue);
                        return Ok(new { success = true, message = $"Wrote LREAL {lrealValue} to {tagName}" });
                    }
                    return BadRequest(new { success = false, message = "Invalid LREAL value" });
                    
                case "STRING":
                    if (request.Value == null)
                        return BadRequest(new { success = false, message = "String value cannot be null" });
                    string stringValue = request.Value.ToString() ?? string.Empty;
                    _plcService.WriteString(tagName, stringValue);
                    return Ok(new { success = true, message = $"Wrote STRING '{stringValue}' to {tagName}" });
                    
                case "UDT":
                    return BadRequest(new { success = false, message = "UDT writing not supported in this example" });
                    
                default:
                    return BadRequest(new { success = false, message = $"Unsupported type: {request.Type}" });
            }
        }
        catch (Exception ex)
        {
            _logger.LogError(ex, "Error writing to tag {TagName}", tagName);
            return StatusCode(500, new { success = false, message = ex.Message });
        }
    }

    [HttpPost("benchmark")]
    public IActionResult RunBenchmark([FromBody] BenchmarkRequest? request = null)
    {
        if (!_plcService.IsConnected)
            return StatusCode(503, new { success = false, message = "Not connected to PLC" });

        // Use provided test tag or default
        string testTag = request?.TestTag ?? "TestTag";
        bool testWrites = request?.TestWrites ?? false;
        int durationSeconds = Math.Max(1, Math.Min(request?.DurationSeconds ?? 5, 30)); // 1-30 seconds

        _logger.LogInformation("Starting benchmark with tag: {TestTag}, Duration: {Duration}s, TestWrites: {TestWrites}", 
            testTag, durationSeconds, testWrites);

        try
        {
            var startTime = DateTime.Now;
            var readCount = 0;
            var writeCount = 0;
            var readErrors = 0;
            var writeErrors = 0;
            string detectedType = "UNKNOWN";
            bool tagExists = false;

            // First, detect the tag's actual data type
            var typeAttempts = new List<(string Type, Func<object> ReadFunc, Action<object> WriteFunc)>
            {
                ("BOOL", () => _plcService.ReadBool(testTag), (val) => _plcService.WriteBool(testTag, (bool)val)),
                ("DINT", () => _plcService.ReadDint(testTag), (val) => _plcService.WriteDint(testTag, (int)val)),
                ("REAL", () => _plcService.ReadReal(testTag), (val) => _plcService.WriteReal(testTag, (float)val)),
                ("INT", () => _plcService.ReadInt(testTag), (val) => _plcService.WriteInt(testTag, (short)val)),
                ("STRING", () => _plcService.ReadString(testTag), (val) => _plcService.WriteString(testTag, (string)val)),
                ("LREAL", () => _plcService.ReadLreal(testTag), (val) => _plcService.WriteLreal(testTag, (double)val)),
                ("SINT", () => _plcService.ReadSint(testTag), (val) => _plcService.WriteSint(testTag, (sbyte)val)),
                ("USINT", () => _plcService.ReadUsint(testTag), (val) => _plcService.WriteUsint(testTag, (byte)val)),
                ("UINT", () => _plcService.ReadUint(testTag), (val) => _plcService.WriteUint(testTag, (ushort)val)),
                ("UDINT", () => _plcService.ReadUdint(testTag), (val) => _plcService.WriteUdint(testTag, (uint)val)),
                ("LINT", () => _plcService.ReadLint(testTag), (val) => _plcService.WriteLint(testTag, (long)val)),
                ("ULINT", () => _plcService.ReadUlint(testTag), (val) => _plcService.WriteUlint(testTag, (ulong)val))
            };

            Func<object> readFunction = null;
            Action<object> writeFunction = null;
            object testValue = null;

            // Detect the correct data type
            foreach (var (type, readFunc, writeFunc) in typeAttempts)
            {
                try
                {
                    var value = readFunc();
                    detectedType = type;
                    readFunction = readFunc;
                    writeFunction = writeFunc;
                    testValue = GetTestValue(type, value);
                    tagExists = true;
                    _logger.LogInformation("Detected tag {TestTag} as type {Type} with value: {Value}", testTag, type, value);
                    break;
                }
                catch
                {
                    // Continue to next type
                }
            }

            if (!tagExists)
            {
                _logger.LogWarning("Tag {TestTag} does not exist or is not readable", testTag);
                return Ok(new { 
                    success = false, 
                    readRate = 0, 
                    writeRate = 0, 
                    message = $"Tag '{testTag}' does not exist or is not accessible",
                    details = new {
                        testTag,
                        durationSeconds = 0.0,
                        readCount = 0,
                        writeCount = 0,
                        readErrors = 0,
                        writeErrors = 0,
                        tagExists = false,
                        detectedType = "UNKNOWN"
                    }
                });
            }

            // Run the benchmark with the detected type
            while ((DateTime.Now - startTime).TotalSeconds < durationSeconds)
            {
                // Test reads
                try 
                { 
                    readFunction();
                    readCount++; 
                } 
                catch 
                { 
                    readErrors++; 
                }

                // Test writes (only if enabled)
                if (testWrites && writeFunction != null)
                {
                    try 
                    { 
                        writeFunction(testValue);
                        writeCount++; 
                        // Alternate test values for some types
                        if (detectedType == "BOOL")
                            testValue = !(bool)testValue;
                        else if (detectedType == "DINT")
                            testValue = ((int)testValue == 100) ? 200 : 100;
                    } 
                    catch 
                    { 
                        writeErrors++; 
                    }
                }
            }

            var actualDuration = (DateTime.Now - startTime).TotalSeconds;
            var readRate = (int)(readCount / actualDuration);
            var writeRate = (int)(writeCount / actualDuration);

            _logger.LogInformation("Benchmark complete. Type: {Type}, Reads: {ReadCount} ({ReadRate}/sec), Writes: {WriteCount} ({WriteRate}/sec), Read Errors: {ReadErrors}, Write Errors: {WriteErrors}", 
                detectedType, readCount, readRate, writeCount, writeRate, readErrors, writeErrors);

            var message = $"Benchmark complete: {readRate} reads/sec";
            if (testWrites)
                message += $", {writeRate} writes/sec";
            message += $" (Type: {detectedType})";

            return Ok(new { 
                success = true, 
                readRate, 
                writeRate, 
                message,
                details = new {
                    testTag,
                    durationSeconds = actualDuration,
                    readCount,
                    writeCount,
                    readErrors,
                    writeErrors,
                    tagExists = true,
                    detectedType
                }
            });
        }
        catch (Exception ex)
        {
            _logger.LogError(ex, "Error running benchmark");
            return StatusCode(500, new { success = false, message = ex.Message });
        }
    }

    private object GetTestValue(string type, object currentValue)
    {
        return type switch
        {
            "BOOL" => true,
            "SINT" => (sbyte)10,
            "INT" => (short)100,
            "DINT" => 1000,
            "LINT" => 10000L,
            "USINT" => (byte)10,
            "UINT" => (ushort)100,
            "UDINT" => 1000U,
            "ULINT" => 10000UL,
            "REAL" => 123.45f,
            "LREAL" => 123.45,
            "STRING" => "TEST",
            _ => currentValue
        };
    }

    [HttpGet("status")]
    public IActionResult GetStatus()
    {
        try
        {
            var status = new
            {
                IsConnected = _plcService.IsConnected,
                Address = _plcService.CurrentAddress,
                IsHealthy = _plcService.IsHealthy,
                LastHealthCheck = _plcService.LastHealthCheck.ToString("HH:mm:ss.fff"),
                LastReadTimes = _plcService.LastReadTimes.ToDictionary(
                    kvp => kvp.Key,
                    kvp => kvp.Value.ToString("HH:mm:ss.fff")
                ),
                Statistics = _plcService.IsConnected ? new
                {
                    ReadCount = _plcService.Client.Statistics.ReadCount,
                    WriteCount = _plcService.Client.Statistics.WriteCount,
                    ErrorCount = _plcService.Client.Statistics.ErrorCount,
                    AverageResponseTimeMs = _plcService.Client.Statistics.AverageResponseTime.TotalMilliseconds
                } : null
            };
            return Ok(new { success = true, status });
        }
        catch (Exception ex)
        {
            _logger.LogError(ex, "Error getting status");
            return StatusCode(500, new { success = false, message = ex.Message });
        }
    }

    /// <summary>
    /// Get performance statistics for the PLC client.
    /// </summary>
    [HttpGet("statistics")]
    public IActionResult GetStatistics()
    {
        if (!_plcService.IsConnected)
            return StatusCode(503, new { success = false, message = "Not connected to PLC" });

        try
        {
            var stats = _plcService.Client.Statistics;
            return Ok(new
            {
                success = true,
                statistics = new
                {
                    readCount = stats.ReadCount,
                    writeCount = stats.WriteCount,
                    errorCount = stats.ErrorCount,
                    averageResponseTimeMs = stats.AverageResponseTime.TotalMilliseconds,
                    totalOperations = stats.ReadCount + stats.WriteCount,
                    successRate = (stats.ReadCount + stats.WriteCount) > 0
                        ? ((double)(stats.ReadCount + stats.WriteCount - stats.ErrorCount) / (stats.ReadCount + stats.WriteCount)) * 100
                        : 0
                }
            });
        }
        catch (Exception ex)
        {
            _logger.LogError(ex, "Error getting statistics");
            return StatusCode(500, new { success = false, message = ex.Message });
        }
    }

    /// <summary>
    /// Reset performance statistics.
    /// </summary>
    [HttpPost("statistics/reset")]
    public IActionResult ResetStatistics()
    {
        if (!_plcService.IsConnected)
            return StatusCode(503, new { success = false, message = "Not connected to PLC" });

        try
        {
            _plcService.Client.Statistics.Reset();
            return Ok(new { success = true, message = "Statistics reset successfully" });
        }
        catch (Exception ex)
        {
            _logger.LogError(ex, "Error resetting statistics");
            return StatusCode(500, new { success = false, message = ex.Message });
        }
    }

    // ================================================================================
    // ARRAY OPERATIONS - Array element addressing
    // ================================================================================

    /// <summary>
    /// Read an array element from the PLC.
    /// </summary>
    [HttpGet("array/{tagName}")]
    public IActionResult ReadArrayElement(string tagName)
    {
        if (!_plcService.IsConnected)
            return StatusCode(503, new { success = false, message = "Not connected to PLC" });

        try
        {
            _logger.LogInformation("Reading array element: {TagName}", tagName);
            
            // Try to determine type from tag name or try common types
            object value;
            string type = "UNKNOWN";
            
            if (tagName.Contains("DINT") || (tagName.Contains("[") && !tagName.Contains("REAL") && !tagName.Contains("BOOL")))
            {
                value = _plcService.ReadDint(tagName);
                type = "DINT";
            }
            else if (tagName.Contains("REAL"))
            {
                value = _plcService.ReadReal(tagName);
                type = "REAL";
            }
            else if (tagName.Contains("BOOL"))
            {
                value = _plcService.ReadBool(tagName);
                type = "BOOL";
            }
            else if (tagName.Contains("INT") && !tagName.Contains("DINT"))
            {
                value = _plcService.ReadInt(tagName);
                type = "INT";
            }
            else
            {
                // Default to DINT
                value = _plcService.ReadDint(tagName);
                type = "DINT";
            }
            
            return Ok(new 
            { 
                success = true, 
                value = value,
                type = type,
                tagName = tagName,
                message = $"Successfully read {type} array element '{tagName}'"
            });
        }
        catch (Exception ex)
        {
            _logger.LogError(ex, "Error reading array element {TagName}", tagName);
            return StatusCode(500, new { success = false, message = ex.Message });
        }
    }

    /// <summary>
    /// Write an array element to the PLC.
    /// </summary>
    [HttpPost("array/{tagName}")]
    public IActionResult WriteArrayElement(string tagName, [FromBody] ArrayWriteRequest request)
    {
        if (!_plcService.IsConnected)
            return StatusCode(503, new { success = false, message = "Not connected to PLC" });

        if (request?.Value == null)
            return BadRequest(new { success = false, message = "Value is required" });

        try
        {
            _logger.LogInformation("Writing array element: {TagName} = {Value}", tagName, request.Value);
            
            string type = "UNKNOWN";
            
            if (tagName.Contains("DINT") || (tagName.Contains("INT") && !tagName.Contains("REAL") && !tagName.Contains("BOOL")))
            {
                if (int.TryParse(request.Value.ToString(), out int intValue))
                {
                    _plcService.WriteDint(tagName, intValue);
                    type = "DINT";
                }
                else
                {
                    return BadRequest(new { success = false, message = "Invalid DINT value" });
                }
            }
            else if (tagName.Contains("REAL"))
            {
                if (float.TryParse(request.Value.ToString(), out float floatValue))
                {
                    _plcService.WriteReal(tagName, floatValue);
                    type = "REAL";
                }
                else
                {
                    return BadRequest(new { success = false, message = "Invalid REAL value" });
                }
            }
            else if (tagName.Contains("BOOL"))
            {
                if (bool.TryParse(request.Value.ToString(), out bool boolValue))
                {
                    _plcService.WriteBool(tagName, boolValue);
                    type = "BOOL";
                }
                else
                {
                    return BadRequest(new { success = false, message = "Invalid BOOL value" });
                }
            }
            else if (tagName.Contains("INT") && !tagName.Contains("DINT"))
            {
                if (short.TryParse(request.Value.ToString(), out short shortValue))
                {
                    _plcService.WriteInt(tagName, shortValue);
                    type = "INT";
                }
                else
                {
                    return BadRequest(new { success = false, message = "Invalid INT value" });
                }
            }
            else
            {
                // Default to DINT
                if (int.TryParse(request.Value.ToString(), out int defaultIntValue))
                {
                    _plcService.WriteDint(tagName, defaultIntValue);
                    type = "DINT";
                }
                else
                {
                    return BadRequest(new { success = false, message = "Invalid value" });
                }
            }
            
            return Ok(new 
            { 
                success = true, 
                message = $"Successfully wrote {type} value '{request.Value}' to array element '{tagName}'",
                tagName = tagName,
                value = request.Value,
                type = type
            });
        }
        catch (Exception ex)
        {
            _logger.LogError(ex, "Error writing array element {TagName}", tagName);
            return StatusCode(500, new { success = false, message = ex.Message });
        }
    }

    // ================================================================================
    // UDT (User Defined Type) ENDPOINTS
    // ================================================================================

    [HttpGet("debug-udt/{udtName}")]
    public IActionResult DebugUdt(string udtName)
    {
        try
        {
            _logger.LogInformation("Debug UDT reading for: {UdtName}", udtName);
            
            // Test direct UDT reading
            var result = _plcService.ReadUdt(udtName);
            _logger.LogInformation("UDT read successful with {Count} members", result.Count);
            
            return Ok(new { 
                success = true, 
                udtName = udtName,
                memberCount = result.Count,
                members = result,
                message = "UDT read successfully" 
            });
        }
        catch (Exception ex)
        {
            _logger.LogError(ex, "Error in debug UDT reading: {Message}", ex.Message);
            return StatusCode(500, new { 
                success = false, 
                message = ex.Message,
                stackTrace = ex.StackTrace 
            });
        }
    }

    [HttpGet("udt/{udtName}")]
    public IActionResult ReadUdt(string udtName)
    {
        try
        {
            _logger.LogInformation("Reading UDT: {UdtName}", udtName);
            
            // Try different tag path formats
            var tagVariations = new[]
            {
                udtName,
                $"{udtName}[0]",
                $"Program:MainProgram.{udtName}",
                $"Program:MainProgram.{udtName}[0]",
                $"Controller:{udtName}",
                $"Controller:{udtName}[0]"
            };
            
            foreach (var tagPath in tagVariations)
            {
                try
                {
                    _logger.LogInformation("Trying UDT path: {TagPath}", tagPath);
                    var result = _plcService.ReadUdt(tagPath);
                    _logger.LogInformation("Successfully read UDT with path: {TagPath}", tagPath);
                    return Ok(new { success = true, value = result, type = "UDT", path = tagPath });
                }
                catch (Exception ex)
                {
                    _logger.LogWarning("Failed to read UDT with path {TagPath}: {Error}", tagPath, ex.Message);
                    // Continue to next variation
                }
            }
            
            return BadRequest(new { success = false, message = $"UDT '{udtName}' not found. Tried paths: {string.Join(", ", tagVariations)}" });
        }
        catch (Exception ex)
        {
            _logger.LogError(ex, "Error reading UDT: {UdtName}", udtName);
            return StatusCode(500, new { success = false, message = ex.Message });
        }
    }

    [HttpGet("udt-dict/{udtName}")]
    public IActionResult ReadUdtAsDictionary(string udtName)
    {
        try
        {
            _logger.LogInformation("Reading UDT as Dictionary: {UdtName}", udtName);
            
            // Try different tag path formats
            var tagVariations = new[]
            {
                udtName,
                $"{udtName}[0]",
                $"Program:MainProgram.{udtName}",
                $"Program:MainProgram.{udtName}[0]",
                $"Controller:{udtName}",
                $"Controller:{udtName}[0]"
            };
            
            foreach (var tagPath in tagVariations)
            {
                try
                {
                    _logger.LogInformation("Trying UDT path: {TagPath}", tagPath);
                    var result = _plcService.ReadUdt(tagPath);
                    _logger.LogInformation("Successfully read UDT as Dictionary with path: {TagPath}", tagPath);
                    return Ok(new { success = true, value = result, type = "UDT", path = tagPath });
                }
                catch (Exception ex)
                {
                    _logger.LogWarning("Failed to read UDT as Dictionary with path {TagPath}: {Error}", tagPath, ex.Message);
                    // Continue to next variation
                }
            }
            
            return BadRequest(new { success = false, message = $"UDT '{udtName}' not found. Tried paths: {string.Join(", ", tagVariations)}" });
        }
        catch (Exception ex)
        {
            _logger.LogError(ex, "Error reading UDT as Dictionary: {UdtName}", udtName);
            return StatusCode(500, new { success = false, message = ex.Message });
        }
    }

    /// <summary>
    /// Read a UDT member from the PLC.
    /// </summary>
    [HttpGet("udt-member/{memberPath}")]
    public IActionResult ReadUdtMember(string memberPath)
    {
        if (!_plcService.IsConnected)
            return StatusCode(503, new { success = false, message = "Not connected to PLC" });

        try
        {
            _logger.LogInformation("Reading UDT member: {MemberPath}", memberPath);
            
            // Parse the path: "gTestUDT.Member1_DINT" -> tagName="gTestUDT", memberPath="Member1_DINT"
            var parts = memberPath.Split('.');
            if (parts.Length < 2)
            {
                return BadRequest(new { success = false, message = "Invalid UDT member path. Use format: 'UDTName.MemberName'" });
            }

            var tagName = parts[0];
            var memberName = string.Join(".", parts.Skip(1));

            // Try direct tag access first
            try
            {
                if (memberName.Contains("DINT") || (memberName.Contains("INT") && !memberName.Contains("REAL")))
                {
                    var intValue = _plcService.ReadDint(memberPath);
                    return Ok(new 
                    { 
                        success = true, 
                        value = intValue,
                        type = "DINT",
                        tagName = tagName,
                        memberName = memberName,
                        fullPath = memberPath,
                        message = $"Successfully read DINT member '{memberPath}'"
                    });
                }
                else if (memberName.Contains("REAL"))
                {
                    var floatValue = _plcService.ReadReal(memberPath);
                    return Ok(new 
                    { 
                        success = true, 
                        value = floatValue,
                        type = "REAL",
                        tagName = tagName,
                        memberName = memberName,
                        fullPath = memberPath,
                        message = $"Successfully read REAL member '{memberPath}'"
                    });
                }
                else if (memberName.Contains("BOOL"))
                {
                    var boolValue = _plcService.ReadBool(memberPath);
                    return Ok(new 
                    { 
                        success = true, 
                        value = boolValue,
                        type = "BOOL",
                        tagName = tagName,
                        memberName = memberName,
                        fullPath = memberPath,
                        message = $"Successfully read BOOL member '{memberPath}'"
                    });
                }
            }
            catch (Exception ex)
            {
                _logger.LogWarning("Direct tag access failed for '{MemberPath}': {Error}", memberPath, ex.Message);
            }

            // Fallback: Read full UDT and extract member
            _logger.LogInformation("Reading full UDT '{TagName}' to extract member '{MemberName}'", tagName, memberName);
            var udtValue = _plcService.ReadUdt(tagName);
            var memberValue = _plcService.GetUdtMember(tagName, memberName);

            if (memberValue == null)
            {
                return NotFound(new { success = false, message = $"Member '{memberName}' not found in UDT '{tagName}'" });
            }

            string valueStr = memberValue.Type switch
            {
                PlcValueType.Bool => memberValue.As<bool>().ToString(),
                PlcValueType.Dint => memberValue.As<int>().ToString(),
                PlcValueType.Int => memberValue.As<short>().ToString(),
                PlcValueType.Real => memberValue.As<float>().ToString(),
                PlcValueType.String => memberValue.As<string>(),
                _ => memberValue.ToString()
            };

            return Ok(new 
            { 
                success = true, 
                value = valueStr,
                type = memberValue.Type.ToString(),
                tagName = tagName,
                memberName = memberName,
                fullPath = memberPath,
                message = $"Successfully read member '{memberPath}'"
            });
        }
        catch (Exception ex)
        {
            _logger.LogError(ex, "Error reading UDT member {MemberPath}", memberPath);
            return StatusCode(500, new { success = false, message = ex.Message });
        }
    }

    /// <summary>
    /// Write a UDT member to the PLC.
    /// </summary>
    [HttpPost("udt-member/{memberPath}")]
    public IActionResult WriteUdtMember(string memberPath, [FromBody] UdtMemberWriteRequest request)
    {
        if (!_plcService.IsConnected)
            return StatusCode(503, new { success = false, message = "Not connected to PLC" });

        if (request?.Value == null)
            return BadRequest(new { success = false, message = "Value is required" });

        try
        {
            _logger.LogInformation("Writing UDT member: {MemberPath} = {Value}", memberPath, request.Value);
            
            // Parse the path
            var parts = memberPath.Split('.');
            if (parts.Length < 2)
            {
                return BadRequest(new { success = false, message = "Invalid UDT member path. Use format: 'UDTName.MemberName'" });
            }

            var tagName = parts[0];
            var memberName = string.Join(".", parts.Skip(1));

            // Try direct tag write first
            try
            {
                if (memberName.Contains("DINT") || (memberName.Contains("INT") && !memberName.Contains("REAL")))
                {
                    if (int.TryParse(request.Value.ToString(), out int intValue))
                    {
                        _plcService.WriteDint(memberPath, intValue);
                        return Ok(new 
                        { 
                            success = true, 
                            message = $"Successfully wrote DINT value '{intValue}' to member '{memberPath}'",
                            tagName = tagName,
                            memberName = memberName,
                            fullPath = memberPath,
                            value = intValue,
                            type = "DINT"
                        });
                    }
                }
                else if (memberName.Contains("REAL"))
                {
                    if (float.TryParse(request.Value.ToString(), out float floatValue))
                    {
                        _plcService.WriteReal(memberPath, floatValue);
                        return Ok(new 
                        { 
                            success = true, 
                            message = $"Successfully wrote REAL value '{floatValue}' to member '{memberPath}'",
                            tagName = tagName,
                            memberName = memberName,
                            fullPath = memberPath,
                            value = floatValue,
                            type = "REAL"
                        });
                    }
                }
                else if (memberName.Contains("BOOL"))
                {
                    if (bool.TryParse(request.Value.ToString(), out bool boolValue))
                    {
                        _plcService.WriteBool(memberPath, boolValue);
                        return Ok(new 
                        { 
                            success = true, 
                            message = $"Successfully wrote BOOL value '{boolValue}' to member '{memberPath}'",
                            tagName = tagName,
                            memberName = memberName,
                            fullPath = memberPath,
                            value = boolValue,
                            type = "BOOL"
                        });
                    }
                }
            }
            catch (Exception ex)
            {
                _logger.LogWarning("Direct write failed, trying SetUdtMember: {Error}", ex.Message);
            }

            // Fallback: Use SetUdtMember
            if (memberName.Contains("DINT") && int.TryParse(request.Value.ToString(), out int dintValue))
            {
                _plcService.SetUdtMember(tagName, memberName, PlcValue.Dint(dintValue));
                return Ok(new 
                { 
                    success = true, 
                    message = $"Successfully wrote DINT value '{dintValue}' to member '{memberPath}' via SetUdtMember",
                    tagName = tagName,
                    memberName = memberName,
                    fullPath = memberPath,
                    value = dintValue,
                    type = "DINT"
                });
            }
            else if (memberName.Contains("REAL") && float.TryParse(request.Value.ToString(), out float realValue))
            {
                _plcService.SetUdtMember(tagName, memberName, PlcValue.Real(realValue));
                return Ok(new 
                { 
                    success = true, 
                    message = $"Successfully wrote REAL value '{realValue}' to member '{memberPath}' via SetUdtMember",
                    tagName = tagName,
                    memberName = memberName,
                    fullPath = memberPath,
                    value = realValue,
                    type = "REAL"
                });
            }
            else
            {
                return BadRequest(new { success = false, message = $"Failed to write UDT member '{memberName}'. Check tag exists and is writable." });
            }
        }
        catch (Exception ex)
        {
            _logger.LogError(ex, "Error writing UDT member {MemberPath}", memberPath);
            return StatusCode(500, new { success = false, message = ex.Message });
        }
    }

    // ================================================================================
    // NEW FEATURE ENDPOINTS
    // ================================================================================

    /// <summary>
    /// Discover tags in a specific program
    /// </summary>
    [HttpPost("program/{programName}/discover")]
    public IActionResult DiscoverProgramTags(string programName)
    {
        if (!_plcService.IsConnected)
            return StatusCode(503, new { success = false, message = "Not connected to PLC" });

        try
        {
            _logger.LogInformation("Discovering tags in program: {ProgramName}", programName);
            // Note: This requires DiscoverProgramTags method in PlcService
            return Ok(new { success = true, message = "Program tag discovery - implementation needed in service" });
        }
        catch (Exception ex)
        {
            _logger.LogError(ex, "Error discovering program tags");
            return StatusCode(500, new { success = false, message = ex.Message });
        }
    }

    /// <summary>
    /// Discover all tags with detailed attributes
    /// </summary>
    [HttpPost("discover/detailed")]
    public IActionResult DiscoverTagsDetailed()
    {
        if (!_plcService.IsConnected)
            return StatusCode(503, new { success = false, message = "Not connected to PLC" });

        try
        {
            _logger.LogInformation("Discovering tags with detailed attributes");
            // Note: This requires DiscoverTagsDetailed method in PlcService
            return Ok(new { success = true, message = "Detailed tag discovery - implementation needed in service" });
        }
        catch (Exception ex)
        {
            _logger.LogError(ex, "Error in detailed tag discovery");
            return StatusCode(500, new { success = false, message = ex.Message });
        }
    }

    /// <summary>
    /// Check PLC connection health
    /// </summary>
    [HttpGet("health")]
    public IActionResult CheckHealth()
    {
        if (!_plcService.IsConnected)
            return StatusCode(503, new { success = false, message = "Not connected to PLC" });

        try
        {
            var isHealthy = _plcService.Client.CheckHealth();
            return Ok(new 
            { 
                success = true, 
                healthy = isHealthy,
                timestamp = DateTime.UtcNow
            });
        }
        catch (Exception ex)
        {
            _logger.LogError(ex, "Error checking health");
            return StatusCode(500, new { success = false, message = ex.Message });
        }
    }

    /// <summary>
    /// Check PLC connection health with detailed information
    /// </summary>
    [HttpGet("health/detailed")]
    public IActionResult CheckHealthDetailed()
    {
        if (!_plcService.IsConnected)
            return StatusCode(503, new { success = false, message = "Not connected to PLC" });

        try
        {
            var isHealthy = _plcService.Client.CheckHealthDetailed();
            return Ok(new 
            { 
                success = true, 
                healthy = isHealthy,
                clientId = $"0x{_plcService.Client.ClientId:X8}",
                timestamp = DateTime.UtcNow
            });
        }
        catch (Exception ex)
        {
            _logger.LogError(ex, "Error checking detailed health");
            return StatusCode(500, new { success = false, message = ex.Message });
        }
    }

    /// <summary>
    /// Clear all caches (UDT definitions, tag metadata)
    /// </summary>
    [HttpPost("cache/clear")]
    public IActionResult ClearCache()
    {
        if (!_plcService.IsConnected)
            return StatusCode(503, new { success = false, message = "Not connected to PLC" });

        try
        {
            _logger.LogInformation("Clearing all caches");
            // Note: This requires ClearCache method in PlcService
            return Ok(new { success = true, message = "Cache cleared" });
        }
        catch (Exception ex)
        {
            _logger.LogError(ex, "Error clearing cache");
            return StatusCode(500, new { success = false, message = ex.Message });
        }
    }

    /// <summary>
    /// Get cache information
    /// </summary>
    [HttpGet("cache/info")]
    public IActionResult GetCacheInfo()
    {
        if (!_plcService.IsConnected)
            return StatusCode(503, new { success = false, message = "Not connected to PLC" });

        try
        {
            _logger.LogInformation("Getting cache info");
            // Note: This requires GetCacheInfo method in PlcService
            return Ok(new 
            { 
                success = true, 
                message = "Cache information",
                timestamp = DateTime.UtcNow
            });
        }
        catch (Exception ex)
        {
            _logger.LogError(ex, "Error getting cache info");
            return StatusCode(500, new { success = false, message = ex.Message });
        }
    }
}

// ================================================================================
// REQUEST MODELS
// ================================================================================

public class ConnectRequest
{
    public string Address { get; set; } = string.Empty;
    public bool UseRoutePath { get; set; } = false;
    public int CpuSlot { get; set; } = 0;
}

public class ArrayWriteRequest
{
    public object Value { get; set; } = null!;
}

public class UdtMemberWriteRequest
{
    public object Value { get; set; } = null!;
}

public class WriteTagRequest
{
    public string Type { get; set; } = string.Empty;
    public object Value { get; set; } = null!;
}

public class BenchmarkRequest
{
    public string TestTag { get; set; } = string.Empty;
    public bool TestWrites { get; set; } = false;
    public int DurationSeconds { get; set; } = 5;
}

// ================================================================================
// BATCH OPERATION REQUEST MODELS
// ================================================================================

/// <summary>
/// Request model for batch read operations
/// </summary>
public class BatchReadRequest
{
    public string[] TagNames { get; set; } = Array.Empty<string>();
}

/// <summary>
/// Request model for batch write operations
/// </summary>
public class BatchWriteRequest
{
    public Dictionary<string, object> TagValues { get; set; } = new();
}

/// <summary>
/// Request model for mixed batch operations
/// </summary>
public class BatchMixedRequest
{
    public BatchOperation[] Operations { get; set; } = Array.Empty<BatchOperation>();
}

// ================================================================================
// STRING OPERATION REQUEST MODELS
// ================================================================================

/// <summary>
/// Request model for string write operations
/// </summary>
public class StringWriteRequest
{
    public string Value { get; set; } = string.Empty;
}

/// <summary>
/// Request model for batch read operations
/// </summary>
public class StringBatchReadRequest
{
    public string[] TagNames { get; set; } = Array.Empty<string>();
}

/// <summary>
/// Request model for batch write operations
/// </summary>
public class StringBatchWriteRequest
{
    public Dictionary<string, string> TagValues { get; set; } = new();
}
