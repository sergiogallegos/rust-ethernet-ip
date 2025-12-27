export type PlcStatus = 'connected' | 'disconnected' | 'connecting' | 'error' | 'unknown';

export interface PlcTagValue {
  tag: string;
  value: boolean | number | string;
}

export async function connectToPlc(ipAddress: string): Promise<boolean> {
  const res = await fetch('/api/connect', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ ipAddress }),
  });
  if (!res.ok) throw new Error(await res.text());
  return true;
}

export async function disconnectPlc(): Promise<void> {
  await fetch('/api/disconnect', { method: 'POST' });
}

export async function readTag(tag: string, type: string): Promise<PlcTagValue> {
  const res = await fetch(`/api/tag?tag=${encodeURIComponent(tag)}&type=${encodeURIComponent(type)}`);
  if (!res.ok) throw new Error(await res.text());
  const data = await res.json();
  return data;
}

export async function writeTag(tag: string, value: any, type: string): Promise<boolean> {
  const res = await fetch('/api/tag', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ tag, type, value }),
  });
  if (!res.ok) throw new Error(await res.text());
  return true;
}

export function subscribeToTagUpdates(onUpdate: (data: PlcTagValue) => void): () => void {
  const ws = new WebSocket(`ws://${window.location.hostname}:8080/ws`);
  ws.onmessage = (event) => {
    try {
      const data = JSON.parse(event.data);
      onUpdate(data);
    } catch {}
  };
  return () => ws.close();
}

export async function batchReadTags(tags: { tag: string; type: string }[]): Promise<Record<string, any>> {
  const res = await fetch('/api/batch', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ tags }),
  });
  if (!res.ok) throw new Error(await res.text());
  return await res.json();
}

export async function batchWriteTags(tagObjs: { tag: string; type: string; value: any }[]): Promise<{ success: boolean; error?: string }> {
  const res = await fetch('/api/batch', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ writes: tagObjs }),
  });
  if (!res.ok) throw new Error(await res.text());
  return await res.json();
}

export async function runBenchmark(tag: string, type: string, write: boolean): Promise<{ success: boolean; readCount: number; writeCount: number; elapsedMs: number; readRate: number; writeRate: number; error?: string }> {
  const res = await fetch('/api/benchmark', {
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
  const res = await fetch(`/api/taginfo?tag=${encodeURIComponent(tag)}`);
  if (!res.ok) throw new Error(await res.text());
  const data = await res.json();
  return data.type as string;
}

export async function debugReadTag(tag: string, typeStr: string): Promise<any> {
  const res = await fetch(`/api/test-read?tag=${encodeURIComponent(tag)}&type=${encodeURIComponent(typeStr)}`);
  return await res.json();
}

export async function testArrays(testType: 'controller' | 'program' | 'bool' | 'all'): Promise<any> {
  const res = await fetch('http://localhost:8080/api/test-arrays', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ testType }),
  });
  if (!res.ok) throw new Error(await res.text());
  return await res.json();
} 