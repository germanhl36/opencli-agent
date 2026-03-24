import { useState, useEffect, useRef } from 'react';
import type { ActionRequest, RiskLevel } from '../../lib/types';
import styles from './ApprovalDialog.module.css';

interface ApprovalDialogProps {
  request: ActionRequest | null;
  onApprove: () => void;
  onReject: () => void;
}

const ACTION_ICONS: Record<string, string> = {
  file_write: '📝',
  file_delete: '🗑️',
  dir_create: '📁',
  shell_run: '💻',
};

const RISK_LABELS: Record<RiskLevel, string> = {
  low: 'Low Risk',
  medium: 'Medium Risk',
  high: 'HIGH RISK',
};

function RiskBadge({ risk }: { risk: RiskLevel }) {
  return (
    <span className={`${styles.riskBadge} ${styles[`risk_${risk}`]}`}>
      {RISK_LABELS[risk]}
    </span>
  );
}

export default function ApprovalDialog({
  request,
  onApprove,
  onReject,
}: ApprovalDialogProps) {
  const [confirmText, setConfirmText] = useState('');
  const confirmInputRef = useRef<HTMLInputElement>(null);
  const dialogRef = useRef<HTMLDivElement>(null);

  const isHighRisk = request?.risk === 'high';
  const canApprove = !isHighRisk || confirmText === 'confirm';

  useEffect(() => {
    if (request) {
      setConfirmText('');
      // Focus confirm input for high-risk, else focus reject button
      setTimeout(() => {
        if (isHighRisk && confirmInputRef.current) {
          confirmInputRef.current.focus();
        }
      }, 100);
    }
  }, [request, isHighRisk]);

  // Handle escape key
  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if (e.key === 'Escape' && request) {
        onReject();
      }
    };
    document.addEventListener('keydown', handler);
    return () => document.removeEventListener('keydown', handler);
  }, [request, onReject]);

  if (!request) return null;

  const icon = ACTION_ICONS[request.action] || '⚡';

  return (
    <div
      className={styles.overlay}
      onClick={(e) => {
        if (e.target === e.currentTarget) onReject();
      }}
    >
      <div
        ref={dialogRef}
        role="dialog"
        aria-modal="true"
        aria-labelledby="approval-title"
        className={styles.dialog}
      >
        <div className={styles.header}>
          <span className={styles.actionIcon} aria-hidden="true">{icon}</span>
          <h2 id="approval-title" className={styles.title}>
            Action Requires Approval
          </h2>
          <RiskBadge risk={request.risk} />
        </div>

        <div className={styles.body}>
          <div className={styles.field}>
            <label className={styles.fieldLabel}>Action</label>
            <code className={styles.fieldValue}>{request.action}</code>
          </div>

          <div className={styles.field}>
            <label className={styles.fieldLabel}>Target Path</label>
            <code className={`${styles.fieldValue} ${styles.targetPath}`}>
              {request.targetPath || '—'}
            </code>
          </div>

          <div className={styles.field}>
            <label className={styles.fieldLabel}>Description</label>
            <p className={styles.description}>{request.description}</p>
          </div>

          {Object.keys(request.args).length > 0 && (
            <div className={styles.field}>
              <label className={styles.fieldLabel}>Arguments</label>
              <pre className={styles.argsPreview}>
                {JSON.stringify(request.args, null, 2)}
              </pre>
            </div>
          )}

          {isHighRisk && (
            <div className={styles.confirmSection}>
              <p className={styles.confirmPrompt}>
                This is a HIGH RISK action. Type <strong>confirm</strong> to approve.
              </p>
              <input
                ref={confirmInputRef}
                type="text"
                className={styles.confirmInput}
                value={confirmText}
                onChange={(e) => setConfirmText(e.target.value)}
                placeholder='Type "confirm"'
                aria-label='Type "confirm" to enable approve button'
                autoComplete="off"
                spellCheck={false}
              />
            </div>
          )}
        </div>

        <div className={styles.footer}>
          <button
            className={styles.rejectButton}
            onClick={onReject}
            autoFocus={!isHighRisk}
          >
            Reject
          </button>
          <button
            className={styles.approveButton}
            onClick={onApprove}
            disabled={!canApprove}
            aria-disabled={!canApprove}
          >
            Approve
          </button>
        </div>
      </div>
    </div>
  );
}
