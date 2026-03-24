import React, { useEffect, useRef, useState, useCallback } from 'react';
import type { Message } from '../../lib/types';
import styles from './ChatPanel.module.css';

interface ChatPanelProps {
  messages: Message[];
  isLoading: boolean;
  streamingContent?: string;
  onSend: (content: string) => void;
  onReset: () => void;
}

function RoleBadge({ role }: { role: string }) {
  return (
    <span className={`${styles.roleBadge} ${styles[`role_${role}`]}`}>
      {role}
    </span>
  );
}

function MessageBubble({ message }: { message: Message }) {
  return (
    <div className={`${styles.messageBubble} ${styles[`bubble_${message.role}`]}`}>
      <RoleBadge role={message.role} />
      <div className={styles.messageContent}>
        <pre className={styles.messageText}>{message.content}</pre>
        {message.toolCalls && message.toolCalls.length > 0 && (
          <div className={styles.toolCalls}>
            {message.toolCalls.map((call, i) => (
              <div key={i} className={styles.toolCallCard}>
                <span className={styles.toolCallBadge}>Tool Call</span>
                <pre>{JSON.stringify(call, null, 2)}</pre>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}

function StreamingBubble({ content }: { content: string }) {
  return (
    <div className={`${styles.messageBubble} ${styles.bubble_assistant} ${styles.streaming}`}>
      <RoleBadge role="assistant" />
      <div className={styles.messageContent}>
        <pre className={styles.messageText}>{content}<span className={styles.cursor}>|</span></pre>
      </div>
    </div>
  );
}

export default function ChatPanel({
  messages,
  isLoading,
  streamingContent,
  onSend,
  onReset,
}: ChatPanelProps) {
  const [input, setInput] = useState('');
  const bottomRef = useRef<HTMLDivElement>(null);
  const textareaRef = useRef<HTMLTextAreaElement>(null);

  // Auto-scroll to bottom on new messages
  useEffect(() => {
    bottomRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [messages, streamingContent]);

  const handleSend = useCallback(() => {
    const trimmed = input.trim();
    if (!trimmed || isLoading) return;
    setInput('');
    onSend(trimmed);
  }, [input, isLoading, onSend]);

  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent<HTMLTextAreaElement>) => {
      if (e.key === 'Enter' && !e.shiftKey) {
        e.preventDefault();
        handleSend();
      }
    },
    [handleSend],
  );

  return (
    <div className={styles.container}>
      <div className={styles.header}>
        <h2 className={styles.title}>Chat</h2>
        <button
          className={styles.resetButton}
          onClick={onReset}
          title="Reset session"
          aria-label="Reset session"
        >
          Reset
        </button>
      </div>

      <div className={styles.messageList} role="log" aria-live="polite">
        {messages.length === 0 && (
          <div className={styles.emptyState}>
            Start a conversation. Press Enter to send, Shift+Enter for newline.
          </div>
        )}
        {messages.map((msg, i) => (
          <MessageBubble key={i} message={msg} />
        ))}
        {streamingContent && isLoading && (
          <StreamingBubble content={streamingContent} />
        )}
        {isLoading && !streamingContent && (
          <div className={styles.thinkingIndicator} aria-label="Thinking">
            <span className={styles.dot} />
            <span className={styles.dot} />
            <span className={styles.dot} />
          </div>
        )}
        <div ref={bottomRef} />
      </div>

      <div className={styles.inputArea}>
        <textarea
          ref={textareaRef}
          className={styles.textarea}
          value={input}
          onChange={(e) => setInput(e.target.value)}
          onKeyDown={handleKeyDown}
          placeholder="Type a message... (Enter to send, Shift+Enter for newline)"
          rows={3}
          disabled={isLoading}
          aria-label="Message input"
        />
        <button
          className={styles.sendButton}
          onClick={handleSend}
          disabled={isLoading || !input.trim()}
          aria-label="Send message"
        >
          {isLoading ? '...' : 'Send'}
        </button>
      </div>
    </div>
  );
}
