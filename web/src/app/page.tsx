"use client";

import React, { useEffect, useRef, useState } from "react";
import Sidebar from "@/components/Sidebar";
import ChatInput from "@/components/ChatInput";
import MessageBubble from "@/components/MessageBubble";
import { useChat } from "@/lib/useChat";
import { Bot, Wifi, WifiOff, Loader2 } from "lucide-react";

export default function Home() {
  const {
    messages,
    status,
    isStreaming,
    error,
    sendMessage,
    cancel,
    connect,
    disconnect,
  } = useChat();

  const [activeSessionId, setActiveSessionId] = useState<string | null>(null);
  const messagesEndRef = useRef<HTMLDivElement>(null);

  // Auto-scroll to bottom when messages change
  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [messages]);

  const handleSelectSession = (id: string) => {
    if (id === activeSessionId) return;
    disconnect();
    setActiveSessionId(id);
    connect(id);
  };

  const statusColor =
    status === "connected"
      ? "#22c55e"
      : status === "connecting"
      ? "#eab308"
      : "#ef4444";

  return (
    <div className="app-layout">
      <Sidebar
        activeSessionId={activeSessionId}
        onSelectSession={handleSelectSession}
      />

      <main className="chat-main">
        {/* Header */}
        <header className="chat-header">
          <div className="chat-header-left">
            <Bot size={22} />
            <span className="chat-header-title">
              {activeSessionId
                ? `Session ${activeSessionId.slice(0, 8)}…`
                : "Claw Code"}
            </span>
          </div>
          <div className="chat-header-right">
            <div className="status-indicator" title={status}>
              {status === "connecting" ? (
                <Loader2 size={16} className="spin" />
              ) : status === "connected" ? (
                <Wifi size={16} />
              ) : (
                <WifiOff size={16} />
              )}
              <span className="status-dot" style={{ background: statusColor }} />
              <span className="status-text">{status}</span>
            </div>
          </div>
        </header>

        {/* Messages area */}
        <div className="chat-messages">
          {!activeSessionId && (
            <div className="chat-empty">
              <div className="chat-empty-icon">🦀</div>
              <h2>Welcome to Claw Code</h2>
              <p>
                Click <strong>&ldquo;New Chat&rdquo;</strong> in the sidebar to create a
                session and start chatting with Claude.
              </p>
            </div>
          )}

          {activeSessionId && messages.length === 0 && !isStreaming && (
            <div className="chat-empty">
              <div className="chat-empty-icon">💬</div>
              <h2>Start a conversation</h2>
              <p>Type a message below to begin chatting.</p>
            </div>
          )}

          {messages.map((msg) => (
            <MessageBubble key={msg.id} message={msg} />
          ))}

          {error && (
            <div className="chat-error">
              <span>⚠️ {error}</span>
            </div>
          )}

          <div ref={messagesEndRef} />
        </div>

        {/* Input */}
        <ChatInput
          onSend={sendMessage}
          onCancel={cancel}
          isStreaming={isStreaming}
          disabled={status !== "connected"}
        />
      </main>
    </div>
  );
}
