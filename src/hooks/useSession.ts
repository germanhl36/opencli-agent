import { useState, useCallback } from 'react';
import type { Message } from '../lib/types';
import { sendMessage, resetSession, getHistory, startSession } from '../lib/tauri';

export interface UseSessionReturn {
  messages: Message[];
  isLoading: boolean;
  error: string | null;
  send: (content: string) => Promise<void>;
  reset: () => Promise<void>;
  start: (provider?: string, model?: string, workingDirectory?: string) => Promise<void>;
  loadHistory: () => Promise<void>;
}

export function useSession(): UseSessionReturn {
  const [messages, setMessages] = useState<Message[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const start = useCallback(async (
    provider?: string,
    model?: string,
    workingDirectory?: string,
  ) => {
    setError(null);
    setMessages([]);
    await startSession(provider, model, workingDirectory);
  }, []);

  const send = useCallback(async (content: string) => {
    if (!content.trim()) return;
    setIsLoading(true);
    setError(null);

    // Optimistically add user message
    const userMsg: Message = {
      role: 'user',
      content,
      timestamp: Date.now() / 1000,
      toolCalls: [],
    };
    setMessages((prev) => [...prev, userMsg]);

    try {
      const response = await sendMessage(content);
      const assistantMsg: Message = {
        role: 'assistant',
        content: response,
        timestamp: Date.now() / 1000,
        toolCalls: [],
      };
      setMessages((prev) => [...prev, assistantMsg]);
    } catch (err) {
      setError(String(err));
      // Remove optimistic user message on error
      setMessages((prev) => prev.filter((m) => m !== userMsg));
    } finally {
      setIsLoading(false);
    }
  }, []);

  const reset = useCallback(async () => {
    setError(null);
    setMessages([]);
    await resetSession();
  }, []);

  const loadHistory = useCallback(async () => {
    const history = await getHistory();
    setMessages(history);
  }, []);

  return { messages, isLoading, error, send, reset, start, loadHistory };
}
