// REST client for the claw-api-server backend.
// Backend runs on :3000; frontend on :3001.

import type { SessionSummary, SessionDetail } from "./protocol";

const API_BASE = process.env.NEXT_PUBLIC_API_URL ?? "http://localhost:3000";

async function json<T>(res: Response): Promise<T> {
  if (!res.ok) {
    const body = await res.text().catch(() => "");
    throw new Error(`API ${res.status}: ${body}`);
  }
  return res.json() as Promise<T>;
}

// POST /api/sessions → { session_id, created_at_ms }
export async function createSession(): Promise<{
  session_id: string;
  created_at_ms: number;
}> {
  const res = await fetch(`${API_BASE}/api/sessions`, { method: "POST" });
  return json(res);
}

// GET /api/sessions → SessionSummary[]
export async function listSessions(): Promise<SessionSummary[]> {
  const res = await fetch(`${API_BASE}/api/sessions`);
  return json(res);
}

// GET /api/sessions/:id → SessionDetail
export async function getSession(id: string): Promise<SessionDetail> {
  const res = await fetch(`${API_BASE}/api/sessions/${encodeURIComponent(id)}`);
  return json(res);
}

// Build a WebSocket URL for a given session.
export function wsUrl(sessionId: string): string {
  const base = API_BASE.replace(/^http/, "ws");
  return `${base}/ws/${encodeURIComponent(sessionId)}`;
}
