'use client'

import { useState, useCallback } from 'react'

interface MessageProps {
  role: 'user' | 'assistant'
  content: string
  isStreaming?: boolean
}

export function Message({ role, content, isStreaming = false }: MessageProps) {
  const isUser = role === 'user'
  const [copied, setCopied] = useState(false)

  const handleCopy = useCallback(() => {
    navigator.clipboard.writeText(content).then(() => {
      setCopied(true)
      setTimeout(() => setCopied(false), 2000)
    })
  }, [content])

  if (isUser) {
    return (
      <div className="group flex justify-end mb-4">
        <div className="max-w-[85%] px-5 py-3.5 rounded-2xl bg-white/10 text-white">
          <p className="whitespace-pre-wrap leading-relaxed">{content}</p>
        </div>
      </div>
    )
  }

  return (
    <div className="group relative mb-4 mt-6">
      <div className="max-w-[85%] px-5 py-3.5 rounded-2xl bg-[#1a1a1a] text-white/90">
        {!isStreaming && content === '' && (
          <span className="flex items-center gap-1.5 text-white/30 py-1">
            <span className="w-2 h-2 bg-white/30 rounded-full animate-pulse" />
            <span className="w-2 h-2 bg-white/30 rounded-full animate-pulse delay-150" />
            <span className="w-2 h-2 bg-white/30 rounded-full animate-pulse delay-300" />
          </span>
        )}
        {content !== '' && <RenderContent content={content} />}
        {!isStreaming && content !== '' && (
          <div className="flex items-center justify-end gap-2 mt-3 opacity-0 group-hover:opacity-100 transition-opacity">
            <button
              className="text-xs text-white/40 hover:text-white/70 transition-colors flex items-center gap-1"
              onClick={handleCopy}
            >
              {copied ? (
                <>
                  <svg className="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 13l4 4L19 7" />
                  </svg>
                  Copied
                </>
              ) : (
                <>
                  <svg className="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z" />
                  </svg>
                  Copy
                </>
              )}
            </button>
          </div>
        )}
      </div>
    </div>
  )
}

function RenderContent({ content }: { content: string }) {
  const parts = splitByCodeBlocks(content)
  if (parts.length === 0) {
    return <p className="leading-relaxed">{content}</p>
  }
  return (
    <>
      {parts.map((part, i) =>
        part.type === 'code' ? (
          <CodeBlock code={part.value} lang={part.lang} key={i} />
        ) : (
          <RenderMarkdown text={part.value} key={i} />
        )
      )}
    </>
  )
}

function splitByCodeBlocks(text: string): Array<{ type: 'text' | 'code'; value: string; lang?: string }> {
  const parts: Array<{ type: 'text' | 'code'; value: string; lang?: string }> = []
  const regex = /```(\w*)\n*([\s\S]*?)```/g
  let lastIndex = 0
  let match
  while ((match = regex.exec(text)) !== null) {
    if (match.index > lastIndex) {
      const beforeText = text.slice(lastIndex, match.index)
      if (beforeText.trim() || beforeText.includes('\n')) {
        parts.push({ type: 'text', value: beforeText })
      }
    }
    parts.push({ type: 'code', value: match[2], lang: match[1] || undefined })
    lastIndex = match.index + match[0].length
  }
  if (lastIndex < text.length) {
    const remaining = text.slice(lastIndex)
    if (remaining.trim() || remaining.includes('\n')) {
      parts.push({ type: 'text', value: remaining })
    }
  }
  return parts
}

function CodeBlock({ code, lang }: { code: string; lang?: string }) {
  const [copied, setCopied] = useState(false)
  const handleCopy = () => {
    navigator.clipboard.writeText(code.trim())
    setCopied(true)
    setTimeout(() => setCopied(false), 2000)
  }
  return (
    <div className="my-3 rounded-lg overflow-hidden border border-white/10">
      <div className="flex items-center justify-between bg-white/5 px-4 py-2 text-xs">
        <span className="text-white/50">{lang || 'text'}</span>
        <button
          onClick={handleCopy}
          className="text-white/40 hover:text-white/70 transition-colors"
        >
          {copied ? 'Copied!' : 'Copy'}
        </button>
      </div>
      <pre className="bg-[#0d0d0d] p-4 overflow-x-auto">
        <code className="text-sm text-white/85 leading-relaxed font-mono">
          {code.trim()}
        </code>
      </pre>
    </div>
  )
}

function RenderMarkdown({ text }: { text: string }) {
  const lines = text.split('\n')
  const elements: React.ReactNode[] = []
  let inList = false
  let listItems: React.ReactNode[] = []

  const flushList = () => {
    if (listItems.length > 0) {
      elements.push(
        <ul key={`list-${elements.length}`} className="list-disc list-inside my-2 space-y-1">
          {listItems}
        </ul>
      )
      listItems = []
      inList = false
    }
  }

  lines.forEach((line, i) => {
    if (line.startsWith('### ')) {
      flushList()
      elements.push(<h3 key={i} className="text-sm font-semibold mt-4 mb-2 text-white">{line.slice(4)}</h3>)
    } else if (line.startsWith('## ')) {
      flushList()
      elements.push(<h2 key={i} className="text-base font-semibold mt-4 mb-2 text-white">{line.slice(3)}</h2>)
    } else if (line.startsWith('# ')) {
      flushList()
      elements.push(<h1 key={i} className="text-lg font-semibold mt-4 mb-2 text-white">{line.slice(2)}</h1>)
    } else if (line.match(/^[\s]*[-*]\s+(.+)$/)) {
      inList = true
      listItems.push(
        <li key={i} className="text-white/80">
          <InlineCode text={line.replace(/^[\s]*[-*]\s+/, '')} />
        </li>
      )
    } else if (inList && !line.trim()) {
      flushList()
    } else if (inList) {
      flushList()
    } else if (!line.trim()) {
      elements.push(<div key={i} className="h-2" />)
    } else {
      elements.push(
        <p key={i} className="whitespace-pre-wrap leading-relaxed">
          <InlineCode text={line} />
        </p>
      )
    }
  })

  flushList()
  return <>{elements}</>
}

function InlineCode({ text }: { text: string }) {
  const parts = text.split(/(`[^`]+`)/g)
  if (parts.length <= 1) {
    return <>{text}</>
  }
  return (
    <>
      {parts.map((part, i) => {
        if (part.startsWith('`') && part.endsWith('`')) {
          return (
            <code key={i} className="bg-white/10 px-1.5 py-0.5 rounded text-sm font-mono text-white/90">
              {part.slice(1, -1)}
            </code>
          )
        }
        return <span key={i}>{part}</span>
      })}
    </>
  )
}
