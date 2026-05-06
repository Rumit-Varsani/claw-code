"use client";

import React from "react";
import ReactMarkdown from "react-markdown";
import type { UIMessage } from "@/lib/protocol";
import { User, Bot } from "lucide-react";

interface MessageBubbleProps {
  message: UIMessage;
}

export default function MessageBubble({ message }: MessageBubbleProps) {
  const isUser = message.role === "user";

  return (
    <div className={`message-row ${isUser ? "message-row-user" : "message-row-assistant"}`}>
      <div className={`message-avatar ${isUser ? "avatar-user" : "avatar-assistant"}`}>
        {isUser ? <User size={18} /> : <Bot size={18} />}
      </div>
      <div className={`message-bubble ${isUser ? "bubble-user" : "bubble-assistant"}`}>
        <div className="message-content">
          {isUser ? (
            <p>{message.content}</p>
          ) : (
            <ReactMarkdown
              components={{
                pre: ({ children }) => (
                  <pre className="code-block">{children}</pre>
                ),
                code: ({ children, className }) => {
                  const isInline = !className;
                  return isInline ? (
                    <code className="inline-code">{children}</code>
                  ) : (
                    <code className={className}>{children}</code>
                  );
                },
              }}
            >
              {message.content}
            </ReactMarkdown>
          )}
          {message.streaming && <span className="cursor-blink">▊</span>}
        </div>
        <div className="message-time">
          {new Date(message.timestamp).toLocaleTimeString([], {
            hour: "2-digit",
            minute: "2-digit",
          })}
        </div>
      </div>
    </div>
  );
}
