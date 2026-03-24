import { invoke } from '@tauri-apps/api/core';
import { listen, UnlistenFn } from '@tauri-apps/api/event';
import type {
  AppConfig,
  ContextSnapshot,
  DirEntry,
  Message,
  ModelInfo,
  TokenEvent,
  UnifiedDiff,
  ActionRequest,
} from './types';

// ---- Session Commands ----

export async function startSession(
  provider?: string,
  model?: string,
  workingDirectory?: string,
): Promise<string> {
  return invoke<string>('start_session', { provider, model, workingDirectory });
}

export async function sendMessage(content: string): Promise<string> {
  return invoke<string>('send_message', { content });
}

export async function resetSession(): Promise<void> {
  return invoke<void>('reset_session');
}

export async function getHistory(): Promise<Message[]> {
  return invoke<Message[]>('get_history');
}

export async function getContext(): Promise<ContextSnapshot> {
  return invoke<ContextSnapshot>('get_context');
}

export async function resolveApproval(approvalId: string, approved: boolean): Promise<void> {
  return invoke<void>('resolve_approval', { approvalId, approved });
}

export async function undoLast(): Promise<string | null> {
  return invoke<string | null>('undo_last');
}

// ---- FS Commands ----

export async function readDir(path: string): Promise<DirEntry[]> {
  return invoke<DirEntry[]>('read_dir', { path });
}

export async function applyPatch(path: string, newContent: string): Promise<UnifiedDiff> {
  return invoke<UnifiedDiff>('apply_patch', { path, newContent });
}

export async function deleteFile(path: string): Promise<void> {
  return invoke<void>('delete_file', { path });
}

export async function readFileContent(path: string): Promise<string> {
  return invoke<string>('read_file_content', { path });
}

// ---- Shell Commands ----

export interface CommandOutput {
  stdout: string;
  stderr: string;
  exitCode: number;
  timedOut: boolean;
}

export async function runCommand(command: string): Promise<CommandOutput> {
  return invoke<CommandOutput>('run_command', { command });
}

export async function cancelCommand(): Promise<void> {
  return invoke<void>('cancel_command');
}

// ---- Config Commands ----

export async function loadConfig(): Promise<AppConfig> {
  return invoke<AppConfig>('load_config');
}

export async function saveConfig(config: AppConfig): Promise<void> {
  return invoke<void>('save_config', { config });
}

export async function listModels(): Promise<ModelInfo[]> {
  return invoke<ModelInfo[]>('list_models');
}

export async function storeApiKey(provider: string, key: string): Promise<void> {
  return invoke<void>('store_api_key', { provider, key });
}

export async function reloadPlugins(): Promise<void> {
  return invoke<void>('reload_plugins');
}

export async function activateSkill(skillName: string, userInput: string): Promise<string> {
  return invoke<string>('activate_skill', { skillName, userInput });
}

export async function startAgent(agentName: string): Promise<unknown[]> {
  return invoke<unknown[]>('start_agent', { agentName });
}

// ---- Event Listeners ----

export async function onTokenStream(
  handler: (event: TokenEvent) => void,
): Promise<UnlistenFn> {
  return listen<TokenEvent>('token-stream', (event) => handler(event.payload));
}

export async function onApprovalRequested(
  handler: (request: ActionRequest) => void,
): Promise<UnlistenFn> {
  return listen<ActionRequest>('approval-requested', (event) => handler(event.payload));
}

export async function onShellOutput(
  handler: (line: string) => void,
): Promise<UnlistenFn> {
  return listen<string>('shell-output', (event) => handler(event.payload));
}

export async function onSessionReset(handler: () => void): Promise<UnlistenFn> {
  return listen<void>('session-reset', () => handler());
}

export async function onContextChanged(
  handler: (snapshot: ContextSnapshot) => void,
): Promise<UnlistenFn> {
  return listen<ContextSnapshot>('context-changed', (event) => handler(event.payload));
}
