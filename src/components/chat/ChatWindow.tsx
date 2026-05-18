import React, { useState, useEffect, useRef } from 'react';
import { Send, Sparkles, Play, Wrench, Lightbulb, AlertCircle, Check, X, Code, Copy, Plus, GitBranch, ChevronUp, Brain, Zap, Database } from 'lucide-react';
import type { LucideIcon } from 'lucide-react';
import { invoke } from '@tauri-apps/api/core';
import { useChatStore, useConnectionStore } from '../../store';
import styles from './ChatWindow.module.css';

type AIMode = 'create' | 'plan' | 'fix' | 'query';

const modes = [
  { id: 'create', label: 'Create', icon: Sparkles, desc: 'Execute commands directly' },
  { id: 'plan', label: 'Plan', icon: Play, desc: 'Propose plan first' },
  { id: 'fix', label: 'Fix', icon: Wrench, desc: 'Find and fix issues' },
  { id: 'query', label: 'Query', icon: Lightbulb, desc: 'Answer questions' },
];

type ContextScope = 'full-scene' | 'selected-only' | 'active-figure' | 'workspace-root';

interface ContextOption {
  id: ContextScope;
  label: string;
  desc: string;
}

const contexts: ContextOption[] = [
  { id: 'full-scene', label: 'Full Scene', desc: 'Analyze the entire scene hierarchy' },
  { id: 'selected-only', label: 'Selected Only', desc: 'Focus on active selection details' },
  { id: 'active-figure', label: 'Active Figure', desc: 'Target active Genesis skeleton' },
  { id: 'workspace-root', label: 'Worktree', desc: 'Scope to Daz project directory' },
];

type ModelId = 'gemini-3-flash' | 'gemini-3-pro' | 'claude-3-5-sonnet' | 'gpt-4o' | 'local-gguf';

interface ModelOption {
  id: ModelId;
  label: string;
  desc: string;
  icon: LucideIcon;
  color: string;
}

const models: ModelOption[] = [
  { id: 'gemini-3-flash', label: 'Gemini 3 Flash', desc: 'Fast, multimodal reasoning', icon: Sparkles, color: '#a855f7' },
  { id: 'gemini-3-pro', label: 'Gemini 3 Pro', desc: 'High-quality analytical reasoning', icon: Sparkles, color: '#8b5cf6' },
  { id: 'claude-3-5-sonnet', label: 'Claude 3.5 Sonnet', desc: 'Expert scripting & logic', icon: Brain, color: '#f97316' },
  { id: 'gpt-4o', label: 'GPT-4o', desc: 'Versatile problem solver', icon: Zap, color: '#10b981' },
  { id: 'local-gguf', label: 'Local GGUF (phi-2)', desc: 'Offline private LLM', icon: Database, color: '#06b6d4' },
];

// Subcomponent to render DazScript Macros beautifully
function MacroCodeBlock({ code, language }: { code: string; language: string }) {
  const [copied, setCopied] = useState(false);
  const [collapsed, setCollapsed] = useState(false);

  const handleCopy = () => {
    navigator.clipboard.writeText(code);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  return (
    <div className={styles.macroContainer}>
      <div className={styles.macroHeader}>
        <div className={styles.macroTitle}>
          <Code size={14} />
          <span>{language.toUpperCase()} MACRO</span>
        </div>
        <div className={styles.macroActions}>
          <button 
            onClick={() => setCollapsed(!collapsed)}
            className="text-xs px-2 py-1 text-slate-400 hover:text-slate-200 bg-transparent border-none cursor-pointer"
          >
            {collapsed ? 'Expand' : 'Collapse'}
          </button>
          <button 
            onClick={handleCopy}
            className="text-xs px-2 py-1 text-slate-400 hover:text-slate-200 bg-transparent border-none cursor-pointer flex items-center gap-1"
          >
            {copied ? 'Copied' : <Copy size={12} />}
          </button>
        </div>
      </div>
      {!collapsed && (
        <pre className={styles.macroCode}>
          <code>{code}</code>
        </pre>
      )}
      <div className={`${styles.macroBadge} ${styles.success}`}>
        <Check size={10} />
        <span>✓ Automatically Executed on Main Thread</span>
      </div>
    </div>
  );
}

interface ActionParam {
  key: string;
  value: string;
}

interface StructuredAction {
  command: string;
  target?: string;
  parameters?: ActionParam[];
  confidence: number;
}

// Subcomponent to render Action Confirmation dialogs
function InteractiveActionCard({
  userQuery,
  commandName,
}: {
  userQuery: string;
  commandName: string;
}) {
  const [action, setAction] = useState<StructuredAction | null>(null);
  const [loading, setLoading] = useState(false);
  const [status, setStatus] = useState<{ type: 'success' | 'error' | 'rejected'; msg: string } | null>(null);

  useEffect(() => {
    async function loadAction() {
      try {
        const parsed = await invoke<StructuredAction>('parse_ai_action', { input: userQuery });
        if (parsed) {
          setAction(parsed);
        } else {
          setAction({
            command: commandName,
            target: 'Genesis 8',
            parameters: [
              { key: 'details', value: userQuery }
            ],
            confidence: 0.9,
          });
        }
      } catch (err) {
        console.error('Failed to parse action:', err);
        setAction({
          command: commandName,
          target: 'selected',
          parameters: [{ key: 'details', value: userQuery }],
          confidence: 0.8,
        });
      }
    }
    loadAction();
  }, [userQuery, commandName]);

  const handleConfirm = async () => {
    if (!action) return;
    setLoading(true);
    try {
      const res = await invoke<{ success: boolean; message?: string }>('execute_ai_action', { action });
      if (res && res.success) {
        setStatus({ type: 'success', msg: res.message || 'Action executed successfully!' });
      } else {
        setStatus({ type: 'error', msg: res?.message || 'Execution failed.' });
      }
    } catch (err: unknown) {
      const errMsg = err instanceof Error ? err.message : String(err);
      setStatus({ type: 'error', msg: errMsg || 'Execution error.' });
    } finally {
      setLoading(false);
    }
  };

  const handleReject = () => {
    setStatus({ type: 'rejected', msg: 'Action execution cancelled by user.' });
  };

  if (status) {
    return (
      <div className={`${styles.statusContainer}`}>
        {status.type === 'success' && <Check size={14} className="text-emerald-400" />}
        {status.type === 'error' && <X size={14} className="text-rose-400" />}
        {status.type === 'rejected' && <X size={14} className="text-slate-400" />}
        <span className="text-xs text-slate-300">{status.msg}</span>
      </div>
    );
  }

  if (!action) {
    return <div className="text-xs text-slate-500 animate-pulse mt-2">Loading action details...</div>;
  }

  return (
    <div className={styles.actionCard}>
      <div className={styles.actionHeader}>
        <AlertCircle size={18} />
        <span className={styles.actionTitle}>Action Requires Confirmation</span>
      </div>
      <div className={styles.actionDesc}>
        The assistant proposed a high-risk operation. Review the parameters below:
      </div>
      <table className={styles.actionParamsTable}>
        <thead>
          <tr>
            <th>Parameter</th>
            <th>Value</th>
          </tr>
        </thead>
        <tbody>
          <tr>
            <td>Command</td>
            <td><span className="text-cyan-400 font-bold">{action.command}</span></td>
          </tr>
          <tr>
            <td>Target</td>
            <td>{action.target || 'selected'}</td>
          </tr>
          {action.parameters && action.parameters.map((p: ActionParam, idx: number) => (
            <tr key={idx}>
              <td>{p.key}</td>
              <td className="text-purple-300">{p.value}</td>
            </tr>
          ))}
        </tbody>
      </table>
      <div className={styles.actionButtons}>
        <button 
          onClick={handleReject}
          disabled={loading}
          className="px-3 py-1.5 text-xs font-medium text-slate-400 hover:text-slate-200 bg-transparent border border-transparent rounded cursor-pointer transition-all"
        >
          Reject
        </button>
        <button
          onClick={handleConfirm}
          disabled={loading}
          className="px-3 py-1.5 text-xs font-medium bg-cyan-500 hover:bg-cyan-400 text-black border-none rounded cursor-pointer transition-all flex items-center gap-1.5 font-semibold"
        >
          {loading ? 'Executing...' : 'Confirm & Execute'}
        </button>
      </div>
    </div>
  );
}

export default function ChatWindow() {
  const { messages, input, isLoading, setInput, sendMessage } = useChatStore();
  const { status } = useConnectionStore();

  const [mode, setMode] = useState<AIMode>('plan');
  const [selectedModel, setSelectedModel] = useState<ModelId>('gemini-3-flash');
  const [selectedContext, setSelectedContext] = useState<ContextScope>('full-scene');
  const [attachedImages, setAttachedImages] = useState<string[]>([]);
  const [lightboxImage, setLightboxImage] = useState<string | null>(null);

  // Dropdown visibility states
  const [showModelDropdown, setShowModelDropdown] = useState(false);
  const [showContextDropdown, setShowContextDropdown] = useState(false);
  const [showModeDropdown, setShowModeDropdown] = useState(false);

  const initialized = useRef(false);
  const messagesEndRef = useRef<HTMLDivElement>(null);
  const fileInputRef = useRef<HTMLInputElement>(null);
  const textareaRef = useRef<HTMLTextAreaElement>(null);

  useEffect(() => {
    if (!initialized.current && messages.length === 0) {
      initialized.current = true;
      useChatStore.getState().addMessage({
        role: 'assistant',
        content:
          "Hello! I'm your DAZPilot assistant. I can help you create 3D scenes, manage assets, and work with Genesis 8/9 figures. Just describe what you want to do!",
        loading: false,
      });
    }
  }, [messages.length]);

  // Scroll to bottom on new messages
  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [messages, isLoading]);

  // Click outside listener to close dropdowns
  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      const target = event.target as HTMLElement;
      if (!target.closest(`.${styles.dropdownContainer}`)) {
        setShowModelDropdown(false);
        setShowContextDropdown(false);
        setShowModeDropdown(false);
      }
    };
    document.addEventListener('mousedown', handleClickOutside);
    return () => document.removeEventListener('mousedown', handleClickOutside);
  }, []);

  // Auto-grow textarea height on value change
  useEffect(() => {
    if (textareaRef.current) {
      textareaRef.current.style.height = 'auto';
      textareaRef.current.style.height = `${Math.min(textareaRef.current.scrollHeight, 180)}px`;
    }
  }, [input]);

  const handleSend = () => {
    if ((!input.trim() && attachedImages.length === 0) || isLoading) return;
    sendMessage(input, attachedImages);
    setInput('');
    setAttachedImages([]);
    if (textareaRef.current) {
      textareaRef.current.style.height = 'auto';
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent<HTMLTextAreaElement>) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      handleSend();
    }
  };

  const handleFileAttach = (e: React.ChangeEvent<HTMLInputElement>) => {
    const files = e.target.files;
    if (!files) return;

    Array.from(files).forEach((file) => {
      const reader = new FileReader();
      reader.onloadend = () => {
        if (typeof reader.result === 'string') {
          setAttachedImages((prev) => [...prev, reader.result as string]);
        }
      };
      reader.readAsDataURL(file);
    });

    if (fileInputRef.current) {
      fileInputRef.current.value = '';
    }
  };

  const removeAttachedImage = (idxToRemove: number) => {
    setAttachedImages((prev) => prev.filter((_, idx) => idx !== idxToRemove));
  };

  const parseMessageContent = (content: string) => {
    const regex = /```(javascript|js|json|bash|sh|css)?\n([\s\S]*?)\n```/g;
    const parts: React.ReactNode[] = [];
    let lastIndex = 0;
    let match;

    while ((match = regex.exec(content)) !== null) {
      if (match.index > lastIndex) {
        parts.push(<span key={lastIndex}>{content.substring(lastIndex, match.index)}</span>);
      }
      const lang = match[1] || 'js';
      const code = match[2];
      parts.push(<MacroCodeBlock key={match.index} code={code} language={lang} />);
      lastIndex = regex.lastIndex;
    }

    if (lastIndex < content.length) {
      parts.push(<span key={lastIndex}>{content.substring(lastIndex)}</span>);
    }

    return parts.length > 0 ? parts : [content];
  };

  const getPreviousUserQuery = (currentIndex: number) => {
    for (let i = currentIndex - 1; i >= 0; i--) {
      if (messages[i].role === 'user') {
        return messages[i].content;
      }
    }
    return '';
  };

  const getThemeClass = (modelId: ModelId) => {
    switch (modelId) {
      case 'gemini-3-flash':
      case 'gemini-3-pro':
        return styles.themeGemini;
      case 'claude-3-5-sonnet':
        return styles.themeClaude;
      case 'gpt-4o':
        return styles.themeGpt;
      default:
        return styles.themeLocal;
    }
  };

  const activeModelInfo = models.find((m) => m.id === selectedModel) || models[0];
  const activeContextInfo = contexts.find((c) => c.id === selectedContext) || contexts[0];
  const activeModeInfo = modes.find((m) => m.id === mode) || modes[0];

  const ModelIcon = activeModelInfo.icon;
  const ModeIcon = activeModeInfo.icon;

  return (
    <div className={`${styles.container} ${getThemeClass(selectedModel)}`}>
      {/* Lightbox for visual attachments */}
      {lightboxImage && (
        <div className={styles.lightbox} onClick={() => setLightboxImage(null)}>
          <div className={styles.lightboxContent} onClick={(e) => e.stopPropagation()}>
            <img src={lightboxImage} alt="lightbox-preview" className={styles.lightboxImg} />
            <button className={styles.lightboxClose} onClick={() => setLightboxImage(null)}>
              <X size={20} />
            </button>
          </div>
        </div>
      )}

      <div className={styles.header}>
        <div className={styles.headerTitleContainer}>
          <Sparkles className={styles.sparklesHeaderIcon} size={18} />
          <h2 className={styles.headerTitle}>AI Co-Pilot Console</h2>
        </div>
        <div className={styles.headerRight}>
          <div className={`${styles.statusBadge} ${styles[status]}`}>
            <div className={styles.statusDot} />
            <span>{status === 'connected' ? 'Daz Active' : 'Disconnected'}</span>
          </div>
        </div>
      </div>

      <div className={styles.messagesList}>
        {messages.map((msg, index) => {
          const isConfirmation = msg.role === 'assistant' && msg.content.includes('needs confirmation before execution');
          let commandName = '';
          if (isConfirmation) {
            const commandMatch = /Planned action '([a-zA-Z0-9_]+)' needs confirmation/.exec(msg.content);
            commandName = commandMatch ? commandMatch[1] : '';
          }

          return (
            <div key={msg.id} className={`${styles.message} ${styles[msg.role]}`}>
              <div className={styles.messageContent}>
                {msg.loading ? (
                  <span className={styles.processing}>Processing...</span>
                ) : (
                  <>
                    {/* Render message attachments */}
                    {msg.images && msg.images.length > 0 && (
                      <div className={styles.msgAttachedImagesGrid}>
                        {msg.images.map((imgUrl, i) => (
                          <div
                            key={i}
                            className={styles.msgImageWrapper}
                            onClick={() => setLightboxImage(imgUrl)}
                            title="Click to enlarge"
                          >
                            <img src={imgUrl} alt={`attached-${i}`} className={styles.msgImage} />
                          </div>
                        ))}
                      </div>
                    )}
                    <div className="whitespace-pre-wrap">{parseMessageContent(msg.content)}</div>
                    {isConfirmation && (
                      <InteractiveActionCard
                        userQuery={getPreviousUserQuery(index)}
                        commandName={commandName}
                      />
                    )}
                  </>
                )}
              </div>
            </div>
          );
        })}
        {isLoading && (
          <div className={`${styles.message} ${styles.assistant}`}>
            <div className={styles.messageContent}>
              <span className={styles.processing}>
                <span className={styles.spinner} />
                Thinking...
              </span>
            </div>
          </div>
        )}
        <div ref={messagesEndRef} />
      </div>

      {/* Modern custom input container with embedded widgets */}
      <div className={styles.inputArea}>
        <input
          type="file"
          multiple
          accept="image/*"
          ref={fileInputRef}
          className={styles.hiddenFileInput}
          onChange={handleFileAttach}
        />

        <div className={styles.inputContainerCard}>
          {/* Thumbnails preview bar inside prompt card */}
          {attachedImages.length > 0 && (
            <div className={styles.attachmentsPreviewContainer}>
              {attachedImages.map((img, idx) => (
                <div key={idx} className={styles.previewImageChip}>
                  <img src={img} alt={`upload-preview-${idx}`} className={styles.previewThumbnail} />
                  <button
                    className={styles.removePreviewBtn}
                    onClick={() => removeAttachedImage(idx)}
                    title="Remove attachment"
                  >
                    <X size={10} />
                  </button>
                </div>
              ))}
            </div>
          )}

          {/* Autogrow Prompt input */}
          <textarea
            ref={textareaRef}
            rows={1}
            value={input}
            onChange={(e) => setInput(e.target.value)}
            onKeyDown={handleKeyDown}
            className={styles.inputPromptTextarea}
            placeholder={`Ask ${activeModelInfo.label} in ${activeModeInfo.label} mode... (e.g., 'Rotate figure by 45deg')`}
            disabled={isLoading}
          />

          {/* Advanced Bottom Toolbar containing widgets */}
          <div className={styles.inputToolbar}>
            <div className={styles.toolbarLeft}>
              {/* Attach Plus Button */}
              <button
                className={styles.toolbarActionBtn}
                onClick={() => fileInputRef.current?.click()}
                title="Attach images (Vision)"
                disabled={isLoading}
              >
                <Plus size={16} />
              </button>

              {/* Context Dropdown Widget */}
              <div className={styles.dropdownContainer}>
                <button
                  className={`${styles.toolbarDropdownTrigger} ${showContextDropdown ? styles.active : ''}`}
                  onClick={() => {
                    setShowContextDropdown(!showContextDropdown);
                    setShowModelDropdown(false);
                    setShowModeDropdown(false);
                  }}
                  title="Workspace context scope"
                  disabled={isLoading}
                >
                  <GitBranch size={13} className={styles.widgetIcon} />
                  <span>{activeContextInfo.label}</span>
                </button>

                {showContextDropdown && (
                  <div className={styles.floatingMenuCard}>
                    <div className={styles.menuHeader}>Workspace Scope</div>
                    <div className={styles.menuOptionsList}>
                      {contexts.map((ctx) => (
                        <button
                          key={ctx.id}
                          className={`${styles.menuOptionRow} ${selectedContext === ctx.id ? styles.active : ''}`}
                          onClick={() => {
                            setSelectedContext(ctx.id);
                            setShowContextDropdown(false);
                          }}
                        >
                          <div className={styles.optionLabel}>{ctx.label}</div>
                          <div className={styles.optionDesc}>{ctx.desc}</div>
                        </button>
                      ))}
                    </div>
                  </div>
                )}
              </div>
            </div>

            <div className={styles.toolbarRight}>
              {/* Model Selector Dropdown Widget */}
              <div className={styles.dropdownContainer}>
                <button
                  className={`${styles.toolbarDropdownTrigger} ${showModelDropdown ? styles.active : ''}`}
                  onClick={() => {
                    setShowModelDropdown(!showModelDropdown);
                    setShowContextDropdown(false);
                    setShowModeDropdown(false);
                  }}
                  title="Change AI model"
                  disabled={isLoading}
                >
                  <ModelIcon size={13} className={styles.widgetIcon} />
                  <span>{activeModelInfo.label}</span>
                  <ChevronUp size={12} className={`${styles.chevron} ${showModelDropdown ? styles.open : ''}`} />
                </button>

                {showModelDropdown && (
                  <div className={styles.floatingMenuCard}>
                    <div className={styles.menuHeader}>Select AI Engine</div>
                    <div className={styles.menuOptionsList}>
                      {models.map((mod) => {
                        const ItemIcon = mod.icon;
                        return (
                          <button
                            key={mod.id}
                            style={{ '--accent-color': mod.color } as React.CSSProperties}
                            className={`${styles.menuOptionRow} ${styles.modelOption} ${
                              selectedModel === mod.id ? styles.active : ''
                            }`}
                            onClick={() => {
                              setSelectedModel(mod.id);
                              setShowModelDropdown(false);
                            }}
                          >
                            <div className={styles.optionRowHeader}>
                              <ItemIcon size={14} className={styles.modelIcon} />
                              <div className={styles.optionLabel}>{mod.label}</div>
                            </div>
                            <div className={styles.optionDesc}>{mod.desc}</div>
                          </button>
                        );
                      })}
                    </div>
                  </div>
                )}
              </div>

              {/* Mode Dropdown Widget */}
              <div className={styles.dropdownContainer}>
                <button
                  className={`${styles.toolbarDropdownTrigger} ${showModeDropdown ? styles.active : ''}`}
                  onClick={() => {
                    setShowModeDropdown(!showModeDropdown);
                    setShowContextDropdown(false);
                    setShowModelDropdown(false);
                  }}
                  title="Change assistant execution mode"
                  disabled={isLoading}
                >
                  <ModeIcon size={13} className={styles.widgetIcon} />
                  <span>{activeModeInfo.label}</span>
                  <ChevronUp size={12} className={`${styles.chevron} ${showModeDropdown ? styles.open : ''}`} />
                </button>

                {showModeDropdown && (
                  <div className={styles.floatingMenuCard}>
                    <div className={styles.menuHeader}>Execution Mode</div>
                    <div className={styles.menuOptionsList}>
                      {modes.map((m) => {
                        const ItemIcon = m.icon;
                        return (
                          <button
                            key={m.id}
                            className={`${styles.menuOptionRow} ${mode === m.id ? styles.active : ''}`}
                            onClick={() => {
                              setMode(m.id as AIMode);
                              setShowModeDropdown(false);
                            }}
                          >
                            <div className={styles.optionRowHeader}>
                              <ItemIcon size={14} className={styles.modeIcon} />
                              <div className={styles.optionLabel}>{m.label}</div>
                            </div>
                            <div className={styles.optionDesc}>{m.desc}</div>
                          </button>
                        );
                      })}
                    </div>
                  </div>
                )}
              </div>

              {/* Send Button */}
              <button
                className={styles.promptSendBtn}
                onClick={handleSend}
                disabled={isLoading || (!input.trim() && attachedImages.length === 0)}
                title="Send query"
              >
                <Send size={14} />
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
