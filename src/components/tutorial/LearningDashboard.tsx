import { useEffect, useCallback } from 'react';
import {
  GraduationCap,
  BookOpen,
  Sun,
  Dumbbell,
  Brush,
  Camera,
  Sparkles,
  TrendingUp,
  CheckCircle2,
  Lightbulb,
} from 'lucide-react';
import { useLearningStore } from '../../store/learningStore';
import type { TutorialCategory } from '../../data/tutorials';
import { tutorials } from '../../data/tutorials';
import styles from './TutorialPanel.module.css';

const CATEGORY_META: Record<
  TutorialCategory,
  { label: string; Icon: typeof BookOpen; color: string }
> = {
  basics: { label: 'Basics', Icon: BookOpen, color: '#22d3ee' },
  lighting: { label: 'Lighting', Icon: Sun, color: '#fbbf24' },
  posing: { label: 'Posing', Icon: Dumbbell, color: '#f87171' },
  materials: { label: 'Materials', Icon: Brush, color: '#34d399' },
  animation: { label: 'Animation', Icon: Camera, color: '#a78bfa' },
};

export default function LearningDashboard({
  onStartTutorial,
}: {
  onStartTutorial?: (id: string) => void;
}) {
  const {
    discoveredConcepts,
    totalConcepts,
    discoveredCount,
    categoryBreakdown,
    suggestedTutorials,
    refresh,
  } = useLearningStore();

  useEffect(() => {
    refresh();
  }, [refresh]);

  const handleRefresh = useCallback(() => {
    refresh();
  }, [refresh]);

  const allDiscovered = Object.values(discoveredConcepts);
  const recentlyUsed = allDiscovered
    .filter((d) => d.discovered)
    .sort((a, b) => b.count - a.count)
    .slice(0, 6);

  const progressPercent =
    totalConcepts > 0 ? Math.round((discoveredCount / totalConcepts) * 100) : 0;

  return (
    <div className={styles.dashboard}>
      <div className={styles.dashboardHeader}>
        <div className={styles.dashboardTitleRow}>
          <TrendingUp size={18} />
          <h3 className={styles.dashboardTitle}>Learning Progress</h3>
        </div>
        <button className={styles.dashboardRefreshBtn} onClick={handleRefresh}>
          <Sparkles size={12} />
          Refresh
        </button>
      </div>

      <div className={styles.overallCard}>
        <div className={styles.overallStat}>
          <span className={styles.overallNumber}>{discoveredCount}</span>
          <span className={styles.overallSep}>/</span>
          <span className={styles.overallTotal}>{totalConcepts}</span>
        </div>
        <div className={styles.overallLabel}>DAZ3D Concepts Explored</div>
        <div className={styles.overallBar}>
          <div className={styles.overallBarFill} style={{ width: `${progressPercent}%` }} />
        </div>
        <div className={styles.overallPercent}>{progressPercent}% complete</div>
      </div>

      <div className={styles.dashboardSection}>
        <div className={styles.sectionHeader}>
          <CheckCircle2 size={14} />
          <span>Skills by Category</span>
        </div>
        <div className={styles.categoryBreakdown}>
          {(['basics', 'lighting', 'posing', 'materials', 'animation'] as const).map((cat) => {
            const meta = CATEGORY_META[cat];
            const data = categoryBreakdown[cat] || { total: 0, discovered: 0 };
            const pct = data.total > 0 ? Math.round((data.discovered / data.total) * 100) : 0;
            return (
              <div key={cat} className={styles.categoryRow}>
                <div className={styles.categoryRowHeader}>
                  <meta.Icon size={13} style={{ color: meta.color }} />
                  <span className={styles.categoryRowLabel}>{meta.label}</span>
                  <span className={styles.categoryRowStat}>
                    {data.discovered}/{data.total}
                  </span>
                </div>
                <div className={styles.categoryRowBar}>
                  <div
                    className={styles.categoryRowFill}
                    style={{ width: `${pct}%`, background: meta.color }}
                  />
                </div>
              </div>
            );
          })}
        </div>
      </div>

      {recentlyUsed.length > 0 && (
        <div className={styles.dashboardSection}>
          <div className={styles.sectionHeader}>
            <TrendingUp size={14} />
            <span>Most Used Concepts</span>
          </div>
          <div className={styles.recentList}>
            {recentlyUsed.map((d) => {
              const meta = CATEGORY_META[d.concept.category];
              return (
                <div key={d.concept.id} className={styles.recentItem}>
                  <div className={styles.recentItemLeft}>
                    <meta.Icon size={12} style={{ color: meta.color }} />
                    <span className={styles.recentItemName}>{d.concept.name}</span>
                  </div>
                  <span className={styles.recentItemCount}>{d.count}x</span>
                </div>
              );
            })}
          </div>
        </div>
      )}

      {suggestedTutorials.length > 0 && (
        <div className={styles.dashboardSection}>
          <div className={styles.sectionHeader}>
            <Lightbulb size={14} />
            <span>Suggested Tutorials</span>
          </div>
          <div className={styles.suggestList}>
            {suggestedTutorials.map((tId) => {
              const tut = tutorials.find((t) => t.id === tId);
              if (!tut) return null;
              const meta = CATEGORY_META[tut.category];
              const alreadyExplored = tut.steps.every(
                (s) =>
                  s.aiAction &&
                  allDiscovered.find(
                    (d) => d.concept.command === s.aiAction!.command && d.discovered
                  )
              );
              return (
                <div key={tId} className={styles.suggestItem}>
                  <div className={styles.suggestItemLeft}>
                    <meta.Icon size={13} style={{ color: meta.color }} />
                    <div className={styles.suggestItemInfo}>
                      <span className={styles.suggestItemTitle}>{tut.title}</span>
                      <span className={styles.suggestItemDesc}>{tut.steps.length} steps</span>
                    </div>
                  </div>
                  <button className={styles.suggestStartBtn} onClick={() => onStartTutorial?.(tId)}>
                    {alreadyExplored ? 'Review' : 'Start'}
                  </button>
                </div>
              );
            })}
          </div>
        </div>
      )}

      {discoveredCount === 0 && (
        <div className={styles.dashboardEmpty}>
          <GraduationCap size={24} />
          <p>No concepts discovered yet.</p>
          <p className={styles.dashboardEmptyHint}>
            Send commands to DAZ Studio through the chat to start tracking your learning progress.
          </p>
        </div>
      )}
    </div>
  );
}
