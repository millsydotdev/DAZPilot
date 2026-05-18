import React, { useState, useEffect, useRef } from 'react';
import { Send, Sparkles, Play, Wrench, Lightbulb, AlertCircle, Check, X, Code, Copy } from 'lucide-react';
import { invoke } from '@tauri-apps/api/core';
import { useChatStore } from '../../store';
import { Button, Input } from '../ui';
import styles from './ChatWindow.module.css';

type AIMode = 'create' | 'plan' | 'fix' | 'query';

const modes = [
  { id: 'create', label: 'Create', icon: Sparkles, desc: 'Execute commands directly' },
  { id: 'plan', label: 'Plan', icon: Play, desc: 'Propose plan first' },
  { id: 'fix', label: 'Fix', icon: Wrench, desc: 'Find and fix issues' },
  { id: 'query', label: 'Query', icon: Lightbulb, desc: 'Answer questions' },
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

  const [mode, setMode] = useState<AIMode>('create');
  const initialized = useRef(false);
  const messagesEndRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!initialized.current && messages.length === 0) {
      initialized.current = true;
      useChatStore.getState().addMessage({
        role: 'assistant',
        content:
          "Hello! I'm your DazPilot assistant. I can help you create 3D scenes, manage assets, and work with Genesis 8/9 figures. Just describe what you want to do!",
        loading: false,
      });
    }
  }, [messages.length]);

  // Scroll to bottom on new messages
  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [messages, isLoading]);

  const handleSend = () => {
    if (!input.trim() || isLoading) return;
    sendMessage(input);
    setInput('');
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

  return (
    <div className={styles.container}>
      <div className={styles.header}>
        <div className={styles.modeSelector}>
          {modes.map((m) => (
            <button
              key={m.id}
              className={`${styles.modeButton} ${mode === m.id ? styles.active : ''}`}
              onClick={() => setMode(m.id as AIMode)}
            >
              <m.icon size={16} />
              <span>{m.label}</span>
            </button>
          ))}
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
              <span className={styles.processing}>Thinking...</span>
            </div>
          </div>
        )}
        <div ref={messagesEndRef} />
      </div>

      <div className={styles.inputArea}>
        <Input
          className={styles.input}
          value={input}
          onChange={(e) => setInput(e.target.value)}
          onKeyDown={(e) => e.key === 'Enter' && handleSend()}
          placeholder="Describe what you want to create... (e.g., 'Select my Genesis 8 figure')"
          disabled={isLoading}
        />
        <Button
          className={styles.sendButton}
          onClick={handleSend}
          disabled={isLoading || !input.trim()}
          icon={<Send size={20} />}
        >
          Send
        </Button>
      </div>
    </div>
  );
}
