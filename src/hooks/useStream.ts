import { useState, useEffect, useRef, useCallback } from 'react';
import { onTokenStream } from '../lib/tauri';
import type { TokenEvent } from '../lib/types';

export interface UseStreamReturn {
  streamingContent: string;
  isStreaming: boolean;
  clearStream: () => void;
}

export function useStream(): UseStreamReturn {
  const [streamingContent, setStreamingContent] = useState('');
  const [isStreaming, setIsStreaming] = useState(false);
  const unlistenRef = useRef<(() => void) | null>(null);

  useEffect(() => {
    let mounted = true;

    onTokenStream((event: TokenEvent) => {
      if (!mounted) return;

      if (event.type === 'text') {
        setIsStreaming(true);
        setStreamingContent((prev) => prev + event.delta);
      } else if (event.type === 'stop') {
        setIsStreaming(false);
      } else if (event.type === 'error') {
        setIsStreaming(false);
        console.error('Stream error:', event.message);
      }
    }).then((unlisten) => {
      if (mounted) {
        unlistenRef.current = unlisten;
      } else {
        unlisten();
      }
    });

    return () => {
      mounted = false;
      unlistenRef.current?.();
    };
  }, []);

  const clearStream = useCallback(() => {
    setStreamingContent('');
    setIsStreaming(false);
  }, []);

  return { streamingContent, isStreaming, clearStream };
}
