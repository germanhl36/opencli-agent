// TypeScript mirror of all Rust IPC structs

// ---- Config ----

export interface AppConfig {
  activeProvider: string;
  activeModel: string;
  theme: string;
  fontSize: number;
  commandTimeoutS: number;
  sandboxEnabled: boolean;
  sandboxImage: string | null;
  workingDirectory: string | null;
}

export interface ProjectConfig {
  modelOverride: string | null;
  providerOverride: string | null;
  autoApprove: AutoApproveRule[];
  sandboxEnabled: boolean | null;
  sandboxImage: string | null;
  ignorePatterns: string[];
}

export interface AutoApproveRule {
  action: string;
  commandPrefix: string | null;
  pathGlob: string | null;
}

// ---- Session ----

export interface Message {
  role: 'user' | 'assistant' | 'system';
  content: string;
  timestamp: number;
  toolCalls: unknown[];
}

export interface SessionInfo {
  id: string;
  provider: string;
  model: string;
  workingDirectory: string | null;
}

// ---- Context ----

export interface FileEntry {
  path: string;
  mimeType: string;
  sizeBytes: number;
  modifiedAt: number;
  excerpt: string | null;
}

export interface ContextSnapshot {
  files: FileEntry[];
  totalTokens: number;
  truncated: boolean;
}

// ---- Approval ----

export type ActionType = 'file_write' | 'file_delete' | 'dir_create' | 'shell_run';

export type RiskLevel = 'low' | 'medium' | 'high';

export type ApprovalOutcome = 'approved' | 'rejected';

export interface ActionRequest {
  id: string;
  action: ActionType;
  targetPath: string;
  args: Record<string, unknown>;
  description: string;
  risk: RiskLevel;
}

// ---- LLM ----

export type TokenEvent =
  | { type: 'text'; delta: string }
  | { type: 'tool'; call: ToolCall }
  | { type: 'stop'; reason: StopReason }
  | { type: 'error'; message: string };

export type StopReason = 'end_turn' | 'max_tokens' | 'tool_call' | 'cancelled';

export interface ModelInfo {
  id: string;
  name: string;
  provider: string;
  contextLength: number | null;
}

export interface ToolCall {
  id: string;
  name: string;
  arguments: Record<string, unknown>;
}

// ---- Diff ----

export type DiffLineKind = 'added' | 'removed' | 'context';

export interface DiffLine {
  kind: DiffLineKind;
  content: string;
  oldLineno: number | null;
  newLineno: number | null;
}

export interface Hunk {
  header: string;
  lines: DiffLine[];
}

export interface UnifiedDiff {
  path: string;
  hunks: Hunk[];
  isNewFile: boolean;
  isDeleted: boolean;
}

// ---- Audit ----

export type AuditStatus = 'success' | 'rejected' | 'error';

export interface AuditEntry {
  ts: string;
  sessionId: string;
  action: string;
  target: string;
  status: AuditStatus;
  auto: boolean;
}

// ---- DirEntry ----

export interface DirEntry {
  name: string;
  path: string;
  isDir: boolean;
  sizeBytes: number;
  modifiedAt: number;
}

// ---- Plugins ----

export interface Skill {
  name: string;
  description: string;
  prompt: string;
  contextFiles: string[];
}

export interface AgentStep {
  goal: string;
  prompt: string;
  allowedTools: string[];
}

export interface Agent {
  name: string;
  description: string;
  steps: AgentStep[];
}
