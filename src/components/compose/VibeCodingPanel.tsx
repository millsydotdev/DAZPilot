import { useState, useCallback } from 'react';
import { Wand2, Loader2, Check, X, AlertCircle } from 'lucide-react';
import { invoke } from '@tauri-apps/api/core';
import { Button, Card, Text } from '../ui';
import { useToastStore } from '../../store';
import styles from './VibeCodingPanel.module.css';

interface StepResult {
  step: number;
  action: string;
  status: 'pending' | 'running' | 'success' | 'failed';
  message?: string;
}

export default function VibeCodingPanel() {
  const [description, setDescription] = useState('');
  const [composing, setComposing] = useState(false);
  const [steps, setSteps] = useState<StepResult[]>([]);
  const [result, setResult] = useState<{ data?: Record<string, unknown>; message?: string } | null>(
    null
  );
  const { addToast } = useToastStore();

  const handleCompose = useCallback(async () => {
    if (!description.trim()) return;

    setComposing(true);
    setSteps([]);
    setResult(null);

    try {
      const raw = await invoke<Record<string, unknown>>('execute_scene_composition', {
        description: description.trim(),
      });

      const actionSteps: StepResult[] = (Array.isArray(raw) ? raw : [raw]).map((s, i) => ({
        step: i + 1,
        action: (s.action as string) || 'Completed',
        status: (s.status as StepResult['status']) || 'success',
        message: s.message as string | undefined,
      }));

      setSteps(actionSteps);

      if (actionSteps.every((s) => s.status === 'success')) {
        setResult({ data: raw as Record<string, unknown> });
        addToast('Scene composed successfully', 'success');
      } else {
        const failed = actionSteps.find((s) => s.status === 'failed');
        setResult({ message: failed?.message || 'Some steps failed' });
        addToast('Composition completed with errors', 'warning');
      }
    } catch (err) {
      const message = typeof err === 'string' ? err : 'Composition failed';
      setSteps([{ step: 1, action: 'Composition', status: 'failed', message }]);
      setResult({ message });
      addToast(message, 'error');
    } finally {
      setComposing(false);
    }
  }, [description, addToast]);

  const statusIcon = (status: StepResult['status']) => {
    switch (status) {
      case 'pending':
        return <div className={styles.pendingDot} />;
      case 'running':
        return <Loader2 size={14} className={styles.spinner} />;
      case 'success':
        return <Check size={14} className={styles.checkIcon} />;
      case 'failed':
        return <X size={14} className={styles.xIcon} />;
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' && (e.ctrlKey || e.metaKey)) {
      handleCompose();
    }
  };

  return (
    <div className={styles.container}>
      <div className={styles.header}>
        <h2 className={styles.title}>
          <Wand2 size={18} className={styles.wandIcon} />
          Vibe Coding
        </h2>
        <Text size="sm" className={styles.subtitle}>
          Describe a scene in natural language and let AI compose it
        </Text>
      </div>

      <div className={styles.inputArea}>
        <textarea
          className={styles.textarea}
          placeholder="e.g. A cozy mountain cabin at sunset with warm fireplace glow and snow falling outside..."
          value={description}
          onChange={(e) => setDescription(e.target.value)}
          onKeyDown={handleKeyDown}
          disabled={composing}
          rows={4}
        />
        <div className={styles.inputFooter}>
          <Text size="xs" className={styles.hint}>
            Press Ctrl+Enter to compose
          </Text>
          <Button
            onClick={handleCompose}
            disabled={composing || !description.trim()}
            loading={composing}
            icon={<Wand2 size={14} />}
          >
            Compose
          </Button>
        </div>
      </div>

      {steps.length > 0 && (
        <div className={styles.stepsSection}>
          <h3 className={styles.sectionTitle}>Steps</h3>
          <div className={styles.stepsList}>
            {steps.map((step) => (
              <div key={step.step} className={`${styles.stepItem} ${styles[step.status]}`}>
                <div className={styles.stepStatus}>{statusIcon(step.status)}</div>
                <div className={styles.stepContent}>
                  <span className={styles.stepAction}>{step.action}</span>
                  {step.message && <span className={styles.stepMessage}>{step.message}</span>}
                </div>
              </div>
            ))}
          </div>
        </div>
      )}

      {result?.data && (
        <Card className={styles.resultCard}>
          <div className={styles.resultHeader}>
            <Check size={14} className={styles.checkIcon} />
            <Text bold>Scene Composed Successfully</Text>
          </div>
          <pre className={styles.resultJson}>{JSON.stringify(result.data, null, 2)}</pre>
        </Card>
      )}

      {result?.message && !result.data && (
        <Card className={styles.errorCard}>
          <div className={styles.resultHeader}>
            <AlertCircle size={14} className={styles.errorIcon} />
            <Text bold>Composition Result</Text>
          </div>
          <Text size="sm">{result.message}</Text>
        </Card>
      )}
    </div>
  );
}
