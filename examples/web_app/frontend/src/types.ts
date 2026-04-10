export interface PlcIdentity {
  vendor_id: number;
  vendor_name: string;
  device_type: number;
  device_type_name: string;
  product_code: number;
  product_name: string;
  revision_major: number;
  revision_minor: number;
  firmware: string;
  status: number;
  serial_number: number;
}

export interface ConnectionStatus {
  connected: boolean;
  address: string | null;
  use_route_path: boolean;
  slot: number | null;
  connected_at_ms: number | null;
  plc_identity: PlcIdentity | null;
}

export type PlcValueJson =
  | { type: 'BOOL'; value: boolean }
  | { type: 'SINT'; value: number }
  | { type: 'INT'; value: number }
  | { type: 'DINT'; value: number }
  | { type: 'LINT'; value: number }
  | { type: 'USINT'; value: number }
  | { type: 'UINT'; value: number }
  | { type: 'UDINT'; value: number }
  | { type: 'ULINT'; value: number }
  | { type: 'REAL'; value: number }
  | { type: 'LREAL'; value: number }
  | { type: 'STRING'; value: string };

export interface DemoSignalConfig {
  key: string;
  label: string;
  tag_name: string;
  unit: string | null;
  emphasis: string;
}

export interface OverviewResponse {
  status: ConnectionStatus;
  supported_features: string[];
  validated_notes: string[];
  known_limitations: string[];
  default_tags: DemoSignalConfig[];
}

export interface KpiCard {
  key: string;
  label: string;
  value: string;
  tone: 'good' | 'watch' | 'bad' | 'neutral' | string;
  detail: string;
}

export interface SignalReading {
  key: string;
  label: string;
  tag_name: string;
  data_type: string | null;
  unit: string | null;
  value_text: string | null;
  quality: 'good' | 'bad' | string;
  emphasis: string;
  error: string | null;
}

export interface SnapshotResponse {
  refreshed_at_ms: number;
  latency_ms: number;
  kpis: KpiCard[];
  signals: SignalReading[];
}

export interface BenchmarkMetric {
  name: string;
  iterations: number;
  logical_ops: number;
  elapsed_ms: number;
  avg_call_ms: number;
  ops_per_sec: number;
}

export interface BenchmarkResponse {
  iterations: number;
  generated_at_ms: number;
  metrics: BenchmarkMetric[];
}

export interface TraceabilityRecord {
  id: number;
  part_id: string;
  product_code: string;
  lot_code: string;
  station: string;
  status: string;
  notes: string;
  timestamp_ms: number;
}

export interface TraceabilityResponse {
  items: TraceabilityRecord[];
}

export const DATA_TYPES = [
  'BOOL',
  'SINT',
  'INT',
  'DINT',
  'LINT',
  'USINT',
  'UINT',
  'UDINT',
  'ULINT',
  'REAL',
  'LREAL',
  'STRING',
] as const;

export type DataType = typeof DATA_TYPES[number];
