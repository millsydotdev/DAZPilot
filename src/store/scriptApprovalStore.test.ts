import { describe, it, expect, vi, beforeEach } from 'vitest';
import { act } from 'react';
import { useScriptApprovalStore } from './scriptApprovalStore';
import { invoke } from '@tauri-apps/api/core';
import { useToastStore } from './toastStore';

const initialState = {
  suggestions: [],
  isOpen: false,
  activeScriptId: null,
  isLoading: false,
};

describe('scriptApprovalStore', () => {
  beforeEach(() => {
    vi.mocked(invoke).mockReset();
    act(() => useToastStore.setState({ toasts: [] }));
  });

  it('addSuggestion adds and opens panel', () => {
    act(() => useScriptApprovalStore.setState(initialState));
    useScriptApprovalStore.getState().addSuggestion({
      id: 's1',
      script: 'print("hello")',
      context: 'test',
      timestamp: 'now',
    });
    const s = useScriptApprovalStore.getState();
    expect(s.suggestions).toHaveLength(1);
    expect(s.suggestions[0].status).toBe('pending');
    expect(s.isOpen).toBe(true);
  });

  it('approveScript calls invoke and updates status', async () => {
    act(() => useScriptApprovalStore.setState(initialState));
    useScriptApprovalStore.getState().addSuggestion({
      id: 's1',
      script: 'print("x")',
      context: 'test',
      timestamp: 'now',
    });
    vi.mocked(invoke).mockResolvedValue(undefined);
    const id = useScriptApprovalStore.getState().suggestions[0].id;
    await useScriptApprovalStore.getState().approveScript(id);
    expect(useScriptApprovalStore.getState().suggestions[0].status).toBe('approved');
  });

  it('approveScript no-ops if suggestion not found', async () => {
    act(() => useScriptApprovalStore.setState(initialState));
    await useScriptApprovalStore.getState().approveScript('nope');
    expect(useScriptApprovalStore.getState().suggestions).toHaveLength(0);
  });

  it('approveScript handles invoke error', async () => {
    act(() => useScriptApprovalStore.setState(initialState));
    useScriptApprovalStore.getState().addSuggestion({
      id: 's1',
      script: 'print',
      context: 'test',
      timestamp: 'now',
    });
    vi.mocked(invoke).mockRejectedValue(new Error('exec fail'));
    const id = useScriptApprovalStore.getState().suggestions[0].id;
    await useScriptApprovalStore.getState().approveScript(id);
    expect(useScriptApprovalStore.getState().suggestions[0].status).toBe('pending');
  });

  it('rejectScript updates status', () => {
    act(() => useScriptApprovalStore.setState(initialState));
    useScriptApprovalStore.getState().addSuggestion({
      id: 's1',
      script: 'print',
      context: 'test',
      timestamp: 'now',
    });
    const id = useScriptApprovalStore.getState().suggestions[0].id;
    useScriptApprovalStore.getState().rejectScript(id);
    expect(useScriptApprovalStore.getState().suggestions[0].status).toBe('rejected');
  });

  it('updateScript updates script content', () => {
    act(() => useScriptApprovalStore.setState(initialState));
    useScriptApprovalStore.getState().addSuggestion({
      id: 's1',
      script: 'old',
      context: 'test',
      timestamp: 'now',
    });
    const id = useScriptApprovalStore.getState().suggestions[0].id;
    useScriptApprovalStore.getState().updateScript(id, 'new script');
    expect(useScriptApprovalStore.getState().suggestions[0].script).toBe('new script');
  });

  it('setActiveScript', () => {
    act(() => useScriptApprovalStore.setState(initialState));
    useScriptApprovalStore.getState().setActiveScript('s1');
    expect(useScriptApprovalStore.getState().activeScriptId).toBe('s1');
    useScriptApprovalStore.getState().setActiveScript(null);
    expect(useScriptApprovalStore.getState().activeScriptId).toBeNull();
  });

  it('executeScript succeeds', async () => {
    act(() => useScriptApprovalStore.setState(initialState));
    useScriptApprovalStore.getState().addSuggestion({
      id: 's1',
      script: 'print',
      context: 'test',
      timestamp: 'now',
    });
    vi.mocked(invoke).mockResolvedValue(undefined);
    const id = useScriptApprovalStore.getState().suggestions[0].id;
    await useScriptApprovalStore.getState().executeScript(id);
    expect(useScriptApprovalStore.getState().suggestions[0].status).toBe('approved');
    expect(useScriptApprovalStore.getState().isLoading).toBe(false);
  });

  it('executeScript no-ops if suggestion not found', async () => {
    act(() => useScriptApprovalStore.setState(initialState));
    await useScriptApprovalStore.getState().executeScript('nope');
    expect(useScriptApprovalStore.getState().isLoading).toBe(false);
  });

  it('executeScript handles invoke error', async () => {
    act(() => useScriptApprovalStore.setState(initialState));
    useScriptApprovalStore.getState().addSuggestion({
      id: 's1',
      script: 'print',
      context: 'test',
      timestamp: 'now',
    });
    vi.mocked(invoke).mockRejectedValue(new Error('fail'));
    const id = useScriptApprovalStore.getState().suggestions[0].id;
    await useScriptApprovalStore.getState().executeScript(id);
    expect(useScriptApprovalStore.getState().isLoading).toBe(false);
  });

  it('clearHistory resets suggestions', () => {
    act(() => useScriptApprovalStore.setState(initialState));
    useScriptApprovalStore.getState().addSuggestion({
      id: 's1',
      script: 'print',
      context: 'test',
      timestamp: 'now',
    });
    useScriptApprovalStore.getState().clearHistory();
    expect(useScriptApprovalStore.getState().suggestions).toHaveLength(0);
  });

  it('togglePanel toggles isOpen', () => {
    act(() => useScriptApprovalStore.setState(initialState));
    useScriptApprovalStore.getState().togglePanel();
    expect(useScriptApprovalStore.getState().isOpen).toBe(true);
    useScriptApprovalStore.getState().togglePanel();
    expect(useScriptApprovalStore.getState().isOpen).toBe(false);
  });

  it('setOpen sets isOpen', () => {
    act(() => useScriptApprovalStore.setState(initialState));
    useScriptApprovalStore.getState().setOpen(true);
    expect(useScriptApprovalStore.getState().isOpen).toBe(true);
    useScriptApprovalStore.getState().setOpen(false);
    expect(useScriptApprovalStore.getState().isOpen).toBe(false);
  });
});
