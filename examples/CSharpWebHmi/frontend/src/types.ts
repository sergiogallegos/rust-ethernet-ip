export type Quality = 'Good' | 'Bad';
export type ConnectionState = 'Connected' | 'Degraded' | 'Simulated' | 'Fallback';

export interface DashboardSignal {
  id: string;
  label: string;
  tag: string;
  scope: string;
  dataType: string;
  value: string | number | boolean | null;
  displayValue: string;
  unit: string | null;
  quality: Quality;
}

export interface ScopeSummary {
  id: string;
  label: string;
  detail: string;
  good: number;
  total: number;
  state: 'Healthy' | 'Attention';
}

export interface DashboardNotice {
  severity: 'Info' | 'Warning' | 'Alarm';
  code: string;
  message: string;
}

export interface DashboardSnapshot {
  mode: string;
  connectionState: ConnectionState;
  target: string;
  slot: number;
  controller: string;
  firmware: string;
  libraryVersion: string;
  abiVersion: number;
  writesEnabled: boolean;
  refreshedAt: string;
  scanTimeMs: number;
  goodSignals: number;
  totalSignals: number;
  operatorMessage: string;
  signals: DashboardSignal[];
  scopes: ScopeSummary[];
  analogProfile: number[];
  counterProfile: number[];
  digitalProfile: boolean[];
  notices: DashboardNotice[];
}

export interface CommandResult {
  success: boolean;
  message: string;
}
