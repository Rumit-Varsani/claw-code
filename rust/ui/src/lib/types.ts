// Types for the Claw Code UI

export interface Message {
  id: string
  role: 'user' | 'assistant' | 'system' | 'tool'
  content: string
  tool_use_id?: string
  tool_name?: string
  tool_result?: string
  is_error?: boolean
  timestamp?: number
}

export interface Session {
  id: string
  title: string
  messages: Message[]
  model: string
  created_at: number
  updated_at: number
  fork_parent?: string | null
  fork_branch?: string | null
}

export interface Model {
  id: string
  provider: string
  label: string
  default: boolean
}

export interface SessionListItem {
  id: string
  title: string
  updated_at: number
  message_count: number
  is_active: boolean
}

export interface UsageStats {
  input_tokens: number
  output_tokens: number
  cache_creation_input_tokens: number
  cache_read_input_tokens: number
  total_tokens: number
}

export interface TokenUsage {
  latest: UsageStats
  cumulative: UsageStats
  estimated_tokens: number
}

export interface ApiConfig {
  apiUrl: string
  apiKey: string
  model: string
}

export interface StatusReport {
  status?: 'ok' | 'degraded'
  model: string
  permission_mode: string
  usage: {
    messages: number
    turns: number
    latest_total: number
    cumulative_input: number
    cumulative_output: number
    cumulative_total: number
    estimated_tokens: number
  }
  workspace: {
    cwd: string
    project_root: string | null
    git_branch: string | null
    git_state: string
    changed_files: number
    session: string
  }
  sandbox: {
    enabled: boolean
    active: boolean
    supported: boolean
  }
}

export interface MessageResponse {
  id: string
  kind: string
  model: string
  role: string
  content: Array<{
    type: string
    text?: string
    id?: string
    name?: string
    input?: string
  }>
  stop_reason: string | null
  usage: UsageStats
}

export type StreamingEvent =
  | { type: 'content_delta'; text: string }
  | { type: 'tool_use'; id: string; name: string; input: string }
  | { type: 'usage'; usage: UsageStats }
  | { type: 'status'; status: 'loading' | 'done' | 'error'; message?: string }
