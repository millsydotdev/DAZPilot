import { describe, it, expect, vi } from 'vitest';
import { act } from 'react';
import { usePlanStore } from './planStore';
import { invoke } from '@tauri-apps/api/core';

const initialState = {
  currentPlan: null,
  pendingQuestions: [],
  isExecuting: false,
};

describe('planStore', () => {
  it('setPlan sets current plan', () => {
    act(() => usePlanStore.setState(initialState));
    const plan = {
      id: 'p1',
      name: 'Plan A',
      steps: [],
      currentStepIndex: 0,
      createdAt: 100,
      status: 'ready' as const,
    };
    usePlanStore.getState().setPlan(plan);
    expect(usePlanStore.getState().currentPlan?.id).toBe('p1');
    usePlanStore.getState().setPlan(null);
    expect(usePlanStore.getState().currentPlan).toBeNull();
  });

  it('addStep creates new plan when none exists', () => {
    act(() => usePlanStore.setState(initialState));
    usePlanStore.getState().addStep('first step');
    const plan = usePlanStore.getState().currentPlan;
    expect(plan).not.toBeNull();
    expect(plan!.steps).toHaveLength(1);
    expect(plan!.steps[0].description).toBe('first step');
    expect(plan!.name).toBe('New Plan');
  });

  it('addStep appends to existing plan', () => {
    act(() => usePlanStore.setState(initialState));
    usePlanStore.getState().addStep('step 1');
    usePlanStore.getState().addStep('step 2');
    expect(usePlanStore.getState().currentPlan!.steps).toHaveLength(2);
  });

  it('updateStep updates fields', () => {
    act(() => usePlanStore.setState(initialState));
    usePlanStore.getState().addStep('s1');
    const id = usePlanStore.getState().currentPlan!.steps[0].id;
    usePlanStore.getState().updateStep(id, { status: 'completed', result: 'done' });
    const step = usePlanStore.getState().currentPlan!.steps[0];
    expect(step.status).toBe('completed');
    expect(step.result).toBe('done');
  });

  it('updateStep no-ops if no plan', () => {
    act(() => usePlanStore.setState(initialState));
    usePlanStore.getState().updateStep('x', { status: 'completed' });
    expect(usePlanStore.getState().currentPlan).toBeNull();
  });

  it('removeStep removes step', () => {
    act(() => usePlanStore.setState(initialState));
    usePlanStore.getState().addStep('s1');
    usePlanStore.getState().addStep('s2');
    const id = usePlanStore.getState().currentPlan!.steps[0].id;
    usePlanStore.getState().removeStep(id);
    expect(usePlanStore.getState().currentPlan!.steps).toHaveLength(1);
    expect(usePlanStore.getState().currentPlan!.steps[0].description).toBe('s2');
  });

  it('removeStep no-ops if no plan', () => {
    act(() => usePlanStore.setState(initialState));
    usePlanStore.getState().removeStep('x');
    expect(usePlanStore.getState().currentPlan).toBeNull();
  });

  it('reorderSteps changes order', () => {
    act(() => usePlanStore.setState(initialState));
    usePlanStore.getState().addStep('a');
    usePlanStore.getState().addStep('b');
    usePlanStore.getState().addStep('c');
    usePlanStore.getState().reorderSteps(0, 2);
    const steps = usePlanStore.getState().currentPlan!.steps;
    expect(steps[0].description).toBe('b');
    expect(steps[1].description).toBe('c');
    expect(steps[2].description).toBe('a');
  });

  it('reorderSteps no-ops if no plan', () => {
    act(() => usePlanStore.setState(initialState));
    usePlanStore.getState().reorderSteps(0, 1);
    expect(usePlanStore.getState().currentPlan).toBeNull();
  });

  it('executePlan executes all steps', async () => {
    vi.mocked(invoke).mockReset();
    vi.mocked(invoke).mockResolvedValue('ok');
    act(() => usePlanStore.setState(initialState));
    usePlanStore.getState().addStep('s1');
    usePlanStore.getState().addStep('s2');
    await usePlanStore.getState().executePlan();
    const plan = usePlanStore.getState().currentPlan;
    expect(plan!.status).toBe('completed');
    expect(plan!.steps.every((s) => s.status === 'completed')).toBe(true);
    expect(usePlanStore.getState().isExecuting).toBe(false);
  });

  it('executePlan stops on step failure', async () => {
    vi.mocked(invoke).mockReset();
    vi.mocked(invoke).mockRejectedValueOnce(new Error('step fail'));
    vi.mocked(invoke).mockResolvedValue('ok');
    act(() => usePlanStore.setState(initialState));
    usePlanStore.getState().addStep('fail step');
    usePlanStore.getState().addStep('ok step');
    await usePlanStore.getState().executePlan();
    const plan = usePlanStore.getState().currentPlan;
    expect(plan!.status).toBe('failed');
    expect(plan!.steps[0].status).toBe('failed');
    expect(plan!.steps[0].error).toBe('Error: step fail');
    expect(usePlanStore.getState().isExecuting).toBe(false);
  });

  it('executePlan no-ops with no plan', async () => {
    act(() => usePlanStore.setState(initialState));
    await usePlanStore.getState().executePlan();
    expect(usePlanStore.getState().isExecuting).toBe(false);
  });

  it('executeStep completes step', async () => {
    vi.mocked(invoke).mockReset();
    vi.mocked(invoke).mockResolvedValue('result');
    act(() => usePlanStore.setState(initialState));
    usePlanStore.getState().addStep('test step');
    await usePlanStore.getState().executeStep(0);
    const step = usePlanStore.getState().currentPlan!.steps[0];
    expect(step.status).toBe('completed');
    expect(step.result).toBe('result');
  });

  it('executeStep fails step', async () => {
    vi.mocked(invoke).mockReset();
    vi.mocked(invoke).mockRejectedValue(new Error('fail'));
    act(() => usePlanStore.setState(initialState));
    usePlanStore.getState().addStep('bad step');
    await expect(usePlanStore.getState().executeStep(0)).rejects.toThrow('fail');
    const step = usePlanStore.getState().currentPlan!.steps[0];
    expect(step.status).toBe('failed');
    expect(step.error).toBe('Error: fail');
  });

  it('executeStep no-ops if no plan', async () => {
    act(() => usePlanStore.setState(initialState));
    await usePlanStore.getState().executeStep(0);
    expect(usePlanStore.getState().currentPlan).toBeNull();
  });

  it('cancelExecution resets to ready', () => {
    act(() => usePlanStore.setState({ ...initialState, isExecuting: true }));
    usePlanStore.getState().addStep('s1');
    usePlanStore.getState().cancelExecution();
    expect(usePlanStore.getState().isExecuting).toBe(false);
    expect(usePlanStore.getState().currentPlan!.status).toBe('ready');
  });

  it('cancelExecution no-ops if no plan', () => {
    act(() => usePlanStore.setState(initialState));
    usePlanStore.getState().cancelExecution();
    expect(usePlanStore.getState().currentPlan).toBeNull();
  });

  it('setQuestion adds pending question', () => {
    act(() => usePlanStore.setState(initialState));
    const q = {
      id: 'q1',
      question: 'Continue?',
      options: [{ label: 'Yes', value: 'yes' }],
      allowCustom: false,
      timestamp: 100,
    };
    usePlanStore.getState().setQuestion(q);
    expect(usePlanStore.getState().pendingQuestions).toHaveLength(1);
    expect(usePlanStore.getState().pendingQuestions[0].question).toBe('Continue?');
  });

  it('answerQuestion removes question', () => {
    act(() => usePlanStore.setState(initialState));
    usePlanStore
      .getState()
      .setQuestion({ id: 'q1', question: '?', options: [], allowCustom: false, timestamp: 1 });
    usePlanStore.getState().answerQuestion('q1', 'yes');
    expect(usePlanStore.getState().pendingQuestions).toHaveLength(0);
  });

  it('dismissQuestion removes question', () => {
    act(() => usePlanStore.setState(initialState));
    usePlanStore
      .getState()
      .setQuestion({ id: 'q1', question: '?', options: [], allowCustom: false, timestamp: 1 });
    usePlanStore.getState().dismissQuestion('q1');
    expect(usePlanStore.getState().pendingQuestions).toHaveLength(0);
  });

  it('clearPlan resets plan and executing', () => {
    act(() => usePlanStore.setState({ ...initialState, isExecuting: true }));
    usePlanStore.getState().addStep('s1');
    usePlanStore.getState().clearPlan();
    expect(usePlanStore.getState().currentPlan).toBeNull();
    expect(usePlanStore.getState().isExecuting).toBe(false);
  });
});
