// React store for session and chat state

import { create } from 'zustand'
import type { Session, Message, Model, SessionListItem } from '@/lib/types'
import * as api from '@/lib/api'

interface ChatState {
  session: Session | null
  sessionId: string | null
  messages: Message[]
  activeSessionId: string | null
  isLoading: boolean
  currentModel: string
  availableModels: Model[] | null
  sessions: SessionListItem[]
  status: any
  error: string | null

  // Actions
  loadSessions: () => Promise<void>
  createSession: () => Promise<void>
  switchToSession: (id: string) => Promise<void>
  send: (prompt: string) => Promise<void>
  abort: () => void
  setModel: (model: string) => void
  setStatus: (status: any) => void
  clearError: () => void
  refreshStatus: () => Promise<void>
}

// Generate a unique ID
const genId = () => `msg-${Date.now()}-${Math.random().toString(36).slice(2, 9)}`

// Helper to extract title from first user message
function extractTitle(content: string): string {
  const firstLine = content.trim().split('\n')[0]
  return firstLine.length > 50 ? firstLine.slice(0, 47) + '...' : firstLine || 'New conversation'
}

export const useChatStore = create<ChatState>((set, get) => ({
  session: null,
  sessionId: null,
  messages: [],
  activeSessionId: null,
  isLoading: false,
  currentModel: 'claude-sonnet-4-6',
  availableModels: null,
  sessions: [],
  status: null,
  error: null,

  loadSessions: async () => {
    try {
      const sessions = await api.fetchSessions()
      set({ sessions })
    } catch {
      set({ sessions: [] })
    }
  },

  createSession: async () => {
    const { loadSessions } = get()
    try {
      const result = await api.createSession()
      const newSession: Session = {
        id: result.id,
        title: 'New conversation',
        messages: [],
        model: get().currentModel,
        created_at: Date.now(),
        updated_at: Date.now(),
      }
      set({
        session: newSession,
        sessionId: result.id,
        messages: [],
        activeSessionId: result.id,
      })
      await loadSessions()
    } catch {
      set({ error: 'Failed to create new session' })
    }
  },

  switchToSession: async (id: string) => {
    const { loadSessions } = get()
    try {
      const session = await api.loadSession(id)
      set({
        session,
        sessionId: id,
        messages: session.messages,
        activeSessionId: id,
        currentModel: session.model,
      })
      await loadSessions()
    } catch {
      set({ error: 'Failed to load session' })
    }
  },

  send: async (prompt: string) => {
    const { session, currentModel } = get()
    if (!session || !prompt.trim()) return

    const userId = genId()
    const userMsg: Message = {
      id: userId,
      role: 'user',
      content: prompt.trim(),
      timestamp: Date.now(),
    }

    const assistantId = genId()
    const assistantMsg: Message = {
      id: assistantId,
      role: 'assistant',
      content: '',
      timestamp: Date.now(),
    }

    set(prev => ({
      messages: [...prev.messages, userMsg, assistantMsg],
      isLoading: true,
      error: null,
    }))

    let accumulated = ''
    let abortController: AbortController | null = null

    try {
      abortController = new AbortController()
      const generator = api.streamMessage(session, prompt, () => {}, abortController.signal)

      for await (const chunk of generator) {
        accumulated += chunk.content
        set(prev => ({
          messages: prev.messages.map(m =>
            m.id === assistantId ? { ...m, content: accumulated } : m,
          ),
        }))
      }

      // Update session in store
      set(prev => ({
        session: prev.session ? {
          ...prev.session,
          messages: [...prev.messages.map(m =>
            m.id === assistantId ? { ...m, content: accumulated } : m
          ), userMsg],
          updated_at: Date.now(),
        } : null,
        isLoading: false,
      }))

      await get().loadSessions()
    } catch (err: any) {
      if (err.name !== 'AbortError') {
        set({
          messages: prev => prev.map(m =>
            m.id === assistantId ? { ...m, content: `Error: ${err.message}` } : m,
          ),
          isLoading: false,
          error: err.message,
        })
      }
    } finally {
      abortController = null
    }
  },

  abort: () => {
    set({ isLoading: false })
  },

  setModel: async (model: string) => {
    const { sessions } = get()
    set({ currentModel: model, error: null })
    // Update current session model
    const { session } = get()
    if (session) {
      const newSession = { ...session, model }
      set({ session: newSession })
    }
  },

  setStatus: (status: any) => {
    set({ status })
  },

  clearError: () => {
    set({ error: null })
  },

  refreshStatus: async () => {
    try {
      const status = await api.fetchStatus()
      set({ status })
    } catch {
      set({ status: null })
    }
  },
}))
