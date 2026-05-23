import { useEffect, useState } from 'react';
import { X, Shield, Check, XIcon, Copy, Trash2, FileCode, Edit, Play } from 'lucide-react';
import { listen } from '@tauri-apps/api/event';
import { useScriptApprovalStore, type ScriptSuggestion } from '../store/scriptApprovalStore';
import ScriptEditor from './scripting/ScriptEditor';
import styles from './ScriptApprovalPanel.module.css';

interface ScriptSuggestionEvent {
  id: string;
  script: string;
  context: string;
  timestamp: string;
}

export function ScriptApprovalPanel() {
  const {
    suggestions,
    isOpen,
    addSuggestion,
    rejectScript,
    updateScript,
    executeScript,
    isLoading,
    setOpen,
  } = useScriptApprovalStore();
  const [isEditorOpen, setIsEditorOpen] = useState(false);
  const [editingScriptId, setEditingScriptId] = useState<string | null>(null);

  useEffect(() => {
    let unlisten: (() => void) | undefined;

    const setupListener = async () => {
      unlisten = await listen<ScriptSuggestionEvent>('script-suggestion', (event) => {
        addSuggestion(event.payload);
      });
    };

    setupListener();
    return () => {
      if (unlisten) unlisten();
    };
  }, [addSuggestion]);

  const handleCopy = async (script: string) => {
    try {
      await navigator.clipboard.writeText(script);
    } catch (e) {
      console.error('Failed to copy script:', e);
    }
  };

  const formatTimestamp = (timestamp: string) => {
    try {
      const date = new Date(timestamp);
      return date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
    } catch {
      return timestamp;
    }
  };

  const getStatusClass = (status: ScriptSuggestion['status']) => {
    switch (status) {
      case 'pending':
        return styles.statusPending;
      case 'approved':
        return styles.statusApproved;
      case 'rejected':
        return styles.statusRejected;
      case 'draft':
        return styles.statusDraft;
    }
  };

  const getStatusLabel = (status: ScriptSuggestion['status']) => {
    switch (status) {
      case 'pending':
        return 'Pending Review';
      case 'approved':
        return 'Approved';
      case 'rejected':
        return 'Rejected';
      case 'draft':
        return 'Draft';
    }
  };

  const pendingCount = suggestions.filter((s) => s.status === 'pending').length;

  const handleExecuteScript = async (id: string) => {
    await executeScript(id);
  };

  const handleEditScript = (id: string) => {
    setEditingScriptId(id);
    setIsEditorOpen(true);
  };

  const handleSaveScript = (script: string) => {
    if (editingScriptId) {
      updateScript(editingScriptId, script);
      setIsEditorOpen(false);
      setEditingScriptId(null);
    }
  };

  if (!isOpen) return null;

  return (
    <>
      <div
        className={styles.overlay}
        role="button"
        tabIndex={0}
        onClick={() => setOpen(false)}
        onKeyDown={(e) => {
          if (e.key === 'Enter' || e.key === ' ') {
            setOpen(false);
          }
        }}
      />
      <div className={styles.panel}>
        <div className={styles.header}>
          <div className={styles.headerLeft}>
            <Shield size={20} className={styles.headerIcon} />
            <h3 className={styles.title}>
              Script Approval
              {pendingCount > 0 && (
                <span style={{ marginLeft: '8px', fontSize: '12px', opacity: 0.7 }}>
                  ({pendingCount} pending)
                </span>
              )}
            </h3>
          </div>
          <div className={styles.headerActions}>
            <button
              className={styles.iconButton}
              onClick={() => setOpen(false)}
              title="Close panel"
              aria-label="Close panel"
            >
              <X size={18} />
            </button>
          </div>
        </div>

        <div className={styles.content}>
          {suggestions.length === 0 ? (
            <div className={styles.empty}>
              <FileCode size={48} className={styles.emptyIcon} />
              <p className={styles.emptyText}>
                No script suggestions yet. When the AI suggests DazScript macros, they will appear
                here for your review.
              </p>
            </div>
          ) : (
            suggestions.map((suggestion) => (
              <div key={suggestion.id} className={styles.suggestionCard}>
                <div className={styles.cardHeader}>
                  <div className={styles.cardHeaderLeft}>
                    <div className={`${styles.statusDot} ${getStatusClass(suggestion.status)}`} />
                    <span className={`${getStatusClass(suggestion.status)}`}>
                      <span className={styles.statusText}>{getStatusLabel(suggestion.status)}</span>
                    </span>
                  </div>
                  <span className={styles.timestamp}>{formatTimestamp(suggestion.timestamp)}</span>
                </div>

                <div className={styles.context}>
                  <div className={styles.contextLabel}>User Request</div>
                  {suggestion.context}
                </div>

                <div className={styles.scriptContainer} aria-live="polite">
                  {editingScriptId === suggestion.id && isEditorOpen ? (
                    <ScriptEditor
                      initialScript={suggestion.script}
                      onScriptSaved={handleSaveScript}
                      onScriptExecuted={async () => {
                        await handleExecuteScript(suggestion.id);
                        setIsEditorOpen(false);
                        setEditingScriptId(null);
                      }}
                    />
                  ) : (
                    <pre className={styles.scriptCode}>{suggestion.script}</pre>
                  )}
                </div>

                {suggestion.status === 'pending' ||
                  (suggestion.status === 'draft' && (
                    <div className={styles.cardActions}>
                      <button
                        className={styles.approveButton}
                        onClick={() => handleExecuteScript(suggestion.id)}
                        disabled={isLoading}
                        aria-label="Approve script"
                      >
                        {isLoading ? (
                          <>
                            <Play size={14} className="animate-spin" />
                            Executing...
                          </>
                        ) : (
                          <>
                            <Check size={14} />
                            Approve & Execute
                          </>
                        )}
                      </button>
                      <button
                        className={styles.rejectButton}
                        onClick={() => rejectScript(suggestion.id)}
                        aria-label="Deny script"
                      >
                        <XIcon size={14} />
                        Reject
                      </button>
                      <button
                        className={styles.editButton}
                        onClick={() => handleEditScript(suggestion.id)}
                        aria-label="Edit script"
                      >
                        <Edit size={14} />
                        Edit
                      </button>
                      <button
                        className={styles.copyButton}
                        onClick={() => handleCopy(suggestion.script)}
                        title="Copy script"
                        aria-label="Copy script"
                      >
                        <Copy size={14} />
                      </button>
                    </div>
                  ))}

                {suggestion.status !== 'pending' && suggestion.status !== 'draft' && (
                  <div className={styles.cardActions}>
                    <button
                      className={styles.copyButton}
                      onClick={() => handleCopy(suggestion.script)}
                      title="Copy script"
                      aria-label="Copy script"
                    >
                      <Copy size={14} />
                      Copy Script
                    </button>
                    <button
                      className={styles.editButton}
                      onClick={() => handleEditScript(suggestion.id)}
                      aria-label="Edit script"
                    >
                      <Edit size={14} />
                      Edit
                    </button>
                  </div>
                )}
              </div>
            ))
          )}
        </div>

        {suggestions.length > 0 && (
          <div className={styles.footer}>
            <button
              className={styles.clearButton}
              onClick={() => {
                setIsEditorOpen(false);
                setEditingScriptId(null);
              }}
              aria-label="Clear history"
            >
              <Trash2 size={12} />
              Clear History
            </button>
          </div>
        )}
      </div>
    </>
  );
}
