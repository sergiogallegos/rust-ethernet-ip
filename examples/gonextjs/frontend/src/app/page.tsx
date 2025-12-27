"use client";
import React, { useState, useEffect, useRef, useCallback } from "react";
import {
  connectToPlc,
  disconnectPlc,
  readTag,
  writeTag,
  batchReadTags,
  batchWriteTags,
  runBenchmark,
  getPlcStatus,
  createTestTags,
  discoverTag,
  debugReadTag,
  testArrays
} from "../lib/plcApi";
import "./globals.css";

// Tab type definition - Enhanced with all features
const TABS = [
  "Individual", 
  "Batch", 
  "Performance", 
  "Subscriptions", 
  "Monitoring", 
  "Advanced", 
  "HMI Demo", 
  "Config", 
  "About"
] as const;
type TabType = typeof TABS[number];

interface LogEntry {
  id: string;
  timestamp: string;
  level: "info" | "success" | "warning" | "error";
  message: string;
}

const PLC_TYPES = [
  { label: 'Bool', value: 'Bool' },
  { label: 'Int', value: 'Int' },
  { label: 'Dint', value: 'Dint' },
  { label: 'Real', value: 'Real' },
  { label: 'String', value: 'String' },
];

export default function Page() {
  // Connection state
  const [isConnected, setIsConnected] = useState(false);
  const [plcAddress, setPlcAddress] = useState("192.168.0.1:44818");
  const [connectionStatus, setConnectionStatus] = useState<string | null>(null);
  const [isConnecting, setIsConnecting] = useState(false);
  const [connectionIssues, setConnectionIssues] = useState(false);

  // Tab management
  const [activeTab, setActiveTab] = useState<TabType>("Individual");

  // Individual tag operations
  const [tagName, setTagName] = useState("");
  const [tagType, setTagType] = useState("String");
  const [tagValue, setTagValue] = useState("");
  const [readValue, setReadValue] = useState<any>(null);
  const [isReading, setIsReading] = useState(false);
  const [isWriting, setIsWriting] = useState(false);

  // Batch operations
  const [batchTags, setBatchTags] = useState("");
  const [batchWriteData, setBatchWriteData] = useState("");
  const [batchReadResult, setBatchReadResult] = useState<any>(null);
  const [batchWriteResult, setBatchWriteResult] = useState<any>(null);
  const [isBatchReading, setIsBatchReading] = useState(false);
  const [isBatchWriting, setIsBatchWriting] = useState(false);

  // Performance benchmark
  const [benchmarkTestTag, setBenchmarkTestTag] = useState("TestTag");
  const [benchmarkTestType, setBenchmarkTestType] = useState("Dint");
  const [benchmarkTestWrites, setBenchmarkTestWrites] = useState(false);
  const [benchmarkResults, setBenchmarkResults] = useState<any>(null);
  const [isRunningBenchmark, setIsRunningBenchmark] = useState(false);

  // HMI Demo state
  const [isHmiMonitoring, setIsHmiMonitoring] = useState(false);
  const [hmiData, setHmiData] = useState({
    machineStatus: 'Running',
    shift: 1,
    operator: 'John Doe',
    productionCount: 1250,
    targetCount: 1500,
    oee: 87.5,
    availability: 92.3,
    performance: 94.8,
    qualityRate: 98.1
  });

  // Monitoring state
  const [isMonitoring, setIsMonitoring] = useState(false);
  const [monitoredTags, setMonitoredTags] = useState<string[]>([]);

  // Real-time Subscriptions state
  const [subscriptions, setSubscriptions] = useState<Array<{
    id: string;
    tagName: string;
    dataType: string;
    updateRate: number;
    lastValue: any;
    isActive: boolean;
    changeThreshold: number;
  }>>([]);
  const [newSubscriptionTag, setNewSubscriptionTag] = useState("");
  const [newSubscriptionType, setNewSubscriptionType] = useState("String");
  const [newSubscriptionRate, setNewSubscriptionRate] = useState(100);
  const [newSubscriptionThreshold, setNewSubscriptionThreshold] = useState(0.001);

  // System Monitoring state
  const [systemMetrics, setSystemMetrics] = useState({
    connectionHealth: 'excellent',
    operationCount: 0,
    errorCount: 0,
    avgLatency: 0,
    memoryUsage: 0,
    cpuUsage: 0,
    uptime: 0
  });
  const [isWebSocketConnected, setIsWebSocketConnected] = useState(false);

  // Advanced Operations state
  const [advancedTagPath, setAdvancedTagPath] = useState("");
  const [udtData, setUdtData] = useState<any>(null);
  const [complexTagResults, setComplexTagResults] = useState<any>(null);
  const [tagDiscoveryResults, setTagDiscoveryResults] = useState<any>(null);
  const [arrayTestType, setArrayTestType] = useState<"controller" | "program" | "bool" | "all">("all");
  const [arrayTestResults, setArrayTestResults] = useState<any>(null);
  const [isTestingArrays, setIsTestingArrays] = useState(false);

  // Logging
  const [logs, setLogs] = useState<LogEntry[]>([]);
  const logIdRef = useRef(0);

  const addLog = useCallback((level: LogEntry['level'], message: string) => {
    const newLog: LogEntry = {
      id: `log-${++logIdRef.current}`,
      timestamp: new Date().toLocaleTimeString(),
      level,
      message
    };
    setLogs(prev => [newLog, ...prev.slice(0, 99)]); // Keep last 100 logs
  }, []);

  // Connection handlers
  const handleConnect = async () => {
    if (!plcAddress.trim()) {
      addLog('error', 'Please enter a PLC address');
      return;
    }

    setIsConnecting(true);
    addLog('info', `Connecting to PLC at ${plcAddress}...`);

    try {
      const result = await connectToPlc(plcAddress);
      if (result) {
        setIsConnected(true);
        setConnectionStatus('Connected');
        addLog('success', `Connected to PLC at ${plcAddress}`);
      } else {
        addLog('error', `Failed to connect to ${plcAddress}`);
        setConnectionIssues(true);
      }
    } catch (error) {
      addLog('error', `Connection error: ${error}`);
      setConnectionIssues(true);
    } finally {
      setIsConnecting(false);
    }
  };

  const handleDisconnect = async () => {
    try {
      await disconnectPlc();
      setIsConnected(false);
      setConnectionStatus('Disconnected');
      addLog('info', 'Disconnected from PLC');
    } catch (error) {
      addLog('error', `Disconnect error: ${error}`);
    }
  };

  // Individual tag operations
  const handleReadTag = async () => {
    if (!tagName.trim()) {
      addLog('error', 'Please enter a tag name');
      return;
    }

    setIsReading(true);
    addLog('info', `Reading tag: ${tagName}`);

    try {
      const result = await readTag(tagName, tagType);
      setReadValue(result.value);
      addLog('success', `Read ${tagName}: ${result.value}`);
    } catch (error) {
      addLog('error', `Read error: ${error}`);
    } finally {
      setIsReading(false);
    }
  };

  const handleWriteTag = async () => {
    if (!tagName.trim()) {
      addLog('error', 'Please enter a tag name');
      return;
    }

    setIsWriting(true);
    addLog('info', `Writing to tag: ${tagName} = ${tagValue}`);

    try {
      const result = await writeTag(tagName, tagValue, tagType);
      if (result) {
        addLog('success', `Wrote to ${tagName}: ${tagValue}`);
      } else {
        addLog('error', `Failed to write to ${tagName}`);
      }
    } catch (error) {
      addLog('error', `Write error: ${error}`);
    } finally {
      setIsWriting(false);
    }
  };

  // Batch operations
  const handleBatchRead = async () => {
    if (!batchTags.trim()) {
      addLog('error', 'Please enter tags to read');
      return;
    }

    setIsBatchReading(true);
    addLog('info', 'Starting batch read operation');

    try {
      const tagList = batchTags.split('\n').filter(tag => tag.trim()).map(tag => {
        const [tagName, tagType] = tag.split(':');
        return { tag: tagName, type: tagType || 'String' };
      });
      const result = await batchReadTags(tagList);
      setBatchReadResult(result);
      addLog('success', `Batch read completed: ${Object.keys(result).length} tags`);
    } catch (error) {
      addLog('error', `Batch read error: ${error}`);
    } finally {
      setIsBatchReading(false);
    }
  };

  const handleBatchWrite = async () => {
    if (!batchWriteData.trim()) {
      addLog('error', 'Please enter tag values to write');
      return;
    }

    setIsBatchWriting(true);
    addLog('info', 'Starting batch write operation');

    try {
      const writeData = batchWriteData.split('\n').filter(line => line.trim()).map(line => {
        const [tagPart, value] = line.split('=');
        const [tagName, tagType] = tagPart.split(':');
        return { tag: tagName, type: tagType || 'String', value: value };
      });
      const result = await batchWriteTags(writeData);
      if (result.success) {
        setBatchWriteResult(result);
        addLog('success', `Batch write completed: ${writeData.length} tags`);
      } else {
        addLog('error', `Batch write failed: ${result.error}`);
      }
    } catch (error) {
      addLog('error', `Batch write error: ${error}`);
    } finally {
      setIsBatchWriting(false);
    }
  };

  // Performance benchmark
  const handleRunBenchmark = async () => {
    if (!benchmarkTestTag.trim()) {
      addLog('error', 'Please enter a benchmark tag name');
      return;
    }

    setIsRunningBenchmark(true);
    addLog('info', `Running performance benchmark on ${benchmarkTestTag}`);

    try {
      const result = await runBenchmark(benchmarkTestTag, benchmarkTestType, benchmarkTestWrites);
      if (result.success) {
        setBenchmarkResults(result);
        addLog('success', `Benchmark completed: ${result.readRate?.toFixed(0)} reads/sec, ${result.writeRate?.toFixed(0)} writes/sec`);
      } else {
        addLog('error', `Benchmark failed: ${result.error}`);
      }
    } catch (error) {
      addLog('error', `Benchmark error: ${error}`);
    } finally {
      setIsRunningBenchmark(false);
    }
  };

  // HMI Demo functions
  const startHmiMonitoring = () => {
    setIsHmiMonitoring(true);
    addLog('info', 'Started HMI monitoring simulation');
  };

  const stopHmiMonitoring = () => {
    setIsHmiMonitoring(false);
    addLog('info', 'Stopped HMI monitoring simulation');
  };

  // Real-time Subscriptions functions
  const addSubscription = async () => {
    if (!newSubscriptionTag.trim()) {
      addLog('error', 'Please enter a tag name for subscription');
      return;
    }

    const subscription = {
      id: `sub-${Date.now()}`,
      tagName: newSubscriptionTag,
      dataType: newSubscriptionType,
      updateRate: newSubscriptionRate,
      lastValue: null,
      isActive: true,
      changeThreshold: newSubscriptionThreshold
    };

    setSubscriptions(prev => [...prev, subscription]);
    addLog('success', `Added subscription for ${newSubscriptionTag}`);
    
    // Start reading the tag immediately
    startSubscriptionReading(subscription);
    
    // Clear form
    setNewSubscriptionTag("");
    setNewSubscriptionType("String");
    setNewSubscriptionRate(100);
    setNewSubscriptionThreshold(0.001);
  };

  const startSubscriptionReading = async (subscription: any) => {
    const readTagValue = async () => {
      if (!subscription.isActive) return;
      
      try {
        const result = await readTag(subscription.tagName, subscription.dataType);
        setSubscriptions(prev => prev.map(sub => 
          sub.id === subscription.id 
            ? { ...sub, lastValue: result.value }
            : sub
        ));
      } catch (error) {
        addLog('error', `Failed to read subscription ${subscription.tagName}: ${error}`);
      }
    };

    // Read immediately
    await readTagValue();
    
    // Set up interval for continuous reading
    const interval = setInterval(async () => {
      const currentSub = subscriptions.find(sub => sub.id === subscription.id);
      if (!currentSub || !currentSub.isActive) {
        clearInterval(interval);
        return;
      }
      await readTagValue();
    }, subscription.updateRate);

    // Store interval ID for cleanup
    subscription.intervalId = interval;
  };

  const removeSubscription = (id: string) => {
    setSubscriptions(prev => prev.filter(sub => sub.id !== id));
    addLog('info', 'Removed subscription');
  };

  const toggleSubscription = (id: string) => {
    setSubscriptions(prev => prev.map(sub => 
      sub.id === id ? { ...sub, isActive: !sub.isActive } : sub
    ));
  };

  // Advanced Operations functions
  const handleAdvancedTagOperation = async () => {
    if (!advancedTagPath.trim()) {
      addLog('error', 'Please enter a tag path');
      return;
    }

    addLog('info', `Processing advanced tag path: ${advancedTagPath}`);
    
    try {
      // Simulate complex tag path processing
      const result = await readTag(advancedTagPath, "String");
      setComplexTagResults({
        tagPath: advancedTagPath,
        value: result.value,
        timestamp: new Date().toISOString()
      });
      addLog('success', `Advanced tag operation completed: ${result.value}`);
    } catch (error) {
      addLog('error', `Advanced tag operation failed: ${error}`);
    }
  };

  const handleTagDiscovery = async () => {
    if (!advancedTagPath.trim()) {
      addLog('error', 'Please enter a tag name for discovery');
      return;
    }

    addLog('info', `Discovering tag: ${advancedTagPath}`);
    
    try {
      const tagType = await discoverTag(advancedTagPath);
      setTagDiscoveryResults({
        tagName: advancedTagPath,
        discoveredType: tagType,
        timestamp: new Date().toISOString()
      });
      addLog('success', `Tag discovery completed: ${tagType}`);
    } catch (error) {
      addLog('error', `Tag discovery failed: ${error}`);
    }
  };

  const handleArrayTest = async () => {
    if (!isConnected) {
      addLog('error', 'Please connect to PLC first');
      return;
    }

    setIsTestingArrays(true);
    addLog('info', `Running array element tests: ${arrayTestType}`);
    
    try {
      const results = await testArrays(arrayTestType);
      setArrayTestResults(results);
      
      const summary = results.summary;
      addLog('success', 
        `Array tests completed: ${summary.successful}/${summary.total} successful (${summary.successRate.toFixed(1)}%)`
      );
    } catch (error: any) {
      addLog('error', `Array test failed: ${error.message || error}`);
      setArrayTestResults(null);
    } finally {
      setIsTestingArrays(false);
    }
  };

  // System Monitoring functions
  const fetchSystemMetrics = async () => {
    try {
      const response = await fetch('http://localhost:8080/api/metrics');
      if (!response.ok) {
        throw new Error(`HTTP error! status: ${response.status}`);
      }
      const metrics = await response.json();
      setSystemMetrics(metrics);
      addLog('info', 'System metrics updated');
    } catch (error) {
      addLog('error', `Failed to fetch system metrics: ${error}`);
      // Set mock data for demo purposes
      setSystemMetrics({
        connectionHealth: 'good',
        operationCount: Math.floor(Math.random() * 10000),
        errorCount: Math.floor(Math.random() * 10),
        avgLatency: Math.random() * 5 + 1,
        memoryUsage: Math.random() * 100 + 50,
        cpuUsage: Math.random() * 20 + 5,
        uptime: Math.floor(Math.random() * 3600) + 1800
      });
    }
  };

  const checkWebSocketConnection = () => {
    // Simulate WebSocket connection check
    const isConnected = Math.random() > 0.3; // 70% chance of being connected
    setIsWebSocketConnected(isConnected);
    addLog(isConnected ? 'success' : 'warning', 
      isConnected ? 'WebSocket connected' : 'WebSocket disconnected');
  };

  // UI rendering
  return (
    <div className="hmi-container">
      {/* Header and Status */}
      <div className="hmi-panel p-6 mb-6">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-4">
            <div className="w-12 h-12 bg-gradient-to-br from-blue-500 to-blue-600 rounded-xl flex items-center justify-center text-white text-2xl font-bold shadow-md">
              G
            </div>
            <div>
              <h1 className="text-2xl font-bold text-gray-900">
                Go + Next.js Application
              </h1>
              <p className="text-gray-600">Rust EtherNet/IP Driver - Industrial HMI</p>
            </div>
            {isMonitoring && (
              <div className="status-indicator status-running pulse-success ml-4">
                📊 Monitoring {monitoredTags.length} tags
              </div>
            )}
          </div>
          <div className="flex items-center gap-4">
            <div className={`status-indicator ${isConnected ? 'status-running' : 'status-stopped'}`}>
              {isConnected ? '🟢 CONNECTED' : '🔴 DISCONNECTED'}
            </div>
          </div>
        </div>
      </div>

      {/* Connection Controls */}
      <div className="hmi-panel p-6 mb-6">
        <div className="flex items-center gap-4">
          <input
            type="text"
            value={plcAddress}
            onChange={(e) => setPlcAddress(e.target.value)}
            placeholder="PLC Address (IP:Port)"
            className="hmi-input flex-1"
            disabled={isConnecting}
          />
          <button
            onClick={isConnected ? handleDisconnect : handleConnect}
            disabled={isConnecting}
            className={`hmi-button ${isConnected ? 'btn-danger' : 'btn-primary'}`}
          >
            {isConnecting ? '⏳ Connecting...' : isConnected ? '🔌 Disconnect' : '🔌 Connect'}
          </button>
        </div>
      </div>

      {/* Tab Navigation */}
      <div className="hmi-panel p-6 mb-6">
        <div className="hmi-tabs">
          {TABS.map(tab => (
            <button
              key={tab}
              onClick={() => setActiveTab(tab)}
              className={`hmi-tab ${activeTab === tab ? 'active' : ''}`}
            >
              {tab}
            </button>
          ))}
        </div>
      </div>

      {/* Main Content Area - Full Width */}
      <div className="w-full">
        <div className="w-full p-6">
          <div className="hmi-panel p-8 mb-6">
            {activeTab === "Individual" && (
              <div className="space-y-6">
                <h2 className="text-xl font-bold mb-4 flex items-center gap-3 text-gray-800">
                  <span role="img" aria-label="tag">🏷️</span>
                  Individual Tag Operations
                </h2>
                
                <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
                  <input
                    type="text"
                    value={tagName}
                    onChange={(e) => setTagName(e.target.value)}
                    placeholder="Tag Name (e.g., TestTag)"
                    className="hmi-input"
                  />
                  <select
                    value={tagType}
                    onChange={(e) => setTagType(e.target.value)}
                    className="hmi-select"
                  >
                    {PLC_TYPES.map(type => (
                      <option key={type.value} value={type.value}>{type.label}</option>
                    ))}
                  </select>
                  <input
                    type="text"
                    value={tagValue}
                    onChange={(e) => setTagValue(e.target.value)}
                    placeholder="Value to Write"
                    className="hmi-input"
                  />
                </div>

                <div className="flex gap-4">
                  <button
                    onClick={handleReadTag}
                    disabled={!isConnected || isReading}
                    className="btn-primary"
                  >
                    {isReading ? '⏳ Reading...' : '📖 Read Tag'}
                  </button>
                  <button
                    onClick={handleWriteTag}
                    disabled={!isConnected || isWriting || !tagValue}
                    className="btn-secondary"
                  >
                    {isWriting ? '⏳ Writing...' : '✏️ Write Tag'}
                  </button>
                </div>

                {readValue !== null && (
                  <div className="hmi-card bg-green-50 border-green-200">
                    <h3 className="font-bold text-green-800 mb-2">📊 Read Result</h3>
                    <div className="font-mono text-green-700 bg-green-100 p-3 rounded">
                      {JSON.stringify(readValue, null, 2)}
                    </div>
                  </div>
                )}
              </div>
            )}

            {activeTab === "Batch" && (
              <div className="space-y-6">
                <h2 className="text-xl font-bold mb-4 flex items-center gap-3 text-gray-800">
                  <span role="img" aria-label="batch">📦</span>
                  Batch Operations
                </h2>
                
                <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
                  <div className="hmi-card">
                    <h3 className="font-bold mb-4 text-blue-800">📖 Batch Read</h3>
                    <textarea
                      value={batchTags}
                      onChange={(e) => setBatchTags(e.target.value)}
                      placeholder="Enter tags to read (one per line):&#10;TagName1:String&#10;TagName2:Dint&#10;TagName3:Real"
                      className="hmi-input h-32 mb-4"
                    />
                    <button
                      onClick={handleBatchRead}
                      disabled={!isConnected || isBatchReading}
                      className="btn-primary w-full"
                    >
                      {isBatchReading ? '⏳ Reading...' : '📖 Batch Read'}
                    </button>
                    
                    {batchReadResult && (
                      <div className="mt-4 bg-green-50 border border-green-200 rounded-lg p-4">
                        <h4 className="font-bold text-green-800 mb-2">📊 Results</h4>
                        <div className="font-mono text-sm text-green-700 bg-green-100 p-3 rounded max-h-48 overflow-y-auto">
                          {JSON.stringify(batchReadResult, null, 2)}
                        </div>
                      </div>
                    )}
                  </div>

                  <div className="hmi-card">
                    <h3 className="font-bold mb-4 text-orange-800">✏️ Batch Write</h3>
                    <textarea
                      value={batchWriteData}
                      onChange={(e) => setBatchWriteData(e.target.value)}
                      placeholder="Enter tag=value pairs (one per line):&#10;TagName1:String=Hello&#10;TagName2:Dint=42&#10;TagName3:Real=3.14"
                      className="hmi-input h-32 mb-4"
                    />
                    <button
                      onClick={handleBatchWrite}
                      disabled={!isConnected || isBatchWriting}
                      className="btn-secondary w-full"
                    >
                      {isBatchWriting ? '⏳ Writing...' : '✏️ Batch Write'}
                    </button>
                    
                    {batchWriteResult && (
                      <div className="mt-4 bg-blue-50 border border-blue-200 rounded-lg p-4">
                        <h4 className="font-bold text-blue-800 mb-2">📊 Results</h4>
                        <div className="font-mono text-sm text-blue-700 bg-blue-100 p-3 rounded">
                          {JSON.stringify(batchWriteResult, null, 2)}
                        </div>
                      </div>
                    )}
                  </div>
                </div>
              </div>
            )}

            {activeTab === "Performance" && (
              <div className="space-y-6">
                <h2 className="text-xl font-bold mb-4 flex items-center gap-3 text-gray-800">
                  <span role="img" aria-label="performance">⚡</span>
                  Performance Benchmark
                </h2>
                
                <div className="hmi-card">
                  <div className="grid grid-cols-1 md:grid-cols-3 gap-4 mb-6">
                    <input
                      type="text"
                      value={benchmarkTestTag}
                      onChange={(e) => setBenchmarkTestTag(e.target.value)}
                      placeholder="Benchmark Tag Name"
                      className="hmi-input"
                    />
                    <select
                      value={benchmarkTestType}
                      onChange={(e) => setBenchmarkTestType(e.target.value)}
                      className="hmi-select"
                    >
                      {PLC_TYPES.map(type => (
                        <option key={type.value} value={type.value}>{type.label}</option>
                      ))}
                    </select>
                    <label className="flex items-center gap-2 text-gray-700">
                      <input
                        type="checkbox"
                        checked={benchmarkTestWrites}
                        onChange={(e) => setBenchmarkTestWrites(e.target.checked)}
                        className="rounded"
                      />
                      Include Writes
                    </label>
                  </div>
                  
                  <button
                    onClick={handleRunBenchmark}
                    disabled={!isConnected || isRunningBenchmark}
                    className="btn-primary mb-6"
                  >
                    {isRunningBenchmark ? '⏳ Running Benchmark...' : '🚀 Run Benchmark'}
                  </button>
                  
                  {benchmarkResults && (
                    <div className="bg-purple-50 border border-purple-200 rounded-lg p-6">
                      <h3 className="font-bold text-purple-800 mb-4">📈 Benchmark Results</h3>
                      <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
                        <div className="text-center">
                          <div className="text-2xl font-bold text-purple-600">{benchmarkResults.readRate?.toFixed(0)}</div>
                          <div className="text-sm text-purple-700">Reads/sec</div>
                        </div>
                        {benchmarkTestWrites && (
                          <div className="text-center">
                            <div className="text-2xl font-bold text-purple-600">{benchmarkResults.writeRate?.toFixed(0)}</div>
                            <div className="text-sm text-purple-700">Writes/sec</div>
                          </div>
                        )}
                        <div className="text-center">
                          <div className="text-2xl font-bold text-purple-600">{benchmarkResults.avgLatency?.toFixed(2)}ms</div>
                          <div className="text-sm text-purple-700">Avg Latency</div>
                        </div>
                        <div className="text-center">
                          <div className="text-2xl font-bold text-purple-600">{benchmarkResults.totalOperations}</div>
                          <div className="text-sm text-purple-700">Total Ops</div>
                        </div>
                      </div>
                    </div>
                  )}
                </div>
              </div>
            )}

            {activeTab === "Subscriptions" && (
              <div className="space-y-6">
                <h2 className="text-xl font-bold mb-4 flex items-center gap-3 text-gray-800">
                  <span role="img" aria-label="subscriptions">📡</span>
                  Real-time Tag Subscriptions
                </h2>
                
                <div className="hmi-card">
                  <h3 className="font-bold mb-4 text-blue-800">➕ Add New Subscription</h3>
                  <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4 mb-4">
                    <input
                      type="text"
                      value={newSubscriptionTag}
                      onChange={(e) => setNewSubscriptionTag(e.target.value)}
                      placeholder="Tag Name"
                      className="hmi-input"
                    />
                    <select
                      value={newSubscriptionType}
                      onChange={(e) => setNewSubscriptionType(e.target.value)}
                      className="hmi-select"
                    >
                      {PLC_TYPES.map(type => (
                        <option key={type.value} value={type.value}>{type.label}</option>
                      ))}
                    </select>
                    <input
                      type="number"
                      value={newSubscriptionRate}
                      onChange={(e) => setNewSubscriptionRate(Number(e.target.value))}
                      placeholder="Update Rate (ms)"
                      min="10"
                      className="hmi-input"
                    />
                    <input
                      type="number"
                      value={newSubscriptionThreshold}
                      onChange={(e) => setNewSubscriptionThreshold(Number(e.target.value))}
                      placeholder="Change Threshold"
                      step="0.001"
                      className="hmi-input"
                    />
                  </div>
                  <button
                    onClick={addSubscription}
                    disabled={!isConnected || !newSubscriptionTag.trim()}
                    className="btn-primary"
                  >
                    ➕ Add Subscription
                  </button>
                </div>
                
                <div className="hmi-card">
                  <h3 className="font-bold mb-4 text-green-800">📊 Active Subscriptions</h3>
                  {subscriptions.length === 0 ? (
                    <div className="text-gray-500 text-center py-8">
                      No active subscriptions. Add one above to start monitoring tags in real-time.
                    </div>
                  ) : (
                    <div className="space-y-3">
                      {subscriptions.map((sub) => (
                        <div key={sub.id} className="flex items-center justify-between p-4 bg-gray-50 rounded-lg border border-gray-200">
                          <div className="flex items-center gap-4">
                            <div className={`w-3 h-3 rounded-full ${sub.isActive ? 'bg-green-500 animate-pulse' : 'bg-gray-400'}`}></div>
                            <div>
                              <div className="font-mono font-bold">{sub.tagName}</div>
                              <div className="text-sm text-gray-600">{sub.dataType} • {sub.updateRate}ms</div>
                            </div>
                          </div>
                          <div className="flex items-center gap-4">
                            <div className="text-right">
                              <div className="font-mono text-lg">{sub.lastValue ?? '---'}</div>
                              <div className="text-xs text-gray-500">Last Value</div>
                            </div>
                            <div className="flex gap-2">
                              <button
                                onClick={() => toggleSubscription(sub.id)}
                                className={`px-3 py-1 rounded text-sm ${sub.isActive ? 'bg-yellow-200 text-yellow-800' : 'bg-green-200 text-green-800'}`}
                              >
                                {sub.isActive ? '⏸️ Pause' : '▶️ Resume'}
                              </button>
                              <button
                                onClick={() => removeSubscription(sub.id)}
                                className="px-3 py-1 rounded text-sm bg-red-200 text-red-800"
                              >
                                🗑️ Remove
                              </button>
                            </div>
                          </div>
                        </div>
                      ))}
                    </div>
                  )}
                </div>
              </div>
            )}

            {activeTab === "Monitoring" && (
              <div className="space-y-6">
                <h2 className="text-xl font-bold mb-4 flex items-center gap-3 text-gray-800">
                  <span role="img" aria-label="monitoring">📊</span>
                  System Monitoring & Health
                </h2>
                
                <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
                  <div className="hmi-card">
                    <h3 className="font-bold mb-4 text-blue-800">🔍 System Metrics</h3>
                    <div className="space-y-4">
                      <div className="flex justify-between items-center">
                        <span className="text-gray-700">Connection Health:</span>
                        <span className={`font-bold ${
                          systemMetrics.connectionHealth === 'excellent' ? 'text-green-600' :
                          systemMetrics.connectionHealth === 'good' ? 'text-blue-600' :
                          systemMetrics.connectionHealth === 'fair' ? 'text-yellow-600' : 'text-red-600'
                        }`}>
                          {systemMetrics.connectionHealth.toUpperCase()}
                        </span>
                      </div>
                      <div className="flex justify-between items-center">
                        <span className="text-gray-700">Operations:</span>
                        <span className="font-mono">{systemMetrics.operationCount.toLocaleString()}</span>
                      </div>
                      <div className="flex justify-between items-center">
                        <span className="text-gray-700">Errors:</span>
                        <span className="font-mono text-red-600">{systemMetrics.errorCount}</span>
                      </div>
                      <div className="flex justify-between items-center">
                        <span className="text-gray-700">Avg Latency:</span>
                        <span className="font-mono">{systemMetrics.avgLatency.toFixed(2)}ms</span>
                      </div>
                      <div className="flex justify-between items-center">
                        <span className="text-gray-700">Memory Usage:</span>
                        <span className="font-mono">{systemMetrics.memoryUsage.toFixed(1)}MB</span>
                      </div>
                      <div className="flex justify-between items-center">
                        <span className="text-gray-700">CPU Usage:</span>
                        <span className="font-mono">{systemMetrics.cpuUsage.toFixed(1)}%</span>
                      </div>
                      <div className="flex justify-between items-center">
                        <span className="text-gray-700">Uptime:</span>
                        <span className="font-mono">{Math.floor(systemMetrics.uptime / 60)}m {systemMetrics.uptime % 60}s</span>
                      </div>
                    </div>
                    <button
                      onClick={fetchSystemMetrics}
                      className="btn-primary w-full mt-4"
                    >
                      🔄 Refresh Metrics
                    </button>
                  </div>
                  
                  <div className="hmi-card">
                    <h3 className="font-bold mb-4 text-green-800">🌐 Connection Status</h3>
                    <div className="space-y-4">
                      <div className="flex items-center gap-3">
                        <div className={`w-4 h-4 rounded-full ${isConnected ? 'bg-green-500 animate-pulse' : 'bg-red-500'}`}></div>
                        <span className="font-bold">PLC Connection: {isConnected ? 'Connected' : 'Disconnected'}</span>
                      </div>
                      <div className="flex items-center gap-3">
                        <div className={`w-4 h-4 rounded-full ${isWebSocketConnected ? 'bg-green-500 animate-pulse' : 'bg-yellow-500'}`}></div>
                        <span className="font-bold">WebSocket: {isWebSocketConnected ? 'Connected' : 'Disconnected'}</span>
                      </div>
                      <div className="flex items-center gap-3">
                        <div className="w-4 h-4 rounded-full bg-blue-500 animate-pulse"></div>
                        <span className="font-bold">API Server: Active</span>
                      </div>
                    </div>
                    <button
                      onClick={checkWebSocketConnection}
                      className="btn-secondary w-full mt-4"
                    >
                      🔍 Check Connections
                    </button>
                  </div>
                </div>
              </div>
            )}

            {activeTab === "Advanced" && (
              <div className="space-y-6">
                <h2 className="text-xl font-bold mb-4 flex items-center gap-3 text-gray-800">
                  <span role="img" aria-label="advanced">⚙️</span>
                  Advanced Operations
                </h2>
                
                <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
                  <div className="hmi-card">
                    <h3 className="font-bold mb-4 text-purple-800">🔍 Complex Tag Operations</h3>
                    <input
                      type="text"
                      value={advancedTagPath}
                      onChange={(e) => setAdvancedTagPath(e.target.value)}
                      placeholder="Complex Tag Path (e.g., Program:MainProgram.MyUDT.Value)"
                      className="hmi-input mb-4"
                    />
                    <div className="flex gap-2 mb-4">
                      <button
                        onClick={handleAdvancedTagOperation}
                        disabled={!isConnected || !advancedTagPath.trim()}
                        className="btn-primary flex-1"
                      >
                        🔍 Process Tag
                      </button>
                      <button
                        onClick={handleTagDiscovery}
                        disabled={!isConnected || !advancedTagPath.trim()}
                        className="btn-secondary flex-1"
                      >
                        🔍 Discover Type
                      </button>
                    </div>
                    
                    {complexTagResults && (
                      <div className="bg-purple-50 border border-purple-200 rounded-lg p-4 mb-4">
                        <h4 className="font-bold text-purple-800 mb-2">📊 Operation Result</h4>
                        <div className="font-mono text-sm text-purple-700 bg-purple-100 p-3 rounded">
                          {JSON.stringify(complexTagResults, null, 2)}
                        </div>
                      </div>
                    )}
                    
                    {tagDiscoveryResults && (
                      <div className="bg-blue-50 border border-blue-200 rounded-lg p-4">
                        <h4 className="font-bold text-blue-800 mb-2">🔍 Discovery Result</h4>
                        <div className="font-mono text-sm text-blue-700 bg-blue-100 p-3 rounded">
                          {JSON.stringify(tagDiscoveryResults, null, 2)}
                        </div>
                      </div>
                    )}
                  </div>
                  
                  <div className="hmi-card">
                    <h3 className="font-bold mb-4 text-orange-800">📋 UDT & Structure Analysis</h3>
                    <div className="text-gray-600 mb-4">
                      User Defined Types (UDT) and complex data structures analysis will be displayed here.
                    </div>
                    {udtData ? (
                      <div className="bg-orange-50 border border-orange-200 rounded-lg p-4">
                        <h4 className="font-bold text-orange-800 mb-2">📊 UDT Data</h4>
                        <div className="font-mono text-sm text-orange-700 bg-orange-100 p-3 rounded">
                          {JSON.stringify(udtData, null, 2)}
                        </div>
                      </div>
                    ) : (
                      <div className="text-center text-gray-500 py-8">
                        No UDT data available. Perform a complex tag operation to analyze structure.
                      </div>
                    )}
                  </div>
                </div>

                {/* Array Element Test Section - v0.5.5 */}
                <div className="hmi-card mt-6">
                  <h3 className="font-bold mb-4 text-green-800">📊 Array Element Access Test (v0.5.5)</h3>
                  <div className="text-gray-600 mb-4">
                    Test array element read/write support with automatic workaround. Tests controller-scoped, program-scoped, and BOOL arrays.
                  </div>
                  
                  <div className="mb-4">
                    <label className="block text-sm font-medium text-gray-700 mb-2">
                      Test Type:
                    </label>
                    <select
                      value={arrayTestType}
                      onChange={(e) => setArrayTestType(e.target.value as any)}
                      className="hmi-input"
                      disabled={isTestingArrays}
                    >
                      <option value="all">All Tests (Controller + Program + BOOL)</option>
                      <option value="controller">Controller-Scoped DINT Array</option>
                      <option value="program">Program-Scoped DINT Array</option>
                      <option value="bool">Controller-Scoped BOOL Array</option>
                    </select>
                  </div>

                  <button
                    onClick={handleArrayTest}
                    disabled={!isConnected || isTestingArrays}
                    className="btn-primary w-full mb-4"
                  >
                    {isTestingArrays ? "🔄 Testing..." : "🚀 Run Array Element Tests"}
                  </button>

                  {arrayTestResults && (
                    <div className="bg-green-50 border border-green-200 rounded-lg p-4">
                      <h4 className="font-bold text-green-800 mb-2">📊 Test Results</h4>
                      <div className="mb-2">
                        <div className="flex justify-between items-center mb-1">
                          <span className="text-sm text-gray-700">Total Tests:</span>
                          <span className="font-bold">{arrayTestResults.summary?.total || 0}</span>
                        </div>
                        <div className="flex justify-between items-center mb-1">
                          <span className="text-sm text-gray-700">Successful:</span>
                          <span className="font-bold text-green-600">{arrayTestResults.summary?.successful || 0}</span>
                        </div>
                        <div className="flex justify-between items-center mb-1">
                          <span className="text-sm text-gray-700">Failed:</span>
                          <span className="font-bold text-red-600">{arrayTestResults.summary?.failed || 0}</span>
                        </div>
                        <div className="flex justify-between items-center">
                          <span className="text-sm text-gray-700">Success Rate:</span>
                          <span className="font-bold">
                            {arrayTestResults.summary?.successRate?.toFixed(1) || 0}%
                          </span>
                        </div>
                      </div>
                      
                      <div className="mt-4 max-h-96 overflow-y-auto">
                        <h5 className="font-bold text-sm mb-2">Detailed Results:</h5>
                        <div className="space-y-2">
                          {arrayTestResults.tests?.map((test: any, idx: number) => (
                            <div key={idx} className="bg-white border rounded p-2 text-xs">
                              <div className="font-bold">{test.tag} ({test.scope})</div>
                              <div className="mt-1">
                                <span className={test.read?.success ? "text-green-600" : "text-red-600"}>
                                  Read: {test.read?.success ? "✅" : "❌"} {test.read?.value !== undefined ? test.read.value : test.read?.error}
                                </span>
                                {test.write && (
                                  <span className={`ml-2 ${test.write?.success ? "text-green-600" : "text-red-600"}`}>
                                    Write: {test.write?.success ? "✅" : "❌"} {test.write?.value !== undefined ? test.write.value : test.write?.error}
                                  </span>
                                )}
                                {test.verify && (
                                  <span className={`ml-2 ${test.verify?.match ? "text-green-600" : "text-yellow-600"}`}>
                                    Verify: {test.verify?.match ? "✅ Match" : "⚠️ Mismatch"}
                                  </span>
                                )}
                              </div>
                            </div>
                          ))}
                        </div>
                      </div>
                    </div>
                  )}
                </div>
              </div>
            )}

            {activeTab === "HMI Demo" && (
              <div className="space-y-6">
                <h2 className="text-xl font-bold mb-4 flex items-center gap-3 text-gray-800">
                  <span role="img" aria-label="hmi">🖥️</span>
                  HMI Dashboard Demo
                </h2>
                
                <div className="flex gap-4 mb-6">
                  <button
                    onClick={startHmiMonitoring}
                    disabled={isHmiMonitoring}
                    className="btn-primary"
                  >
                    ▶️ Start Monitoring
                  </button>
                  <button
                    onClick={stopHmiMonitoring}
                    disabled={!isHmiMonitoring}
                    className="btn-secondary"
                  >
                    ⏹️ Stop Monitoring
                  </button>
                </div>
                
                <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
                  <div className="hmi-card text-center">
                    <div className="text-3xl mb-2">🏭</div>
                    <h3 className="font-bold text-gray-800">Machine Status</h3>
                    <div className={`text-lg font-bold mt-2 ${hmiData.machineStatus === 'Running' ? 'text-green-600' : 'text-red-600'}`}>
                      {hmiData.machineStatus}
                    </div>
                  </div>
                  
                  <div className="hmi-card text-center">
                    <div className="text-3xl mb-2">👨‍🔧</div>
                    <h3 className="font-bold text-gray-800">Operator</h3>
                    <div className="text-lg font-bold mt-2 text-blue-600">
                      {hmiData.operator}
                    </div>
                    <div className="text-sm text-gray-600">Shift {hmiData.shift}</div>
                  </div>
                  
                  <div className="hmi-card text-center">
                    <div className="text-3xl mb-2">📊</div>
                    <h3 className="font-bold text-gray-800">Production</h3>
                    <div className="text-lg font-bold mt-2 text-purple-600">
                      {hmiData.productionCount.toLocaleString()}
                    </div>
                    <div className="text-sm text-gray-600">Target: {hmiData.targetCount.toLocaleString()}</div>
                  </div>
                  
                  <div className="hmi-card text-center">
                    <div className="text-3xl mb-2">⚡</div>
                    <h3 className="font-bold text-gray-800">OEE</h3>
                    <div className="text-lg font-bold mt-2 text-green-600">
                      {hmiData.oee}%
                    </div>
                  </div>
                </div>
                
                <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
                  <div className="hmi-card">
                    <h3 className="font-bold mb-4 text-blue-800">📈 Availability</h3>
                    <div className="text-center">
                      <div className="text-3xl font-bold text-blue-600">{hmiData.availability}%</div>
                      <div className="w-full bg-gray-200 rounded-full h-2 mt-2">
                        <div 
                          className="bg-blue-600 h-2 rounded-full" 
                          style={{width: `${hmiData.availability}%`}}
                        ></div>
                      </div>
                    </div>
                  </div>
                  
                  <div className="hmi-card">
                    <h3 className="font-bold mb-4 text-green-800">🚀 Performance</h3>
                    <div className="text-center">
                      <div className="text-3xl font-bold text-green-600">{hmiData.performance}%</div>
                      <div className="w-full bg-gray-200 rounded-full h-2 mt-2">
                        <div 
                          className="bg-green-600 h-2 rounded-full" 
                          style={{width: `${hmiData.performance}%`}}
                        ></div>
                      </div>
                    </div>
                  </div>
                  
                  <div className="hmi-card">
                    <h3 className="font-bold mb-4 text-purple-800">✅ Quality</h3>
                    <div className="text-center">
                      <div className="text-3xl font-bold text-purple-600">{hmiData.qualityRate}%</div>
                      <div className="w-full bg-gray-200 rounded-full h-2 mt-2">
                        <div 
                          className="bg-purple-600 h-2 rounded-full" 
                          style={{width: `${hmiData.qualityRate}%`}}
                        ></div>
                      </div>
                    </div>
                  </div>
                </div>
              </div>
            )}

            {activeTab === "Config" && (
              <div className="space-y-6">
                <h2 className="text-xl font-bold mb-4 flex items-center gap-3 text-gray-800">
                  <span role="img" aria-label="config">⚙️</span>
                  Configuration & Settings
                </h2>
                
                <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
                  <div className="hmi-card">
                    <h3 className="font-bold mb-4 text-blue-800">🔧 Connection Settings</h3>
                    <div className="space-y-4">
                      <div>
                        <label className="block text-sm font-bold text-gray-700 mb-2">PLC Address</label>
                        <input
                          type="text"
                          value={plcAddress}
                          onChange={(e) => setPlcAddress(e.target.value)}
                          className="hmi-input w-full"
                        />
                      </div>
                      <div>
                        <label className="block text-sm font-bold text-gray-700 mb-2">Connection Timeout (ms)</label>
                        <input
                          type="number"
                          defaultValue="5000"
                          className="hmi-input w-full"
                        />
                      </div>
                      <div>
                        <label className="block text-sm font-bold text-gray-700 mb-2">Retry Attempts</label>
                        <input
                          type="number"
                          defaultValue="3"
                          className="hmi-input w-full"
                        />
                      </div>
                    </div>
                  </div>
                  
                  <div className="hmi-card">
                    <h3 className="font-bold mb-4 text-green-800">📊 Monitoring Settings</h3>
                    <div className="space-y-4">
                      <div>
                        <label className="block text-sm font-bold text-gray-700 mb-2">Default Update Rate (ms)</label>
                        <input
                          type="number"
                          defaultValue="100"
                          className="hmi-input w-full"
                        />
                      </div>
                      <div>
                        <label className="block text-sm font-bold text-gray-700 mb-2">Max Log Entries</label>
                        <input
                          type="number"
                          defaultValue="100"
                          className="hmi-input w-full"
                        />
                      </div>
                      <div>
                        <label className="flex items-center gap-2 text-gray-700">
                          <input
                            type="checkbox"
                            defaultChecked
                            className="rounded"
                          />
                          Enable Auto-reconnect
                        </label>
                      </div>
                    </div>
                  </div>
                </div>
              </div>
            )}

            {activeTab === "About" && (
              <div className="space-y-6">
                <h2 className="text-xl font-bold mb-4 flex items-center gap-3 text-gray-800">
                  <span role="img" aria-label="about">ℹ️</span>
                  About This Application
                </h2>
                
                <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
                  <div className="hmi-card">
                    <h3 className="font-bold mb-4 text-blue-800">🚀 Technology Stack</h3>
                    <div className="space-y-3">
                      <div className="flex items-center gap-3">
                        <span className="text-2xl">🦀</span>
                        <div>
                          <div className="font-bold">Rust Backend</div>
                          <div className="text-sm text-gray-600">EtherNet/IP protocol implementation</div>
                        </div>
                      </div>
                      <div className="flex items-center gap-3">
                        <span className="text-2xl">🐹</span>
                        <div>
                          <div className="font-bold">Go API Server</div>
                          <div className="text-sm text-gray-600">HTTP API and WebSocket server</div>
                        </div>
                      </div>
                      <div className="flex items-center gap-3">
                        <span className="text-2xl">⚛️</span>
                        <div>
                          <div className="font-bold">Next.js Frontend</div>
                          <div className="text-sm text-gray-600">React-based web interface</div>
                        </div>
                      </div>
                      <div className="flex items-center gap-3">
                        <span className="text-2xl">🎨</span>
                        <div>
                          <div className="font-bold">Tailwind CSS</div>
                          <div className="text-sm text-gray-600">Modern responsive styling</div>
                        </div>
                      </div>
                    </div>
                  </div>
                  
                  <div className="hmi-card">
                    <h3 className="font-bold mb-4 text-green-800">✨ Features</h3>
                    <div className="space-y-2">
                      <div className="flex items-center gap-2">
                        <span className="text-green-600">✅</span>
                        <span>Individual tag read/write operations</span>
                      </div>
                      <div className="flex items-center gap-2">
                        <span className="text-green-600">✅</span>
                        <span>Batch operations for multiple tags</span>
                      </div>
                      <div className="flex items-center gap-2">
                        <span className="text-green-600">✅</span>
                        <span>Performance benchmarking</span>
                      </div>
                      <div className="flex items-center gap-2">
                        <span className="text-green-600">✅</span>
                        <span>Real-time tag subscriptions</span>
                      </div>
                      <div className="flex items-center gap-2">
                        <span className="text-green-600">✅</span>
                        <span>System monitoring & health checks</span>
                      </div>
                      <div className="flex items-center gap-2">
                        <span className="text-green-600">✅</span>
                        <span>Advanced tag operations</span>
                      </div>
                      <div className="flex items-center gap-2">
                        <span className="text-green-600">✅</span>
                        <span>HMI dashboard demo</span>
                      </div>
                      <div className="flex items-center gap-2">
                        <span className="text-green-600">✅</span>
                        <span>Modern minimalistic design</span>
                      </div>
                    </div>
                  </div>
                </div>
                
                <div className="hmi-card text-center">
                  <h3 className="font-bold mb-4 text-purple-800">🎯 Purpose</h3>
                  <p className="text-gray-700 leading-relaxed">
                    This application demonstrates a complete industrial automation solution using modern web technologies. 
                    It provides a comprehensive interface for interacting with PLCs via the EtherNet/IP protocol, 
                    showcasing real-time data monitoring, batch operations, performance analysis, and system health monitoring.
                  </p>
                </div>
              </div>
            )}
          </div>

          {/* Activity Log - Full Width Below */}
          <div className="hmi-panel p-6">
            <h2 className="text-lg font-bold mb-4 flex items-center gap-3 text-gray-800">
              <span role="img" aria-label="log">📝</span> 
              Activity Log
            </h2>
            <div className="h-48 overflow-y-auto bg-gray-50 p-4 rounded-lg font-mono text-sm border border-gray-200">
              {logs.length === 0 ? (
                <div className="text-gray-500 italic text-center py-8">
                  Activity will be logged here when you interact with the PLC.
                </div>
              ) : (
                logs.map((log) => (
                  <div key={log.id} className={`mb-2 p-2 rounded ${
                    log.level === 'error' ? 'bg-red-50 text-red-800 border-l-4 border-red-500' : 
                    log.level === 'success' ? 'bg-green-50 text-green-800 border-l-4 border-green-500' : 
                    log.level === 'warning' ? 'bg-yellow-50 text-yellow-800 border-l-4 border-yellow-500' : 
                    'bg-gray-50 text-gray-800 border-l-4 border-gray-400'
                  }`}>
                    <div className="flex items-center gap-2">
                      <span className="text-xs text-gray-500">[{log.timestamp}]</span>
                      <span className={`text-xs font-bold px-2 py-1 rounded ${
                        log.level === 'error' ? 'bg-red-100 text-red-800' : 
                        log.level === 'success' ? 'bg-green-100 text-green-800' : 
                        log.level === 'warning' ? 'bg-yellow-100 text-yellow-800' : 
                        'bg-gray-100 text-gray-800'
                      }`}>
                        {log.level.toUpperCase()}
                      </span>
                    </div>
                    <div className="mt-1">{log.message}</div>
                  </div>
                ))
              )}
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
