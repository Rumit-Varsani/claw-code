interface StatusBarProps {
  model?: string
  permissionMode?: string
  sessionId?: string
  tokenCount?: number
  estimatedCost?: number
  gitBranch?: string
}

export function StatusBar({ tokenCount = 0, estimatedCost = 0 }: StatusBarProps) {
  return (
    <div className="flex items-center justify-between px-4 py-2 text-xs border-t border-white/10">
      <div className="flex items-center gap-4">
        <div className="flex items-center gap-2">
          <span className="text-white/40 flex items-center gap-1.5">
            <div className="w-2 h-2 bg-green-400 rounded-full animate-pulse" />
            <span>Online</span>
          </span>
          <span className="text-white/40">•</span>
          <span className="text-white/40">Read</span>
        </div>
        <div className="flex items-center gap-2">
          <span className="text-white/40">Auto</span>
        </div>
      </div>

      <div className="flex items-center gap-4 text-xs">
        <div className="flex items-center gap-3">
          <div className="flex items-center gap-2">
            <span className="text-white/50">Model:</span>
            <span className="text-white font-medium">Claude Sonnet 4</span>
          </div>
        </div>
        <div className="flex items-center gap-2">
          <span className="text-white/40 flex items-center gap-1">
            <span>🌳</span>
            <span>main</span>
          </span>
        </div>
      </div>
    </div>
  );
}