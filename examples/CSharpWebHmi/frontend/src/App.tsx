import { useCallback, useEffect, useMemo, useState } from 'react';
import type { CommandResult, DashboardSignal, DashboardSnapshot } from './types';

type IconName = 'overview' | 'signals' | 'structure' | 'diagnostics' | 'refresh' | 'pulse' | 'route';

const POLL_INTERVAL_MS = 1800;

function Icon({ name, size = 22 }: { name: IconName; size?: number }) {
  const common = {
    width: size,
    height: size,
    viewBox: '0 0 24 24',
    fill: 'none',
    stroke: 'currentColor',
    strokeWidth: 1.9,
    strokeLinecap: 'round' as const,
    strokeLinejoin: 'round' as const,
    'aria-hidden': true,
  };

  const paths: Record<IconName, React.ReactNode> = {
    overview: <><rect x="3" y="3" width="7" height="7" /><rect x="14" y="3" width="7" height="7" /><rect x="3" y="14" width="7" height="7" /><rect x="14" y="14" width="7" height="7" /></>,
    signals: <><path d="M4 17V9" /><path d="M9 17V5" /><path d="M14 17v-7" /><path d="M19 17V3" /><path d="M3 21h18" /></>,
    structure: <><rect x="4" y="3" width="6" height="5" /><rect x="14" y="16" width="6" height="5" /><rect x="4" y="16" width="6" height="5" /><path d="M7 8v4h10v4" /><path d="M7 12v4" /></>,
    diagnostics: <><circle cx="12" cy="12" r="3" /><path d="M19.4 15a1.7 1.7 0 0 0 .3 1.9l.1.1-2.8 2.8-.1-.1a1.7 1.7 0 0 0-1.9-.3 1.7 1.7 0 0 0-1 1.6v.2h-4V21a1.7 1.7 0 0 0-1-1.6 1.7 1.7 0 0 0-1.9.3l-.1.1L4.2 17l.1-.1a1.7 1.7 0 0 0 .3-1.9A1.7 1.7 0 0 0 3 14H2.8v-4H3a1.7 1.7 0 0 0 1.6-1 1.7 1.7 0 0 0-.3-1.9L4.2 7 7 4.2l.1.1A1.7 1.7 0 0 0 9 4a1.7 1.7 0 0 0 1-1.6v-.2h4v.2A1.7 1.7 0 0 0 15 4a1.7 1.7 0 0 0 1.9.3l.1-.1L19.8 7l-.1.1a1.7 1.7 0 0 0-.3 1.9 1.7 1.7 0 0 0 1.6 1h.2v4H21a1.7 1.7 0 0 0-1.6 1Z" /></>,
    refresh: <><path d="M20 6v5h-5" /><path d="M4 18v-5h5" /><path d="M18.2 9A7 7 0 0 0 6.5 6.5L4 9" /><path d="M5.8 15A7 7 0 0 0 17.5 17.5L20 15" /></>,
    pulse: <><path d="M3 12h4l2-7 4 14 2-7h6" /></>,
    route: <><circle cx="5" cy="5" r="2" /><circle cx="19" cy="19" r="2" /><path d="M7 5h5a3 3 0 0 1 3 3v8" /><path d="m12 13 3 3 3-3" /></>,
  };

  return <svg {...common}>{paths[name]}</svg>;
}

function App() {
  const [snapshot, setSnapshot] = useState<DashboardSnapshot | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);
  const [autoRefresh, setAutoRefresh] = useState(true);
  const [clock, setClock] = useState(new Date());
  const [commandMessage, setCommandMessage] = useState<string | null>(null);

  const loadSnapshot = useCallback(async () => {
    try {
      const response = await fetch('/api/dashboard', { cache: 'no-store' });
      if (!response.ok) throw new Error(`Dashboard API returned ${response.status}`);
      setSnapshot(await response.json() as DashboardSnapshot);
      setError(null);
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : 'Dashboard data is unavailable');
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    void loadSnapshot();
  }, [loadSnapshot]);

  useEffect(() => {
    if (!autoRefresh) return;
    const timer = window.setInterval(() => void loadSnapshot(), POLL_INTERVAL_MS);
    return () => window.clearInterval(timer);
  }, [autoRefresh, loadSnapshot]);

  useEffect(() => {
    const timer = window.setInterval(() => setClock(new Date()), 1000);
    return () => window.clearInterval(timer);
  }, []);

  const pulseTestTag = async () => {
    setCommandMessage('Pulsing allowlisted test tag…');
    try {
      const response = await fetch('/api/dashboard/pulse', { method: 'POST' });
      const result = await response.json() as CommandResult;
      setCommandMessage(result.message);
      if (result.success) await loadSnapshot();
    } catch {
      setCommandMessage('Test pulse request failed.');
    }
  };

  const signalById = useMemo(() => new Map(snapshot?.signals.map((signal) => [signal.id, signal]) ?? []), [snapshot]);
  const udtSignals = snapshot?.signals.filter((signal) => signal.scope === 'UDT Structure') ?? [];
  const programSignals = snapshot?.signals.filter((signal) => signal.scope === 'Program Scope') ?? [];
  const qualityPercent = snapshot ? Math.round((snapshot.goodSignals / snapshot.totalSignals) * 100) : 0;
  const isLive = snapshot?.mode === 'Live PLC';
  const isRouted = snapshot?.target.startsWith('Communication module route') ?? true;

  return (
    <div className="hmi-shell">
      <header className="topbar">
        <div className="brand-lockup" aria-label="Rust EtherNet/IP Web HMI">
          <div className="brand-mark"><span>EI</span></div>
          <div><strong>Rust EtherNet/IP</strong><small>Web HMI Demo</small></div>
        </div>
        <div className="header-divider" />
        <div className="cell-id"><span>VALIDATION CELL</span><strong>CELL-01</strong></div>
        <div className={`connection-chip ${isLive ? 'live' : 'sim'}`}>
          <span className="status-dot" />
          {snapshot?.connectionState ?? (loading ? 'Connecting' : 'Offline')}
        </div>
        <div className="topbar-spacer" />
        <div className="header-metric"><span>CONTROLLER</span><strong>{snapshot?.controller ?? 'Loading target'}</strong></div>
        <time className="clock">{clock.toLocaleTimeString([], { hour12: false })}</time>
        <div className="operator-chip"><span className="operator-avatar">OP</span><span>Demo Operator</span></div>
      </header>

      <aside className="nav-rail" aria-label="Demo navigation">
        <button className="nav-item active" type="button"><Icon name="overview" /><span>Overview</span></button>
        <button className="nav-item" type="button"><Icon name="signals" /><span>Signals</span></button>
        <button className="nav-item" type="button"><Icon name="structure" /><span>Structures</span></button>
        <button className="nav-item" type="button"><Icon name="diagnostics" /><span>Diagnostics</span></button>
        <div className="rail-version"><span>DEMO</span><strong>v1.2.1</strong></div>
      </aside>

      <div className={`message-strip ${error ? 'alarm' : ''}`}>
        <span className="message-code">{error ? 'C-500' : 'S-100'}</span>
        <strong>{error ?? snapshot?.operatorMessage ?? 'Establishing dashboard data source…'}</strong>
        <span className="message-action">{error ? '{Check backend connection}' : isLive ? '{Monitoring live tags}' : '{Simulation active}'}</span>
      </div>

      <main className="workspace">
        <section className="screen-heading">
          <div><p className="eyebrow">LOGIX / ETHERNET/IP EXPLICIT MESSAGING</p><h1>Validation Cell Overview</h1></div>
          <div className="heading-meta"><span>Last scan</span><strong>{snapshot ? formatTime(snapshot.refreshedAt) : '—'}</strong></div>
        </section>

        <section className="kpi-grid" aria-label="Connection metrics">
          <MetricCard label="Signal Quality" value={`${qualityPercent}`} unit="%" detail={`${snapshot?.goodSignals ?? 0} / ${snapshot?.totalSignals ?? 0} good`} tone={qualityPercent === 100 ? 'ok' : 'alarm'} />
          <MetricCard label="Acquisition Scan" value={snapshot?.scanTimeMs.toFixed(1) ?? '—'} unit="ms" detail="C# wrapper read cycle" />
          <MetricCard label="Connection Path" value={isRouted ? `SLOT ${snapshot?.slot ?? 0}` : 'DIRECT'} detail={snapshot?.firmware ? `Firmware ${snapshot.firmware}` : 'Awaiting target'} icon="route" />
          <MetricCard label="Native Core" value={`v${snapshot?.libraryVersion ?? '1.2.1'}`} detail={`C ABI ${snapshot?.abiVersion ?? 3} · Rust core`} />
        </section>

        <section className="dashboard-grid">
          <article className="panel process-panel">
            <PanelHeader overline="LIVE PROFILE" title="Controller Analog Array" meta="gTestArray_REAL[0..7]" />
            <div className="chart-wrap">
              <LineChart values={snapshot?.analogProfile ?? []} />
              <div className="chart-legend"><span><i className="legend-line" />Live REAL values</span><span>8 channels / current scan</span></div>
            </div>
            <div className="analog-readouts">
              {[0, 1, 2, 3].map((index) => <SignalReadout key={index} signal={signalById.get(`real-${index}`)} />)}
            </div>
          </article>

          <article className="panel scope-panel">
            <PanelHeader overline="COMMUNICATION PATH" title="Validated Data Domains" meta={snapshot?.target ?? 'Loading route'} />
            <div className="scope-flow">
              {snapshot?.scopes.map((scope, index) => (
                <div className="scope-step" key={scope.id}>
                  <div className={`scope-node ${scope.state === 'Healthy' ? 'healthy' : 'attention'}`}>
                    <span className="scope-index">0{index + 1}</span>
                    <span className="scope-lamp" />
                    <strong>{scope.label}</strong>
                    <small>{scope.detail}</small>
                    <div className="scope-count"><b>{scope.good}</b><span>/ {scope.total || snapshot.totalSignals} good</span></div>
                  </div>
                  {index < snapshot.scopes.length - 1 && <span className="scope-connector" />}
                </div>
              )) ?? <PanelSkeleton />}
            </div>
          </article>

          <article className="panel digital-panel">
            <PanelHeader overline="PACKED BOOL ACCESS" title="Digital Signal Bank" meta="gTestArray_BOOL[0..11]" />
            <div className="digital-grid">
              {(snapshot?.digitalProfile ?? Array.from({ length: 12 }, () => false)).map((value, index) => (
                <div className="digital-channel" key={index}>
                  <span className={`pilot ${value ? 'on' : ''}`} />
                  <strong>{String(index + 1).padStart(2, '0')}</strong>
                  <small>{value ? 'ON' : 'OFF'}</small>
                </div>
              ))}
            </div>
          </article>

          <article className="panel structure-panel">
            <PanelHeader overline="STRUCTURED DATA" title="UDT Member Snapshot" meta="gTestUDT" />
            <div className="structure-table">
              {udtSignals.length > 0 ? udtSignals.map((signal) => <StructureRow key={signal.id} signal={signal} />) : <PanelSkeleton />}
            </div>
          </article>

          <article className="panel program-panel">
            <PanelHeader overline="PROGRAM SCOPE" title="TestProgram Signals" meta="Program:TestProgram" />
            <div className="program-values">
              {programSignals.length > 0 ? programSignals.map((signal) => <SignalReadout key={signal.id} signal={signal} compact />) : <PanelSkeleton />}
            </div>
          </article>

          <article className="panel notices-panel">
            <PanelHeader overline="SYSTEM MESSAGES" title="Communication Status" meta={`${snapshot?.notices.length ?? 0} active`} />
            <div className="notice-list">
              {snapshot?.notices.map((notice) => (
                <div className={`notice ${notice.severity.toLowerCase()}`} key={notice.code}>
                  <span className="notice-dot" /><div><strong>{notice.code}</strong><p>{notice.message}</p></div>
                </div>
              )) ?? <PanelSkeleton />}
              {commandMessage && <div className="notice info"><span className="notice-dot" /><div><strong>COMMAND</strong><p>{commandMessage}</p></div></div>}
            </div>
          </article>
        </section>
      </main>

      <section className="control-dock" aria-label="Demo controls">
        <div className="dock-label"><span>MONITORING</span><strong>Dashboard Controls</strong></div>
        <label className="toggle-control">
          <input type="checkbox" checked={autoRefresh} onChange={(event) => setAutoRefresh(event.target.checked)} />
          <span className="toggle-track"><span /></span>
          <span><b>Auto Refresh</b><small>{POLL_INTERVAL_MS / 1000}s interval</small></span>
        </label>
        <button className="control-button primary" type="button" onClick={() => void loadSnapshot()} disabled={loading}>
          <Icon name="refresh" /><span><b>Refresh Now</b><small>Read configured tags</small></span>
        </button>
        <button className="control-button" type="button" onClick={() => void pulseTestTag()} disabled={!snapshot?.writesEnabled} title={snapshot?.writesEnabled ? 'Pulse and restore the allowlisted BOOL tag' : 'Set HMI_ALLOW_WRITES=true in live mode'}>
          <Icon name="pulse" /><span><b>Test Pulse</b><small>{snapshot?.writesEnabled ? 'Pulse + restore BOOL[0]' : 'Writes disabled'}</small></span>
        </button>
        <div className="dock-spacer" />
        <div className={`mode-panel ${isLive ? 'live' : 'sim'}`}><span className="status-dot" /><div><small>DATA SOURCE</small><strong>{snapshot?.mode ?? 'Starting'}</strong></div></div>
      </section>

      <footer className="statusbar">
        <span><i className={`footer-dot ${isLive ? 'ok' : 'info'}`} />PLC: {snapshot?.connectionState ?? 'Starting'}</span>
        <span>Route: Slot {snapshot?.slot ?? 0}</span>
        <span>Scan: {snapshot?.scanTimeMs.toFixed(1) ?? '—'} ms</span>
        <span>Protocol: EtherNet/IP · CIP</span>
        <span className="statusbar-spacer" />
        <span className="mono">RUST CORE {snapshot?.libraryVersion ?? '1.2.1'} · C# · REACT</span>
      </footer>
    </div>
  );
}

function MetricCard({ label, value, unit, detail, tone, icon }: { label: string; value: string; unit?: string; detail: string; tone?: 'ok' | 'alarm'; icon?: IconName }) {
  return <article className={`metric-card ${tone ?? ''}`}>
    <div className="metric-top"><span>{label}</span>{icon ? <Icon name={icon} size={19} /> : <span className={`metric-state ${tone ?? ''}`} />}</div>
    <div className="metric-value">{value}<small>{unit}</small></div>
    <p>{detail}</p>
  </article>;
}

function PanelHeader({ overline, title, meta }: { overline: string; title: string; meta: string }) {
  return <header className="panel-header"><div><span>{overline}</span><h2>{title}</h2></div><code>{meta}</code></header>;
}

function SignalReadout({ signal, compact = false }: { signal?: DashboardSignal; compact?: boolean }) {
  return <div className={`signal-readout ${compact ? 'compact' : ''}`}>
    <div><span>{signal?.label ?? 'Loading signal'}</span><code>{signal?.dataType ?? '—'}</code></div>
    <strong>{signal?.displayValue ?? '—'}{signal?.unit && <small>{signal.unit}</small>}</strong>
    {compact && <p>{signal?.tag ?? 'Awaiting tag'}</p>}
  </div>;
}

function StructureRow({ signal }: { signal: DashboardSignal }) {
  return <div className="structure-row">
    <span className={`quality-dot ${signal.quality.toLowerCase()}`} />
    <div><strong>{signal.label}</strong><code>{signal.tag.split('.').at(-1)}</code></div>
    <span className="type-badge">{signal.dataType}</span>
    <b>{signal.displayValue}</b>
  </div>;
}

function LineChart({ values }: { values: number[] }) {
  const width = 640;
  const height = 190;
  const padding = 18;
  const safeValues = values.length > 1 ? values : [0, 0, 0, 0, 0, 0, 0, 0];
  const min = Math.min(...safeValues);
  const max = Math.max(...safeValues);
  const range = max - min || 1;
  const points = safeValues.map((value, index) => {
    const x = padding + (index / (safeValues.length - 1)) * (width - padding * 2);
    const y = height - padding - ((value - min) / range) * (height - padding * 2);
    return `${x},${y}`;
  }).join(' ');

  return <svg className="line-chart" viewBox={`0 0 ${width} ${height}`} role="img" aria-label="Current controller REAL array profile">
    {[0.2, 0.4, 0.6, 0.8].map((fraction) => <line key={fraction} x1={padding} y1={height * fraction} x2={width - padding} y2={height * fraction} className="grid-line" />)}
    <polyline points={points} className="profile-line" />
    {points.split(' ').map((point, index) => {
      const [cx, cy] = point.split(',');
      return <circle key={index} cx={cx} cy={cy} r="4" className="profile-point" />;
    })}
  </svg>;
}

function PanelSkeleton() {
  return <div className="skeleton"><span /><span /><span /></div>;
}

function formatTime(value: string) {
  return new Date(value).toLocaleTimeString([], { hour12: false });
}

export default App;
