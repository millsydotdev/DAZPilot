import { describe, it, expect, vi, beforeEach } from 'vitest';
import { act } from 'react';
import { useConnectionStore } from './connectionStore';
import { useToastStore } from './toastStore';

const initialState = {
  status: 'not_connected' as const,
  isConnecting: false,
  settings: { host: 'localhost', port: 8765, autoConnect: true, timeout: 30 },
  aiModel: { name: 'ollama', size: 0, loaded: false },
  error: null,
};

describe('connectionStore', () => {
  beforeEach(() => {
    act(() => useToastStore.setState({ toasts: [] }));
  });

  it('setStatus updates status', () => {
    act(() => useConnectionStore.setState(initialState));
    useConnectionStore.getState().setStatus('connected');
    expect(useConnectionStore.getState().status).toBe('connected');
  });

  it('setSettings persists and updates state', async () => {
    act(() => useConnectionStore.setState(initialState));
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockReset();
    vi.mocked(invoke).mockResolvedValue(undefined);
    await useConnectionStore.getState().setSettings({ host: '192.168.1.1', port: 9000 });
    expect(useConnectionStore.getState().settings.host).toBe('192.168.1.1');
    expect(useConnectionStore.getState().settings.port).toBe(9000);
  });

  it('setSettings handles invoke error gracefully', async () => {
    act(() => useConnectionStore.setState(initialState));
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockRejectedValue(new Error('save fail'));
    await useConnectionStore.getState().setSettings({ host: 'new' });
    expect(useConnectionStore.getState().settings.host).toBe('new');
  });

  it('loadSettings loads from invoke', async () => {
    act(() => useConnectionStore.setState(initialState));
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockReset();
    vi.mocked(invoke).mockResolvedValueOnce('192.168.1.1');
    vi.mocked(invoke).mockResolvedValueOnce('9999');
    vi.mocked(invoke).mockResolvedValueOnce('false');
    vi.mocked(invoke).mockResolvedValueOnce('60');
    await useConnectionStore.getState().loadSettings();
    const s = useConnectionStore.getState().settings;
    expect(s.host).toBe('192.168.1.1');
    expect(s.port).toBe(9999);
    expect(s.autoConnect).toBe(false);
    expect(s.timeout).toBe(60);
  });

  it('loadSettings uses defaults on error', async () => {
    act(() => useConnectionStore.setState(initialState));
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockRejectedValue(new Error('fail'));
    await useConnectionStore.getState().loadSettings();
    expect(useConnectionStore.getState().settings.host).toBe('localhost');
  });

  it('setAiModel', () => {
    act(() => useConnectionStore.setState(initialState));
    useConnectionStore.getState().setAiModel({ name: 'llama', size: 100, loaded: true });
    expect(useConnectionStore.getState().aiModel.name).toBe('llama');
    expect(useConnectionStore.getState().aiModel.loaded).toBe(true);
  });

  it('setError', () => {
    act(() => useConnectionStore.setState(initialState));
    useConnectionStore.getState().setError('oops');
    expect(useConnectionStore.getState().error).toBe('oops');
  });

  it('connect succeeds', async () => {
    act(() => useConnectionStore.setState(initialState));
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockReset();
    vi.mocked(invoke).mockResolvedValueOnce(undefined);
    vi.mocked(invoke).mockResolvedValueOnce('connected');
    vi.mocked(invoke).mockResolvedValueOnce({ name: 'ollama', size: 0, loaded: false });
    await useConnectionStore.getState().connect();
    expect(useConnectionStore.getState().status).toBe('connected');
  });

  it('connect fails with error', async () => {
    act(() => useConnectionStore.setState(initialState));
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockRejectedValue(new Error('connection refused'));
    await useConnectionStore.getState().connect();
    expect(useConnectionStore.getState().status).toBe('error');
    expect(useConnectionStore.getState().error).toContain('connection refused');
  });

  it('disconnect succeeds', async () => {
    act(() => useConnectionStore.setState({ ...initialState, status: 'connected' }));
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockResolvedValue(undefined);
    await useConnectionStore.getState().disconnect();
    expect(useConnectionStore.getState().status).toBe('not_connected');
  });

  it('disconnect sets not_connected even on error', async () => {
    act(() => useConnectionStore.setState({ ...initialState, status: 'connected' }));
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockRejectedValue(new Error('fail'));
    await useConnectionStore.getState().disconnect();
    expect(useConnectionStore.getState().status).toBe('not_connected');
  });

  it('checkStatus updates from invoke', async () => {
    act(() => useConnectionStore.setState(initialState));
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockReset();
    vi.mocked(invoke).mockResolvedValueOnce('connected');
    vi.mocked(invoke).mockResolvedValueOnce({ name: 'llama3', size: 100, loaded: true });
    await useConnectionStore.getState().checkStatus();
    expect(useConnectionStore.getState().status).toBe('connected');
    expect(useConnectionStore.getState().aiModel.name).toBe('llama3');
  });

  it('checkStatus sets error on failure', async () => {
    act(() => useConnectionStore.setState(initialState));
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockRejectedValue(new Error('check fail'));
    await useConnectionStore.getState().checkStatus();
    expect(useConnectionStore.getState().error).toBe('Error: check fail');
  });

  it('reset restores initial state', () => {
    act(() => useConnectionStore.setState(initialState));
    useConnectionStore.getState().setStatus('connected');
    useConnectionStore.getState().reset();
    expect(useConnectionStore.getState().status).toBe('not_connected');
    expect(useConnectionStore.getState().error).toBeNull();
  });
});
