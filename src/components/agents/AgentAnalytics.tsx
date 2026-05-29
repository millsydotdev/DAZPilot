import { useState, useEffect, useCallback, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';
import {
  Activity,
  CheckCircle2,
  XCircle,
  Clock,
  TrendingUp,
  RefreshCw,
  Loader2,
  AlertCircle,
} from 'lucide-react';
import type { AgentMetrics } from '../../store/agentAnalyticsStore';
import styles from './AgentAnalytics.module.css';

function formatDuration(ms: number): string {
  if (ms < 1000) return `${ms.toFixed(0)}ms`;
  if (ms < 60000) return `${(ms / 1000).toFixed(1)}s`;
  return `${(ms / 60000).toFixed(1)}m`;
}

function formatTime(iso: string | null): string {
  if (!iso) return 'Never';
  const d = new Date(iso);
  const now = new Date();
  const diffMs = now.getTime() - d.getTime();
  const diffMin = Math.floor(diffMs / 60000);
  if (diffMin < 1) return 'Just now';
  if (diffMin < 60) return `${diffMin}m ago`;
  const diffHr = Math.floor(diffMin / 60);
  if (diffHr < 24) return `${diffHr}h ago`;
  return d.toLocaleDateString();
}

function successRate(metrics: AgentMetrics): number {
  if (metrics.total_executions === 0) return 0;
  return (metrics.successful_executions / metrics.total_executions) * 100;
}

function getSuccessColor(rate: number): string {
  if (rate >= 90) return styles.green;
  if (rate >= 60) return styles.yellow;
  return styles.red;
}

interface SummaryCardProps {
  icon: React.ReactNode;
  label: string;
  value: string;
  subtitle?: string;
  variant?: 'default' | 'success' | 'warning' | 'danger';
}

function SummaryCard({ icon, label, value, subtitle, variant = 'default' }: SummaryCardProps) {
  return (
    <div className={`${styles.summaryCard} ${styles[variant]}`}>
      <div className={styles.summaryIcon}>{icon}</div>
      <div className={styles.summaryBody}>
        <span className={styles.summaryValue}>{value}</span>
        <span className={styles.summaryLabel}>{label}</span>
        {subtitle && <span className={styles.summarySubtitle}>{subtitle}</span>}
      </div>
    </div>
  );
}

interface AgentRowProps {
  metrics: AgentMetrics;
}

function AgentRow({ metrics }: AgentRowProps) {
  const rate = successRate(metrics);
  const rateStr = metrics.total_executions > 0 ? `${rate.toFixed(0)}%` : 'N/A';

  return (
    <div className={styles.agentRow}>
      <div className={styles.agentRowName}>
        <span className={styles.agentDot} />
        <span className={styles.agentType}>{metrics.agent_type}</span>
      </div>
      <div className={styles.agentRowStat}>{metrics.total_executions}</div>
      <div className={styles.agentRowStat}>
        <span className={`${styles.rateBadge} ${getSuccessColor(rate)}`}>{rateStr}</span>
      </div>
      <div className={styles.agentRowStat}>{formatDuration(metrics.average_duration_ms)}</div>
      <div className={styles.agentRowStat}>{formatTime(metrics.last_execution)}</div>
      <div className={styles.agentRowStat}>
        {metrics.last_success ? (
          <CheckCircle2 size={14} className={styles.successIcon} />
        ) : (
          <XCircle size={14} className={styles.failIcon} />
        )}
      </div>
    </div>
  );
}

export default function AgentAnalytics() {
  const [metrics, setMetrics] = useState<AgentMetrics[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [autoRefresh, setAutoRefresh] = useState(false);
  const intervalRef = useRef<ReturnType<typeof setInterval> | null>(null);

  const fetchMetrics = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const result = await invoke<AgentMetrics[]>('get_agent_metrics');
      setMetrics(result);
    } catch (err) {
      setError(String(err));
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    fetchMetrics();
  }, [fetchMetrics]);

  useEffect(() => {
    if (autoRefresh) {
      intervalRef.current = setInterval(fetchMetrics, 5000);
    } else {
      if (intervalRef.current) {
        clearInterval(intervalRef.current);
        intervalRef.current = null;
      }
    }
    return () => {
      if (intervalRef.current) {
        clearInterval(intervalRef.current);
        intervalRef.current = null;
      }
    };
  }, [autoRefresh, fetchMetrics]);

  const totalExecutions = metrics.reduce((sum, m) => sum + m.total_executions, 0);
  const totalSuccess = metrics.reduce((sum, m) => sum + m.successful_executions, 0);
  const totalFailed = metrics.reduce((sum, m) => sum + m.failed_executions, 0);
  const overallRate = totalExecutions > 0 ? (totalSuccess / totalExecutions) * 100 : 0;
  const avgDuration =
    totalExecutions > 0
      ? metrics.reduce((sum, m) => sum + m.total_duration_ms, 0) / totalExecutions
      : 0;

  return (
    <div className={styles.container}>
      <div className={styles.header}>
        <Activity size={16} />
        <span>Agent Analytics</span>
        <span className={styles.headerCount}>{metrics.length} agents</span>
        <div className={styles.headerActions}>
          <button
            className={`${styles.refreshBtn} ${autoRefresh ? styles.refreshBtnActive : ''}`}
            onClick={() => setAutoRefresh(!autoRefresh)}
            title="Auto-refresh every 5s"
          >
            <RefreshCw size={14} className={autoRefresh ? styles.spin : ''} />
            <span>{autoRefresh ? 'Auto' : 'Manual'}</span>
          </button>
          <button className={styles.refreshBtn} onClick={fetchMetrics} title="Refresh now">
            <RefreshCw size={14} />
          </button>
        </div>
      </div>

      {loading && metrics.length === 0 ? (
        <div className={styles.centered}>
          <Loader2 size={24} className={styles.spin} />
          <span className={styles.loadingText}>Loading metrics...</span>
        </div>
      ) : error ? (
        <div className={styles.centered}>
          <AlertCircle size={24} className={styles.errorIcon} />
          <span className={styles.errorText}>{error}</span>
        </div>
      ) : (
        <>
          <div className={styles.summaryRow}>
            <SummaryCard
              icon={<TrendingUp size={20} />}
              label="Total Executions"
              value={totalExecutions.toLocaleString()}
              variant={totalExecutions > 0 ? 'success' : 'default'}
            />
            <SummaryCard
              icon={<CheckCircle2 size={20} />}
              label="Success Rate"
              value={totalExecutions > 0 ? `${overallRate.toFixed(1)}%` : 'N/A'}
              subtitle={`${totalSuccess} ok, ${totalFailed} failed`}
              variant={overallRate >= 90 ? 'success' : overallRate >= 60 ? 'warning' : 'danger'}
            />
            <SummaryCard
              icon={<Clock size={20} />}
              label="Avg Response Time"
              value={formatDuration(avgDuration)}
              variant="default"
            />
            <SummaryCard
              icon={<Activity size={20} />}
              label="Active Agents"
              value={metrics.length.toString()}
              subtitle={metrics.filter((m) => m.total_executions > 0).length + ' with data'}
              variant="default"
            />
          </div>

          <div className={styles.tableContainer}>
            <div className={styles.tableHeader}>
              <span className={styles.colAgent}>Agent</span>
              <span className={styles.colStat}>Executions</span>
              <span className={styles.colStat}>Success Rate</span>
              <span className={styles.colStat}>Avg Time</span>
              <span className={styles.colStat}>Last Run</span>
              <span className={styles.colStat}>Status</span>
            </div>
            <div className={styles.tableBody}>
              {metrics.length === 0 ? (
                <div className={styles.empty}>
                  No metrics recorded yet. Execute some agents to see data.
                </div>
              ) : (
                metrics.map((m) => <AgentRow key={m.agent_type} metrics={m} />)
              )}
            </div>
          </div>
        </>
      )}
    </div>
  );
}
