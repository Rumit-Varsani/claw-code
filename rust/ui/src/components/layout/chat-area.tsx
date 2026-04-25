'use client'

import { useState, useRef, useEffect, useCallback } from 'react'
import type { Message } from '@/lib/types'
import { Message as MessageComponent } from '@/components/chat/message'
import { ScrollContainer } from '@/components/ui/scroll-container'
import { Input } from '@/components/ui/input'

const MAX_WIDTH = '720px'

export function ChatArea() {
  const [messages, setMessages] = useState<Message[]>([])
  const [input, setInput] = useState('')
  const [isLoading, setIsLoading] = useState(false)
  const [isStreaming, setIsStreaming] = useState(false)
  const scrollRef = useRef<HTMLDivElement>(null)
  const messageContainerRef = useRef<HTMLDivElement>(null)

  const scrollToBottom = useCallback(() => {
    if (messageContainerRef.current) {
      messageContainerRef.current.scrollIntoView({ behavior: 'smooth', block: 'end' })
    }
  }, [])

  useEffect(() => {
    scrollToBottom()
  }, [messages, scrollToBottom, isLoading])

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    if (!input.trim() || isLoading) return

    const userId = Date.now().toString()
    const assistantId = (Date.now() + 1).toString()

    const userMessage: Message = {
      id: userId,
      role: 'user',
      content: input.trim(),
    }

    const assistantMessage: Message = {
      id: assistantId,
      role: 'assistant',
      content: '',
    }

    setMessages(prev => [...prev, userMessage, assistantMessage])
    setInput('')
    setIsLoading(true)
    setIsStreaming(true)

    // Simulate streaming response
    const fullResponse = generateResponse(input.trim())
    const chars = fullResponse.split('')
    let currentContent = ''

    // Clear previous assistant message content
    setMessages(prev =>
      prev.map(msg =>
        msg.id === assistantId ? { ...msg, content: '' } : msg
      )
    )

    await new Promise<void>(resolve => {
      const interval = setInterval(() => {
        if (currentContent.length < chars.length) {
          currentContent = chars.slice(0, currentContent.length + Math.floor(Math.random() * 3) + 1).join('')
          setMessages(prev =>
            prev.map(msg =>
              msg.id === assistantId
                ? { ...msg, content: currentContent }
                : msg
            )
          )
          scrollToBottom()
        } else {
          clearInterval(interval)
          setIsStreaming(false)
          setIsLoading(false)
          resolve()
        }
      }, 20)
    })
  }

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault()
      if (input.trim() && !isLoading) {
        handleSubmit(e)
      }
    }
  }

  return (
    <main className="h-full flex flex-col bg-black relative">
      {/* Messages area */}
      <div className="flex-1 overflow-y-auto">
        {!messages.length ? (
          <div className="flex flex-col items-center justify-center h-full text-center px-4 py-12">
            <div className="text-6xl mb-6 opacity-80">💬</div>
            <h2 className="text-xl font-semibold text-white/90 mb-2">Start a conversation</h2>
            <p className="text-sm text-white/50 max-w-md leading-relaxed">
              Send a message to Claude or use the Rust CLI directly. Type something to get started.
            </p>
          </div>
        ) : (
          <div className="space-y-4 py-6">
            {messages.map((message) => (
              <div key={message.id}>
                <div
                  className={`flex ${
                    message.role === 'user' ? 'justify-end mb-4' : 'justify-start mt-6 mb-4'
                  }`}
                >
                  <MessageComponent
                    role={message.role}
                    content={message.content}
                    isStreaming={message.role === 'assistant' && isStreaming && message.content !== ''}
                  />
                </div>
              </div>
            ))}
            <div ref={messageContainerRef} />
          </div>
        )}
      </div>

      {/* Input area */}
      <div className="sticky bottom-0 bg-black/95 backdrop-blur-xl w-full border-t border-white/10 z-10">
        <div className="max-w-3xl mx-auto px-4 py-4">
          <form onSubmit={handleSubmit} className="w-full">
            <div className="relative flex items-center h-12 bg-[#1a1a1a] border border-white/10 rounded-xl pl-4 pr-2 hover:border-white/20 transition-colors focus-within:border-white/30 focus-within:shadow-lg focus-within:shadow-white/5">
              <Input
                value={input}
                onChange={(e) => setInput(e.target.value)}
                onKeyDown={handleKeyDown}
                disabled={isLoading || isStreaming}
                placeholder="Message Claude..."
                className="bg-transparent border-none outline-none text-white placeholder:text-white/30 flex-1 h-full py-0"
              />
              {/* Send Button */}
              <button
                type="submit"
                disabled={!input.trim() || isLoading || isStreaming}
                className="w-9 h-9 flex items-center justify-center rounded-lg bg-white text-black disabled:opacity-50 disabled:cursor-not-allowed disabled:grayscale transition-all duration-200 hover:scale-105 active:scale-95"
              >
                <svg className="w-5 h-5 ml-0.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2.5} d="M14 5l7 7m0 0l-7 7m7-7H3" />
                </svg>
              </button>
            </div>
            <p className="flex justify-center mt-3 text-xs text-white/40">
              Press Enter to send • Shift + Enter for new line
            </p>
          </form>
        </div>
      </div>
    </main>
  )
}

// Generate a realistic-looking response based on input
function generateResponse(prompt: string): string {
  const lower = prompt.toLowerCase()

  if (lower.includes('hello') || lower.includes('hi') || lower.includes('hey')) {
    return `Hi there! 👋\n\nI see you're building the Claw Code CLI interface. This is a Rust-based AI coding assistant that supports:\n\n## Core Features\n\n- Interactive REPL with tab completions\n- Session management with automatic persistence\n- Tool execution (bash, file read/write, search)\n- MCP server integration\n- Multiple LLM providers (Anthropic, OpenAI, xAI)\n- Sandbox execution\n\nWould you like help with any specific part of the UI or backend?`
  }

  if (lower.includes('rust') || lower.includes('cargo') || lower.includes('compile')) {
    return `Great question about the Rust side!\n\nThe project structure consists of several interconnected crates:\n\n\`\`\`rust\n// Core workspace structure\ncrates/\n├── rusty-claude-cli     # Main CLI binary\n├── api                  # Provider client (Anthropic/OpenAI)\n├── runtime              # Session/conversation runtime\n├── tools                # Built-in tool registry\n├── commands             # Slash command handling\n└── plugins              # Plugin system\n\`\`\`\n\nKey architectural decisions:\n\n1. **Sandbox first** - All tool execution is sandboxed\n2. **Streaming by default** - Real-time feedback to the user\n3. **Session persistence** - Conversations saved to \`./claw/sessions/\` \n4. **Provider abstraction** - Works with Anthropic, OpenAI, xAI, and Ollama\n\nWant me to explore any specific crate in more detail?`
  }

  if (lower.includes('ui') || lower.includes('next') || lower.includes('frontend')) {
    return `The UI is a Next.js 15 application in the \`ui/\` directory.\n\n## Current Architecture\n\n\`\`\`\nui/src/\n├── app/                 # Next.js App Router\n│   ├── layout.tsx       # Root layout\n│   └── page.tsx         # Home page\n├── components/\n│   ├── layout/          # Sidebar, ChatArea, StatusBar\n│   ├── chat/            # Message component\n│   └── ui/              # Primitives (Button, Input, etc.)\n└── lib/\n    ├── api.ts           # API client for backend\n    ├── store.ts         # Zustand state management\n    └── types.ts         # TypeScript interfaces\n\`\`\`\n\n## Design System\n\n- Colors: \`background: #000000\`, \`foreground: #FFFFFF\`, \`muted: #A1A1AA\`\n- Strict black/white palette\n- 8px grid spacing system\n- Inter font family\n- Custom thin scrollbars\n\n## What's Left\n\n- [x] Basic layout structure\n- [x] Message component with markdown rendering\n- [x] Streaming simulation\n- [ ] Full API integration with Rust backend\n- [ ] Session sidebar with conversation list\n- [ ] Model selector dropdown\n- [ ] Markdown code syntax highlighting\n\`\`\`

I've built the component with proper TypeScript types and Tailwind classes following your design system. What would you like to work on next?`
  }

  if (lower.includes('session') || lower.includes('persist') || lower.includes('save')) {
    return `Session management in Claw Code uses file-based persistence.\n\n## How Sessions Work\n\nSessions are stored in \`./claw/sessions/<session-id>.jsonl\` with the following structure:\n\n\`\`\`typescript\ninterface Session {\n  id: string\n  title: string\n  messages: Message[]\n  model: string\n  created_at: number\n  updated_at: number\n}\n\`\`\`\n\nEach line in the file is a JSON-encoded message. The session state includes:\n\n1. **Conversation history** - All messages stored in append-only fashion\n2. **Prompt history** - User prompts with timestamps\n3. **Fork tracking** - Session lineage for branching conversations\n4. **Compaction metadata** - Tokens and message counts for auto-compaction\n\n## Managing Sessions\n\nThe CLI provides these commands:\n\n\`\`\`\n/claw --resume latest       # Resume most recent session\n/claw --resume <id>         # Resume specific session\n/claw /session list         # List all managed sessions\n/claw /session fork <name>  # Fork current session\n/claw /session delete <id>  # Delete a session\n\`\`\`\n\nIn the UI, we'd integrate with \`claw export\` and \`claw --resume\` to provide a full session browser. Want me to add that?`
  }

  // Default response
  const responses = [
    `I understand your question: "${prompt}"\n\nLet me break down the key points:\n\n## Context\n\nBased on the Claw Code project structure, you're working with:\n\n- A Rust CLI tool (\`claw\`) that integrates with AI providers\n- A Next.js UI layer for the chat interface\n- Session management with automatic persistence\n\n## Key Considerations\n\n1. The backend (\`main.rs\` at 12,933 lines) handles CLI parsing, session management, and API communication\n2. The UI (\`ui/\`) provides a cleaner interface over the same backend\n3. Both use the same \`claw\` binary as the execution engine\n\nWould you like me to elaborate on any part of this? I can help with:\n\n- **UI components** - Build out specific parts\n- **API integration** - Connect UI to the Rust backend\n- **New features** - Sessions, model switching, etc.`,

    `That's an interesting topic!\n\nHere's my analysis:\n\n## Background\n\nThe Claw Code project is a Rust-based AI coding assistant with:\n\n- **CLI layer**: Comprehensive argument parsing, session state, tool execution\n- **API layer**: Provider abstraction for Anthropic/OpenAI/xAI\n- **Runtime**: Conversation management, permissions, MCP integration\n- **UI layer**: Next.js frontend with the Claude aesthetic\n\n## Relevant Details\n\n\`\`\`rust\nconst DEFAULT_MODEL: &str = "claude-opus-4-6";\nconst VERSION: &str = env!("CARGO_PKG_VERSION");\nconst GIT_SHA: Option<&str> = option_env!("GIT_SHA");\n\`\`\`\n\nThe project uses model aliases (\`opus\`, \`sonnet\`, \`haiku\`), config-based overrides, and environment variable fallback for model selection.\n\nWhat specific aspect should we dive deeper into?`,

    `Great question about "${prompt}"!\n\nLet me provide some context:\n\n## Architecture Overview\n\nThe project follows a layered architecture:\n\n\`\`\`\n┌─────────────┐     ┌─────────────┐\n│     UI      │────▶│    API      │\n│ (Next.js)   │     │ (Rust)      │\n└─────────────┘     └─────────────┘\n                          │\n                    ┌─────▼─────┐\n                    │  AI       │\n                    │ Providers │\n                    └───────────┘\n\`\`\`\n\n## Current Status\n\n- CLI: Production-ready with full tool integration\n- API: Anthropic + OpenAI compat providers\n- UI: Partially implemented (I'm building it!)\n- Sessions: File-based with auto-save\n\nFeel free to ask me to:\n\n1. Build more UI components\n2. Add mock API responses for testing\n3. Explain any Rust code section\n4. Help with project structure or architecture`
  ]

  return responses[Math.floor(Math.random() * responses.length)]
}
