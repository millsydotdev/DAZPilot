import { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { PlusCircle, Loader2, CheckCircle2, AlertCircle } from 'lucide-react';
import styles from './CustomSubAgentForm.module.css';

interface CustomSubAgentFormProps {
  onRegistered?: () => void;
}

export default function CustomSubAgentForm({ onRegistered }: CustomSubAgentFormProps) {
  const [agentType, setAgentType] = useState('');
  const [description, setDescription] = useState('');
  const [parent, setParent] = useState('scene_composer');
  const [capabilities, setCapabilities] = useState('');
  const [loading, setLoading] = useState(false);
  const [message, setMessage] = useState<string | null>(null);
  const [isError, setIsError] = useState(false);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!agentType.trim() || !description.trim()) {
      setMessage('Agent type and description are required');
      setIsError(true);
      return;
    }

    setLoading(true);
    setMessage(null);
    try {
      const caps = capabilities
        .split(',')
        .map((c) => c.trim())
        .filter(Boolean);
      const result = await invoke<string>('register_sub_agent', {
        agentType: agentType.trim(),
        description: description.trim(),
        parent,
        capabilities: caps,
      });
      setMessage(result);
      setIsError(false);
      setAgentType('');
      setDescription('');
      setCapabilities('');
      onRegistered?.();
    } catch (err) {
      setMessage(String(err));
      setIsError(true);
    } finally {
      setLoading(false);
    }
  };

  return (
    <form className={styles.form} onSubmit={handleSubmit}>
      <div className={styles.header}>
        <PlusCircle size={16} />
        <span>Register Custom Sub-Agent</span>
      </div>

      <label className={styles.field}>
        <span>Agent ID</span>
        <input
          className={styles.input}
          value={agentType}
          onChange={(e) => setAgentType(e.target.value)}
          placeholder="my_custom_agent"
        />
      </label>

      <label className={styles.field}>
        <span>Description</span>
        <input
          className={styles.input}
          value={description}
          onChange={(e) => setDescription(e.target.value)}
          placeholder="What this agent handles"
        />
      </label>

      <label className={styles.field}>
        <span>Parent Agent</span>
        <select className={styles.input} value={parent} onChange={(e) => setParent(e.target.value)}>
          <option value="task_planner">task_planner</option>
          <option value="asset_selection">asset_selection</option>
          <option value="animation">animation</option>
          <option value="render">render</option>
          <option value="scene_composer">scene_composer</option>
          <option value="physics">physics</option>
          <option value="conflict_resolution">conflict_resolution</option>
        </select>
      </label>

      <label className={styles.field}>
        <span>Capabilities (comma-separated keywords)</span>
        <input
          className={styles.input}
          value={capabilities}
          onChange={(e) => setCapabilities(e.target.value)}
          placeholder="custom, workflow, special"
        />
      </label>

      <button type="submit" className={styles.submit} disabled={loading}>
        {loading ? <Loader2 size={14} className={styles.spin} /> : 'Register Agent'}
      </button>

      {message && (
        <div className={`${styles.message} ${isError ? styles.error : styles.success}`}>
          {isError ? <AlertCircle size={14} /> : <CheckCircle2 size={14} />}
          <span>{message}</span>
        </div>
      )}
    </form>
  );
}
