export interface ConnectionStatus {
  connected: boolean;
  address: string | null;
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

