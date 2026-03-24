import { useState, useEffect, useRef, useCallback } from 'react';
import { onApprovalRequested, resolveApproval } from '../lib/tauri';
import type { ActionRequest } from '../lib/types';

type ApprovalState =
  | { status: 'idle' }
  | { status: 'pending'; request: ActionRequest }
  | { status: 'resolved'; outcome: 'approved' | 'rejected' };

export interface UseApprovalReturn {
  approvalState: ApprovalState;
  approve: () => Promise<void>;
  reject: () => Promise<void>;
  currentRequest: ActionRequest | null;
}

export function useApproval(): UseApprovalReturn {
  const [approvalState, setApprovalState] = useState<ApprovalState>({ status: 'idle' });
  const unlistenRef = useRef<(() => void) | null>(null);

  useEffect(() => {
    let mounted = true;

    onApprovalRequested((request: ActionRequest) => {
      if (!mounted) return;
      setApprovalState({ status: 'pending', request });
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

  const approve = useCallback(async () => {
    if (approvalState.status !== 'pending') return;
    const { request } = approvalState;
    setApprovalState({ status: 'resolved', outcome: 'approved' });
    await resolveApproval(request.id, true);
    // Reset to idle after a short delay
    setTimeout(() => setApprovalState({ status: 'idle' }), 500);
  }, [approvalState]);

  const reject = useCallback(async () => {
    if (approvalState.status !== 'pending') return;
    const { request } = approvalState;
    setApprovalState({ status: 'resolved', outcome: 'rejected' });
    await resolveApproval(request.id, false);
    setTimeout(() => setApprovalState({ status: 'idle' }), 500);
  }, [approvalState]);

  const currentRequest =
    approvalState.status === 'pending' ? approvalState.request : null;

  return { approvalState, approve, reject, currentRequest };
}
