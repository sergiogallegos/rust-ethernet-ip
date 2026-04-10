import React, { useEffect, useMemo, useState } from 'react';
import './App.css';
import {
  BenchmarkResponse,
  ConnectionStatus,
  DataType,
  DATA_TYPES,
  OverviewResponse,
  PlcValueJson,
  SignalReading,
  SnapshotResponse,
  TraceabilityRecord,
  TraceabilityResponse,
} from './types';

const API_BASE = process.env.REACT_APP_API_URL || 'http://localhost:3000';

const emptyStatus: ConnectionStatus = {
  connected: false,
  address: null,
  use_route_path: false,
  slot: null,
  connected_at_ms: null,
  plc_identity: null,
};

type HistoryPoint = {
  timestampMs: number;
  throughput: number;
  quality: number;
  cycle: number;
  availability: number;
};

type OpsAlert = {
  title: string;
  detail: string;
  tone: 'good' | 'watch' | 'bad';
};

function App() {
  const [connectionStatus, setConnectionStatus] = useState<ConnectionStatus>(emptyStatus);
  const [overview, setOverview] = useState<OverviewResponse | null>(null);
  const [snapshot, setSnapshot] = useState<SnapshotResponse | null>(null);
  const [history, setHistory] = useState<HistoryPoint[]>([]);
  const [benchmark, setBenchmark] = useState<BenchmarkResponse | null>(null);
  const [traceability, setTraceability] = useState<TraceabilityRecord[]>([]);
  const [activity, setActivity] = useState<string[]>([
    'Dashboard initialized. Waiting for PLC session.',
  ]);
  const [address, setAddress] = useState('192.168.0.101:44818');
  const [useRoutePath, setUseRoutePath] = useState(true);
  const [slot, setSlot] = useState('0');
  const [loading, setLoading] = useState(false);
  const [snapshotLoading, setSnapshotLoading] = useState(false);
  const [benchmarkLoading, setBenchmarkLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [readTagName, setReadTagName] = useState('Program:TestProgram.gTestArray_DINT[0]');
  const [readResult, setReadResult] = useState<{
    value: PlcValueJson | null;
    dataType: string | null;
    error: string | null;
  } | null>(null);
  const [writeTagName, setWriteTagName] = useState('gTestArray_DINT[5]');
  const [writeDataType, setWriteDataType] = useState<DataType>('DINT');
  const [writeValue, setWriteValue] = useState('12345');
  const [writeResult, setWriteResult] = useState<{ success: boolean; error: string | null } | null>(null);
  const [traceForm, setTraceForm] = useState({
    part_id: 'PART-10027',
    product_code: 'SKU-AXUM-01',
    lot_code: 'LOT-2026-04A',
    station: 'Station-3',
    status: 'Completed',
    notes: 'MacBook demo event captured from the dashboard.',
  });

  const pushActivity = (message: string) => {
    setActivity((current) => [message, ...current].slice(0, 10));
  };

  const fetchStatus = async () => {
    const response = await fetch(`${API_BASE}/api/status`);
    const data = await response.json();
    setConnectionStatus(data);
    return data as ConnectionStatus;
  };

  const fetchOverview = async () => {
    const response = await fetch(`${API_BASE}/api/overview`);
    const data = await response.json();
    setOverview(data);
    return data as OverviewResponse;
  };

  const fetchSnapshot: () => Promise<void> = async () => {
    if (!connectionStatus.connected) {
      return;
    }

    setSnapshotLoading(true);
    try {
      const response = await fetch(`${API_BASE}/api/demo/snapshot`);
      const data = await response.json();
      if (response.ok) {
        setSnapshot(data);
      } else {
        throw new Error(data.error || 'Failed to refresh snapshot');
      }
    } catch (fetchError) {
      setError(fetchError instanceof Error ? fetchError.message : 'Snapshot failed');
    } finally {
      setSnapshotLoading(false);
    }
  };

  const fetchTraceability = async () => {
    const response = await fetch(`${API_BASE}/api/traceability`);
    const data: TraceabilityResponse = await response.json();
    setTraceability(data.items);
  };

  useEffect(() => {
    const loadInitial = async () => {
      try {
        await fetchStatus();
        await fetchOverview();
        await fetchTraceability();
      } catch (loadError) {
        setError(loadError instanceof Error ? loadError.message : 'Initialization failed');
      }
    };

    loadInitial();
  }, []);

  useEffect(() => {
    if (!connectionStatus.connected) {
      return;
    }

    const refreshSnapshot = async () => {
      setSnapshotLoading(true);
      try {
        const response = await fetch(`${API_BASE}/api/demo/snapshot`);
        const data = await response.json();
        if (response.ok) {
          setSnapshot(data);
        } else {
          throw new Error(data.error || 'Failed to refresh snapshot');
        }
      } catch (fetchError) {
        setError(fetchError instanceof Error ? fetchError.message : 'Snapshot failed');
      } finally {
        setSnapshotLoading(false);
      }
    };

    refreshSnapshot();
    const interval = setInterval(refreshSnapshot, 3000);
    return () => clearInterval(interval);
  }, [connectionStatus.connected]);

  useEffect(() => {
    if (!snapshot) {
      return;
    }

    const throughput = signalNumber(snapshot, 'line_speed');
    const quality = signalNumber(snapshot, 'quality_proxy');
    const cycle = signalNumber(snapshot, 'cycle_time');
    const machineReady = signalBoolean(snapshot, 'machine_ready');
    const autoMode = signalBoolean(snapshot, 'auto_mode');
    const availability = machineReady ? (autoMode ? 98 : 84) : 42;

    setHistory((current) => {
      if (current[current.length - 1]?.timestampMs === snapshot.refreshed_at_ms) {
        return current;
      }

      return [
        ...current.slice(-23),
        {
          timestampMs: snapshot.refreshed_at_ms,
          throughput,
          quality,
          cycle,
          availability,
        },
      ];
    });
  }, [snapshot]);

  const productionSummary = useMemo(() => {
    const throughput = signalNumber(snapshot, 'line_speed');
    const quality = signalNumber(snapshot, 'quality_proxy');
    const productionCount = signalNumber(snapshot, 'production_count');
    const workOrder = signalText(snapshot, 'work_order');
    const recipe = signalText(snapshot, 'active_recipe');
    const cycle = signalNumber(snapshot, 'cycle_time');
    const ready = signalBoolean(snapshot, 'machine_ready');
    const auto = signalBoolean(snapshot, 'auto_mode');

    const oee = Math.min(
      100,
      ((ready ? (auto ? 98 : 84) : 42) * Math.min(quality, 100) * productivityScore(throughput, cycle)) /
        10000
    );
    const downtimeReason = inferDowntimeReason(ready, auto, cycle, quality);
    const alarmCount = inferAlarmCount(ready, auto, cycle, quality);

    return {
      throughput,
      quality,
      productionCount,
      workOrder,
      recipe,
      cycle,
      ready,
      auto,
      availability: ready ? (auto ? 98 : 84) : 42,
      oee,
      alarmCount,
      downtimeReason,
      concerns: buildConcerns(snapshot, quality, cycle, ready, auto),
      alerts: buildOpsAlerts(snapshot, quality, cycle, ready, auto),
    };
  }, [snapshot]);

  const topSignals = useMemo(() => snapshot?.signals ?? [], [snapshot]);

  const handleConnect = async (event: React.FormEvent) => {
    event.preventDefault();
    setLoading(true);
    setError(null);

    try {
      const response = await fetch(`${API_BASE}/api/connect`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          address: address.trim(),
          use_route_path: useRoutePath,
          slot: useRoutePath ? Number(slot) : undefined,
        }),
      });

      const data = await response.json();
      if (!response.ok || !data.success) {
        throw new Error(data.error || data.message || 'Failed to connect');
      }

      setConnectionStatus(data.status);
      pushActivity(`Connected to ${data.status.address}${data.status.use_route_path ? ` via slot ${data.status.slot}` : ''}.`);
      await fetchOverview();
      await fetchSnapshot();
    } catch (connectError) {
      setError(connectError instanceof Error ? connectError.message : 'Connection failed');
    } finally {
      setLoading(false);
    }
  };

  const handleDisconnect = async () => {
    setLoading(true);
    setError(null);

    try {
      const response = await fetch(`${API_BASE}/api/disconnect`, { method: 'POST' });
      const data = await response.json();
      setConnectionStatus(data.status);
      setSnapshot(null);
      setBenchmark(null);
      setHistory([]);
      pushActivity('Disconnected from PLC.');
      await fetchOverview();
    } catch (disconnectError) {
      setError(disconnectError instanceof Error ? disconnectError.message : 'Disconnect failed');
    } finally {
      setLoading(false);
    }
  };

  const handleReadTag = async (event: React.FormEvent) => {
    event.preventDefault();
    setReadResult(null);

    try {
      const response = await fetch(`${API_BASE}/api/read`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ tag_name: readTagName.trim() }),
      });

      const data = await response.json();
      setReadResult({
        value: data.value ?? null,
        dataType: data.data_type ?? null,
        error: data.error ?? null,
      });
      pushActivity(data.error ? `Read failed for ${readTagName.trim()}.` : `Read ${readTagName.trim()} successfully.`);
    } catch (readError) {
      setReadResult({
        value: null,
        dataType: null,
        error: readError instanceof Error ? readError.message : 'Read failed',
      });
    }
  };

  const handleWriteTag = async (event: React.FormEvent) => {
    event.preventDefault();
    setWriteResult(null);

    try {
      const value = parseWriteValue(writeDataType, writeValue);
      const response = await fetch(`${API_BASE}/api/write`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ tag_name: writeTagName.trim(), value }),
      });

      const data = await response.json();
      const result = { success: !!data.success, error: data.error ?? null };
      setWriteResult(result);
      pushActivity(result.success ? `Write applied to ${writeTagName.trim()}.` : `Write rejected for ${writeTagName.trim()}.`);
      await fetchSnapshot();
    } catch (writeError) {
      setWriteResult({
        success: false,
        error: writeError instanceof Error ? writeError.message : 'Write failed',
      });
    }
  };

  const handleRunBenchmark = async () => {
    setBenchmarkLoading(true);
    setError(null);

    try {
      const response = await fetch(`${API_BASE}/api/demo/benchmark`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ iterations: 25 }),
      });

      const data = await response.json();
      if (!response.ok) {
        throw new Error(data.error || 'Benchmark failed');
      }

      setBenchmark(data);
      pushActivity(`Benchmark completed with ${data.iterations} iterations.`);
    } catch (benchError) {
      setError(benchError instanceof Error ? benchError.message : 'Benchmark failed');
    } finally {
      setBenchmarkLoading(false);
    }
  };

  const handleTraceabilitySubmit = async (event: React.FormEvent) => {
    event.preventDefault();
    setError(null);

    try {
      const response = await fetch(`${API_BASE}/api/traceability`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(traceForm),
      });

      const data = await response.json();
      if (!response.ok) {
        throw new Error(data.error || 'Failed to save traceability event');
      }

      pushActivity(`Traceability record saved for ${data.part_id}.`);
      await fetchTraceability();
    } catch (traceError) {
      setError(traceError instanceof Error ? traceError.message : 'Traceability save failed');
    }
  };

  return (
    <div className="app-shell">
      <div className="page-backdrop" />
      <main className="dashboard">
        <section className="hero-panel panel panel-highlight">
          <div className="hero-copy">
            <span className="eyebrow">Rust Axum + React Dashboard</span>
            <h1>Production visibility from the MacBook using the real test tags already in the PLC.</h1>
            <p>
              This version is tuned for manufacturing and production audiences: OEE-style rollups,
              machine availability, quality rate, downtime cues, work order context, benchmarked
              response time, and a lightweight traceability story that stays on the MacBook.
            </p>
            <div className="hero-metrics">
              <div>
                <strong>{snapshot ? snapshot.kpis[0]?.value : '--'}</strong>
                <span>Live scan latency</span>
              </div>
              <div>
                <strong>{productionSummary.workOrder || 'Pending read'}</strong>
                <span>Active work order</span>
              </div>
              <div>
                <strong>{productionSummary.oee ? `${productionSummary.oee.toFixed(1)}%` : '--'}</strong>
                <span>Estimated OEE</span>
              </div>
            </div>
          </div>

          <div className="hero-card">
            <h2>Connection</h2>
            <form className="stack-form" onSubmit={handleConnect}>
              <label>
                PLC Address
                <input
                  value={address}
                  onChange={(event) => setAddress(event.target.value)}
                  placeholder="192.168.0.101:44818"
                  disabled={loading}
                />
              </label>

              <div className="form-inline">
                <label className="toggle-row">
                  <input
                    type="checkbox"
                    checked={useRoutePath}
                    onChange={(event) => setUseRoutePath(event.target.checked)}
                    disabled={loading}
                  />
                  Use ControlLogix route path
                </label>

                <label>
                  Slot
                  <input
                    value={slot}
                    onChange={(event) => setSlot(event.target.value)}
                    disabled={loading || !useRoutePath}
                  />
                </label>
              </div>

              <div className="button-row">
                <button className="btn btn-solid" type="submit" disabled={loading}>
                  {loading ? 'Connecting...' : connectionStatus.connected ? 'Reconnect' : 'Connect'}
                </button>
                <button
                  className="btn btn-ghost"
                  type="button"
                  disabled={loading || !connectionStatus.connected}
                  onClick={handleDisconnect}
                >
                  Disconnect
                </button>
              </div>
            </form>

            <div className={`status-chip ${connectionStatus.connected ? 'online' : 'offline'}`}>
              {connectionStatus.connected ? 'Connected' : 'Offline'}
              <span>{connectionStatus.address ?? 'No session active'}</span>
            </div>
          </div>
        </section>

        {error && <section className="panel error-banner">{error}</section>}

        <section className="grid-two">
          <article className="panel">
            <div className="section-header">
              <div>
                <span className="eyebrow">Controller Overview</span>
                <h2>Identity and validated scope</h2>
              </div>
              <button className="btn btn-ghost" onClick={() => fetchOverview()}>
                Refresh
              </button>
            </div>

            <div className="identity-grid">
              <IdentityItem label="Model" value={connectionStatus.plc_identity?.product_name ?? 'Not available'} />
              <IdentityItem label="Firmware" value={connectionStatus.plc_identity?.firmware ?? 'Not available'} />
              <IdentityItem label="Vendor" value={connectionStatus.plc_identity?.vendor_name ?? 'Not available'} />
              <IdentityItem
                label="Path"
                value={connectionStatus.use_route_path ? `Route path slot ${connectionStatus.slot ?? 0}` : 'Direct connection'}
              />
            </div>

            <div className="capsule-list">
              {overview?.supported_features.map((item) => (
                <span key={item} className="capsule">
                  {item}
                </span>
              ))}
            </div>

            <div className="note-columns">
              <div>
                <h3>Validated Notes</h3>
                <ul>
                  {overview?.validated_notes.map((item) => (
                    <li key={item}>{item}</li>
                  ))}
                </ul>
              </div>
              <div>
                <h3>Known Limits</h3>
                <ul>
                  {overview?.known_limitations.map((item) => (
                    <li key={item}>{item}</li>
                  ))}
                </ul>
              </div>
            </div>
          </article>

          <article className="panel">
            <div className="section-header">
              <div>
                <span className="eyebrow">Production Priorities</span>
                <h2>What operations would watch first</h2>
              </div>
              <button className="btn btn-ghost" onClick={() => fetchSnapshot()} disabled={!connectionStatus.connected}>
                {snapshotLoading ? 'Refreshing...' : 'Refresh Live Data'}
              </button>
            </div>

            <div className="priority-grid">
              <GaugeCard label="Availability" value={productionSummary.availability} suffix="%" tone={productionSummary.availability >= 95 ? 'good' : 'watch'} />
              <GaugeCard label="First Pass Yield" value={productionSummary.quality} suffix="%" tone={productionSummary.quality >= 95 ? 'good' : 'watch'} />
              <GaugeCard label="Cycle Time" value={productionSummary.cycle} suffix=" s" tone={productionSummary.cycle <= 35 ? 'good' : 'watch'} inverse />
            </div>

            <div className="concern-list">
              {productionSummary.concerns.map((item) => (
                <div key={item.title} className={`concern-card ${item.tone}`}>
                  <strong>{item.title}</strong>
                  <span>{item.detail}</span>
                </div>
              ))}
            </div>
          </article>
        </section>

        <section className="grid-two">
          <article className="panel">
            <div className="section-header">
              <div>
                <span className="eyebrow">Downtime Desk</span>
                <h2>Downtime and response context</h2>
              </div>
            </div>

            <div className="downtime-card">
              <div className={`downtime-state ${productionSummary.ready ? 'good' : 'bad'}`}>
                <strong>{productionSummary.ready ? 'Line available' : 'Downtime active'}</strong>
                <span>{productionSummary.downtimeReason}</span>
              </div>

              <div className="downtime-metrics">
                <ProductionTile label="Estimated OEE" value={`${productionSummary.oee.toFixed(1)}%`} detail="Derived from availability, quality, and runtime pace" />
                <ProductionTile label="Active Alarms" value={String(productionSummary.alarmCount)} detail="Derived alert count for this demo pass" />
                <ProductionTile label="Run Mode" value={productionSummary.auto ? 'Auto' : 'Manual / Setup'} detail="From the program-scoped auto mode signal" />
              </div>
            </div>
          </article>

          <article className="panel">
            <div className="section-header">
              <div>
                <span className="eyebrow">Alarm Console</span>
                <h2>Operator-facing alerts</h2>
              </div>
            </div>

            <div className="alarm-list">
              {productionSummary.alerts.map((item) => (
                <div key={item.title} className={`alarm-card ${item.tone}`}>
                  <strong>{item.title}</strong>
                  <span>{item.detail}</span>
                </div>
              ))}
            </div>
          </article>
        </section>

        <section className="panel">
          <div className="section-header">
            <div>
              <span className="eyebrow">Live Operations</span>
              <h2>Live line snapshot</h2>
            </div>
            <div className="live-meta">
              <span>{snapshot ? formatTimestamp(snapshot.refreshed_at_ms) : 'No refresh yet'}</span>
              <span>{snapshot ? `${snapshot.latency_ms.toFixed(2)} ms` : '--'}</span>
            </div>
          </div>

          <div className="kpi-grid">
            {snapshot?.kpis.map((item) => (
              <div key={item.key} className={`kpi-card tone-${item.tone}`}>
                <span>{item.label}</span>
                <strong>{item.value}</strong>
                <small>{item.detail}</small>
              </div>
            ))}
          </div>

          <div className="production-board">
            <ProductionTile label="Work Order" value={productionSummary.workOrder || '--'} detail="Program:TestProgram.gTestArray_DINT[5]" />
            <ProductionTile label="Recipe / SKU" value={productionSummary.recipe || '--'} detail="Program:TestProgram.gTestUDT.Member5_String" />
            <ProductionTile label="Total Count" value={formatNumber(productionSummary.productionCount)} detail="Program:TestProgram.gTestArray_DINT[0]" />
            <ProductionTile label="Cell State" value={productionSummary.ready ? (productionSummary.auto ? 'Producing' : 'Ready / Manual') : 'Stopped / Attention'} detail="From machine_ready and auto_mode" />
          </div>

          <div className="signal-grid">
            {topSignals.map((signal) => (
              <SignalCard key={signal.key} signal={signal} />
            ))}
          </div>
        </section>

        <section className="grid-two charts-layout">
          <article className="panel">
            <div className="section-header">
              <div>
                <span className="eyebrow">Trend View</span>
                <h2>Throughput and yield over time</h2>
              </div>
            </div>

            <div className="chart-stack">
              <TrendChart
                title="Throughput"
                subtitle="Program:TestProgram.gTestArray_REAL[0]"
                color="#2c909a"
                data={history.map((point) => point.throughput)}
                labels={history.map((point) => shortTime(point.timestampMs))}
              />
              <TrendChart
                title="Yield Rate"
                subtitle="Program:TestProgram.gTestArray_REAL[1]"
                color="#2ea77c"
                data={history.map((point) => point.quality)}
                labels={history.map((point) => shortTime(point.timestampMs))}
              />
            </div>
          </article>

          <article className="panel">
            <div className="section-header">
              <div>
                <span className="eyebrow">Runtime Trend</span>
                <h2>Cycle time and availability history</h2>
              </div>
            </div>

            <div className="chart-stack">
              <TrendChart
                title="Cycle Time"
                subtitle="Program:TestProgram.gTestUDT.Member2_REAL"
                color="#ea8d45"
                data={history.map((point) => point.cycle)}
                labels={history.map((point) => shortTime(point.timestampMs))}
              />
              <TrendChart
                title="Availability"
                subtitle="Derived from machine_ready + auto_mode"
                color="#214f5c"
                data={history.map((point) => point.availability)}
                labels={history.map((point) => shortTime(point.timestampMs))}
              />
            </div>
          </article>
        </section>

        <section className="grid-two">
          <article className="panel">
            <div className="section-header">
              <div>
                <span className="eyebrow">PLC Workstation</span>
                <h2>Direct read and write</h2>
              </div>
            </div>

            <div className="workstation-grid">
              <form className="stack-form" onSubmit={handleReadTag}>
                <h3>Read tag</h3>
                <label>
                  Tag name
                  <input value={readTagName} onChange={(event) => setReadTagName(event.target.value)} />
                </label>
                <button className="btn btn-solid" type="submit" disabled={!connectionStatus.connected}>
                  Read
                </button>
                {readResult && (
                  <div className={`result-card ${readResult.error ? 'error' : 'success'}`}>
                    {readResult.error ? (
                      <span>{readResult.error}</span>
                    ) : (
                      <>
                        <strong>{formatPlcValue(readResult.value)}</strong>
                        <span>{readResult.dataType}</span>
                      </>
                    )}
                  </div>
                )}
              </form>

              <form className="stack-form" onSubmit={handleWriteTag}>
                <h3>Write tag</h3>
                <label>
                  Tag name
                  <input value={writeTagName} onChange={(event) => setWriteTagName(event.target.value)} />
                </label>
                <div className="form-inline">
                  <label>
                    Data type
                    <select value={writeDataType} onChange={(event) => setWriteDataType(event.target.value as DataType)}>
                      {DATA_TYPES.map((type) => (
                        <option key={type} value={type}>
                          {type}
                        </option>
                      ))}
                    </select>
                  </label>

                  <label>
                    Value
                    <input value={writeValue} onChange={(event) => setWriteValue(event.target.value)} />
                  </label>
                </div>
                <button className="btn btn-solid" type="submit" disabled={!connectionStatus.connected}>
                  Write
                </button>
                {writeResult && (
                  <div className={`result-card ${writeResult.success ? 'success' : 'error'}`}>
                    <span>{writeResult.success ? 'Write completed.' : writeResult.error}</span>
                  </div>
                )}
              </form>
            </div>
          </article>

          <article className="panel">
            <div className="section-header">
              <div>
                <span className="eyebrow">Speed Demo</span>
                <h2>Polling efficiency and response speed</h2>
              </div>
              <button className="btn btn-solid" onClick={handleRunBenchmark} disabled={!connectionStatus.connected || benchmarkLoading}>
                {benchmarkLoading ? 'Running...' : 'Run 25-Iteration Benchmark'}
              </button>
            </div>

            <div className="benchmark-table">
              <div className="benchmark-row benchmark-head">
                <span>Operation</span>
                <span>Avg call</span>
                <span>Ops/sec</span>
              </div>
              {benchmark?.metrics.map((metric) => (
                <div key={metric.name} className="benchmark-row">
                  <span>{metric.name}</span>
                  <span>{metric.avg_call_ms.toFixed(2)} ms</span>
                  <span>{metric.ops_per_sec.toFixed(0)}</span>
                </div>
              ))}
            </div>

            <ul className="activity-list compact">
              {activity.map((item, index) => (
                <li key={`${item}-${index}`}>{item}</li>
              ))}
            </ul>
          </article>
        </section>

        <section className="grid-two">
          <article className="panel">
            <div className="section-header">
              <div>
                <span className="eyebrow">Traceability</span>
                <h2>Part-tracking event capture</h2>
              </div>
            </div>

            <form className="stack-form" onSubmit={handleTraceabilitySubmit}>
              <div className="form-inline">
                <label>
                  Part ID
                  <input
                    value={traceForm.part_id}
                    onChange={(event) => setTraceForm({ ...traceForm, part_id: event.target.value })}
                  />
                </label>
                <label>
                  Product Code
                  <input
                    value={traceForm.product_code}
                    onChange={(event) => setTraceForm({ ...traceForm, product_code: event.target.value })}
                  />
                </label>
              </div>

              <div className="form-inline">
                <label>
                  Lot Code
                  <input
                    value={traceForm.lot_code}
                    onChange={(event) => setTraceForm({ ...traceForm, lot_code: event.target.value })}
                  />
                </label>
                <label>
                  Station
                  <input
                    value={traceForm.station}
                    onChange={(event) => setTraceForm({ ...traceForm, station: event.target.value })}
                  />
                </label>
              </div>

              <div className="form-inline">
                <label>
                  Status
                  <select value={traceForm.status} onChange={(event) => setTraceForm({ ...traceForm, status: event.target.value })}>
                    <option>Completed</option>
                    <option>In Process</option>
                    <option>Hold</option>
                    <option>Rejected</option>
                  </select>
                </label>
                <label className="notes-field">
                  Notes
                  <textarea value={traceForm.notes} onChange={(event) => setTraceForm({ ...traceForm, notes: event.target.value })} />
                </label>
              </div>

              <button className="btn btn-solid" type="submit">
                Save Traceability Record
              </button>
            </form>
          </article>

          <article className="panel">
            <div className="section-header">
              <div>
                <span className="eyebrow">Recorded Events</span>
                <h2>Local persistence for the demo</h2>
              </div>
              <button className="btn btn-ghost" onClick={() => fetchTraceability()}>
                Reload
              </button>
            </div>

            <div className="trace-list">
              {traceability.map((item) => (
                <div key={item.id} className="trace-item">
                  <div>
                    <strong>{item.part_id}</strong>
                    <span>{item.product_code}</span>
                  </div>
                  <div>
                    <span>{item.station}</span>
                    <span>{item.status}</span>
                  </div>
                  <div>
                    <span>{item.lot_code}</span>
                    <span>{formatTimestamp(item.timestamp_ms)}</span>
                  </div>
                </div>
              ))}
            </div>
          </article>
        </section>
      </main>
    </div>
  );
}

function IdentityItem({ label, value }: { label: string; value: string }) {
  return (
    <div className="identity-item">
      <span>{label}</span>
      <strong>{value}</strong>
    </div>
  );
}

function ProductionTile({ label, value, detail }: { label: string; value: string; detail: string }) {
  return (
    <div className="production-tile">
      <span>{label}</span>
      <strong>{value}</strong>
      <small>{detail}</small>
    </div>
  );
}

function SignalCard({ signal }: { signal: SignalReading }) {
  return (
    <div className={`signal-card ${signal.quality === 'good' ? 'good' : 'bad'}`}>
      <div className="signal-card-top">
        <span>{signal.label}</span>
        <small>{signal.data_type ?? 'N/A'}</small>
      </div>
      <strong>{signal.value_text ?? 'Unavailable'}</strong>
      <p>{signal.tag_name}</p>
      {signal.error && <small className="signal-error">{signal.error}</small>}
    </div>
  );
}

function GaugeCard({
  label,
  value,
  suffix,
  tone,
  inverse,
}: {
  label: string;
  value: number;
  suffix: string;
  tone: string;
  inverse?: boolean;
}) {
  const normalized = Math.max(0, Math.min(100, inverse ? 100 - Math.min(value * 2, 100) : value));
  return (
    <div className={`gauge-card tone-${tone}`}>
      <div className="gauge-shell">
        <div className="gauge-fill" style={{ '--gauge': `${normalized}%` } as React.CSSProperties} />
        <strong>{Number.isFinite(value) ? `${value.toFixed(1)}${suffix}` : '--'}</strong>
      </div>
      <span>{label}</span>
    </div>
  );
}

function TrendChart({
  title,
  subtitle,
  color,
  data,
  labels,
}: {
  title: string;
  subtitle: string;
  color: string;
  data: number[];
  labels: string[];
}) {
  const width = 520;
  const height = 160;
  const safeData = data.length > 1 ? data : [0, 0];
  const max = Math.max(...safeData, 1);
  const min = Math.min(...safeData, 0);
  const range = Math.max(max - min, 1);

  const points = safeData
    .map((value, index) => {
      const x = (index / (safeData.length - 1)) * (width - 24) + 12;
      const y = height - 12 - ((value - min) / range) * (height - 28);
      return `${x},${y}`;
    })
    .join(' ');

  return (
    <div className="trend-card">
      <div className="trend-header">
        <div>
          <strong>{title}</strong>
          <span>{subtitle}</span>
        </div>
        <small>{safeData[safeData.length - 1]?.toFixed(2) ?? '--'}</small>
      </div>
      <svg viewBox={`0 0 ${width} ${height}`} className="trend-svg" role="img" aria-label={title}>
        <polyline
          fill="none"
          stroke={color}
          strokeWidth="4"
          strokeLinecap="round"
          strokeLinejoin="round"
          points={points}
        />
      </svg>
      <div className="trend-labels">
        <span>{labels[0] ?? '--'}</span>
        <span>{labels[labels.length - 1] ?? '--'}</span>
      </div>
    </div>
  );
}

function parseWriteValue(type: DataType, rawValue: string): PlcValueJson {
  switch (type) {
    case 'BOOL':
      return {
        type,
        value: rawValue.toLowerCase() === 'true' || rawValue === '1',
      };
    case 'REAL':
    case 'LREAL':
      return { type, value: Number(rawValue) };
    case 'STRING':
      return { type, value: rawValue };
    default:
      return { type, value: Number(rawValue) };
  }
}

function formatPlcValue(value: PlcValueJson | null): string {
  if (!value) {
    return 'No value';
  }

  if (value.type === 'BOOL') {
    return value.value ? 'True' : 'False';
  }

  return String(value.value);
}

function formatTimestamp(timestampMs: number): string {
  return new Date(timestampMs).toLocaleString();
}

function shortTime(timestampMs: number): string {
  return new Date(timestampMs).toLocaleTimeString([], {
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit',
  });
}

function formatNumber(value: number): string {
  return Number.isFinite(value) ? value.toFixed(0) : '--';
}

function signalByKey(snapshot: SnapshotResponse | null, key: string): SignalReading | undefined {
  return snapshot?.signals.find((signal) => signal.key === key);
}

function signalNumber(snapshot: SnapshotResponse | null, key: string): number {
  const value = signalByKey(snapshot, key)?.value_text;
  if (!value) {
    return 0;
  }
  const parsed = Number(value);
  return Number.isFinite(parsed) ? parsed : 0;
}

function signalBoolean(snapshot: SnapshotResponse | null, key: string): boolean {
  const value = signalByKey(snapshot, key)?.value_text?.toLowerCase();
  return value === 'true';
}

function signalText(snapshot: SnapshotResponse | null, key: string): string {
  return signalByKey(snapshot, key)?.value_text ?? '';
}

function buildConcerns(
  snapshot: SnapshotResponse | null,
  quality: number,
  cycle: number,
  ready: boolean,
  auto: boolean
) {
  const latency = snapshot?.latency_ms ?? 0;
  return [
    {
      title: ready ? 'Machine state healthy' : 'Machine needs attention',
      detail: ready ? 'Readiness signal is active on the current poll.' : 'Readiness signal is off. Review the live tag tiles.',
      tone: ready ? 'good' : 'bad',
    },
    {
      title: auto ? 'Auto mode active' : 'Manual or fallback mode',
      detail: auto ? 'Program-scoped BOOL indicates the cell is in auto.' : 'Auto mode is not active, so output may not reflect target throughput.',
      tone: auto ? 'good' : 'watch',
    },
    {
      title: quality >= 95 ? 'Yield trend on target' : 'Yield trend below target',
      detail: `Current first-pass yield estimate is ${quality.toFixed(1)}%.`,
      tone: quality >= 95 ? 'good' : 'watch',
    },
    {
      title: latency <= 10 ? 'Dashboard latency healthy' : 'Latency above preferred band',
      detail: `Latest batch snapshot returned in ${latency.toFixed(2)} ms.`,
      tone: latency <= 10 ? 'good' : 'watch',
    },
    {
      title: cycle <= 35 ? 'Cycle time steady' : 'Cycle time elevated',
      detail: `Current cycle proxy is ${cycle.toFixed(2)} seconds.`,
      tone: cycle <= 35 ? 'good' : 'watch',
    },
  ];
}

function buildOpsAlerts(
  snapshot: SnapshotResponse | null,
  quality: number,
  cycle: number,
  ready: boolean,
  auto: boolean
): OpsAlert[] {
  const latency = snapshot?.latency_ms ?? 0;
  return [
    {
      title: ready ? 'No downtime event active' : 'Downtime event active',
      detail: ready ? 'The readiness bit is on for the latest scan.' : 'The readiness bit is off. Treat this as an active stop condition.',
      tone: ready ? 'good' : 'bad',
    },
    {
      title: auto ? 'Auto cycle enabled' : 'Manual intervention likely',
      detail: auto ? 'The line is flagged in automatic mode.' : 'Auto mode is off, so throughput and OEE are likely suppressed.',
      tone: auto ? 'good' : 'watch',
    },
    {
      title: quality >= 95 ? 'Scrap risk low' : 'Scrap risk rising',
      detail: `Current yield estimate is ${quality.toFixed(1)}%.`,
      tone: quality >= 95 ? 'good' : 'watch',
    },
    {
      title: cycle <= 35 ? 'Cycle target met' : 'Cycle target missed',
      detail: `Current cycle estimate is ${cycle.toFixed(2)} seconds.`,
      tone: cycle <= 35 ? 'good' : 'watch',
    },
    {
      title: latency <= 10 ? 'Telemetry healthy' : 'Telemetry lag detected',
      detail: `Latest batch snapshot returned in ${latency.toFixed(2)} ms.`,
      tone: latency <= 10 ? 'good' : 'watch',
    },
  ];
}

function productivityScore(throughput: number, cycle: number): number {
  const throughputScore = Math.min(Math.max(throughput, 0), 100);
  const cyclePenalty = Math.min(Math.max(cycle / 6, 0), 100);
  return Math.max(20, Math.min(100, throughputScore - cyclePenalty + 40));
}

function inferDowntimeReason(ready: boolean, auto: boolean, cycle: number, quality: number): string {
  if (!ready && !auto) {
    return 'Cell is stopped with auto mode disabled. Likely setup, hold, or operator intervention.';
  }
  if (!ready) {
    return 'Machine ready signal is off. Treat this as an unplanned stop until cleared.';
  }
  if (!auto) {
    return 'Line is available but not in auto. Production may be in manual recovery or setup mode.';
  }
  if (cycle > 35) {
    return 'Line is running, but the cycle estimate is above target and should be reviewed.';
  }
  if (quality < 95) {
    return 'Line is running, but yield trend suggests increased scrap or quality loss.';
  }
  return 'No active downtime indicator. Line conditions are within the current demo band.';
}

function inferAlarmCount(ready: boolean, auto: boolean, cycle: number, quality: number): number {
  let alarms = 0;
  if (!ready) alarms += 2;
  if (!auto) alarms += 1;
  if (cycle > 35) alarms += 1;
  if (quality < 95) alarms += 1;
  return alarms;
}

export default App;
