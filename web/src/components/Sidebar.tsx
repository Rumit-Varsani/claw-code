"use client";

import React, { useEffect, useState } from "react";
import { Plus, MessageSquare, Trash2 } from "lucide-react";
import type { SessionSummary } from "@/lib/protocol";
import { createSession, listSessions } from "@/lib/api";

interface SidebarProps {
  activeSessionId: string | null;
  onSelectSession: (id: string) => void;
}

export default function Sidebar({
  activeSessionId,
  onSelectSession,
}: SidebarProps) {
  const [sessions, setSessions] = useState<SessionSummary[]>([]);
  const [loading, setLoading] = useState(false);

  const refresh = async () => {
    try {
      const list = await listSessions();
      setSessions(list);
    } catch {
      // backend might not be running yet
    }
  };

  useEffect(() => {
    refresh();
  }, []);

  const handleNew = async () => {
    setLoading(true);
    try {
      const { session_id } = await createSession();
      await refresh();
      onSelectSession(session_id);
    } catch (err) {
      console.error("Failed to create session:", err);
    } finally {
      setLoading(false);
    }
  };

  const formatTime = (ms: number) => {
    const d = new Date(ms);
    const now = new Date();
    if (d.toDateString() === now.toDateString()) {
      return d.toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" });
    }
    return d.toLocaleDateString([], { month: "short", day: "numeric" });
  };

  return (
    <aside className="sidebar">
      <div className="sidebar-header">
        <div className="sidebar-logo">
          <span className="logo-icon">🦀</span>
          <span className="logo-text">Claw Code</span>
        </div>
        <button
          className="new-chat-btn"
          onClick={handleNew}
          disabled={loading}
          title="New chat"
        >
          <Plus size={18} />
          <span>New Chat</span>
        </button>
      </div>

      <div className="sidebar-sessions">
        {sessions.length === 0 && (
          <div className="sidebar-empty">
            No sessions yet. Click &ldquo;New Chat&rdquo; to start.
          </div>
        )}
        {sessions.map((s) => (
          <button
            key={s.session_id}
            className={`session-item ${
              s.session_id === activeSessionId ? "session-item-active" : ""
            }`}
            onClick={() => onSelectSession(s.session_id)}
          >
            <MessageSquare size={16} className="session-icon" />
            <div className="session-info">
              <div className="session-id">
                {s.session_id.slice(0, 8)}…
              </div>
              <div className="session-meta">
                {s.message_count} msgs · {formatTime(s.created_at_ms)}
              </div>
            </div>
          </button>
        ))}
      </div>

      <div className="sidebar-footer">
        <div className="sidebar-footer-text">
          Powered by Claude · Rust backend
        </div>
      </div>
    </aside>
  );
}
