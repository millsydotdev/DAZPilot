import { describe, it, expect, vi } from 'vitest';
import { act } from 'react';
import { useOllamaStore } from './ollamaStore';
import { invoke } from '@tauri-apps/api/core';

const initialState = {
  isRunning: false,
  models: [],
  currentModel: null,
  isLoading: false,
  error: null,
};

describe('ollamaStore', () => {
  it('checkStatus sets isRunning on success', async () => {
    act(() => useOllamaStore.setState(initialState));
    vi.mocked(invoke).mockResolvedValue(true);
    await useOllamaStore.getState().checkStatus();
    expect(useOllamaStore.getState().isRunning).toBe(true);
    expect(useOllamaStore.getState().error).toBeNull();
  });

  it('checkStatus sets error on failure', async () => {
    act(() => useOllamaStore.setState(initialState));
    vi.mocked(invoke).mockRejectedValue(new Error('conn failed'));
    await useOllamaStore.getState().checkStatus();
    expect(useOllamaStore.getState().isRunning).toBe(false);
    expect(useOllamaStore.getState().error).toBe('Error: conn failed');
  });

  it('loadModels sets models on success', async () => {
    act(() => useOllamaStore.setState(initialState));
    const models = [{ name: 'llama2', size: 100, modified_at: 'now' }];
    vi.mocked(invoke).mockResolvedValue(models);
    await useOllamaStore.getState().loadModels();
    expect(useOllamaStore.getState().models).toEqual(models);
    expect(useOllamaStore.getState().isLoading).toBe(false);
  });

  it('loadModels sets error on failure', async () => {
    act(() => useOllamaStore.setState(initialState));
    vi.mocked(invoke).mockRejectedValue(new Error('no ollama'));
    await useOllamaStore.getState().loadModels();
    expect(useOllamaStore.getState().error).toBe('Error: no ollama');
    expect(useOllamaStore.getState().isLoading).toBe(false);
  });

  it('pullModel calls invoke and reloads models', async () => {
    vi.mocked(invoke).mockReset();
    vi.mocked(invoke).mockResolvedValueOnce(undefined);
    vi.mocked(invoke).mockResolvedValueOnce([]);
    act(() => useOllamaStore.setState(initialState));
    await useOllamaStore.getState().pullModel('llama3');
    expect(vi.mocked(invoke)).toHaveBeenCalledWith('pull_ollama_model', { modelName: 'llama3' });
    expect(useOllamaStore.getState().isLoading).toBe(false);
  });

  it('pullModel sets error on failure', async () => {
    act(() => useOllamaStore.setState(initialState));
    vi.mocked(invoke).mockRejectedValue(new Error('pull fail'));
    await useOllamaStore.getState().pullModel('bad');
    expect(useOllamaStore.getState().error).toBe('Error: pull fail');
    expect(useOllamaStore.getState().isLoading).toBe(false);
  });

  it('chat returns null if no currentModel', async () => {
    act(() => useOllamaStore.setState({ ...initialState, currentModel: null }));
    const result = await useOllamaStore.getState().chat([{ role: 'user', content: 'hi' }]);
    expect(result).toBeNull();
    expect(useOllamaStore.getState().error).toBe('No model selected');
  });

  it('chat returns response on success', async () => {
    const response = { message: { role: 'assistant', content: 'hello' }, done: true };
    vi.mocked(invoke).mockResolvedValue(response);
    act(() => useOllamaStore.setState({ ...initialState, currentModel: 'llama2' }));
    const result = await useOllamaStore.getState().chat([{ role: 'user', content: 'hi' }], 0.5);
    expect(result).toEqual(response);
    expect(vi.mocked(invoke)).toHaveBeenCalledWith('ollama_chat', {
      model: 'llama2',
      messages: [{ role: 'user', content: 'hi' }],
      temperature: 0.5,
    });
  });

  it('chat returns null on error', async () => {
    vi.mocked(invoke).mockRejectedValue(new Error('chat fail'));
    act(() => useOllamaStore.setState({ ...initialState, currentModel: 'llama2' }));
    const result = await useOllamaStore.getState().chat([{ role: 'user', content: 'hi' }]);
    expect(result).toBeNull();
    expect(useOllamaStore.getState().error).toBe('Error: chat fail');
  });

  it('setCurrentModel', () => {
    act(() => useOllamaStore.setState(initialState));
    useOllamaStore.getState().setCurrentModel('llama3');
    expect(useOllamaStore.getState().currentModel).toBe('llama3');
  });
});
