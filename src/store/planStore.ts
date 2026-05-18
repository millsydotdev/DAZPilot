import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/core';

export interface PlanStep {
  id: string;
  description: string;
  status: 'pending' | 'in_progress' | 'completed' | 'failed';
  result?: string;
  error?: string;
}

export interface Plan {
  id: string;
  name: string;
  steps: PlanStep[];
  currentStepIndex: number;
  createdAt: number;
  status: 'planning' | 'ready' | 'executing' | 'completed' | 'failed';
}

export interface QuestionOption {
  label: string;
  value: string;
}

export interface PendingQuestion {
  id: string;
  question: string;
  options: QuestionOption[];
  allowCustom: boolean;
  timestamp: number;
}

interface PlanState {
  currentPlan: Plan | null;
  pendingQuestions: PendingQuestion[];
  isExecuting: boolean;
}

interface PlanActions {
  setPlan: (plan: Plan | null) => void;
  addStep: (description: string) => void;
  updateStep: (stepId: string, updates: Partial<PlanStep>) => void;
  removeStep: (stepId: string) => void;
  reorderSteps: (fromIndex: number, toIndex: number) => void;
  executePlan: () => Promise<void>;
  executeStep: (stepIndex: number) => Promise<void>;
  cancelExecution: () => void;
  setQuestion: (question: PendingQuestion) => void;
  answerQuestion: (questionId: string, answer: string) => void;
  dismissQuestion: (questionId: string) => void;
  clearPlan: () => void;
}

export const usePlanStore = create<PlanState & PlanActions>((set, get) => ({
  currentPlan: null,
  pendingQuestions: [],
  isExecuting: false,

  setPlan: (plan) => set({ currentPlan: plan }),

  addStep: (description) => {
    const plan = get().currentPlan;
    if (!plan) {
      const newPlan: Plan = {
        id: `plan-${Date.now()}`,
        name: 'New Plan',
        steps: [{ id: `step-1`, description, status: 'pending' }],
        currentStepIndex: 0,
        createdAt: Date.now(),
        status: 'ready',
      };
      set({ currentPlan: newPlan });
      return;
    }
    const newStep: PlanStep = {
      id: `step-${plan.steps.length + 1}`,
      description,
      status: 'pending',
    };
    set({
      currentPlan: {
        ...plan,
        steps: [...plan.steps, newStep],
        status: 'ready',
      },
    });
  },

  updateStep: (stepId, updates) => {
    const plan = get().currentPlan;
    if (!plan) return;
    set({
      currentPlan: {
        ...plan,
        steps: plan.steps.map((s) => (s.id === stepId ? { ...s, ...updates } : s)),
      },
    });
  },

  removeStep: (stepId) => {
    const plan = get().currentPlan;
    if (!plan) return;
    set({
      currentPlan: {
        ...plan,
        steps: plan.steps.filter((s) => s.id !== stepId),
      },
    });
  },

  reorderSteps: (fromIndex, toIndex) => {
    const plan = get().currentPlan;
    if (!plan) return;
    const steps = [...plan.steps];
    const [removed] = steps.splice(fromIndex, 1);
    steps.splice(toIndex, 0, removed);
    set({ currentPlan: { ...plan, steps } });
  },

  executePlan: async () => {
    const plan = get().currentPlan;
    if (!plan || plan.steps.length === 0) return;
    set({ isExecuting: true });

    const updatedSteps = plan.steps.map((s, i) =>
      i === 0 ? { ...s, status: 'in_progress' as const } : s
    );
    set({
      currentPlan: { ...plan, steps: updatedSteps, status: 'executing', currentStepIndex: 0 },
    });

    for (let i = 0; i < plan.steps.length; i++) {
      set({
        currentPlan: get().currentPlan ? { ...get().currentPlan!, currentStepIndex: i } : null,
      });

      try {
        await get().executeStep(i);
      } catch (e) {
        const currentPlan = get().currentPlan;
        if (currentPlan) {
          set({
            currentPlan: {
              ...currentPlan,
              status: 'failed',
              steps: currentPlan.steps.map((s, idx) =>
                idx === i ? { ...s, status: 'failed' as const, error: String(e) } : s
              ),
            },
            isExecuting: false,
          });
        }
        return;
      }
    }

    set({
      currentPlan: get().currentPlan ? { ...get().currentPlan!, status: 'completed' } : null,
      isExecuting: false,
    });
  },

  executeStep: async (stepIndex) => {
    const plan = get().currentPlan;
    if (!plan) return;

    const step = plan.steps[stepIndex];
    set({
      currentPlan: {
        ...plan,
        steps: plan.steps.map((s, i) => (i === stepIndex ? { ...s, status: 'in_progress' } : s)),
      },
    });

    try {
      const result = await invoke<string>('execute_agent', {
        agentType: 'task_planner',
        input: step.description,
      });

      set({
        currentPlan: {
          ...plan,
          steps: plan.steps.map((s, i) =>
            i === stepIndex
              ? { ...s, status: 'completed' as const, result }
              : i > stepIndex && s.status === 'pending'
                ? { ...s, status: 'in_progress' as const }
                : s
          ),
        },
      });
    } catch (e) {
      set({
        currentPlan: {
          ...plan,
          steps: plan.steps.map((s, i) =>
            i === stepIndex ? { ...s, status: 'failed' as const, error: String(e) } : s
          ),
        },
      });
      throw e;
    }
  },

  cancelExecution: () => {
    const plan = get().currentPlan;
    if (!plan) return;
    set({
      currentPlan: { ...plan, status: 'ready' },
      isExecuting: false,
    });
  },

  setQuestion: (question) => {
    set((state) => ({
      pendingQuestions: [...state.pendingQuestions, question],
    }));
  },

  answerQuestion: (questionId, _answer) => {
    set((state) => ({
      pendingQuestions: state.pendingQuestions.filter((q) => q.id !== questionId),
    }));
  },

  dismissQuestion: (questionId) => {
    set((state) => ({
      pendingQuestions: state.pendingQuestions.filter((q) => q.id !== questionId),
    }));
  },

  clearPlan: () => set({ currentPlan: null, isExecuting: false }),
}));
