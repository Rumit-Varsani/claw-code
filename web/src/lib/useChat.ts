"use client";

import { useCallback, useEffect, useRef, useState } from "react";
import type { ClientMsg, ServerMsg, UIMessage } from "./protocol";
import { wsUrl } from "./api";

export type ConnectionStatus = "disconnected" | "connecting" | "connected";

export interface UseChatReturn {
  messages: UIMessage[];
  status: ConnectionStatus;
  isStreaming: boolean;
  error: string | null;
  sendMessage: (text: string) => void;
  cancel: () => void;
  connect: (sessionId: string) => void;
  disconnect: () => void;
}

let msgCounter = 0;
function nextId(): string {
  msgCounter += 1;
  return `msg-${msgCounter}-${Date.now()}`;
}

export function useChat(): UseChatReturn {
  const [messages, setMessages] = useState<UIMessage[]>([]);
  const [status, setStatus] = useState<ConnectionStatus>("disconnected");
  const [isStreaming, setIsStreaming] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const wsRef = useRef<WebSocket | null>(null);
  const sessionIdRef = useRef<string | null>(null);

  // Cleanup on unmount
  useEffect(() => {
    return () => {
      wsRef.current?.close();
    };
  }, []);

  const disconnect = useCallback(() => {
    wsRef.current?.close();
    wsRef.current = null;
    sessionIdRef.current = null;
    setStatus("disconnected");
    setIsStreaming(false);
  }, []);

  const connect = useCallback(
    (sessionId: string) => {
      // Close existing connection
      if (wsRef.current) {
        wsRef.current.close();
      }

      sessionIdRef.current = sessionId;
      setStatus("connecting");
      setError(null);
      setMessages([]);

      const url = wsUrl(sessionId);
      const ws = new WebSocket(url);
      wsRef.current = ws;

      ws.onopen = () => {
        setStatus("connected");
      };

      ws.onclose = () => {
        setStatus("disconnected");
        setIsStreaming(false);
      };

      ws.onerror = () => {
        setError("WebSocket connection failed");
        setStatus("disconnected");
        setIsStreaming(false);
      };

      ws.onmessage = (event) => {
        let msg: ServerMsg;
        try {
          msg = JSON.parse(event.data as string) as ServerMsg;
        } catch {
          return;
        }

        switch (msg.type) {
          case "text_delta":
            setMessages((prev) => {
              const last = prev[prev.length - 1];
              if (last && last.role === "assistant" && last.streaming) {
                // Append to existing streaming message
                const updated = { ...last, content: last.content + msg.text };
                return [...prev.slice(0, -1), updated];
              }
              // Start a new assistant message
              return [
                ...prev,
                {
                  id: nextId(),
                  role: "assistant" as const,
                  content: msg.text,
                  streaming: true,
                  timestamp: Date.now(),
                },
              ];
            });
            break;

          case "turn_done":
            setMessages((prev) => {
              const last = prev[prev.length - 1];
              if (last && last.role === "assistant" && last.streaming) {
                return [...prev.slice(0, -1), { ...last, streaming: false }];
              }
              return prev;
            });
            setIsStreaming(false);
            break;

          case "error":
            setError(msg.message);
            setIsStreaming(false);
            // Also finalize any streaming message
            setMessages((prev) => {
              const last = prev[prev.length - 1];
              if (last && last.role === "assistant" && last.streaming) {
                return [...prev.slice(0, -1), { ...last, streaming: false }];
              }
              return prev;
            });
            break;

          case "tool_call_started":
            // Display tool use as assistant message
            setMessages((prev) => [
              ...prev,
              {
                id: nextId(),
                role: "assistant" as const,
                content: `🔧 Using tool: **${msg.name}**`,
                streaming: true,
                timestamp: Date.now(),
              },
            ]);
            break;

          case "tool_call_done":
            setMessages((prev) => {
              const last = prev[prev.length - 1];
              if (last && last.streaming) {
                return [
                  ...prev.slice(0, -1),
                  {
                    ...last,
                    content: last.content + `\n\n${msg.is_error ? "❌" : "✅"} Result: ${msg.output}`,
                    streaming: false,
                  },
                ];
              }
              return prev;
            });
            break;

          case "permission_request":
            // For now, auto-display; could add a UI for accept/deny
            setMessages((prev) => [
              ...prev,
              {
                id: nextId(),
                role: "assistant" as const,
                content: `🔒 Permission needed: **${msg.tool_name}**`,
                streaming: false,
                timestamp: Date.now(),
              },
            ]);
            break;
        }
      };
    },
    []
  );

  const sendMessage = useCallback(
    (text: string) => {
      const ws = wsRef.current;
      if (!ws || ws.readyState !== WebSocket.OPEN) {
        setError("Not connected");
        return;
      }

      // Add user message to UI
      setMessages((prev) => [
        ...prev,
        {
          id: nextId(),
          role: "user" as const,
          content: text,
          streaming: false,
          timestamp: Date.now(),
        },
      ]);

      setIsStreaming(true);
      setError(null);

      const payload: ClientMsg = { type: "user_turn", text };
      ws.send(JSON.stringify(payload));
    },
    []
  );

  const cancel = useCallback(() => {
    const ws = wsRef.current;
    if (ws && ws.readyState === WebSocket.OPEN) {
      const payload: ClientMsg = { type: "cancel" };
      ws.send(JSON.stringify(payload));
    }
    setIsStreaming(false);
    // Finalize any streaming message
    setMessages((prev) => {
      const last = prev[prev.length - 1];
      if (last && last.streaming) {
        return [...prev.slice(0, -1), { ...last, streaming: false }];
      }
      return prev;
    });
  }, []);

  return {
    messages,
    status,
    isStreaming,
    error,
    sendMessage,
    cancel,
    connect,
    disconnect,
  };
}
