import { useState, useCallback } from 'react';
import {
  BookOpen,
  GraduationCap,
  Play,
  ArrowLeft,
  ArrowRight,
  ChevronDown,
  ChevronUp,
  Sparkles,
  Lightbulb,
  Sun,
  Brush,
  Camera,
  Dumbbell,
  RotateCcw,
  TrendingUp,
} from 'lucide-react';
import { invoke } from '@tauri-apps/api/core';
import {
  tutorials,
  type Tutorial,
  type TutorialCategory,
  type TutorialStep,
} from '../../data/tutorials';
import { useToastStore } from '../../store';
import LearningDashboard from './LearningDashboard';
import styles from './TutorialPanel.module.css';

const categoryIcons: Record<TutorialCategory, typeof BookOpen> = {
  basics: BookOpen,
  lighting: Sun,
  posing: Dumbbell,
  materials: Brush,
  animation: Camera,
};

const CATEGORY_LABELS: Record<TutorialCategory, string> = {
  basics: 'Basics',
  lighting: 'Lighting',
  posing: 'Posing',
  materials: 'Materials',
  animation: 'Animation',
};

function TutorialCard({
  tutorial,
  onStart,
}: {
  tutorial: Tutorial;
  onStart: (id: string) => void;
}) {
  const CategoryIcon = categoryIcons[tutorial.category];
  const difficultyColor =
    tutorial.difficulty === 'beginner'
      ? '#22d3ee'
      : tutorial.difficulty === 'intermediate'
        ? '#fbbf24'
        : '#f87171';

  return (
    <div className={styles.tutorialCard}>
      <div className={styles.cardHeader}>
        <CategoryIcon size={18} className={styles.categoryIcon} />
        <span className={styles.difficultyBadge} style={{ color: difficultyColor }}>
          {tutorial.difficulty}
        </span>
      </div>
      <h3 className={styles.cardTitle}>{tutorial.title}</h3>
      <p className={styles.cardDesc}>{tutorial.description}</p>
      <div className={styles.cardFooter}>
        <span className={styles.stepCount}>{tutorial.steps.length} steps</span>
        <button className={styles.startBtn} onClick={() => onStart(tutorial.id)}>
          <Play size={12} />
          Start
        </button>
      </div>
    </div>
  );
}

function StepView({
  step,
  stepIndex,
  totalSteps,
  onExecute,
  onNext,
  onPrev,
  onBack,
  isFirst,
  isLast,
  executing,
}: {
  step: TutorialStep;
  stepIndex: number;
  totalSteps: number;
  onExecute: (step: TutorialStep) => void;
  onNext: () => void;
  onPrev: () => void;
  onBack: () => void;
  isFirst: boolean;
  isLast: boolean;
  executing: boolean;
}) {
  const [showManual, setShowManual] = useState(false);

  return (
    <div className={styles.stepView}>
      <button className={styles.backBtn} onClick={onBack}>
        <ArrowLeft size={14} />
        Back to tutorials
      </button>

      <div className={styles.stepProgress}>
        <div className={styles.progressBar}>
          <div
            className={styles.progressFill}
            style={{ width: `${((stepIndex + 1) / totalSteps) * 100}%` }}
          />
        </div>
        <span className={styles.progressText}>
          Step {stepIndex + 1} of {totalSteps}
        </span>
      </div>

      <div className={styles.stepHeader}>
        <span className={styles.conceptBadge}>{step.concept}</span>
        <h2 className={styles.stepTitle}>{step.title}</h2>
      </div>

      <div className={styles.teachSection}>
        <div className={styles.teachSectionHeader}>
          <GraduationCap size={14} />
          <span>What you need to know</span>
        </div>
        <p className={styles.teachText}>{step.teach}</p>
      </div>

      {step.aiAction && (
        <button className={styles.executeBtn} onClick={() => onExecute(step)} disabled={executing}>
          {executing ? (
            'Executing...'
          ) : (
            <>
              <Sparkles size={14} />
              Execute in DAZ Studio
            </>
          )}
        </button>
      )}

      {step.manualInstructions && (
        <div className={styles.manualSection}>
          <button className={styles.manualToggle} onClick={() => setShowManual(!showManual)}>
            <Lightbulb size={14} />
            <span>How to do this manually</span>
            {showManual ? <ChevronUp size={14} /> : <ChevronDown size={14} />}
          </button>
          {showManual && (
            <div className={styles.manualContent}>
              <p>{step.manualInstructions}</p>
            </div>
          )}
        </div>
      )}

      {step.tryYourself && (
        <div className={styles.trySection}>
          <div className={styles.trySectionHeader}>
            <RotateCcw size={14} />
            <span>Try it yourself</span>
          </div>
          <p className={styles.tryText}>{step.tryYourself}</p>
        </div>
      )}

      <div className={styles.stepNav}>
        <button className={styles.navBtn} onClick={onPrev} disabled={isFirst}>
          <ArrowLeft size={14} />
          Previous
        </button>
        <button
          className={`${styles.navBtn} ${styles.navBtnPrimary}`}
          onClick={onNext}
          disabled={isLast}
        >
          {isLast ? 'Finish' : 'Next'}
          {!isLast && <ArrowRight size={14} />}
        </button>
      </div>
    </div>
  );
}

export default function TutorialPanel() {
  const [activeTutorialId, setActiveTutorialId] = useState<string | null>(null);
  const [currentStepIndex, setCurrentStepIndex] = useState(0);
  const [executing, setExecuting] = useState(false);
  const [categoryFilter, setCategoryFilter] = useState<TutorialCategory | 'all'>('all');
  const [subTab, setSubTab] = useState<'browse' | 'dashboard'>('browse');

  const activeTutorial = activeTutorialId
    ? tutorials.find((t) => t.id === activeTutorialId) || null
    : null;
  const currentStep = activeTutorial ? activeTutorial.steps[currentStepIndex] : null;

  const filteredTutorials =
    categoryFilter === 'all' ? tutorials : tutorials.filter((t) => t.category === categoryFilter);

  const handleStart = useCallback((id: string) => {
    setActiveTutorialId(id);
    setCurrentStepIndex(0);
  }, []);

  const handleBack = useCallback(() => {
    setActiveTutorialId(null);
    setCurrentStepIndex(0);
  }, []);

  const handleNext = useCallback(() => {
    if (activeTutorial && currentStepIndex < activeTutorial.steps.length - 1) {
      setCurrentStepIndex((i) => i + 1);
    }
  }, [activeTutorial, currentStepIndex]);

  const handlePrev = useCallback(() => {
    if (currentStepIndex > 0) {
      setCurrentStepIndex((i) => i - 1);
    }
  }, [currentStepIndex]);

  const handleExecute = useCallback(async (step: TutorialStep) => {
    if (!step.aiAction) return;
    setExecuting(true);
    try {
      const result = await invoke<{ success: boolean; message?: string }>('execute_ai_action', {
        action: {
          command: step.aiAction!.command,
          args: step.aiAction!.args,
          confidence: 0.95,
          sdk_refs: [],
          requires_confirmation: false,
        },
      });
      if (result.success) {
        useToastStore.getState().success(`Step completed: ${step.title}`);
      } else {
        useToastStore.getState().error(result.message || 'Execution failed');
      }
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : String(err);
      useToastStore.getState().error(`Execution error: ${msg}`);
    } finally {
      setExecuting(false);
    }
  }, []);

  if (activeTutorial && currentStep) {
    return (
      <div className={styles.container}>
        <StepView
          step={currentStep}
          stepIndex={currentStepIndex}
          totalSteps={activeTutorial.steps.length}
          onExecute={handleExecute}
          onNext={handleNext}
          onPrev={handlePrev}
          onBack={handleBack}
          isFirst={currentStepIndex === 0}
          isLast={currentStepIndex === activeTutorial.steps.length - 1}
          executing={executing}
        />
      </div>
    );
  }

  return (
    <div className={styles.container}>
      <div className={styles.panelHeader}>
        <div className={styles.panelTitleRow}>
          <GraduationCap size={20} />
          <h2 className={styles.panelTitle}>Tutorials</h2>
        </div>
        <p className={styles.panelSubtitle}>
          Learn DAZ Studio concepts with guided, interactive tutorials
        </p>
      </div>

      <div className={styles.subTabRow}>
        <button
          className={`${styles.subTabBtn} ${subTab === 'browse' ? styles.subTabBtnActive : ''}`}
          onClick={() => setSubTab('browse')}
        >
          <BookOpen size={12} />
          Browse
        </button>
        <button
          className={`${styles.subTabBtn} ${subTab === 'dashboard' ? styles.subTabBtnActive : ''}`}
          onClick={() => setSubTab('dashboard')}
        >
          <TrendingUp size={12} />
          Dashboard
        </button>
      </div>

      {subTab === 'browse' && (
        <>
          <div className={styles.filterRow}>
            {(['all', 'basics', 'lighting', 'posing', 'materials', 'animation'] as const).map(
              (cat) => {
                const isActive = categoryFilter === cat;
                const Icon = cat === 'all' ? BookOpen : categoryIcons[cat];
                return (
                  <button
                    key={cat}
                    className={`${styles.filterBtn} ${isActive ? styles.filterBtnActive : ''}`}
                    onClick={() => setCategoryFilter(cat)}
                  >
                    <Icon size={12} />
                    {cat === 'all' ? 'All' : CATEGORY_LABELS[cat]}
                  </button>
                );
              }
            )}
          </div>

          <div className={styles.grid}>
            {filteredTutorials.map((t) => (
              <TutorialCard key={t.id} tutorial={t} onStart={handleStart} />
            ))}
          </div>

          {filteredTutorials.length === 0 && (
            <div className={styles.emptyState}>No tutorials in this category yet.</div>
          )}
        </>
      )}

      {subTab === 'dashboard' && <LearningDashboard onStartTutorial={handleStart} />}
    </div>
  );
}
