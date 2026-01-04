export type PlcStatus = 'connected' | 'disconnected' | 'connecting' | 'error' | 'unknown';

export interface PlcTagValue {
  tag: string;
  value: boolean | number | string;
}

// API base URL - Next.js rewrites will proxy /api/* to http://localhost:8080/api/*
// Use empty string for relative paths (Next.js will handle the rewrite)
const API_BASE = '';

export async function connectToPlc(ipAddress: string, useRoutePath?: boolean, cpuSlot?: number): Promise<boolean> {
  console.log('connectToPlc called with:', { ipAddress, useRoutePath, cpuSlot, API_BASE });
  const res = await fetch(`${API_BASE}/api/connect`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ ipAddress, useRoutePath, cpuSlot }),
  });
  console.log('connectToPlc response status:', res.status, res.statusText);
  if (!res.ok) {
    const errorText = await res.text();
    console.error('connectToPlc error:', errorText);
    throw new Error(errorText);
  }
  const result = await res.json();
  console.log('connectToPlc result:', result);
  return true;
}

export async function disconnectPlc(): Promise<void> {
  await fetch(`${API_BASE}/api/disconnect`, { method: 'POST' });
}

export async function readTag(tag: string, type: string): Promise<PlcTagValue> {
  const res = await fetch(`${API_BASE}/api/tag?tag=${encodeURIComponent(tag)}&type=${encodeURIComponent(type)}`);
  if (!res.ok) throw new Error(await res.text());
  const data = await res.json();
  return data;
}

export async function writeTag(tag: string, value: any, type: string): Promise<boolean> {
  const res = await fetch(`${API_BASE}/api/tag`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ tag, type, value }),
  });
  if (!res.ok) throw new Error(await res.text());
  return true;
}

export function subscribeToTagUpdates(onUpdate: (data: PlcTagValue) => void): () => void {
  // WebSocket needs full URL - Next.js rewrites don't work for WebSockets
  const wsUrl = typeof window !== 'undefined' 
    ? `ws://${window.location.hostname}:8080/ws`
    : 'ws://localhost:8080/ws';
  const ws = new WebSocket(wsUrl);
  ws.onmessage = (event) => {
    try {
      const data = JSON.parse(event.data);
      onUpdate(data);
    } catch {}
  };
  return () => ws.close();
}

export async function batchReadTags(tags: { tag: string; type: string }[]): Promise<Record<string, any>> {
  const res = await fetch(`${API_BASE}/api/batch`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ tags }),
  });
  if (!res.ok) throw new Error(await res.text());
  return await res.json();
}

export async function batchWriteTags(tagObjs: { tag: string; type: string; value: any }[]): Promise<{ success: boolean; error?: string }> {
  const res = await fetch(`${API_BASE}/api/batch`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ writes: tagObjs }),
  });
  if (!res.ok) throw new Error(await res.text());
  return await res.json();
}

export async function runBenchmark(tag: string, type: string, write: boolean): Promise<{ success: boolean; readCount: number; writeCount: number; elapsedMs: number; readRate: number; writeRate: number; error?: string }> {
  const res = await fetch(`${API_BASE}/api/benchmark`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ tag, type, write }),
  });
  if (!res.ok) throw new Error(await res.text());
  return await res.json();
}

export async function getPlcStatus(): Promise<{ status: PlcStatus }> {
  // Not implemented in backend, placeholder for future
  return { status: 'unknown' };
}

export async function createTestTags(): Promise<{ success: boolean; error?: string }> {
  // Not implemented in backend, placeholder for future
  return { success: false, error: 'Create test tags not implemented' };
}

export async function discoverTag(tag: string): Promise<string> {
  const res = await fetch(`${API_BASE}/api/taginfo?tag=${encodeURIComponent(tag)}`);
  if (!res.ok) throw new Error(await res.text());
  const data = await res.json();
  return data.type as string;
}

export async function debugReadTag(tag: string, typeStr: string): Promise<any> {
  const res = await fetch(`${API_BASE}/api/test-read?tag=${encodeURIComponent(tag)}&type=${encodeURIComponent(typeStr)}`);
  return await res.json();
}

export async function testArrays(testType: 'controller' | 'program' | 'bool' | 'all'): Promise<any> {
  const res = await fetch(`${API_BASE}/api/test-arrays`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ testType }),
  });
  if (!res.ok) throw new Error(await res.text());
  return await res.json();
}

// Array operations
export async function readArrayElement(tagName: string): Promise<{ success: boolean; tag: string; value: any; type: string }> {
  const res = await fetch(`${API_BASE}/api/array/${encodeURIComponent(tagName)}`);
  if (!res.ok) throw new Error(await res.text());
  return await res.json();
}

export async function writeArrayElement(tagName: string, value: any): Promise<{ success: boolean; message: string }> {
  const res = await fetch(`${API_BASE}/api/array/${encodeURIComponent(tagName)}`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ value }),
  });
  if (!res.ok) throw new Error(await res.text());
  return await res.json();
}

// UDT operations
export async function readUdt(tagName: string): Promise<{ success: boolean; tag: string; symbolId: number; dataLength: number; data: number[] }> {
  const res = await fetch(`${API_BASE}/api/udt/${encodeURIComponent(tagName)}`);
  if (!res.ok) throw new Error(await res.text());
  return await res.json();
}

export async function writeUdt(tagName: string, symbolId: number, data: number[]): Promise<{ success: boolean; message: string }> {
  const res = await fetch(`${API_BASE}/api/udt/${encodeURIComponent(tagName)}`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ symbolId, data }),
  });
  if (!res.ok) throw new Error(await res.text());
  return await res.json();
}

// UDT member operations
export async function readUdtMember(memberPath: string): Promise<{ success: boolean; tagName: string; memberName: string; value: any }> {
  const res = await fetch(`${API_BASE}/api/udt-member/${encodeURIComponent(memberPath)}`);
  if (!res.ok) throw new Error(await res.text());
  return await res.json();
}

export async function writeUdtMember(memberPath: string, value: any): Promise<{ success: boolean; tagName: string; memberName: string; message: string }> {
  const res = await fetch(`${API_BASE}/api/udt-member/${encodeURIComponent(memberPath)}`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ value }),
  });
  if (!res.ok) throw new Error(await res.text());
  return await res.json();
} 