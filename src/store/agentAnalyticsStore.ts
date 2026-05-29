import { create } from 'zustand';

export interface AgentMetrics {
  agent_type: string;
  total_executions: number;
  successful_executions: number;
  failed_executions: number;
  total_duration_ms: number;
  last_execution: string | null;
  last_success: boolean;
  average_duration_ms: number;
}

export interface AgentAnalyticsState {
  metrics: AgentMetrics[];
  loading: boolean;
  error: string | null;
  autoRefresh: boolean;
}

export interface AgentAnalyticsActions {
  fetchMetrics: () => Promise<void>;
  setAutoRefresh: (enabled: boolean) => void;
}

const initialState: AgentAnalyticsState = {
  metrics: [],
  loading: false,
  error: null,
  autoRefresh: false,
};

export const useAgentAnalyticsStore = create<AgentAnalyticsState & AgentAnalyticsActions>(
  (set) => ({
    ...initialState,

    fetchMetrics: async () => {
      set({ loading: true, error: null });
      try {
        const { invoke } = await import('@tauri-apps/api/core');
        const metrics = await invoke<AgentMetrics[]>('get_agent_metrics');
        set({ metrics, loading: false });
      } catch (err) {
        set({ error: String(err), loading: false });
      }
    },

    setAutoRefresh: (enabled) => set({ autoRefresh: enabled }),
  })
);
