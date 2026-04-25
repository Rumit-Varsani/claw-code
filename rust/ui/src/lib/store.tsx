// Chat store using React Context + useReducer/useState (no Zustand needed)

'use client'

import { createContext, useContext, useState, useCallback, useRef, useEffect } from 'react'
import type { Message } from '@/lib/types'

// ---- Types ----

export interface ChatStore {
  messages: Message[]
  input: string
  isLoading: boolean
  isStreaming: boolean
  sessionId: string | null
  currentModel: string
  error: string | null

  setInput: (input: string) => void
  send: (prompt: string) => Promise<void>
  abort: () => void
  clearMessages: () => void
  switchModel: (model: string) => void
  setError: (error: string | null) => void
}

const ChatContext = createContext<ChatStore | null>(null)

export function useChatContext() {
  const ctx = useContext(ChatContext)
  if (!ctx) throw new Error('useChatContext must be used within ChatProvider')
  return ctx
}

export function ChatProvider({ children }: { children: React.ReactNode }) {
  const [messages, setMessages] = useState<Message[]>([])
  const [input, setInput] = useState('')
  const [isLoading, setIsLoading] = useState(false)
  const [isStreaming, setIsStreaming] = useState(false)
  const [sessionId] = useState<string | null>(() => `session-${Date.now()}`)
  const [error, setError] = useState<string | null>(null)
  const abortRef = useRef<AbortController | null>(null)

  const clearMessages = useCallback(() => {
    setMessages([])
    setInput('')
    setError(null)
  }, [])

  const switchModel = useCallback((model: string) => {
    // Could integrate with API to switch models
  }, [])

  const send = useCallback(async (prompt: string) => {
    if (!prompt.trim() || isLoading) return
    setError(null)

    const userId = `user-${Date.now()}`
    const userMsg: Message = { id: userId, role: 'user', content: prompt.trim() }
    const assistantId = `assistant-${Date.now()}`
    const assistantMsg: Message = { id: assistantId, role: 'assistant', content: '' }

    setMessages(prev => [...prev, userMsg, assistantMsg])
    setInput('')
    setIsLoading(true)
    setIsStreaming(true)

    abortRef.current = new AbortController()
    const signal = abortRef.current.signal

    // Generate context-aware response
    const fullResponse = await generateContextualResponse(prompt)
    const chars = fullResponse.split('')
    let currentContent = ''

    await new Promise<void>((resolve) => {
      const interval = setInterval(() => {
        if (currentContent.length >= chars.length) {
          clearInterval(interval)
          resolve()
          return
        }
        const batchSize = Math.floor(Math.random() * 4) + 1
        currentContent = chars.slice(0, currentContent.length + batchSize).join('')
        setMessages(prev =>
          prev.map(m => (m.id === assistantId ? { ...m, content: currentContent } : m))
        )
      }, 15)
    })

    setIsStreaming(false)
    setIsLoading(false)
  }, [isLoading])

  const abort = useCallback(() => {
    abortRef.current?.abort()
    setIsStreaming(false)
    setIsLoading(false)
  }, [])

  const value = useState({
    messages, setInput, isLoading, isStreaming, sessionId, currentModel: 'claude-sonnet-4-6',
    error, clearMessages, send, abort, switchModel, setError,
  })[0]

  const store: ChatStore = {
    messages, input, isLoading, isStreaming, sessionId,
    currentModel: 'claude-sonnet-4-6', error,
    setInput, send, abort, clearMessages, switchModel, setError,
  }

  return <ChatContext.Provider value={store}>{children}</ChatContext.Provider>
}

// ---- Contextual response generation ----

async function generateContextualResponse(prompt: string): Promise<string> {
  // Simulate network delay
  await new Promise(r => setTimeout(r, 800 + Math.random() * 1200))

  const lower = prompt.toLowerCase()

  if (lower.includes('rust') || lower.includes('cargo') || lower.includes('clippy') || lower.includes('compile')) {
    return `Great question about the Rust side of the project!

## Project Structure

The workspace is organized into several crates in \`crates/\`:

\`\`\`
crates/
├── rusty-claude-cli    # Main CLI binary & orchestration
├── api                 # Provider clients (Anthropic, OpenAI, xAI, Ollama)
├── runtime             # Session management, conversation state, tools
├── tools               # Built-in tool registry (bash, read/write, search)
├── commands            # Slash command specs & dispatch
├── plugins             # Plugin system
├── mock-anthropic-service # Test mock server
└── compat-harness      # Manifest extraction for upstream parity
\`\`\`

## Key Architectural Decisions

1. **Provider abstraction** — \`api/\` supports Anthropic, xAI, OpenAI, and Ollama via the \`ProviderClient\` enum with an \`OpenAiCompatClient\` for non-Anthropic providers
2. **Streaming by default** — All messages are streamed with SSE-style deltas
3. **Session persistence** — File-based JSONL sessions in \`./.claw/sessions/\`
4. **Permission system** — Three modes (read-only, workspace-write, danger-full-access) with per-tool requirements
5. **Tool registry** — Centralized registry with builtin + MCP tool discovery

Want me to explain any specific crate or concept in more detail?
`
  }

  if (lower.includes('ui') || lower.includes('frontend') || lower.includes('react') || lower.includes('next')) {
    return `The UI layer is a Next.js 15 application designed to mirror the Claude aesthetic with a black-and-white palette.

## Architecture

\`\`\`
ui/src/
├── app/                  # Next.js 15 App Router
│   ├── layout.tsx        # Root layout (Inter font, dark mode)
│   ├── page.tsx          # Home page wrapping Layout
│   └── globals.css       # Tailwind + custom scrollbar + selection styles
├── components/
│   ├── layout/
│   │   ├── layout.tsx    # Sidebar + ChatArea + StatusBar container
│   │   ├── sidebar.tsx   # Session list, new conversation button
│   │   ├── chat-area.tsx # Message display, input with send button
│   │   └── status-bar.tsx
│   ├── chat/
│   │   ├── message.tsx   # Message bubbles + markdown/code rendering
│   │   └── index.ts
│   └── ui/
│       ├── button.tsx    # Outline/default variants
│       ├── input.tsx     # Clean input with focus states
│       └── scroll-container.tsx
└── lib/
    ├── api.ts            # API client & session management
    ├── types.ts          # TypeScript interfaces
    └── store.tsx         # React Context-based chat store
\`\`\`

## Design System

- **Colors**: \`background: #000000\`, \`foreground: #FFFFFF\`, \`muted: #A1A1AA\`, \`border: #1A1A1A\`
- **Strictly black & white** — No accent colors, no gradients
- **8px grid** spacing system
- **Inter** font family with wide letter-spacing
- **Thin scrollbars** (4px width with subtle thumb)

## Current Status

- ✅ Layout structure (sidebar, chat area, status bar)
- ✅ Message component with full markdown + code block rendering
- ✅ Input with send button and Enter key support
- ✅ Streaming simulation
- ✅ React Context store (no external deps)
- ⏳ Session sidebar with conversation list
- ⏳ Model selector dropdown
- ⏳ Real API integration`
  }

  if (lower.includes('session') || lower.includes('persist') || lower.includes('save') || lower.includes('history')) {
    return `Session management is one of the core features of Claw Code.

## Session Architecture

Sessions are stored as JSONL files in \`./.claw/sessions/<id>.jsonl\`:

\`\`\`typescript
interface Session {
  id: string
  title: string
  messages: Message[]
  model: string
  fork?: { parent_session_id: string; branch_name: string | null } | null
  compaction: { count: number; removed_message_count: number }
  prompt_history: PromptEntry[]
}
\`\`\`

## Key Features

| Feature | Description |
|---|---|
| Auto-save | Sessions persist after every turn |
| Forking | \`/session fork\` branches from current session |
| Compaction | Auto-compacts when context window is nearing capacity |
| Resume | \`claw --resume SESSION\` or \`/resume latest\` |
| Export | \`claw export\` writes transcript as markdown |

## CLI Commands

\`\`\`bash
claw --resume latest                # Resume newest session
claw --resume session.jsonl         # Resume specific session
claw /session list                  # List all sessions
claw /session fork main             # Fork current session
claw /session switch <id>           # Switch active session
claw export conversation.md         # Export to file
\`\`\`

This is exactly what the sidebar UI will visualize — a list of conversations with the ability to switch, fork, and delete.
`
  }

  if (lower.includes('tool') || lower.includes('bash') || lower.includes('read') || lower.includes('write')) {
    return `Tool execution is a core capability of Claw Code.

## Available Tools

| Tool | Permission | Description |
|---|---|---|
| \`bash\` | workspace-write | Execute shell commands with background/task support |
| \`read_file\` | workspace-write | Read file contents with line ranges |
| \`write_file\` | workspace-write | Create or overwrite files |
| \`edit_file\` | workspace-write | Find-and-replace with structured patch |
| \`glob_search\` | workspace-write | Glob pattern file search |
| \`grep_search\` | workspace-write | Grep with pattern matching |
| \`web_search\` | workspace-write | Web search via Tavily |
| \`write_note\` | workspace-write | Write to .claw/notes/ |

## Tool Architecture

\`\`\`
Tool Definition
     │
     ▼
ToolChoice (auto | any | tool name)
     │
     ▼
PermissionCheck ←─ PermissionPolicy per tool
     │
     ▼
ToolExecutor.execute()
     │
     ├── CliToolExecutor (for CLI)
     │     ├── GlobalToolRegistry (builtin)
     │     └── MCPToolBridge (MCP tools)
     └── SandboxToolExecutor (for sandbox)
\`\`\`

## Permission Tiers

- **read-only** — Search and read tools only
- **workspace-write** — All editing tools
- **danger-full-access** — Unrestricted system access (e.g., \`cargo install\`)

The CLI supports \`--allowedTools\` to restrict which tools are available.`
  }

  if (lower.includes('api') || lower.includes('provider') || lower.includes('anthropic') || lower.includes('ollama')) {
    return `The \`api/\` crate provides provider-agnostic LLM communication.

## Provider Support

\`\`\`
ProviderClient enum:
├── Anthropic(AnthropicClient)     — Direct Anthropic API
├── Xai(XaIClient)                 — xAI (Grok) via OpenAI compat
├── OpenAi(OpenAiCompatClient)     — OpenAI, OpenRouter, Ollama, etc.
└── DashScope(DashScopeClient)     — Alibaba DashScope
\`\`\`

## Key Types

\`\`\`typescript
// Request structure
interface MessageRequest {
  model: string
  max_tokens: u32
  messages: InputMessage[]
  system?: string
  tools?: Vec<ToolDefinition>
  tool_choice: ToolChoice
  stream: bool
  reasoning_effort: Option<String>
}

// Response handling
enum ApiStreamEvent {
  MessageStart,
  ContentBlockStart,
  ContentBlockDelta { text | json_delta | thinking_delta },
  ContentBlockStop,
  MessageDelta { usage },
  MessageStop
}
\`\`\`

## Model Aliases

| Alias | Resolves To |
|---|---|
| \`opus\` | \`claude-opus-4-6\` |
| \`sonnet\` | \`claude-sonnet-4-6\` |
| \`haiku\` | \`claude-haiku-4-5-20251213\` |

## Auth Sources

1. \`ANTHROPIC_API_KEY\` — Standard API key
2. \`ANTHROPIC_AUTH_TOKEN\` — OAuth auth token
3. Config file \`model\` + \`aliases\` overrides`
  }

  // Default contextual response
  return `I'd be happy to help you with the Claw Code project! Here's what I know about it:

## Project Overview

**Claw Code** is a Rust-based AI coding assistant CLI that bridges to LLM providers (Anthropic, OpenAI, xAI, Ollama).

### What's Done

✅ Full CLI with argument parsing, help, version, status
✅ Session management with auto-save (JSONL format)
✅ Provider abstraction (Anthropic primary + OpenAI compat)
✅ Tool execution (bash, file read/write/edit, glob/grep search, web search)
✅ Permission system (read-only, workspace-write, danger-full-access)
✅ MCP server integration
✅ Streaming responses with markdown rendering
✅ Doctor/health diagnostics
✅ Model aliasing and config-based overrides

### What I'm Building Now

The **UI layer** — a Next.js 15 application that provides a modern chat interface mirroring the Claude aesthetic:

- Clean black-and-white design system
- Interactive chat with streaming simulation
- Full markdown + code block rendering
- React Context state management (no external deps)

## What would you like to work on?

You can ask me to:
- Build more UI components (session sidebar, model selector)
- Wire up real API integration
- Explain any Rust code section
- Add new features to the CLI`
}
