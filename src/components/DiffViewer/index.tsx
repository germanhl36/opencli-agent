import { useState } from 'react';
import type { UnifiedDiff, Hunk, DiffLine } from '../../lib/types';
import styles from './DiffViewer.module.css';

interface DiffViewerProps {
  diff: UnifiedDiff;
  onAcceptHunk?: (hunkIndex: number) => void;
  onRejectHunk?: (hunkIndex: number) => void;
  hunkMode?: boolean;
}

function DiffLineView({ line }: { line: DiffLine }) {
  const prefix =
    line.kind === 'added' ? '+' : line.kind === 'removed' ? '-' : ' ';
  return (
    <div className={`${styles.diffLine} ${styles[`line_${line.kind}`]}`}>
      <span className={styles.lineNumber}>{line.oldLineno ?? ''}</span>
      <span className={styles.lineNumber}>{line.newLineno ?? ''}</span>
      <span className={styles.linePrefix}>{prefix}</span>
      <span className={styles.lineContent}>{line.content}</span>
    </div>
  );
}

function HunkView({
  hunk,
  index,
  onAccept,
  onReject,
  hunkMode,
}: {
  hunk: Hunk;
  index: number;
  onAccept?: () => void;
  onReject?: () => void;
  hunkMode?: boolean;
}) {
  return (
    <div className={styles.hunk}>
      <div className={styles.hunkHeader}>
        <span className={styles.hunkHeaderText}>{hunk.header}</span>
        {hunkMode && (
          <div className={styles.hunkActions}>
            <button
              className={`${styles.hunkButton} ${styles.acceptButton}`}
              onClick={onAccept}
              aria-label={`Accept hunk ${index + 1}`}
            >
              Accept
            </button>
            <button
              className={`${styles.hunkButton} ${styles.rejectButton}`}
              onClick={onReject}
              aria-label={`Reject hunk ${index + 1}`}
            >
              Reject
            </button>
          </div>
        )}
      </div>
      <div className={styles.hunkLines}>
        {hunk.lines.map((line, i) => (
          <DiffLineView key={i} line={line} />
        ))}
      </div>
    </div>
  );
}

export default function DiffViewer({
  diff,
  onAcceptHunk,
  onRejectHunk,
  hunkMode = false,
}: DiffViewerProps) {
  const [currentHunk, setCurrentHunk] = useState(0);

  const handleAccept = (index: number) => {
    onAcceptHunk?.(index);
    if (hunkMode && currentHunk < diff.hunks.length - 1) {
      setCurrentHunk((prev) => prev + 1);
    }
  };

  const handleReject = (index: number) => {
    onRejectHunk?.(index);
    if (hunkMode && currentHunk < diff.hunks.length - 1) {
      setCurrentHunk((prev) => prev + 1);
    }
  };

  const displayedHunks = hunkMode
    ? diff.hunks.slice(currentHunk, currentHunk + 1)
    : diff.hunks;

  return (
    <div className={styles.container}>
      <div className={styles.fileHeader}>
        <span className={styles.filePath}>{diff.path}</span>
        {diff.isNewFile && <span className={styles.badge}>NEW</span>}
        {diff.isDeleted && <span className={`${styles.badge} ${styles.badgeDeleted}`}>DELETED</span>}
        {hunkMode && diff.hunks.length > 0 && (
          <span className={styles.hunkProgress}>
            Hunk {currentHunk + 1} / {diff.hunks.length}
          </span>
        )}
      </div>
      <div className={styles.diffContent}>
        {displayedHunks.map((hunk, i) => (
          <HunkView
            key={hunkMode ? currentHunk : i}
            hunk={hunk}
            index={hunkMode ? currentHunk : i}
            onAccept={() => handleAccept(hunkMode ? currentHunk : i)}
            onReject={() => handleReject(hunkMode ? currentHunk : i)}
            hunkMode={hunkMode}
          />
        ))}
        {diff.hunks.length === 0 && (
          <div className={styles.noChanges}>No changes</div>
        )}
      </div>
    </div>
  );
}
