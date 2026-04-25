// API client to communicate with the Rust backend (claw CLI)

import type { Session, MessageResponse, SessionListItem, Message } from './types.js'

const API_BASE = process.env.NEXT_PUBLIC_API_URL || 'http://localhost:4545'

// Check if the CLI binary is available
export async function checkCliAvailable(): Promise<boolean> {
  try {
    const resp = await fetch(`${API_BASE}/health`)
    return resp.ok
  } catch {
    return false
  }
}

// Fetch current session info
export async function fetchStatus(): Promise<any> {
  const resp = await fetch(`${API_BASE}/status`, {
    headers: { 'Accept': 'application/json' },
  })
  if (!resp.ok) throw new Error(`status error: ${resp.status}`)
  return resp.json()
}

// Fetch list of managed sessions
export async function fetchSessions(): Promise<SessionListItem[]> {
  try {
    const resp = await fetch(`${API_BASE}/sessions`, {
      headers: { 'Accept': 'application/json' },
    })
    if (!resp.ok) return []
    const data = await resp.json()
    return (Array.isArray(data) ? data : []).map((s: any) => ({
      id: s.id,
      title: s.title || s.first_message || 'New conversation',
      updated_at: s.updated_at || s.modified_at || 0,
      message_count: s.message_count || s.msgs || 0,
      is_active: s.is_active || false,
    }))
  } catch {
    return []
  }
}

// Start a new session
export async function createSession(): Promise<{ id: string }> {
  try {
    const resp = await fetch(`${API_BASE}/session/new`, {
      method: 'POST',
      headers: { 'Accept': 'application/json' },
    })
    if (!resp.ok) throw new Error('failed to create session')
    return resp.json()
  } catch {
    // Generate a local session id as fallback
    return { id: `local-${Date.now()}` }
  }
}

// Load a session
export async function loadSession(sessionId: string): Promise<Session> {
  try {
    const resp = await fetch(`${API_BASE}/session/${encodeURIComponent(sessionId)}`, {
      headers: { 'Accept': 'application/json' },
    })
    if (!resp.ok) throw new Error('failed to load session')
    const data = await resp.json()
    return {
      id: data.id || sessionId,
      title: data.title || extractTitle(data),
      messages: normalizeMessages(data.messages || []),
      model: data.model || 'claude-sonnet-4-6',
      created_at: data.created_at || Date.now(),
      updated_at: data.updated_at || Date.now(),
      fork_parent: data.fork?.parent_session_id || null,
      fork_branch: data.fork?.branch_name || null,
    }
  } catch {
    // Return synthetic session for fallback
    return {
      id: sessionId,
      title: 'New conversation',
      messages: [],
      model: 'claude-sonnet-4-6',
      created_at: Date.now(),
      updated_at: Date.now(),
    }
  }
}

// Stream a message to the API
export async function* streamMessage(
  session: Session,
  prompt: string,
  onChunk: (event: any) => void,
  signal?: AbortSignal,
): AsyncGenerator<{ content: string; stop_reason?: string }> {
  try {
    const resp = await fetch(`${API_BASE}/sessions/${encodeURIComponent(session.id)}/messages`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'Accept': 'text/event-stream',
      },
      body: JSON.stringify({
        message: {
          role: 'user',
          content: prompt,
        },
        stream: true,
      }),
      signal,
    })

    if (!resp.ok) throw new Error(`request failed: ${resp.status}`)

    const reader = resp.body?.getReader()
    if (!reader) throw new Error('no response reader')

    const decoder = new TextDecoder()
    let buffer = ''

    while (true) {
      const { done, value } = await reader.read()
      if (done) break

      buffer += decoder.decode(value, { stream: true })
      const lines = buffer.split('\n')
      buffer = lines.pop() || ''

      for (const line of lines) {
        if (!line.startsWith('data: ')) continue
        const jsonStr = line.slice(6)
        if (!jsonStr.trim()) continue

        try {
          const data = JSON.parse(jsonStr)
          onChunk(data)

          if (data.type === 'text_delta') {
            yield { content: data.text || '' }
          } else if (data.type === 'message_stop') {
            yield { content: '', stop_reason: data.stop_reason || null }
            return
          }
        } catch {
          // Skip malformed JSON
        }
      }
    }

    // Process remaining buffer
    if (buffer.startsWith('data: ')) {
      const jsonStr = buffer.slice(6)
      if (jsonStr.trim()) {
        try {
          const data = JSON.parse(jsonStr)
          onChunk(data)
        } catch {
          // skip
        }
      }
    }
  } catch (err: any) {
    if (err.name === 'AbortError') return
    throw err
  }
}

// Update session usage stats
export async function updateSessionUsage(sessionId: string, usage: any): Promise<void> {
  try {
    await fetch(`${API_BASE}/sessions/${encodeURIComponent(sessionId)}/usage`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ usage }),
    })
  } catch {
    // silently fail
  }
}

// Export session as markdown
export async function exportSession(sessionId: string, format: 'markdown' | 'text' = 'markdown'): Promise<string> {
  try {
    const resp = await fetch(
      `${API_BASE}/sessions/${encodeURIComponent(sessionId)}/export?format=${format}`,
      { headers: { 'Accept': 'application/json' } },
    )
    if (!resp.ok) throw new Error('export failed')
    const data = await resp.json()
    return data.content || data.markdown || ''
  } catch {
    return ''
  }
}

// Get available models
export async function fetchAvailableModels(): Promise<{ id: string; label: string; provider: string; default: boolean }[]> {
  const models = [
    { id: 'claude-sonnet-4-6', label: 'Claude Sonnet 4', provider: 'anthropic', default: true },
    { id: 'claude-opus-4-6', label: 'Claude Opus 4', provider: 'anthropic', default: false },
    { id: 'claude-opus-4-5', label: 'Claude Opus 4.5', provider: 'anthropic', default: false },
    { id: 'claude-haiku-4-5-20251213', label: 'Claude Haiku 4.5', provider: 'anthropic', default: false },
    { id: 'claude-3-7-sonnet-latest', label: 'Claude 3.7 Sonnet', provider: 'anthropic', default: false },
  ]
  // Check for custom config overrides
  try {
    const configResp = await fetch(`${API_BASE}/config/model`)
    if (configResp.ok) {
      const config = await configResp.json()
      if (config?.model) {
        const existing = models.find(m => m.id === config.model)
        if (existing) {
          existing.default = true
        } else {
          models.unshift({ id: config.model, label: config.model, provider: 'custom', default: true })
        }
      }
    }
  } catch {
    // ignore config errors
  }
  return models
}

// Helpers
function extractTitle(session: any): string {
  const msgs = session.messages || []
  const firstUser = msgs.find((m: Message) => m.role === 'user')?.content
  if (!firstUser) return 'New conversation'
  const title = firstUser.trim().split('\n')[0].slice(0, 50)
  return title || 'New conversation'
}

function normalizeMessages(raw: any[]): any[] {
  return raw
    .filter(
      (m): m is Message =>
        m?.role && (m.role === 'user' || m.role === 'assistant' || m.role === 'system' || m.role === 'tool'),
    )
    .map(
      (m: Message) => ({
        ...m,
        content: m.content || (Array.isArray(m.blocks) ? m.blocks.filter(b => b.type === 'text').map(b => b.text || '').join('') : ''),
        timestamp: m.timestamp || 0,
      }),
    )
}
