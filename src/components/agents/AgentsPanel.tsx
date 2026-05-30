import { useState } from 'react';
import { Network } from 'lucide-react';
import AgentTreeView from './AgentTreeView';
import AgentDetailPanel from './AgentDetailPanel';
import AgentTester from './AgentTester';
import AgentAnalytics from './AgentAnalytics';
import CustomSubAgentForm from './CustomSubAgentForm';
import LearningDashboard from '../tutorial/LearningDashboard';
import styles from './AgentsPanel.module.css';

export default function AgentsPanel() {
  const [selectedAgent, setSelectedAgent] = useState<string | null>(null);
  const [treeKey, setTreeKey] = useState(0);

  return (
    <div className={styles.container}>
      <div className={styles.header}>
        <Network size={20} />
        <h2>Agent System</h2>
        <p className={styles.subtitle}>
          Explore the 14-agent hierarchy, test agents, and review execution analytics.
        </p>
      </div>

      <div className={styles.grid}>
        <div className={styles.treeColumn}>
          <AgentTreeView
            key={treeKey}
            onSelectAgent={setSelectedAgent}
            selectedAgent={selectedAgent}
          />
        </div>
        <div className={styles.detailColumn}>
          {selectedAgent ? (
            <>
              <AgentDetailPanel agentType={selectedAgent} />
              <AgentTester agentType={selectedAgent} />
            </>
          ) : (
            <p className={styles.hint}>
              Select an agent from the tree to view details and test it.
            </p>
          )}
        </div>
      </div>

      <div className={styles.analyticsSection}>
        <AgentAnalytics />
      </div>

      <CustomSubAgentForm onRegistered={() => setTreeKey((k) => k + 1)} />

      <div className={styles.learningSection}>
        <LearningDashboard />
      </div>
    </div>
  );
}
