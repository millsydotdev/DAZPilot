import { useState, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Play, RotateCcw, Loader2, CheckCircle2, XCircle } from 'lucide-react';
import { Button } from '../ui/Button';
import styles from './AgentTester.module.css';

interface AgentResponse {
  success: boolean;
  result: string | null;
  error: string | null;
  actions: Array<{ action_type: string; command: string; args: string[] }>;
  sub_results: Array<{
    agent_type: string;
    success: boolean;
    result: string | null;
  }>;
}

interface AgentTesterProps {
  agentType: string | null;
}

export default function AgentTester({ agentType }: AgentTesterProps) {
  const [input, setInput] = useState('');
  const [response, setResponse] = useState<AgentResponse | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const runTest = useCallback(async () => {
    if (!agentType || !input.trim()) return;
    setLoading(true);
    setError(null);
    setResponse(null);
    try {
      const result = await invoke<AgentResponse>('test_agent', {
        agentType,
        input: input.trim(),
      });
      setResponse(result);
    } catch (err) {
      setError(String(err));
    } finally {
      setLoading(false);
    }
  }, [agentType, input]);

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      runTest();
    }
  };

  const clear = () => {
    setInput('');
    setResponse(null);
    setError(null);
  };

  return (
    <div className={styles.container}>
      <div className={styles.header}>
        <Play size={16} />
        <span>Agent Tester</span>
        {agentType && <span className={styles.target}>{agentType}</span>}
      </div>

      <div className={styles.body}>
        <div className={styles.inputRow}>
          <textarea
            className={styles.textarea}
            placeholder={
              agentType ? `Enter test input for "${agentType}"...` : 'Select an agent first...'
            }
            value={input}
            onChange={(e) => setInput(e.target.value)}
            onKeyDown={handleKeyDown}
            disabled={!agentType}
            rows={3}
          />
        </div>

        <div className={styles.actions}>
          <Button onClick={runTest} disabled={!agentType || !input.trim() || loading} size="sm">
            {loading ? <Loader2 size={14} className={styles.spin} /> : <Play size={14} />}
            {loading ? 'Running...' : 'Run'}
          </Button>
          <Button
            onClick={clear}
            variant="ghost"
            size="sm"
            disabled={!input && !response && !error}
          >
            <RotateCcw size={14} />
            Clear
          </Button>
        </div>

        {error && (
          <div className={styles.error}>
            <XCircle size={14} />
            {error}
          </div>
        )}

        {response && (
          <div className={styles.responsePanel}>
            <div
              className={`${styles.statusBadge} ${
                response.success ? styles.success : styles.failure
              }`}
            >
              {response.success ? <CheckCircle2 size={14} /> : <XCircle size={14} />}
              {response.success ? 'Success' : 'Failed'}
            </div>

            {response.result && (
              <div className={styles.resultBlock}>
                <div className={styles.blockLabel}>Result</div>
                <pre className={styles.pre}>{response.result}</pre>
              </div>
            )}

            {response.error && (
              <div className={styles.resultBlock}>
                <div className={styles.blockLabel}>Error</div>
                <pre className={styles.preError}>{response.error}</pre>
              </div>
            )}

            {response.actions.length > 0 && (
              <div className={styles.resultBlock}>
                <div className={styles.blockLabel}>Actions ({response.actions.length})</div>
                <div className={styles.actionList}>
                  {response.actions.map((a, i) => (
                    <div key={i} className={styles.actionItem}>
                      <span className={styles.actionCmd}>{a.command}</span>
                      <span className={styles.actionType}>{a.action_type}</span>
                      {a.args.length > 0 && (
                        <span className={styles.actionArgs}>{a.args.join(', ')}</span>
                      )}
                    </div>
                  ))}
                </div>
              </div>
            )}

            {response.sub_results.length > 0 && (
              <div className={styles.resultBlock}>
                <div className={styles.blockLabel}>Sub-results ({response.sub_results.length})</div>
                {response.sub_results.map((sr, i) => (
                  <div key={i} className={styles.subResult}>
                    <span className={styles.subAgentType}>{sr.agent_type}</span>
                    <span className={sr.success ? styles.subSuccess : styles.subFailure}>
                      {sr.success ? 'OK' : 'FAIL'}
                    </span>
                    {sr.result && <span className={styles.subText}>{sr.result}</span>}
                  </div>
                ))}
              </div>
            )}
          </div>
        )}
      </div>
    </div>
  );
}
