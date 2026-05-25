import { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Network, ChevronRight, ChevronDown, FileCode, Loader2, AlertCircle } from 'lucide-react';
import styles from './AgentTreeView.module.css';

interface AgentTreeNode {
  agent_type: string;
  description: string;
  capabilities: string[];
  children: AgentTreeNode[];
}

interface AgentTreeViewProps {
  onSelectAgent?: (agentType: string) => void;
  selectedAgent?: string | null;
}

function TreeNode({
  node,
  depth,
  onSelect,
  selectedAgent,
}: {
  node: AgentTreeNode;
  depth: number;
  onSelect: (t: string) => void;
  selectedAgent?: string | null;
}) {
  const [expanded, setExpanded] = useState(depth < 1);
  const hasChildren = node.children.length > 0;
  const isSelected = selectedAgent === node.agent_type;

  return (
    <div>
      <div
        role="treeitem"
        aria-expanded={expanded}
        aria-selected={isSelected}
        className={`${styles.treeNode} ${isSelected ? styles.selected : ''}`}
        style={{ paddingLeft: `${12 + depth * 20}px` }}
        onClick={() => {
          onSelect(node.agent_type);
          if (hasChildren) setExpanded(!expanded);
        }}
        onKeyDown={(e) => {
          if (e.key === 'Enter' || e.key === ' ') {
            e.preventDefault();
            onSelect(node.agent_type);
            if (hasChildren) setExpanded(!expanded);
          }
        }}
        tabIndex={0}
      >
        <span className={styles.chevron}>
          {hasChildren ? (
            expanded ? (
              <ChevronDown size={14} />
            ) : (
              <ChevronRight size={14} />
            )
          ) : (
            <span className={styles.leafSpacer} />
          )}
        </span>
        <FileCode size={16} className={styles.nodeIcon} />
        <span className={styles.nodeName}>{node.agent_type}</span>
        {hasChildren && <span className={styles.childCount}>{node.children.length}</span>}
      </div>
      {expanded &&
        hasChildren &&
        node.children.map((child) => (
          <TreeNode
            key={child.agent_type}
            node={child}
            depth={depth + 1}
            onSelect={onSelect}
            selectedAgent={selectedAgent}
          />
        ))}
    </div>
  );
}

export default function AgentTreeView({ onSelectAgent, selectedAgent }: AgentTreeViewProps) {
  const [tree, setTree] = useState<AgentTreeNode[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const fetchTree = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const result = await invoke<AgentTreeNode[]>('get_agent_tree');
      setTree(result);
    } catch (err) {
      setError(String(err));
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    fetchTree();
  }, [fetchTree]);

  if (loading) {
    return (
      <div className={styles.container}>
        <div className={styles.header}>
          <Network size={16} />
          <span>Agent Hierarchy</span>
        </div>
        <div className={styles.centered}>
          <Loader2 size={20} className={styles.spin} />
          <span className={styles.loadingText}>Loading agents...</span>
        </div>
      </div>
    );
  }

  if (error) {
    return (
      <div className={styles.container}>
        <div className={styles.header}>
          <Network size={16} />
          <span>Agent Hierarchy</span>
        </div>
        <div className={styles.centered}>
          <AlertCircle size={20} className={styles.errorIcon} />
          <span className={styles.errorText}>{error}</span>
        </div>
      </div>
    );
  }

  return (
    <div className={styles.container}>
      <div className={styles.header}>
        <Network size={16} />
        <span>Agent Hierarchy</span>
        <span className={styles.count}>{tree.length} root(s)</span>
      </div>
      <div className={styles.treeContainer} role="tree">
        {tree.map((node) => (
          <TreeNode
            key={node.agent_type}
            node={node}
            depth={0}
            onSelect={(t) => onSelectAgent?.(t)}
            selectedAgent={selectedAgent}
          />
        ))}
        {tree.length === 0 && <div className={styles.empty}>No agents registered.</div>}
      </div>
    </div>
  );
}
