import React, { useEffect, useRef, useCallback } from 'react';
import type { Message } from '../../lib/types';
import styles from './ChatPanel.module.css';

interface SelectedFile {
  name: string;
  content: string;
}

interface ChatPanelProps {
  messages: Message[];
  isLoading: boolean;
  streamingContent?: string;
  inputValue: string;
  onInputChange: (value: string) => void;
  onSend: (content: string) => void;
  onReset: () => void;
  workspacePath?: string | null;
  selectedFile?: SelectedFile | null;
  onClearFile?: () => void;
  onOpenFolder: () => void;
  onOpenFile: () => void;
}

function RoleBadge({ role }: { role: string }) {
  return (
    <span className={`${styles.roleBadge} ${styles[`role_${role}`]}`}>{role}</span>
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
  inputValue,
  onInputChange,
  onSend,
  onReset,
  workspacePath,
  selectedFile,
  onClearFile,
  onOpenFolder,
  onOpenFile,
}: ChatPanelProps) {
  const bottomRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    bottomRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [messages, streamingContent]);

  const handleSend = useCallback(() => {
    const trimmed = inputValue.trim();
    if (!trimmed || isLoading) return;

    // If a file is selected and the user sends from the input, combine them
    onSend(trimmed);
  }, [inputValue, isLoading, onSend]);

  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent<HTMLTextAreaElement>) => {
      if (e.key === 'Enter' && !e.shiftKey) {
        e.preventDefault();
        handleSend();
      }
    },
    [handleSend],
  );

  const emptyAndNoFile = messages.length === 0 && !selectedFile;

  return (
    <div className={styles.container}>
      <div className={styles.header}>
        <h2 className={styles.title}>Chat</h2>
        <div className={styles.headerActions}>
          <button className={styles.openButton} onClick={onOpenFolder} title="Open folder as workspace" aria-label="Open folder">
            📁 Folder
          </button>
          <button className={styles.openButton} onClick={onOpenFile} title="Select a file to use with skills or chat" aria-label="Open file">
            📄 File
          </button>
          <button className={styles.resetButton} onClick={onReset} title="Reset session" aria-label="Reset session">
            Reset
          </button>
        </div>
      </div>

      {/* Context bar — shows active workspace and/or selected file */}
      {(workspacePath || selectedFile) && (
        <div className={styles.contextBar}>
          {workspacePath && (
            <span className={styles.contextItem} title={workspacePath}>
              🗂 <span className={styles.contextPath}>{workspacePath}</span>
            </span>
          )}
          {selectedFile && (
            <span className={styles.contextItem}>
              📄 <span className={styles.contextFileName}>{selectedFile.name}</span>
              <button
                className={styles.clearFileBtn}
                onClick={onClearFile}
                aria-label="Remove selected file"
                title="Remove file from context"
              >
                ✕
              </button>
            </span>
          )}
        </div>
      )}

      <div className={styles.messageList} role="log" aria-live="polite">
        {emptyAndNoFile && (
          <div className={styles.emptyState}>
            <p>Type a message, open a 📁 folder or 📄 file, then press Enter or activate a skill.</p>
          </div>
        )}
        {messages.map((msg, i) => (
          <MessageBubble key={i} message={msg} />
        ))}
        {streamingContent && isLoading && <StreamingBubble content={streamingContent} />}
        {isLoading && !streamingContent && (
          <div className={styles.thinkingIndicator} aria-label="Thinking">
            <span className={styles.dot} /><span className={styles.dot} /><span className={styles.dot} />
          </div>
        )}
        <div ref={bottomRef} />
      </div>

      <div className={styles.inputArea}>
        <textarea
          className={styles.textarea}
          value={inputValue}
          onChange={(e) => onInputChange(e.target.value)}
          onKeyDown={handleKeyDown}
          placeholder={
            selectedFile
              ? `Add instructions for "${selectedFile.name}"… (Enter to send, or activate a skill)`
              : 'Type a message… (Enter to send, Shift+Enter for newline)'
          }
          rows={3}
          disabled={isLoading}
          aria-label="Message input"
        />
        <button
          className={styles.sendButton}
          onClick={handleSend}
          disabled={isLoading || !inputValue.trim()}
          aria-label="Send message"
        >
          {isLoading ? '…' : 'Send'}
        </button>
      </div>
    </div>
  );
}
