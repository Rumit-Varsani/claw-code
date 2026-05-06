"use client";

import React, { useRef, useEffect } from "react";
import { Send, Square } from "lucide-react";

interface ChatInputProps {
  onSend: (text: string) => void;
  onCancel: () => void;
  isStreaming: boolean;
  disabled: boolean;
}

export default function ChatInput({
  onSend,
  onCancel,
  isStreaming,
  disabled,
}: ChatInputProps) {
  const textareaRef = useRef<HTMLTextAreaElement>(null);

  // Auto-focus on mount
  useEffect(() => {
    textareaRef.current?.focus();
  }, []);

  // Auto-resize textarea
  const handleInput = () => {
    const el = textareaRef.current;
    if (!el) return;
    el.style.height = "auto";
    el.style.height = `${Math.min(el.scrollHeight, 200)}px`;
  };

  const handleSubmit = () => {
    const el = textareaRef.current;
    if (!el) return;
    const text = el.value.trim();
    if (!text) return;
    onSend(text);
    el.value = "";
    el.style.height = "auto";
  };

  const handleKeyDown = (e: React.KeyboardEvent<HTMLTextAreaElement>) => {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      if (isStreaming) return;
      handleSubmit();
    }
  };

  return (
    <div className="chat-input-container">
      <div className="chat-input-wrapper">
        <textarea
          ref={textareaRef}
          className="chat-textarea"
          placeholder={disabled ? "Connect to a session first…" : "Send a message… (Enter to send, Shift+Enter for newline)"}
          rows={1}
          disabled={disabled}
          onInput={handleInput}
          onKeyDown={handleKeyDown}
        />
        {isStreaming ? (
          <button
            className="chat-btn chat-btn-cancel"
            onClick={onCancel}
            title="Stop generation"
          >
            <Square size={18} />
          </button>
        ) : (
          <button
            className="chat-btn chat-btn-send"
            onClick={handleSubmit}
            disabled={disabled}
            title="Send message"
          >
            <Send size={18} />
          </button>
        )}
      </div>
    </div>
  );
}
