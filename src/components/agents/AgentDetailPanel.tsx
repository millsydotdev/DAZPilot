import { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Info, Tag, GitBranch, CheckCircle2, XCircle, Loader2 } from 'lucide-react';
import styles from './AgentDetailPanel.module.css';

interface AgentInfo {
  agent_type: string;
  description: string;
  parent: string | null;
  children: string[];
  capabilities: string[];
}

interface AgentDetailPanelProps {
  agentType: string | null;
}

export default function AgentDetailPanel({ agentType }: AgentDetailPanelProps) {
  const [agent, setAgent] = useState<AgentInfo | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const fetchAgent = useCallback(async () => {
    if (!agentType) return;
    setLoading(true);
    setError(null);
    try {
      const result = await invoke<AgentInfo | null>('get_agent_info', {
        agentType,
      });
      setAgent(result);
    } catch (err) {
      setError(String(err));
    } finally {
      setLoading(false);
    }
  }, [agentType]);

  useEffect(() => {
    fetchAgent();
  }, [fetchAgent]);

  if (!agentType) {
    return (
      <div className={styles.container}>
        <div className={styles.header}>
          <Info size={16} />
          <span>Agent Details</span>
        </div>
        <div className={styles.empty}>Select an agent from the tree to view details.</div>
      </div>
    );
  }

  if (loading) {
    return (
      <div className={styles.container}>
        <div className={styles.header}>
          <Info size={16} />
          <span>{agentType}</span>
        </div>
        <div className={styles.centered}>
          <Loader2 size={20} className={styles.spin} />
        </div>
      </div>
    );
  }

  if (error || !agent) {
    return (
      <div className={styles.container}>
        <div className={styles.header}>
          <Info size={16} />
          <span>{agentType}</span>
        </div>
        <div className={styles.centered}>
          <XCircle size={20} className={styles.errorIcon} />
          <span className={styles.errorText}>{error || 'Agent not found'}</span>
        </div>
      </div>
    );
  }

  return (
    <div className={styles.container}>
      <div className={styles.header}>
        <Info size={16} />
        <span>{agent.agent_type}</span>
        {agent.parent && <span className={styles.parent}>child of {agent.parent}</span>}
      </div>

      <div className={styles.body}>
        <div className={styles.section}>
          <div className={styles.label}>Description</div>
          <div className={styles.value}>{agent.description}</div>
        </div>

        <div className={styles.section}>
          <div className={styles.label}>
            <Tag size={12} /> Capabilities
          </div>
          <div className={styles.tags}>
            {agent.capabilities.length > 0 ? (
              agent.capabilities.map((cap) => (
                <span key={cap} className={styles.tag}>
                  {cap}
                </span>
              ))
            ) : (
              <span className={styles.muted}>None</span>
            )}
          </div>
        </div>

        <div className={styles.section}>
          <div className={styles.label}>
            <GitBranch size={12} /> Hierarchy
          </div>
          <div className={styles.hierarchyGrid}>
            <div className={styles.hierarchyLabel}>Parent</div>
            <div className={styles.hierarchyValue}>
              {agent.parent || <span className={styles.muted}>Root agent</span>}
            </div>
            <div className={styles.hierarchyLabel}>Children</div>
            <div className={styles.hierarchyValue}>
              {agent.children.length > 0 ? (
                agent.children.join(', ')
              ) : (
                <span className={styles.muted}>Leaf agent (no children)</span>
              )}
            </div>
          </div>
        </div>

        <div className={styles.section}>
          <div className={styles.label}>
            <CheckCircle2 size={12} /> Status
          </div>
          <div className={styles.statusRow}>
            <span className={styles.statusDot} />
            <span>Registered</span>
          </div>
        </div>
      </div>
    </div>
  );
}
