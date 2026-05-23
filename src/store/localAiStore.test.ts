import { describe, it, expect, vi } from 'vitest';
import { act } from 'react';
import { useLocalAiStore } from './localAiStore';
import { invoke } from '@tauri-apps/api/core';

const initialState = {
  isRunning: false,
  models: [],
  currentModel: null,
  isLoading: false,
  error: null,
  modelsDir: '',
};

describe('localAiStore', () => {
  it('getModelsDir sets dir on success', async () => {
    act(() => useLocalAiStore.setState(initialState));
    vi.mocked(invoke).mockResolvedValue('/models');
    await useLocalAiStore.getState().getModelsDir();
    expect(useLocalAiStore.getState().modelsDir).toBe('/models');
  });

  it('getModelsDir sets error on failure', async () => {
    act(() => useLocalAiStore.setState(initialState));
    vi.mocked(invoke).mockRejectedValue(new Error('no dir'));
    await useLocalAiStore.getState().getModelsDir();
    expect(useLocalAiStore.getState().error).toBe('Error: no dir');
  });

  it('setModelsDir saves and reloads models', async () => {
    vi.mocked(invoke).mockReset();
    vi.mocked(invoke).mockResolvedValueOnce(undefined);
    vi.mocked(invoke).mockResolvedValueOnce([]);
    act(() => useLocalAiStore.setState(initialState));
    await useLocalAiStore.getState().setModelsDir('/new');
    expect(vi.mocked(invoke)).toHaveBeenCalledWith('save_app_setting', {
      key: 'local_ai_models_dir',
      value: '/new',
    });
    expect(useLocalAiStore.getState().modelsDir).toBe('/new');
  });

  it('setModelsDir sets error on failure', async () => {
    act(() => useLocalAiStore.setState(initialState));
    vi.mocked(invoke).mockRejectedValue(new Error('save fail'));
    await useLocalAiStore.getState().setModelsDir('/bad');
    expect(useLocalAiStore.getState().error).toBe('Error: save fail');
  });

  it('checkServerStatus sets isRunning on success', async () => {
    act(() => useLocalAiStore.setState(initialState));
    vi.mocked(invoke).mockResolvedValue(true);
    await useLocalAiStore.getState().checkServerStatus();
    expect(useLocalAiStore.getState().isRunning).toBe(true);
  });

  it('checkServerStatus sets error on failure', async () => {
    act(() => useLocalAiStore.setState(initialState));
    vi.mocked(invoke).mockRejectedValue(new Error('fail'));
    await useLocalAiStore.getState().checkServerStatus();
    expect(useLocalAiStore.getState().isRunning).toBe(false);
    expect(useLocalAiStore.getState().error).toBe('Error: fail');
  });

  it('startServer sets isRunning on success', async () => {
    act(() => useLocalAiStore.setState(initialState));
    vi.mocked(invoke).mockResolvedValue(undefined);
    await useLocalAiStore.getState().startServer('/model.gguf', 8080);
    expect(useLocalAiStore.getState().isRunning).toBe(true);
    expect(useLocalAiStore.getState().isLoading).toBe(false);
  });

  it('startServer sets error on failure', async () => {
    act(() => useLocalAiStore.setState(initialState));
    vi.mocked(invoke).mockRejectedValue(new Error('start fail'));
    await useLocalAiStore.getState().startServer('/model.gguf');
    expect(useLocalAiStore.getState().isRunning).toBe(false);
    expect(useLocalAiStore.getState().error).toBe('Error: start fail');
  });

  it('stopServer sets isRunning false on success', async () => {
    act(() => useLocalAiStore.setState({ ...initialState, isRunning: true }));
    vi.mocked(invoke).mockResolvedValue(undefined);
    await useLocalAiStore.getState().stopServer();
    expect(useLocalAiStore.getState().isRunning).toBe(false);
  });

  it('stopServer sets error on failure', async () => {
    act(() => useLocalAiStore.setState({ ...initialState, isRunning: true }));
    vi.mocked(invoke).mockRejectedValue(new Error('stop fail'));
    await useLocalAiStore.getState().stopServer();
    expect(useLocalAiStore.getState().error).toBe('Error: stop fail');
  });

  it('loadModels sets models on success', async () => {
    act(() => useLocalAiStore.setState(initialState));
    const models = [{ name: 'm1', size_mb: 100, loaded: false }];
    vi.mocked(invoke).mockResolvedValue(models);
    await useLocalAiStore.getState().loadModels();
    expect(useLocalAiStore.getState().models).toEqual(models);
    expect(useLocalAiStore.getState().isLoading).toBe(false);
  });

  it('loadModels sets error on failure', async () => {
    act(() => useLocalAiStore.setState(initialState));
    vi.mocked(invoke).mockRejectedValue(new Error('list fail'));
    await useLocalAiStore.getState().loadModels();
    expect(useLocalAiStore.getState().error).toBe('Error: list fail');
  });

  it('downloadModel succeeds', async () => {
    vi.mocked(invoke).mockReset();
    vi.mocked(invoke).mockResolvedValueOnce('');
    vi.mocked(invoke).mockResolvedValueOnce([]);
    act(() => useLocalAiStore.setState(initialState));
    await useLocalAiStore.getState().downloadModel('http://example.com/model.gguf', 'model.gguf');
    expect(useLocalAiStore.getState().isLoading).toBe(false);
    expect(useLocalAiStore.getState().error).toBeNull();
  });

  it('downloadModel sets error on failure', async () => {
    act(() => useLocalAiStore.setState(initialState));
    vi.mocked(invoke).mockRejectedValue(new Error('dl fail'));
    await useLocalAiStore.getState().downloadModel('http://bad', 'bad.gguf');
    expect(useLocalAiStore.getState().error).toBe('Error: dl fail');
    expect(useLocalAiStore.getState().isLoading).toBe(false);
  });

  it('chat returns response on success', async () => {
    vi.mocked(invoke).mockResolvedValue('response text');
    act(() => useLocalAiStore.setState(initialState));
    const result = await useLocalAiStore.getState().chat('hello', 'model1');
    expect(result).toBe('response text');
  });

  it('chat returns null and sets error on failure', async () => {
    act(() => useLocalAiStore.setState(initialState));
    vi.mocked(invoke).mockRejectedValue(new Error('chat fail'));
    const result = await useLocalAiStore.getState().chat('hello');
    expect(result).toBeNull();
    expect(useLocalAiStore.getState().error).toBe('Error: chat fail');
  });
});
