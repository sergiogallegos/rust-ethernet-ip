using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using System;
using System.Threading.Tasks;
using System.Linq;
using System.Collections.ObjectModel;
using System.Windows;
using System.Windows.Threading;
using WpfExample.Models;
using RustEtherNetIp;
using System.Threading;
using System.Collections.Generic;
using System.Diagnostics;

namespace WpfExample.ViewModels
{
    public partial class MainViewModel : ObservableObject
    {
        private EtherNetIpClient? _plcClient;
        private DispatcherTimer? _refreshTimer;
        private bool _isRefreshing;
        private readonly object _refreshLock = new();
        private const int MAX_RETRIES = 3;
        private const int RETRY_DELAY = 1000;
        private readonly SemaphoreSlim _tagOperationLock = new(1, 1);
        private readonly Dictionary<string, TagSubscription> _activeSubscriptions = new(StringComparer.OrdinalIgnoreCase);
        private readonly Dictionary<string, EventHandler<TagValueChangedEventArgs>> _subscriptionHandlers = new(StringComparer.OrdinalIgnoreCase);
        private static readonly string[] KnownProgramTagSuffixes =
        {
            "gTestArray_DINT[0]",
            "gTestArray_DINT[5]",
            "gTestArray_REAL[0]",
            "gTestArray_BOOL[0]",
            "gTest_STRING",
            "gTestUDT.Member1_DINT",
            "gTestUDT.Member2_REAL",
            "gTestUDT.Member3_BOOL",
            "gTestUDT.Array_DINT[5]"
        };
        private static readonly (string Name, string DataType, object Value)[] SeedableTestTags =
        {
            ("gTestArray_BOOL[0]", "BOOL", true),
            ("gTestArray_INT[0]", "INT", (short)321),
            ("gTestArray_DINT[0]", "DINT", 12345),
            ("gTestArray_DINT[5]", "DINT", 54321),
            ("gTestArray_REAL[0]", "REAL", 123.45f),
            ("Program:TestProgram.gTestArray_DINT[0]", "DINT", 10000),
            ("Program:TestProgram.gTestArray_REAL[0]", "REAL", 100.0f),
            ("Program:TestProgram.gTestArray_BOOL[0]", "BOOL", true)
        };

        [ObservableProperty]
        private string plcAddress = "192.168.0.101:44818";

        [ObservableProperty]
        private bool useRoutePath = true;

        [ObservableProperty]
        private int cpuSlot = 0;

        [ObservableProperty]
        private bool isConnected;

        [ObservableProperty]
        private string connectionStatus = "Disconnected";

        [ObservableProperty]
        private string sessionId = "None";

        [ObservableProperty]
        private int readRate;

        [ObservableProperty]
        private int writeRate;

        [ObservableProperty]
        private string tagToDiscover = string.Empty;

        [ObservableProperty]
        private string tagName = string.Empty;

        [ObservableProperty]
        private string tagValue = string.Empty;

        [ObservableProperty]
        private string selectedDataType = "BOOL";

        // Batch Operations Properties
        [ObservableProperty]
        private string batchReadTags = "gTestArray_DINT[0]\ngTestArray_REAL[0]\ngTestArray_BOOL[0]\ngTestArray_INT[0]\nProgram:TestProgram.gTestArray_DINT[0]";

        [ObservableProperty]
        private string batchWriteTags = "gTestArray_DINT[5]=999\nProgram:TestProgram.gTestArray_DINT[5]=15555\ngTestArray_BOOL[0]=true\ngTestArray_REAL[0]=88.8";

        [ObservableProperty]
        private string batchResults = "";

        [ObservableProperty]
        private string batchPerformance = "";

        // Array Test Properties
        [ObservableProperty]
        private string arrayTagName = "gTestArray_DINT[5]";

        [ObservableProperty]
        private string arrayReadValue = "";

        [ObservableProperty]
        private string arrayWriteValue = "999";

        [ObservableProperty]
        private string arrayResult = "";

        // UDT Test Properties
        [ObservableProperty]
        private string udtTagName = "gTestUDT";

        [ObservableProperty]
        private string udtMemberPath = "gTestUDT.Member1_DINT";

        [ObservableProperty]
        private string udtMemberReadValue = "";

        [ObservableProperty]
        private string udtMemberWriteValue = "500";

        [ObservableProperty]
        private string udtResult = "";

        public ObservableCollection<string> DataTypes { get; } = new()
        {
            "BOOL",    // Boolean values
            "SINT",    // 8-bit signed integer (-128 to 127)
            "INT",     // 16-bit signed integer (-32,768 to 32,767)
            "DINT",    // 32-bit signed integer (-2.1B to 2.1B)
            "LINT",    // 64-bit signed integer
            "USINT",   // 8-bit unsigned integer (0 to 255)
            "UINT",    // 16-bit unsigned integer (0 to 65,535)
            "UDINT",   // 32-bit unsigned integer (0 to 4.3B)
            "ULINT",   // 64-bit unsigned integer
            "REAL",    // 32-bit IEEE 754 float
            "LREAL",   // 64-bit IEEE 754 double
            "STRING"   // Variable-length strings (up to 82 characters)
            // All Allen-Bradley data types are supported, including STRING and UDT.
        };

        public ObservableCollection<PlcTag> Tags { get; } = new();
        public ObservableCollection<string> LogMessages { get; } = new();

        // Detailed Discovery Properties
        [ObservableProperty]
        private ObservableCollection<DetailedTagInfo> detailedTags = new();

        [ObservableProperty]
        private string discoveryFilter = "";

        // Program Tags Properties
        [ObservableProperty]
        private string programName = "TestProgram";

        [ObservableProperty]
        private string programTagName = "gTestArray_DINT[0]";

        [ObservableProperty]
        private ObservableCollection<ProgramTagInfo> programTags = new();

        [ObservableProperty]
        private string programTagReadResult = "";

        // Subscriptions Properties
        [ObservableProperty]
        private string subscribeTagName = "gTestArray_DINT[0]";

        [ObservableProperty]
        private ObservableCollection<string> subscriptions = new();

        [ObservableProperty]
        private ObservableCollection<SubscriptionValue> subscriptionValues = new();

        // Health & Cache Properties
        [ObservableProperty]
        private string healthStatus = "Not Connected";

        [ObservableProperty]
        private string healthStatusColor = "#E81123";

        [ObservableProperty]
        private string healthInfo = "";

        [ObservableProperty]
        private string cacheInfo = "";

        public MainViewModel()
        {
            TagToDiscover = "gTestArray_DINT[0]";
            TagName = "gTestArray_DINT[0]";
            TagValue = "0";
            SelectedDataType = "DINT";
            InitializeTags();
            SetupTimer();
            SetupStatisticsTimer();
        }

        private void InitializeTags()
        {
            // Test tags from PLC_TEST_TAG_DEFINITIONS.md - actual tags used in testing
            // Array tags
            Tags.Add(new PlcTag("gTestArray_DINT[0]", "DINT"));
            Tags.Add(new PlcTag("gTestArray_DINT[5]", "DINT"));
            Tags.Add(new PlcTag("gTestArray_REAL[0]", "REAL"));
            Tags.Add(new PlcTag("gTestArray_BOOL[0]", "BOOL"));
            Tags.Add(new PlcTag("gTestArray_INT[0]", "INT"));
            Tags.Add(new PlcTag("gTestArray_Large[300]", "DINT"));
            
            // UDT tags
            Tags.Add(new PlcTag("gTestUDT", "UDT"));
            Tags.Add(new PlcTag("gTestUDT.Member1_DINT", "DINT"));
            Tags.Add(new PlcTag("gTestUDT.Member2_REAL", "REAL"));
            Tags.Add(new PlcTag("gTestUDT.Member3_BOOL", "BOOL"));
            Tags.Add(new PlcTag("gTestUDT.Array_DINT[5]", "DINT"));
            
            // STRING tags
            Tags.Add(new PlcTag("gTest_STRING", "STRING"));
            
            // Program-scoped tags
            Tags.Add(new PlcTag("Program:TestProgram.gTestArray_DINT[5]", "DINT"));
            
            // More program-scoped tags that are used in the real PLC validation passes
            Tags.Add(new PlcTag("Program:TestProgram.gTestArray_DINT[0]", "DINT"));
            Tags.Add(new PlcTag("Program:TestProgram.gTestArray_REAL[0]", "REAL"));
            Tags.Add(new PlcTag("Program:TestProgram.gTestArray_BOOL[0]", "BOOL"));
            Tags.Add(new PlcTag("Program:TestProgram.gTest_STRING", "STRING"));
        }

        private void SetupTimer()
        {
            _refreshTimer = new DispatcherTimer
            {
                Interval = TimeSpan.FromMilliseconds(100)
            };
            _refreshTimer.Tick += RefreshTimer_Tick;
        }

        [RelayCommand]
        private async Task ConnectAsync()
        {
            try
            {
                LogMessage("🔌 Connecting to PLC...");
                
                // Create and connect on background thread
                await Task.Run(() =>
                {
                    _plcClient = new EtherNetIpClient();

                    bool connected;
                    if (UseRoutePath)
                    {
                        var routePath = new RoutePath().AddSlot((byte)CpuSlot);
                        LogMessage($"📍 Using routed connection: CPU slot {CpuSlot}");
                        connected = _plcClient.ConnectWithRoute(PlcAddress, routePath);
                    }
                    else
                    {
                        LogMessage("📍 Using direct connection (no route path)");
                        connected = _plcClient.Connect(PlcAddress);
                    }

                    if (connected)
                    {
                        LogMessage(UseRoutePath
                            ? "✅ Connected successfully with RoutePath!"
                            : "✅ Connected successfully without RoutePath!");
                    }
                    return connected;
                }).ContinueWith(t =>
                {
                    if (t.Result)
                    {
                        IsConnected = true;
                        ConnectionStatus = "Connected";
                        SessionId = $"0x{_plcClient?.ClientId:X8}";
                        
                        _refreshTimer?.Start();
                        LogMessage($"✅ Connected! Session ID: {SessionId}");
                        LogMessage($"💡 Tip: If tag operations fail, verify the tags exist in your PLC.");
                        
                        // Test connection by trying to read a simple tag
                        _ = Task.Run(async () =>
                        {
                            try
                            {
                                var testResult = _plcClient?.ReadTagWithDetails("gTestArray_INT[0]");
                                if (testResult != null && testResult.Success)
                                {
                                    LogMessage($"✅ Connection verified: Successfully read test tag gTestArray_INT[0] = {testResult.Value}");
                                }
                                else if (testResult != null)
                                {
                                    LogMessage($"⚠️ Connection test: Could not read gTestArray_INT[0] - {testResult.ErrorMessage}");
                                }
                            }
                            catch (Exception ex)
                            {
                                LogMessage($"⚠️ Connection test failed: {ex.Message}");
                            }
                        });
                    }
                    else
                    {
                        LogMessage("❌ Connection failed!");
                        _plcClient?.Dispose();
                        _plcClient = null;
                    }
                }, TaskScheduler.FromCurrentSynchronizationContext());
            }
            catch (Exception ex)
            {
                LogMessage($"❌ Connection error: {ex.Message}");
                _plcClient?.Dispose();
                _plcClient = null;
            }
        }

        [RelayCommand]
        private void Disconnect()
        {
            try
            {
                _refreshTimer?.Stop();
                UnsubscribeAllLocal();
                
                _plcClient?.Dispose();
                _plcClient = null;
                
                IsConnected = false;
                ConnectionStatus = "Disconnected";
                SessionId = "None";
                Subscriptions.Clear();
                SubscriptionValues.Clear();
                
                // Clear tag values
                foreach (var tag in Tags)
                {
                    tag.Value = null;
                    tag.HasError = false;
                    tag.ErrorMessage = null;
                }
                
                LogMessage("📤 Disconnected from PLC");
            }
            catch (Exception ex)
            {
                LogMessage($"⚠️ Disconnect error: {ex.Message}");
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
                        LogMessage($"❌ {operationName} failed after {MAX_RETRIES} attempts: {ex.Message}");
                        throw;
                    }
                    LogMessage($"⚠️ {operationName} attempt {attempt + 1} failed: {ex.Message}");
                    await Task.Delay(RETRY_DELAY * (int)Math.Pow(2, attempt));
                }
            }
            throw new Exception($"{operationName} failed after {MAX_RETRIES} attempts");
        }

        [RelayCommand]
        private async Task DiscoverTag()
        {
            if (!IsConnected || _plcClient == null) return;

            try
            {
                LogMessage($"🔍 Discovering tag: {TagToDiscover}");
                
                await RetryOperation(async () =>
                {
                    // Run the synchronous PLC operations on a background thread
                    return await Task.Run(() =>
                    {
                        // Try to read the tag to determine its type - order matters for proper detection
                        try
                        {
                            var boolValue = _plcClient.ReadBool(TagToDiscover);
                            SelectedDataType = "BOOL";
                            TagName = TagToDiscover;
                            TagValue = boolValue.ToString();
                            LogMessage($"✅ Discovered BOOL tag: {TagToDiscover} = {boolValue}");
                            return true;
                        }
                        catch { }

                        try
                        {
                            var sintValue = _plcClient.ReadSint(TagToDiscover);
                            SelectedDataType = "SINT";
                            TagName = TagToDiscover;
                            TagValue = sintValue.ToString();
                            LogMessage($"✅ Discovered SINT tag: {TagToDiscover} = {sintValue}");
                            return true;
                        }
                        catch { }

                        try
                        {
                            var intValue = _plcClient.ReadInt(TagToDiscover);
                            SelectedDataType = "INT";
                            TagName = TagToDiscover;
                            TagValue = intValue.ToString();
                            LogMessage($"✅ Discovered INT tag: {TagToDiscover} = {intValue}");
                            return true;
                        }
                        catch { }

                        try
                        {
                            var dintValue = _plcClient.ReadDint(TagToDiscover);
                            SelectedDataType = "DINT";
                            TagName = TagToDiscover;
                            TagValue = dintValue.ToString();
                            LogMessage($"✅ Discovered DINT tag: {TagToDiscover} = {dintValue}");
                            return true;
                        }
                        catch { }

                        try
                        {
                            var lintValue = _plcClient.ReadLint(TagToDiscover);
                            SelectedDataType = "LINT";
                            TagName = TagToDiscover;
                            TagValue = lintValue.ToString();
                            LogMessage($"✅ Discovered LINT tag: {TagToDiscover} = {lintValue}");
                            return true;
                        }
                        catch { }

                        try
                        {
                            var usintValue = _plcClient.ReadUsint(TagToDiscover);
                            SelectedDataType = "USINT";
                            TagName = TagToDiscover;
                            TagValue = usintValue.ToString();
                            LogMessage($"✅ Discovered USINT tag: {TagToDiscover} = {usintValue}");
                            return true;
                        }
                        catch { }

                        try
                        {
                            var uintValue = _plcClient.ReadUint(TagToDiscover);
                            SelectedDataType = "UINT";
                            TagName = TagToDiscover;
                            TagValue = uintValue.ToString();
                            LogMessage($"✅ Discovered UINT tag: {TagToDiscover} = {uintValue}");
                            return true;
                        }
                        catch { }

                        try
                        {
                            var udintValue = _plcClient.ReadUdint(TagToDiscover);
                            SelectedDataType = "UDINT";
                            TagName = TagToDiscover;
                            TagValue = udintValue.ToString();
                            LogMessage($"✅ Discovered UDINT tag: {TagToDiscover} = {udintValue}");
                            return true;
                        }
                        catch { }

                        try
                        {
                            var ulintValue = _plcClient.ReadUlint(TagToDiscover);
                            SelectedDataType = "ULINT";
                            TagName = TagToDiscover;
                            TagValue = ulintValue.ToString();
                            LogMessage($"✅ Discovered ULINT tag: {TagToDiscover} = {ulintValue}");
                            return true;
                        }
                        catch { }

                        try
                        {
                            var realValue = _plcClient.ReadReal(TagToDiscover);
                            SelectedDataType = "REAL";
                            TagName = TagToDiscover;
                            TagValue = realValue.ToString();
                            LogMessage($"✅ Discovered REAL tag: {TagToDiscover} = {realValue}");
                            return true;
                        }
                        catch { }

                        try
                        {
                            var lrealValue = _plcClient.ReadLreal(TagToDiscover);
                            SelectedDataType = "LREAL";
                            TagName = TagToDiscover;
                            TagValue = lrealValue.ToString();
                            LogMessage($"✅ Discovered LREAL tag: {TagToDiscover} = {lrealValue}");
                            return true;
                        }
                        catch { }

                        try
                        {
                            var stringValue = _plcClient.ReadString(TagToDiscover);
                            SelectedDataType = "STRING";
                            TagName = TagToDiscover;
                            TagValue = stringValue;
                            LogMessage($"✅ Discovered STRING tag: {TagToDiscover} = \"{stringValue}\"");
                            return true;
                        }
                        catch { }

                        LogMessage($"❌ Could not determine type for tag: {TagToDiscover}");
                        return false;
                    });
                }, "Tag discovery");
            }
            catch (Exception ex)
            {
                LogMessage($"❌ Discovery error: {ex.Message}");
            }
        }

        [RelayCommand]
        private async Task ReadTag()
        {
            if (!IsConnected || _plcClient == null) return;

            try
            {
                LogMessage($"📖 Reading tag: {TagName}");
                
                await RetryOperation(async () =>
                {
                    // Try ReadTagWithDetails first for better error handling
                    try
                    {
                        var result = await Task.Run(() => _plcClient.ReadTagWithDetails(TagName));
                        if (result.Success && result.Value != null)
                        {
                            TagValue = result.Value.ToString() ?? string.Empty;
                            LogMessage($"✅ Read tag: {TagName} = {result.Value}");
                            return true;
                        }
                        else
                        {
                            LogMessage($"⚠️ ReadTagWithDetails returned Success=false: {result.ErrorMessage ?? "Unknown error"}");
                        }
                    }
                    catch (Exception ex)
                    {
                        LogMessage($"⚠️ ReadTagWithDetails exception: {ex.Message}");
                    }
                    
                    // Fallback to type-specific methods
                    return await Task.Run(() =>
                    {
                        object value = SelectedDataType switch
                        {
                            "BOOL" => _plcClient.ReadBool(TagName),
                            "SINT" => _plcClient.ReadSint(TagName),
                            "INT" => _plcClient.ReadInt(TagName),
                            "DINT" => _plcClient.ReadDint(TagName),
                            "LINT" => _plcClient.ReadLint(TagName),
                            "USINT" => _plcClient.ReadUsint(TagName),
                            "UINT" => _plcClient.ReadUint(TagName),
                            "UDINT" => _plcClient.ReadUdint(TagName),
                            "ULINT" => _plcClient.ReadUlint(TagName),
                            "REAL" => _plcClient.ReadReal(TagName),
                            "LREAL" => _plcClient.ReadLreal(TagName),
                            "STRING" => _plcClient.ReadString(TagName),
                            "UDT" => _plcClient.ReadUdt(TagName),
                            _ => throw new Exception($"Unsupported data type: {SelectedDataType}")
                        };
                        
                        TagValue = value.ToString() ?? string.Empty;
                        LogMessage($"✅ Read {SelectedDataType} tag: {TagName} = {value}");
                        return true;
                    });
                }, "Tag read");
            }
            catch (Exception ex)
            {
                LogMessage($"❌ Read error: {ex.Message}");
                LogMessage($"   Exception type: {ex.GetType().Name}");
                if (ex.InnerException != null)
                {
                    LogMessage($"   Inner exception: {ex.InnerException.Message}");
                }
            }
        }

        [RelayCommand]
        private void WriteTag()
        {
            if (!IsConnected || _plcClient == null) return;

            try
            {
                LogMessage($"✏️ Writing tag: {TagName}");
                
                switch (SelectedDataType)
                {
                    case "BOOL":
                        if (bool.TryParse(TagValue, out bool boolValue))
                        {
                            _plcClient.WriteBool(TagName, boolValue);
                            LogMessage($"✅ Wrote BOOL tag: {TagName} = {boolValue}");
                        }
                        else
                        {
                            throw new Exception("Invalid boolean value");
                        }
                        break;
                        
                    case "SINT":
                        if (sbyte.TryParse(TagValue, out sbyte sintValue))
                        {
                            _plcClient.WriteSint(TagName, sintValue);
                            LogMessage($"✅ Wrote SINT tag: {TagName} = {sintValue}");
                        }
                        else
                        {
                            throw new Exception("Invalid SINT value (-128 to 127)");
                        }
                        break;
                        
                    case "INT":
                        if (short.TryParse(TagValue, out short intValue))
                        {
                            _plcClient.WriteInt(TagName, intValue);
                            LogMessage($"✅ Wrote INT tag: {TagName} = {intValue}");
                        }
                        else
                        {
                            throw new Exception("Invalid INT value (-32,768 to 32,767)");
                        }
                        break;
                        
                    case "DINT":
                        if (int.TryParse(TagValue, out int dintValue))
                        {
                            _plcClient.WriteDint(TagName, dintValue);
                            LogMessage($"✅ Wrote DINT tag: {TagName} = {dintValue}");
                        }
                        else
                        {
                            throw new Exception("Invalid DINT value");
                        }
                        break;
                        
                    case "LINT":
                        if (long.TryParse(TagValue, out long lintValue))
                        {
                            _plcClient.WriteLint(TagName, lintValue);
                            LogMessage($"✅ Wrote LINT tag: {TagName} = {lintValue}");
                        }
                        else
                        {
                            throw new Exception("Invalid LINT value");
                        }
                        break;
                        
                    case "USINT":
                        if (byte.TryParse(TagValue, out byte usintValue))
                        {
                            _plcClient.WriteUsint(TagName, usintValue);
                            LogMessage($"✅ Wrote USINT tag: {TagName} = {usintValue}");
                        }
                        else
                        {
                            throw new Exception("Invalid USINT value (0 to 255)");
                        }
                        break;
                        
                    case "UINT":
                        if (ushort.TryParse(TagValue, out ushort uintValue))
                        {
                            _plcClient.WriteUint(TagName, uintValue);
                            LogMessage($"✅ Wrote UINT tag: {TagName} = {uintValue}");
                        }
                        else
                        {
                            throw new Exception("Invalid UINT value (0 to 65,535)");
                        }
                        break;
                        
                    case "UDINT":
                        if (uint.TryParse(TagValue, out uint udintValue))
                        {
                            _plcClient.WriteUdint(TagName, udintValue);
                            LogMessage($"✅ Wrote UDINT tag: {TagName} = {udintValue}");
                        }
                        else
                        {
                            throw new Exception("Invalid UDINT value");
                        }
                        break;
                        
                    case "ULINT":
                        if (ulong.TryParse(TagValue, out ulong ulintValue))
                        {
                            _plcClient.WriteUlint(TagName, ulintValue);
                            LogMessage($"✅ Wrote ULINT tag: {TagName} = {ulintValue}");
                        }
                        else
                        {
                            throw new Exception("Invalid ULINT value");
                        }
                        break;
                        
                    case "REAL":
                        if (float.TryParse(TagValue, out float realValue))
                        {
                            _plcClient.WriteReal(TagName, realValue);
                            LogMessage($"✅ Wrote REAL tag: {TagName} = {realValue}");
                        }
                        else
                        {
                            throw new Exception("Invalid REAL value");
                        }
                        break;
                        
                    case "LREAL":
                        if (double.TryParse(TagValue, out double lrealValue))
                        {
                            _plcClient.WriteLreal(TagName, lrealValue);
                            LogMessage($"✅ Wrote LREAL tag: {TagName} = {lrealValue}");
                        }
                        else
                        {
                            throw new Exception("Invalid LREAL value");
                        }
                        break;
                        
                    case "STRING":
                        _plcClient.WriteString(TagName, TagValue);
                        LogMessage($"✅ Wrote STRING tag: {TagName} = '{TagValue}'");
                        break;
                        
                    case "UDT":
                        LogMessage("❌ UDT writing not supported in this example");
                        break;
                        
                    default:
                        throw new Exception($"Unsupported data type: {SelectedDataType}");
                }
            }
            catch (Exception ex)
            {
                LogMessage($"❌ Write error: {ex.Message}");
            }
        }

        [RelayCommand]
        private async Task RunBenchmarkAsync()
        {
            if (!IsConnected || _plcClient == null) return;

            try
            {
                LogMessage("📊 Running benchmark...");
                
                var startTime = DateTime.Now;
                var readCount = 0;
                var writeCount = 0;
                var originalWriteValue = _plcClient.ReadDint("gTestArray_DINT[5]");
                
                // Run benchmark on background thread
                await Task.Run(() =>
                {
                    while ((DateTime.Now - startTime).TotalSeconds < 5)
                    {
                        try
                        {
                            _plcClient?.ReadDint("gTestArray_DINT[0]");
                            readCount++;
                        }
                        catch { }
                        
                        try
                        {
                            _plcClient?.WriteDint("gTestArray_DINT[5]", 999);
                            writeCount++;
                        }
                        catch { }
                    }
                });

                try
                {
                    _plcClient.WriteDint("gTestArray_DINT[5]", originalWriteValue);
                }
                catch (Exception ex)
                {
                    LogMessage($"⚠️ Benchmark restore failed for gTestArray_DINT[5]: {ex.Message}");
                }
                
                ReadRate = (int)(readCount / 5.0);
                WriteRate = (int)(writeCount / 5.0);
                
                LogMessage($"✅ Benchmark complete: {ReadRate} reads/sec, {WriteRate} writes/sec");
            }
            catch (Exception ex)
            {
                LogMessage($"❌ Benchmark error: {ex.Message}");
            }
        }

        [RelayCommand]
        private async Task CreateTestTags()
        {
            if (!IsConnected || _plcClient == null) return;

            try
            {
                LogMessage("🧪 Seeding existing test tags with known values...");
                LogMessage("ℹ️ This action does not create PLC tags. It only writes verification values to tags that already exist.");

                int successCount = 0;
                int errorCount = 0;

                foreach (var (name, type, value) in SeedableTestTags)
                {
                    try
                    {
                        await Task.Run(() =>
                        {
                            switch (type)
                            {
                                case "BOOL":
                                    _plcClient.WriteBool(name, (bool)value);
                                    break;
                                case "INT":
                                    _plcClient.WriteInt(name, (short)value);
                                    break;
                                case "DINT":
                                    _plcClient.WriteDint(name, (int)value);
                                    break;
                                case "REAL":
                                    _plcClient.WriteReal(name, (float)value);
                                    break;
                            }
                        });

                        LogMessage($"✅ Seeded {type} tag: {name} = {value}");
                        successCount++;
                    }
                    catch (Exception ex)
                    {
                        LogMessage($"❌ Failed to create {name}: {ex.Message}");
                        errorCount++;
                    }
                }

                if (successCount > 0)
                {
                    LogMessage($"✅ Seeded {successCount}/{SeedableTestTags.Length} existing test tags successfully");
                    LogMessage("🚀 The WPF sample is ready for read/write verification against the real PLC.");
                }
                else
                {
                    LogMessage($"❌ Failed to seed any test tags ({errorCount} errors)");
                }
            }
            catch (Exception ex)
            {
                LogMessage($"❌ Error seeding test tags: {ex.Message}");
            }
        }

        [RelayCommand]
        private async Task ExecuteBatchRead()
        {
            if (!IsConnected || _plcClient == null) return;

            try
            {
                var tagNames = BatchReadTags.Split('\n')
                    .Select(line => line.Trim())
                    .Where(line => !string.IsNullOrEmpty(line))
                    .ToArray();

                if (tagNames.Length == 0)
                {
                    LogMessage("❌ Please enter at least one tag name for batch read");
                    return;
                }

                LogMessage($"🚀 Executing batch read for {tagNames.Length} tags...");
                var stopwatch = System.Diagnostics.Stopwatch.StartNew();

                var results = await Task.Run(() => _plcClient.ReadTagsBatch(tagNames));
                
                stopwatch.Stop();
                var totalTime = stopwatch.ElapsedMilliseconds;

                var resultsText = new System.Text.StringBuilder();
                int successCount = 0;

                resultsText.AppendLine($"📊 Batch Read Results ({tagNames.Length} tags in {totalTime}ms):");
                resultsText.AppendLine("".PadRight(50, '='));

                foreach (var result in results)
                {
                    if (result.Value.Success)
                    {
                        resultsText.AppendLine($"✅ {result.Key}: {result.Value.Value} ({result.Value.DataType})");
                        successCount++;
                    }
                    else
                    {
                        resultsText.AppendLine($"❌ {result.Key}: {result.Value.ErrorMessage}");
                    }
                }

                BatchResults = resultsText.ToString();
                BatchPerformance = $"⏱️ Performance: {totalTime}ms total, {(double)totalTime / tagNames.Length:F1}ms avg/tag, {successCount}/{tagNames.Length} successful";
                
                LogMessage($"✅ Batch read completed: {successCount}/{tagNames.Length} successful in {totalTime}ms");
            }
            catch (Exception ex)
            {
                LogMessage($"❌ Batch read error: {ex.Message}");
                BatchResults = $"❌ Batch read failed: {ex.Message}";
                BatchPerformance = "⏱️ Performance: Error occurred";
            }
        }

        [RelayCommand]
        private async Task ExecuteBatchWrite()
        {
            if (!IsConnected || _plcClient == null) return;

            try
            {
                var lines = BatchWriteTags.Split('\n')
                    .Select(line => line.Trim())
                    .Where(line => !string.IsNullOrEmpty(line) && line.Contains('='))
                    .ToArray();

                if (lines.Length == 0)
                {
                    LogMessage("❌ Please enter tag=value pairs (one per line) for batch write");
                    return;
                }

                var tagValues = new Dictionary<string, object>();

                foreach (var line in lines)
                {
                    var parts = line.Split('=', 2);
                    if (parts.Length == 2)
                    {
                        var tagName = parts[0].Trim();
                        var valueStr = parts[1].Trim();

                        object value = InferBatchWriteValue(tagName, valueStr);

                        tagValues[tagName] = value;
                    }
                }

                LogMessage($"✏️ Executing batch write for {tagValues.Count} tags...");
                var stopwatch = System.Diagnostics.Stopwatch.StartNew();

                var results = await Task.Run(() => _plcClient.WriteTagsBatch(tagValues));
                
                stopwatch.Stop();
                var totalTime = stopwatch.ElapsedMilliseconds;

                var resultsText = new System.Text.StringBuilder();
                int successCount = 0;

                resultsText.AppendLine($"✏️ Batch Write Results ({tagValues.Count} tags in {totalTime}ms):");
                resultsText.AppendLine("".PadRight(50, '='));

                foreach (var result in results)
                {
                    var originalValue = tagValues.ContainsKey(result.Key) ? tagValues[result.Key] : "Unknown";
                    
                    if (result.Value.Success)
                    {
                        resultsText.AppendLine($"✅ {result.Key}: {originalValue} → Written successfully");
                        successCount++;
                    }
                    else
                    {
                        resultsText.AppendLine($"❌ {result.Key}: {originalValue} → {result.Value.ErrorMessage}");
                    }
                }

                BatchResults = resultsText.ToString();
                BatchPerformance = $"⏱️ Performance: {totalTime}ms total, {(double)totalTime / tagValues.Count:F1}ms avg/tag, {successCount}/{tagValues.Count} successful";
                
                LogMessage($"✅ Batch write completed: {successCount}/{tagValues.Count} successful in {totalTime}ms");
            }
            catch (Exception ex)
            {
                LogMessage($"❌ Batch write error: {ex.Message}");
                BatchResults = $"❌ Batch write failed: {ex.Message}";
                BatchPerformance = "⏱️ Performance: Error occurred";
            }
        }

        private async void RefreshTimer_Tick(object? sender, EventArgs e)
        {
            if (!IsConnected || _plcClient == null || _isRefreshing) return;

            lock (_refreshLock)
            {
                if (_isRefreshing) return;
                _isRefreshing = true;
            }

            try
            {
                // Read all tags in parallel on background thread
                await Task.Run(() =>
                {
                    Parallel.ForEach(Tags, tag =>
                    {
                        try
                        {
                            object value = tag.DataType switch
                            {
                                "BOOL" => _plcClient?.ReadBool(tag.Name) ?? false,
                                "SINT" => _plcClient?.ReadSint(tag.Name) ?? (sbyte)0,
                                "INT" => _plcClient?.ReadInt(tag.Name) ?? (short)0,
                                "DINT" => _plcClient?.ReadDint(tag.Name) ?? 0,
                                "LINT" => _plcClient?.ReadLint(tag.Name) ?? 0L,
                                "USINT" => _plcClient?.ReadUsint(tag.Name) ?? (byte)0,
                                "UINT" => _plcClient?.ReadUint(tag.Name) ?? (ushort)0,
                                "UDINT" => _plcClient?.ReadUdint(tag.Name) ?? 0U,
                                "ULINT" => _plcClient?.ReadUlint(tag.Name) ?? 0UL,
                                "REAL" => _plcClient?.ReadReal(tag.Name) ?? 0.0f,
                                "LREAL" => _plcClient?.ReadLreal(tag.Name) ?? 0.0,
                                "STRING" => _plcClient?.ReadString(tag.Name) ?? "",
                                _ => "Unknown"
                            };
                            
                            Application.Current.Dispatcher.Invoke(() => tag.UpdateValue(value));
                        }
                        catch (Exception ex)
                        {
                            Application.Current.Dispatcher.Invoke(() => tag.SetError(ex.Message));
                        }
                    });
                });
            }
            catch (Exception ex)
            {
                LogMessage($"⚠️ Refresh error: {ex.Message}");
            }
            finally
            {
                _isRefreshing = false;
            }
        }

        // Array Test Commands
        [RelayCommand]
        private async Task ReadArrayElement()
        {
            if (!IsConnected || _plcClient == null) return;

            try
            {
                LogMessage($"📖 Reading array element: {ArrayTagName}");
                ArrayResult = "Reading...";

                var value = await Task.Run(() =>
                {
                    // Try ReadTagWithDetails first for better error handling
                    try
                    {
                        var result = _plcClient.ReadTagWithDetails(ArrayTagName);
                        if (result.Success && result.Value != null)
                        {
                            return result.Value.ToString() ?? string.Empty;
                        }
                    }
                    catch { }
                    
                    // Fallback to type-specific methods based on tag name
                    if (ArrayTagName.Contains("REAL", StringComparison.OrdinalIgnoreCase))
                    {
                        return _plcClient.ReadReal(ArrayTagName).ToString();
                    }
                    else if (ArrayTagName.Contains("BOOL", StringComparison.OrdinalIgnoreCase))
                    {
                        return _plcClient.ReadBool(ArrayTagName).ToString();
                    }
                    else if (ArrayTagName.Contains("INT", StringComparison.OrdinalIgnoreCase) &&
                             !ArrayTagName.Contains("DINT", StringComparison.OrdinalIgnoreCase) &&
                             !ArrayTagName.Contains("LINT", StringComparison.OrdinalIgnoreCase) &&
                             !ArrayTagName.Contains("UINT", StringComparison.OrdinalIgnoreCase) &&
                             !ArrayTagName.Contains("USINT", StringComparison.OrdinalIgnoreCase) &&
                             !ArrayTagName.Contains("UDINT", StringComparison.OrdinalIgnoreCase) &&
                             !ArrayTagName.Contains("ULINT", StringComparison.OrdinalIgnoreCase))
                    {
                        return _plcClient.ReadInt(ArrayTagName).ToString();
                    }
                    else if (ArrayTagName.Contains("DINT", StringComparison.OrdinalIgnoreCase) || ArrayTagName.Contains("["))
                    {
                        return _plcClient.ReadDint(ArrayTagName).ToString();
                    }
                    else
                    {
                        // Default to DINT
                        return _plcClient.ReadDint(ArrayTagName).ToString();
                    }
                });

                ArrayReadValue = value;
                ArrayResult = $"✅ Success! Value: {value}";
                LogMessage($"✅ Read {ArrayTagName} = {value}");
            }
            catch (Exception ex)
            {
                ArrayResult = $"❌ Error: {ex.Message}";
                LogMessage($"❌ Read error: {ex.Message}");
            }
        }

        [RelayCommand]
        private async Task WriteArrayElement()
        {
            if (!IsConnected || _plcClient == null) return;

            try
            {
                LogMessage($"✏️ Writing array element: {ArrayTagName} = {ArrayWriteValue}");

                await Task.Run(() =>
                {
                    if (ArrayTagName.Contains("REAL", StringComparison.OrdinalIgnoreCase))
                    {
                        if (float.TryParse(ArrayWriteValue, out float floatValue))
                        {
                            _plcClient.WriteReal(ArrayTagName, floatValue);
                        }
                        else
                        {
                            throw new Exception("Invalid REAL value");
                        }
                    }
                    else if (ArrayTagName.Contains("BOOL", StringComparison.OrdinalIgnoreCase))
                    {
                        if (bool.TryParse(ArrayWriteValue, out bool boolValue))
                        {
                            _plcClient.WriteBool(ArrayTagName, boolValue);
                        }
                        else
                        {
                            throw new Exception("Invalid BOOL value");
                        }
                    }
                    else if (ArrayTagName.Contains("INT", StringComparison.OrdinalIgnoreCase) &&
                             !ArrayTagName.Contains("DINT", StringComparison.OrdinalIgnoreCase) &&
                             !ArrayTagName.Contains("LINT", StringComparison.OrdinalIgnoreCase) &&
                             !ArrayTagName.Contains("UINT", StringComparison.OrdinalIgnoreCase) &&
                             !ArrayTagName.Contains("USINT", StringComparison.OrdinalIgnoreCase) &&
                             !ArrayTagName.Contains("UDINT", StringComparison.OrdinalIgnoreCase) &&
                             !ArrayTagName.Contains("ULINT", StringComparison.OrdinalIgnoreCase))
                    {
                        if (short.TryParse(ArrayWriteValue, out short shortValue))
                        {
                            _plcClient.WriteInt(ArrayTagName, shortValue);
                        }
                        else
                        {
                            throw new Exception("Invalid INT value");
                        }
                    }
                    else if (ArrayTagName.Contains("DINT", StringComparison.OrdinalIgnoreCase) || ArrayTagName.Contains("["))
                    {
                        if (int.TryParse(ArrayWriteValue, out int intValue))
                        {
                            _plcClient.WriteDint(ArrayTagName, intValue);
                        }
                        else
                        {
                            throw new Exception("Invalid DINT value");
                        }
                    }
                    else
                    {
                        // Default to DINT
                        if (int.TryParse(ArrayWriteValue, out int defaultIntValue))
                        {
                            _plcClient.WriteDint(ArrayTagName, defaultIntValue);
                        }
                        else
                        {
                            throw new Exception("Invalid value");
                        }
                    }
                });

                ArrayResult = $"✅ Success! Wrote {ArrayWriteValue} to {ArrayTagName}";
                LogMessage($"✅ Wrote {ArrayTagName} = {ArrayWriteValue}");
            }
            catch (Exception ex)
            {
                ArrayResult = $"❌ Error: {ex.Message}";
                LogMessage($"❌ Write error: {ex.Message}");
            }
        }

        // UDT Test Commands
        [RelayCommand]
        private async Task ReadUdt()
        {
            if (!IsConnected || _plcClient == null) return;

            try
            {
                LogMessage($"📖 Reading UDT: {UdtTagName}");
                UdtResult = "Reading...";

                var value = await Task.Run(() => _plcClient.ReadUdt(UdtTagName));

                if (value.IsUdtDataFormat)
                {
                    var udtData = value.UdtData;
                    if (udtData == null)
                    {
                        UdtResult = $"❌ UDT read returned null UdtData for {UdtTagName}";
                        LogMessage($"❌ UDT read returned null UdtData for {UdtTagName}");
                        return;
                    }
                    UdtResult = $"✅ Success!\nTag: {UdtTagName}\nSymbol ID: {udtData.SymbolId}\nData Length: {udtData.Data.Length} bytes\n\n" +
                               $"⚠️ UDT is in UdtData format.\n" +
                               $"To access members, use direct tag paths:\n" +
                               $"  {UdtTagName}.Member1_DINT\n" +
                               $"  {UdtTagName}.Member2_REAL\n" +
                               $"etc.";
                    LogMessage($"✅ Read UDT {UdtTagName}: Symbol ID = {udtData.SymbolId}, Data Length = {udtData.Data.Length} bytes");
                }
                else
                {
                    var memberCount = value.UdtMembers?.Count ?? 0;
                    var memberList = value.UdtMembers != null 
                        ? string.Join("\n", value.UdtMembers.Keys.Take(10).Select(k => $"  - {k}"))
                        : "  (Use GetUdtMember to access)";
                    UdtResult = $"✅ Success!\nTag: {UdtTagName}\nUDT with {memberCount} members\n\nMembers available:\n{memberList}";
                    LogMessage($"✅ Read UDT {UdtTagName} with {memberCount} members");
                }
            }
            catch (Exception ex)
            {
                UdtResult = $"❌ Error: {ex.Message}";
                LogMessage($"❌ Read error: {ex.Message}");
            }
        }

        [RelayCommand]
        private async Task ReadUdtMember()
        {
            if (!IsConnected || _plcClient == null) return;

            try
            {
                LogMessage($"📖 Reading UDT member: {UdtMemberPath}");

                // Parse the path: "gTestUDT.Member1_DINT" -> tagName="gTestUDT", memberPath="Member1_DINT"
                var parts = UdtMemberPath.Split('.');
                if (parts.Length < 2)
                {
                    throw new Exception("Invalid UDT member path. Use format: 'UDTName.MemberName'");
                }

                var tagName = parts[0];
                var memberPath = string.Join(".", parts.Skip(1));

                // Try direct tag access first
                PlcValue? memberValue = null;
                try
                {
                    if (memberPath.Contains("REAL", StringComparison.OrdinalIgnoreCase))
                    {
                        var floatValue = await Task.Run(() => _plcClient.ReadReal(UdtMemberPath));
                        UdtMemberReadValue = floatValue.ToString();
                        UdtResult = $"✅ Success!\nUDT: {tagName}\nMember: {memberPath}\nValue: {UdtMemberReadValue}\nType: REAL";
                        LogMessage($"✅ Read {UdtMemberPath} = {UdtMemberReadValue} (REAL)");
                        return;
                    }
                    else if (memberPath.Contains("BOOL", StringComparison.OrdinalIgnoreCase))
                    {
                        var boolValue = await Task.Run(() => _plcClient.ReadBool(UdtMemberPath));
                        UdtMemberReadValue = boolValue.ToString();
                        UdtResult = $"✅ Success!\nUDT: {tagName}\nMember: {memberPath}\nValue: {UdtMemberReadValue}\nType: BOOL";
                        LogMessage($"✅ Read {UdtMemberPath} = {boolValue} (BOOL)");
                        return;
                    }
                    else if (memberPath.Contains("INT", StringComparison.OrdinalIgnoreCase) &&
                             !memberPath.Contains("DINT", StringComparison.OrdinalIgnoreCase) &&
                             !memberPath.Contains("LINT", StringComparison.OrdinalIgnoreCase) &&
                             !memberPath.Contains("UINT", StringComparison.OrdinalIgnoreCase) &&
                             !memberPath.Contains("USINT", StringComparison.OrdinalIgnoreCase) &&
                             !memberPath.Contains("UDINT", StringComparison.OrdinalIgnoreCase) &&
                             !memberPath.Contains("ULINT", StringComparison.OrdinalIgnoreCase))
                    {
                        var shortValue = await Task.Run(() => _plcClient.ReadInt(UdtMemberPath));
                        UdtMemberReadValue = shortValue.ToString();
                        UdtResult = $"✅ Success!\nUDT: {tagName}\nMember: {memberPath}\nValue: {UdtMemberReadValue}\nType: INT";
                        LogMessage($"✅ Read {UdtMemberPath} = {UdtMemberReadValue} (INT)");
                        return;
                    }
                    else if (memberPath.Contains("DINT", StringComparison.OrdinalIgnoreCase))
                    {
                        var intValue = await Task.Run(() => _plcClient.ReadDint(UdtMemberPath));
                        UdtMemberReadValue = intValue.ToString();
                        UdtResult = $"✅ Success!\nUDT: {tagName}\nMember: {memberPath}\nValue: {UdtMemberReadValue}\nType: DINT";
                        LogMessage($"✅ Read {UdtMemberPath} = {UdtMemberReadValue} (DINT)");
                        return;
                    }
                }
                catch (Exception ex)
                {
                    LogMessage($"⚠️ Direct tag access failed for '{UdtMemberPath}': {ex.Message}");
                }

                // Fallback: Read full UDT and extract member
                LogMessage($"🔧 Reading full UDT '{tagName}' to extract member '{memberPath}'...");
                var udtValue = await Task.Run(() => _plcClient.ReadUdt(tagName));
                memberValue = await Task.Run(() => _plcClient.GetUdtMember(tagName, memberPath));

                if (memberValue == null)
                {
                    throw new Exception($"Member '{memberPath}' not found in UDT '{tagName}'");
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

                UdtMemberReadValue = valueStr;
                UdtResult = $"✅ Success!\nUDT: {tagName}\nMember: {memberPath}\nValue: {valueStr}\nType: {memberValue.Type}";
                LogMessage($"✅ Read {UdtMemberPath} = {valueStr} ({memberValue.Type})");
            }
            catch (Exception ex)
            {
                UdtResult = $"❌ Error: {ex.Message}";
                LogMessage($"❌ Read error: {ex.Message}");
            }
        }

        [RelayCommand]
        private async Task WriteUdtMember()
        {
            if (!IsConnected || _plcClient == null) return;

            try
            {
                LogMessage($"✏️ Writing UDT member: {UdtMemberPath} = {UdtMemberWriteValue}");

                // Parse the path
                var parts = UdtMemberPath.Split('.');
                if (parts.Length < 2)
                {
                    throw new Exception("Invalid UDT member path. Use format: 'UDTName.MemberName'");
                }

                var tagName = parts[0];
                var memberPath = string.Join(".", parts.Skip(1));

                // Try direct tag write first
                try
                {
                    if (memberPath.Contains("REAL", StringComparison.OrdinalIgnoreCase))
                    {
                        if (float.TryParse(UdtMemberWriteValue, out float floatValue))
                        {
                            await Task.Run(() => _plcClient.WriteReal(UdtMemberPath, floatValue));
                            UdtResult = $"✅ Success! Wrote {floatValue} to {UdtMemberPath}";
                            LogMessage($"✅ Wrote {UdtMemberPath} = {floatValue}");
                            return;
                        }
                    }
                    else if (memberPath.Contains("BOOL", StringComparison.OrdinalIgnoreCase))
                    {
                        if (bool.TryParse(UdtMemberWriteValue, out bool boolValue))
                        {
                            await Task.Run(() => _plcClient.WriteBool(UdtMemberPath, boolValue));
                            UdtResult = $"✅ Success! Wrote {boolValue} to {UdtMemberPath}";
                            LogMessage($"✅ Wrote {UdtMemberPath} = {boolValue}");
                            return;
                        }
                    }
                    else if (memberPath.Contains("INT", StringComparison.OrdinalIgnoreCase) &&
                             !memberPath.Contains("DINT", StringComparison.OrdinalIgnoreCase) &&
                             !memberPath.Contains("LINT", StringComparison.OrdinalIgnoreCase) &&
                             !memberPath.Contains("UINT", StringComparison.OrdinalIgnoreCase) &&
                             !memberPath.Contains("USINT", StringComparison.OrdinalIgnoreCase) &&
                             !memberPath.Contains("UDINT", StringComparison.OrdinalIgnoreCase) &&
                             !memberPath.Contains("ULINT", StringComparison.OrdinalIgnoreCase))
                    {
                        if (short.TryParse(UdtMemberWriteValue, out short shortValue))
                        {
                            await Task.Run(() => _plcClient.WriteInt(UdtMemberPath, shortValue));
                            UdtResult = $"✅ Success! Wrote {shortValue} to {UdtMemberPath}";
                            LogMessage($"✅ Wrote {UdtMemberPath} = {shortValue}");
                            return;
                        }
                    }
                    else if (memberPath.Contains("DINT", StringComparison.OrdinalIgnoreCase))
                    {
                        if (int.TryParse(UdtMemberWriteValue, out int intValue))
                        {
                            await Task.Run(() => _plcClient.WriteDint(UdtMemberPath, intValue));
                            UdtResult = $"✅ Success! Wrote {intValue} to {UdtMemberPath}";
                            LogMessage($"✅ Wrote {UdtMemberPath} = {intValue}");
                            return;
                        }
                    }
                }
                catch (Exception ex)
                {
                    LogMessage($"⚠️ Direct write failed, trying SetUdtMember: {ex.Message}");
                }

                // Fallback: Use SetUdtMember
                if (memberPath.Contains("DINT", StringComparison.OrdinalIgnoreCase) && int.TryParse(UdtMemberWriteValue, out int dintValue))
                {
                    await Task.Run(() => _plcClient.SetUdtMember(tagName, memberPath, PlcValue.Dint(dintValue)));
                    UdtResult = $"✅ Success! Wrote {dintValue} to {UdtMemberPath}";
                    LogMessage($"✅ Wrote {UdtMemberPath} = {dintValue} via SetUdtMember");
                }
                else if (memberPath.Contains("REAL", StringComparison.OrdinalIgnoreCase) && float.TryParse(UdtMemberWriteValue, out float realValue))
                {
                    await Task.Run(() => _plcClient.SetUdtMember(tagName, memberPath, PlcValue.Real(realValue)));
                    UdtResult = $"✅ Success! Wrote {realValue} to {UdtMemberPath}";
                    LogMessage($"✅ Wrote {UdtMemberPath} = {realValue} via SetUdtMember");
                }
                else if (memberPath.Contains("BOOL", StringComparison.OrdinalIgnoreCase) && bool.TryParse(UdtMemberWriteValue, out bool boolMemberValue))
                {
                    await Task.Run(() => _plcClient.SetUdtMember(tagName, memberPath, PlcValue.Bool(boolMemberValue)));
                    UdtResult = $"✅ Success! Wrote {boolMemberValue} to {UdtMemberPath}";
                    LogMessage($"✅ Wrote {UdtMemberPath} = {boolMemberValue} via SetUdtMember");
                }
                else if (memberPath.Contains("INT", StringComparison.OrdinalIgnoreCase) &&
                         !memberPath.Contains("DINT", StringComparison.OrdinalIgnoreCase) &&
                         !memberPath.Contains("LINT", StringComparison.OrdinalIgnoreCase) &&
                         !memberPath.Contains("UINT", StringComparison.OrdinalIgnoreCase) &&
                         !memberPath.Contains("USINT", StringComparison.OrdinalIgnoreCase) &&
                         !memberPath.Contains("UDINT", StringComparison.OrdinalIgnoreCase) &&
                         !memberPath.Contains("ULINT", StringComparison.OrdinalIgnoreCase) &&
                         short.TryParse(UdtMemberWriteValue, out short intMemberValue))
                {
                    await Task.Run(() => _plcClient.SetUdtMember(tagName, memberPath, PlcValue.Int(intMemberValue)));
                    UdtResult = $"✅ Success! Wrote {intMemberValue} to {UdtMemberPath}";
                    LogMessage($"✅ Wrote {UdtMemberPath} = {intMemberValue} via SetUdtMember");
                }
                else
                {
                    throw new Exception($"Failed to write UDT member '{memberPath}'. Check tag exists and is writable.");
                }
            }
            catch (Exception ex)
            {
                UdtResult = $"❌ Error: {ex.Message}";
                LogMessage($"❌ Write error: {ex.Message}");
            }
        }

        // STRING Operations Properties
        [ObservableProperty]
        private string stringReadTagName = "gTest_STRING";

        [ObservableProperty]
        private string stringReadResult = "";

        [ObservableProperty]
        private string stringWriteTagName = "gTest_STRING";

        [ObservableProperty]
        private string stringWriteValue = "Hello PLC!";

        [ObservableProperty]
        private string stringWriteResult = "⚠️ This operation will fail due to PLC firmware limitation.";

        [ObservableProperty]
        private string logixStringExample = "";

        // Tag Group Properties
        [ObservableProperty]
        private string tagGroupTagNames = "gTestArray_DINT[0]\ngTestArray_REAL[0]\ngTestArray_BOOL[0]\nProgram:TestProgram.gTestArray_DINT[0]";

        [ObservableProperty]
        private int tagGroupUpdateRate = 500;

        [ObservableProperty]
        private bool isTagGroupActive = false;

        [ObservableProperty]
        private bool isTagGroupSuspended = false;

        [ObservableProperty]
        private string tagGroupStatus = "Status: Not Started";

        [ObservableProperty]
        private ObservableCollection<TagGroupValue> tagGroupValues = new();

        private TagGroup? _tagGroup;

        // Statistics Properties
        [ObservableProperty]
        private long statisticsReadCount;

        [ObservableProperty]
        private long statisticsWriteCount;

        [ObservableProperty]
        private long statisticsErrorCount;

        [ObservableProperty]
        private double statisticsAvgResponseTime;

        private DispatcherTimer? _statisticsTimer;

        private void LogMessage(string message)
        {
            var timestamp = DateTime.Now.ToString("HH:mm:ss");
            var logEntry = $"[{timestamp}] {message}";
            
            Application.Current.Dispatcher.Invoke(() =>
            {
                LogMessages.Insert(0, logEntry);
                
                // Keep only last 100 messages
                while (LogMessages.Count > 100)
                {
                    LogMessages.RemoveAt(LogMessages.Count - 1);
                }
            });
        }

        // STRING Operations Commands
        [RelayCommand]
        private async Task ReadString()
        {
            if (!IsConnected || _plcClient == null) return;

            try
            {
                LogMessage($"📖 Reading STRING tag: {StringReadTagName}");
                var value = await Task.Run(() => _plcClient.ReadString(StringReadTagName));
                StringReadResult = $"✅ Success! Value: \"{value}\" (Length: {value.Length})";
                LogMessage($"✅ Read STRING tag: {StringReadTagName} = \"{value}\"");
            }
            catch (Exception ex)
            {
                StringReadResult = $"❌ Error: {ex.Message}";
                LogMessage($"❌ Read error: {ex.Message}");
            }
        }

        [RelayCommand]
        private async Task WriteString()
        {
            if (!IsConnected || _plcClient == null) return;

            try
            {
                LogMessage($"✏️ Attempting to write STRING tag: {StringWriteTagName} = \"{StringWriteValue}\"");
                await Task.Run(() => _plcClient.WriteString(StringWriteTagName, StringWriteValue));
                StringWriteResult = $"✅ Success! Wrote \"{StringWriteValue}\" to {StringWriteTagName}";
                LogMessage($"✅ Wrote STRING tag: {StringWriteTagName} = \"{StringWriteValue}\"");
            }
            catch (Exception ex)
            {
                string errorMsg = ex.Message;
                if (errorMsg.Contains("0x2107", StringComparison.OrdinalIgnoreCase) ||
                    errorMsg.Contains("2107", StringComparison.OrdinalIgnoreCase) ||
                    errorMsg.Contains("0x1E", StringComparison.OrdinalIgnoreCase) ||
                    errorMsg.Contains("Embedded service error", StringComparison.OrdinalIgnoreCase))
                {
                    errorMsg = "PLC firmware limitation: direct STRING writes can fail on CompactLogix/ControlLogix " +
                              "(commonly surfaced as 0x1E or 0x2107). " +
                              "This is a PLC restriction, not a library bug. " +
                              "For STRING members in UDTs, use the LogixString helper and write the entire UDT.";
                }
                StringWriteResult = $"❌ {errorMsg}";
                LogMessage($"❌ Write error: {errorMsg}");
            }
        }

        // Tag Group Commands
        [RelayCommand]
        private async Task TagGroupStart()
        {
            if (!IsConnected || _plcClient == null) return;

            try
            {
                var tagNames = TagGroupTagNames.Split('\n')
                    .Select(line => line.Trim())
                    .Where(line => !string.IsNullOrEmpty(line))
                    .ToArray();

                if (tagNames.Length == 0)
                {
                    LogMessage("❌ Please enter at least one tag name for the tag group");
                    return;
                }

                _tagGroup?.Dispose();
                _tagGroup = new TagGroup(_plcClient)
                {
                    TagNames = tagNames,
                    UpdateRateMs = TagGroupUpdateRate
                };
                _tagGroup.DataChanged += TagGroup_DataChanged;
                _tagGroup.PollingEvent += TagGroup_PollingEvent;
                _tagGroup.Start();

                IsTagGroupActive = true;
                IsTagGroupSuspended = false;
                TagGroupStatus = $"Status: Active (Polling {tagNames.Length} tags every {TagGroupUpdateRate}ms)";
                LogMessage($"🔄 TagGroup started: {tagNames.Length} tags, {TagGroupUpdateRate}ms update rate");
            }
            catch (Exception ex)
            {
                LogMessage($"❌ TagGroup start error: {ex.Message}");
            }
        }

        [RelayCommand]
        private void TagGroupStop()
        {
            _tagGroup?.Stop();
            if (_tagGroup != null)
            {
                _tagGroup.DataChanged -= TagGroup_DataChanged;
                _tagGroup.PollingEvent -= TagGroup_PollingEvent;
            }
            IsTagGroupActive = false;
            IsTagGroupSuspended = false;
            TagGroupStatus = "Status: Stopped";
            LogMessage("🔄 TagGroup stopped");
        }

        [RelayCommand]
        private void TagGroupSuspend()
        {
            _tagGroup?.Suspend();
            IsTagGroupSuspended = true;
            TagGroupStatus = "Status: Suspended";
            LogMessage("🔄 TagGroup suspended");
        }

        [RelayCommand]
        private void TagGroupResume()
        {
            _tagGroup?.Resume();
            IsTagGroupSuspended = false;
            TagGroupStatus = "Status: Active";
            LogMessage("🔄 TagGroup resumed");
        }

        private void TagGroup_DataChanged(object? sender, GroupDataChangedEventArgs e)
        {
            Application.Current.Dispatcher.Invoke(() =>
            {
                TagGroupValues.Clear();
                foreach (var kvp in e.AllValues)
                {
                    TagGroupValues.Add(new TagGroupValue
                    {
                        TagName = kvp.Key,
                        Value = kvp.Value?.ToString() ?? "N/A",
                        Type = kvp.Value?.Type.ToString() ?? "N/A",
                        LastUpdated = DateTime.Now
                    });
                }

                if (e.ChangedTags.Length > 0)
                {
                    LogMessage($"🔄 TagGroup: {e.ChangedTags.Length} tag(s) changed: {string.Join(", ", e.ChangedTags)}");
                }
            });
        }

        private void TagGroup_PollingEvent(object? sender, TagGroupPollingEventArgs e)
        {
            Application.Current.Dispatcher.Invoke(() =>
            {
                if (e.Kind == TagGroupEventKind.Data)
                {
                    return;
                }

                if (e.Kind == TagGroupEventKind.PartialError)
                {
                    var count = e.Errors?.Count ?? 0;
                    TagGroupStatus = $"Status: Active (Partial errors: {count})";
                    if (count > 0)
                    {
                        var preview = string.Join(", ", e.Errors!.Take(3).Select(kvp => kvp.Key));
                        LogMessage($"⚠️ TagGroup partial error on {count} tag(s): {preview}");
                    }
                    return;
                }

                var category = e.Failure?.Category.ToString() ?? "Unknown";
                TagGroupStatus = $"Status: ReadFailure ({category})";
                LogMessage($"❌ TagGroup read failure: {e.ErrorMessage ?? "Unknown error"}");
            });
        }

        // Statistics Commands
        [RelayCommand]
        private void ResetStatistics()
        {
            if (_plcClient == null) return;
            _plcClient.Statistics.Reset();
            UpdateStatistics();
            LogMessage("📊 Statistics reset");
        }

        private void UpdateStatistics()
        {
            if (_plcClient == null) return;

            try
            {
                var stats = _plcClient.Statistics;
                StatisticsReadCount = stats.ReadCount;
                StatisticsWriteCount = stats.WriteCount;
                StatisticsErrorCount = stats.ErrorCount;
                StatisticsAvgResponseTime = stats.AverageResponseTime.TotalMilliseconds;
            }
            catch { }
        }

        private void SetupStatisticsTimer()
        {
            _statisticsTimer = new DispatcherTimer
            {
                Interval = TimeSpan.FromSeconds(1)
            };
            _statisticsTimer.Tick += (s, e) => UpdateStatistics();
        }

        partial void OnIsConnectedChanged(bool value)
        {
            if (value)
            {
                _statisticsTimer?.Start();
                LogixStringExample = "Example code:\n" +
                                    "var logixString = new LogixString();\n" +
                                    "logixString.SetString(\"Hello\");\n" +
                                    "client.WriteStringAsUdt(\"gTestUDT.Member5_String\", logixString);\n" +
                                    "\nNote: Even this may fail if the STRING is standalone.";
            }
            else
            {
                _statisticsTimer?.Stop();
                _tagGroup?.Stop();
                if (_tagGroup != null)
                {
                    _tagGroup.DataChanged -= TagGroup_DataChanged;
                    _tagGroup.PollingEvent -= TagGroup_PollingEvent;
                }
                _tagGroup?.Dispose();
                _tagGroup = null;
                IsTagGroupActive = false;
                IsTagGroupSuspended = false;
                TagGroupStatus = "Status: Not Started";
            }
        }

        [RelayCommand]
        private async Task DiscoverDetailed()
        {
            if (!IsConnected || _plcClient == null) return;

            try
            {
                LogMessage("🔍 Starting detailed tag discovery...");
                DetailedTags.Clear();

                await Task.Run(() =>
                {
                    try
                    {
                        _plcClient.DiscoverTags();
                        LogMessage("✅ Basic discovery completed, gathering metadata...");
                    }
                    catch (Exception ex)
                    {
                        LogMessage($"⚠️ Basic discovery failed: {ex.Message}");
                        return;
                    }
                });

                // Get metadata for known test tags
                var testTags = new[]
                {
                    "gTestArray_DINT", "gTestArray_REAL", "gTestArray_BOOL", "gTestArray_INT",
                    "gTestUDT", "gTest_STRING",
                    "Program:TestProgram.gTestArray_DINT",
                    "Program:TestProgram.gTestArray_REAL",
                    "Program:TestProgram.gTestArray_BOOL",
                    "Program:TestProgram.gTest_STRING"
                };

                var discovered = new List<DetailedTagInfo>();
                foreach (var tagName in testTags)
                {
                    try
                    {
                        var metadata = _plcClient.GetTagMetadata(tagName);
                        // TagMetadata is a struct, check if it's valid
                        if (metadata.DataType != 0)
                        {
                            discovered.Add(new DetailedTagInfo
                            {
                                Name = tagName,
                                Type = metadata.DataType.ToString(),
                                TypeCode = $"0x{metadata.DataType:X04}",
                                Size = metadata.ArraySize > 0 ? metadata.ArraySize.ToString() : "N/A",
                                Scope = metadata.Scope.ToString(),
                                Readable = true,  // Assume readable if metadata exists
                                Writable = true  // Assume writable if metadata exists
                            });
                        }
                    }
                    catch { }
                }

                DetailedTags = new ObservableCollection<DetailedTagInfo>(discovered);
                LogMessage($"✅ Detailed discovery completed: {discovered.Count} tags found");
            }
            catch (Exception ex)
            {
                LogMessage($"❌ Detailed discovery error: {ex.Message}");
            }
        }

        [RelayCommand]
        private async Task DiscoverProgramTags()
        {
            if (!IsConnected || _plcClient == null) return;

            try
            {
                var discovered = new List<ProgramTagInfo>();
                var prefix = $"Program:{ProgramName}.";

                await Task.Run(() =>
                {
                    foreach (var suffix in KnownProgramTagSuffixes)
                    {
                        var fullTag = prefix + suffix;
                        try
                        {
                            var metadata = _plcClient.GetTagMetadata(fullTag);
                            discovered.Add(new ProgramTagInfo
                            {
                                Name = fullTag,
                                Type = FormatDataTypeCode(metadata.DataType),
                                Size = metadata.ArraySize > 0 ? metadata.ArraySize.ToString() : "Scalar",
                                Scope = $"Program ({metadata.Scope})",
                                LastReadValue = "Not read yet",
                                LastReadStatus = "Discovered"
                            });
                        }
                        catch
                        {
                            // Ignore missing tags in the candidate list.
                        }
                    }
                });

                ProgramTags = new ObservableCollection<ProgramTagInfo>(discovered);
                if (discovered.Count > 0)
                {
                    ProgramTagName = ExtractProgramSuffix(discovered[0].Name);
                }

                LogMessage(discovered.Count > 0
                    ? $"✅ Discovered {discovered.Count} program-scoped test tags under Program:{ProgramName}"
                    : $"⚠️ No known program-scoped test tags were found under Program:{ProgramName}");
            }
            catch (Exception ex)
            {
                LogMessage($"❌ Program tag discovery error: {ex.Message}");
            }
        }

        [RelayCommand]
        private async Task ReadProgramTag()
        {
            if (!IsConnected || _plcClient == null || string.IsNullOrEmpty(ProgramTagName)) return;

            try
            {
                var fullTag = NormalizeProgramTagName(ProgramTagName);
                var result = await Task.Run(() => _plcClient.ReadTagWithDetails(fullTag));
                var programTagRow = ProgramTags.FirstOrDefault(t => t.Name.Equals(fullTag, StringComparison.OrdinalIgnoreCase));
                if (result.Success)
                {
                    var valueText = result.Value?.ToString() ?? "<null>";
                    ProgramTagReadResult = $"✅ {fullTag} = {valueText}";
                    if (programTagRow != null)
                    {
                        programTagRow.LastReadValue = valueText;
                        programTagRow.LastReadStatus = "Read OK";
                    }
                    LogMessage($"✅ Program tag read: {fullTag} = {valueText}");
                }
                else
                {
                    ProgramTagReadResult = $"❌ {fullTag}: {result.ErrorMessage}";
                    if (programTagRow != null)
                    {
                        programTagRow.LastReadValue = "<error>";
                        programTagRow.LastReadStatus = result.ErrorMessage ?? "Read failed";
                    }
                    LogMessage($"❌ Program tag read failed: {fullTag} - {result.ErrorMessage}");
                }
            }
            catch (Exception ex)
            {
                ProgramTagReadResult = $"❌ {NormalizeProgramTagName(ProgramTagName)}: {ex.Message}";
                LogMessage($"❌ Program tag read error: {ex.Message}");
            }
        }

        [RelayCommand]
        private async Task Subscribe()
        {
            if (!IsConnected || _plcClient == null || string.IsNullOrEmpty(SubscribeTagName)) return;

            var tagName = SubscribeTagName.Trim();
            if (_activeSubscriptions.ContainsKey(tagName))
            {
                LogMessage($"⚠️ Subscription already active for {tagName}");
                return;
            }

            try
            {
                var subscription = await Task.Run(() => _plcClient.SubscribeToTag(tagName, new SubscriptionOptions
                {
                    PollIntervalMs = 500
                }));

                EventHandler<TagValueChangedEventArgs> handler = (_, e) =>
                {
                    Application.Current.Dispatcher.Invoke(() =>
                    {
                        var existing = SubscriptionValues.FirstOrDefault(v => v.TagName.Equals(e.TagName, StringComparison.OrdinalIgnoreCase));
                        if (existing == null)
                        {
                            SubscriptionValues.Add(new SubscriptionValue
                            {
                                TagName = e.TagName,
                                Value = e.NewValue?.ToString() ?? "<null>",
                                Updated = DateTime.Now
                            });
                        }
                        else
                        {
                            existing.Value = e.NewValue?.ToString() ?? "<null>";
                            existing.Updated = DateTime.Now;
                        }
                    });
                };

                subscription.ValueChanged += handler;
                _activeSubscriptions[tagName] = subscription;
                _subscriptionHandlers[tagName] = handler;
                Subscriptions.Add(tagName);
                UpsertSubscriptionValue(tagName, subscription.Value);
                LogMessage($"✅ Subscription started for {tagName}");
            }
            catch (Exception ex)
            {
                LogMessage($"❌ Subscription error for {tagName}: {ex.Message}");
            }
        }

        [RelayCommand]
        private async Task Unsubscribe()
        {
            if (!IsConnected || _plcClient == null) return;

            var tagName = SubscribeTagName.Trim();
            if (string.IsNullOrWhiteSpace(tagName))
            {
                LogMessage("⚠️ Enter the tag name to unsubscribe.");
                return;
            }

            await Task.Run(() =>
            {
                if (_activeSubscriptions.TryGetValue(tagName, out var subscription) &&
                    _subscriptionHandlers.TryGetValue(tagName, out var handler))
                {
                    subscription.ValueChanged -= handler;
                }

                _plcClient.UnsubscribeFromTag(tagName);
            });

            _activeSubscriptions.Remove(tagName);
            _subscriptionHandlers.Remove(tagName);

            var existingName = Subscriptions.FirstOrDefault(s => s.Equals(tagName, StringComparison.OrdinalIgnoreCase));
            if (existingName != null)
            {
                Subscriptions.Remove(existingName);
            }

            var existingValue = SubscriptionValues.FirstOrDefault(v => v.TagName.Equals(tagName, StringComparison.OrdinalIgnoreCase));
            if (existingValue != null)
            {
                SubscriptionValues.Remove(existingValue);
            }

            LogMessage($"✅ Unsubscribed from {tagName}");
        }

        [RelayCommand]
        private async Task CheckHealth()
        {
            if (!IsConnected || _plcClient == null) return;
            try
            {
                var isHealthy = _plcClient.CheckHealth();
                HealthStatus = isHealthy ? "✅ Healthy" : "❌ Unhealthy";
                HealthStatusColor = isHealthy ? "#107C10" : "#E81123";
                HealthInfo = $"Health Check: {(isHealthy ? "PASSED" : "FAILED")}\n" +
                            $"Timestamp: {DateTime.Now:yyyy-MM-dd HH:mm:ss}\n" +
                            $"Connection: Active";
                LogMessage($"Health check: {(isHealthy ? "Healthy" : "Unhealthy")}");
            }
            catch (Exception ex)
            {
                LogMessage($"❌ Health check error: {ex.Message}");
            }
        }

        [RelayCommand]
        private async Task CheckHealthDetailed()
        {
            if (!IsConnected || _plcClient == null) return;
            try
            {
                var isHealthy = _plcClient.CheckHealthDetailed();
                HealthStatus = isHealthy ? "✅ Healthy (Detailed)" : "❌ Unhealthy (Detailed)";
                HealthStatusColor = isHealthy ? "#107C10" : "#E81123";
                HealthInfo = $"Detailed Health Check: {(isHealthy ? "PASSED" : "FAILED")}\n" +
                            $"Timestamp: {DateTime.Now:yyyy-MM-dd HH:mm:ss}\n" +
                            $"Connection: Active\n" +
                            $"Client ID: 0x{_plcClient.ClientId:X8}";
                LogMessage($"Detailed health check: {(isHealthy ? "Healthy" : "Unhealthy")}");
            }
            catch (Exception ex)
            {
                LogMessage($"❌ Detailed health check error: {ex.Message}");
            }
        }

        [RelayCommand]
        private async Task ClearCache()
        {
            if (!IsConnected || _plcClient == null) return;
            CacheInfo = "Wrapper cache management is not currently exposed as a public C# API.\n" +
                       "Use 'Discover All Tags (Detailed)' or reconnect to rebuild cached metadata.";
            LogMessage("ℹ️ Clear cache is not exposed in the C# wrapper. Reconnect or rediscover tags to refresh metadata.");
            await RefreshCache();
        }

        [RelayCommand]
        private async Task RefreshCache()
        {
            if (!IsConnected || _plcClient == null) return;
            try
            {
                int discoveredMetadata = 0;
                foreach (var tag in Tags.Take(12))
                {
                    try
                    {
                        _plcClient.GetTagMetadata(tag.Name);
                        discoveredMetadata++;
                    }
                    catch
                    {
                        // Ignore individual tag misses in the summary.
                    }
                }

                CacheInfo = $"Cache Information\n" +
                           $"Timestamp: {DateTime.Now:yyyy-MM-dd HH:mm:ss}\n" +
                           $"Connected: yes\n" +
                           $"Known sample tags with metadata available: {discoveredMetadata}\n" +
                           $"Note: explicit cache clear/list APIs are not currently exposed by the C# wrapper.";
                LogMessage($"ℹ️ Refreshed wrapper cache summary: {discoveredMetadata} sample tags have cached metadata available.");
            }
            catch (Exception ex)
            {
                CacheInfo = $"Cache refresh failed: {ex.Message}";
                LogMessage($"❌ Cache refresh error: {ex.Message}");
            }
        }

        private void UnsubscribeAllLocal()
        {
            if (_plcClient != null)
            {
                try
                {
                    _plcClient.UnsubscribeFromAllTags();
                }
                catch
                {
                    // Best-effort cleanup while disconnecting.
                }
            }

            foreach (var entry in _activeSubscriptions)
            {
                if (_subscriptionHandlers.TryGetValue(entry.Key, out var handler))
                {
                    entry.Value.ValueChanged -= handler;
                }
            }

            _activeSubscriptions.Clear();
            _subscriptionHandlers.Clear();
        }

        private void UpsertSubscriptionValue(string tagName, object? value)
        {
            var existing = SubscriptionValues.FirstOrDefault(v => v.TagName.Equals(tagName, StringComparison.OrdinalIgnoreCase));
            if (existing == null)
            {
                SubscriptionValues.Add(new SubscriptionValue
                {
                    TagName = tagName,
                    Value = value?.ToString() ?? "<null>",
                    Updated = DateTime.Now
                });
                return;
            }

            existing.Value = value?.ToString() ?? "<null>";
            existing.Updated = DateTime.Now;
        }

        private string NormalizeProgramTagName(string tagName)
        {
            if (tagName.StartsWith("Program:", StringComparison.OrdinalIgnoreCase))
            {
                return tagName;
            }

            return $"Program:{ProgramName}.{tagName}";
        }

        private static string ExtractProgramSuffix(string fullTagName)
        {
            var marker = ".";
            var index = fullTagName.IndexOf(marker, StringComparison.Ordinal);
            return index >= 0 && index + 1 < fullTagName.Length
                ? fullTagName[(index + 1)..]
                : fullTagName;
        }

        private static string FormatDataTypeCode(int dataTypeCode)
        {
            return dataTypeCode switch
            {
                0x00C1 => "BOOL",
                0x00C2 => "SINT",
                0x00C3 => "INT",
                0x00C4 => "DINT",
                0x00C5 => "LINT",
                0x00C6 => "USINT",
                0x00C7 => "UINT",
                0x00C8 => "UDINT",
                0x00C9 => "ULINT",
                0x00CA => "REAL",
                0x00CB => "LREAL",
                0x00D0 => "STRING",
                _ => $"0x{dataTypeCode:X4}"
            };
        }

        private static object InferBatchWriteValue(string tagName, string valueStr)
        {
            if (bool.TryParse(valueStr, out bool boolVal))
            {
                return boolVal;
            }

            if (tagName.Contains("LREAL", StringComparison.OrdinalIgnoreCase) && double.TryParse(valueStr, out double doubleVal))
            {
                return doubleVal;
            }

            if (tagName.Contains("REAL", StringComparison.OrdinalIgnoreCase) && float.TryParse(valueStr, out float floatVal))
            {
                return floatVal;
            }

            if (tagName.Contains("USINT", StringComparison.OrdinalIgnoreCase) && byte.TryParse(valueStr, out byte usintVal))
            {
                return usintVal;
            }

            if (tagName.Contains("SINT", StringComparison.OrdinalIgnoreCase) && sbyte.TryParse(valueStr, out sbyte sintVal))
            {
                return sintVal;
            }

            if (tagName.Contains("UINT", StringComparison.OrdinalIgnoreCase) &&
                !tagName.Contains("UDINT", StringComparison.OrdinalIgnoreCase) &&
                ushort.TryParse(valueStr, out ushort uint16Val))
            {
                return uint16Val;
            }

            if (tagName.Contains("INT", StringComparison.OrdinalIgnoreCase) &&
                !tagName.Contains("DINT", StringComparison.OrdinalIgnoreCase) &&
                !tagName.Contains("LINT", StringComparison.OrdinalIgnoreCase) &&
                !tagName.Contains("UINT", StringComparison.OrdinalIgnoreCase) &&
                !tagName.Contains("USINT", StringComparison.OrdinalIgnoreCase) &&
                !tagName.Contains("UDINT", StringComparison.OrdinalIgnoreCase) &&
                !tagName.Contains("ULINT", StringComparison.OrdinalIgnoreCase) &&
                short.TryParse(valueStr, out short int16Val))
            {
                return int16Val;
            }

            if (tagName.Contains("ULINT", StringComparison.OrdinalIgnoreCase) && ulong.TryParse(valueStr, out ulong ulintVal))
            {
                return ulintVal;
            }

            if (tagName.Contains("LINT", StringComparison.OrdinalIgnoreCase) && long.TryParse(valueStr, out long lintVal))
            {
                return lintVal;
            }

            if (tagName.Contains("UDINT", StringComparison.OrdinalIgnoreCase) && uint.TryParse(valueStr, out uint udintVal))
            {
                return udintVal;
            }

            if ((tagName.Contains("DINT", StringComparison.OrdinalIgnoreCase) || tagName.Contains("[", StringComparison.OrdinalIgnoreCase)) &&
                int.TryParse(valueStr, out int intVal))
            {
                return intVal;
            }

            if (float.TryParse(valueStr, out float fallbackFloat))
            {
                return fallbackFloat;
            }

            if (int.TryParse(valueStr, out int fallbackInt))
            {
                return fallbackInt;
            }

            return valueStr;
        }
    }

    public class TagGroupValue
    {
        public string TagName { get; set; } = string.Empty;
        public string Value { get; set; } = string.Empty;
        public string Type { get; set; } = string.Empty;
        public DateTime LastUpdated { get; set; }
    }

    public class DetailedTagInfo
    {
        public string Name { get; set; } = string.Empty;
        public string Type { get; set; } = string.Empty;
        public string TypeCode { get; set; } = string.Empty;
        public string Size { get; set; } = string.Empty;
        public string Scope { get; set; } = string.Empty;
        public bool Readable { get; set; }
        public bool Writable { get; set; }
    }

    public partial class ProgramTagInfo : ObservableObject
    {
        [ObservableProperty]
        private string name = string.Empty;

        [ObservableProperty]
        private string type = string.Empty;

        [ObservableProperty]
        private string size = string.Empty;

        [ObservableProperty]
        private string scope = string.Empty;

        [ObservableProperty]
        private string lastReadValue = "Not read yet";

        [ObservableProperty]
        private string lastReadStatus = "Not read";
    }

    public partial class SubscriptionValue : ObservableObject
    {
        [ObservableProperty]
        private string tagName = string.Empty;

        [ObservableProperty]
        private string value = string.Empty;

        [ObservableProperty]
        private DateTime updated;
    }
}
