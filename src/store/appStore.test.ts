import { describe, it, expect, vi } from 'vitest';
import { act } from 'react';
import { useAppStore } from './appStore';
import { invoke } from '@tauri-apps/api/core';

const initialState = {
  theme: 'dark',
  logLevel: 'info',
  activePanel: 'chat',
  sidebarCollapsed: false,
  autoConnect: true,
  connectionTimeout: 30,
  wizardCompleted: false,
  autoSave: true,
  autoSaveInterval: 10,
  startupWindowMode: 'windowed',
  systemPrompt: expect.any(String),
  temperature: 0.7,
  maxTokens: 2048,
  mockAiMode: false,
  aiProvider: 'local-gguf',
  aiModel: 'phi-2-q4.gguf',
  openaiApiKey: '',
  openaiBaseUrl: 'https://api.openai.com/v1',
  geminiApiKey: '',
  anthropicApiKey: '',
  ollamaHost: 'http://localhost:11434',
} as const;

describe('appStore', () => {
  it('setTheme', () => {
    act(() => useAppStore.setState(initialState));
    useAppStore.getState().setTheme('light');
    expect(useAppStore.getState().theme).toBe('light');
  });

  it('setLogLevel', () => {
    act(() => useAppStore.setState(initialState));
    useAppStore.getState().setLogLevel('error');
    expect(useAppStore.getState().logLevel).toBe('error');
  });

  it('setActivePanel', () => {
    act(() => useAppStore.setState(initialState));
    useAppStore.getState().setActivePanel('assets');
    expect(useAppStore.getState().activePanel).toBe('assets');
  });

  it('toggleSidebar', () => {
    act(() => useAppStore.setState(initialState));
    useAppStore.getState().toggleSidebar();
    expect(useAppStore.getState().sidebarCollapsed).toBe(true);
    useAppStore.getState().toggleSidebar();
    expect(useAppStore.getState().sidebarCollapsed).toBe(false);
  });

  it('setSidebarCollapsed', () => {
    act(() => useAppStore.setState(initialState));
    useAppStore.getState().setSidebarCollapsed(true);
    expect(useAppStore.getState().sidebarCollapsed).toBe(true);
  });

  it('setAutoConnect', () => {
    act(() => useAppStore.setState(initialState));
    useAppStore.getState().setAutoConnect(false);
    expect(useAppStore.getState().autoConnect).toBe(false);
  });

  it('setConnectionTimeout', () => {
    act(() => useAppStore.setState(initialState));
    useAppStore.getState().setConnectionTimeout(60);
    expect(useAppStore.getState().connectionTimeout).toBe(60);
  });

  it('setWizardCompleted', () => {
    act(() => useAppStore.setState(initialState));
    useAppStore.getState().setWizardCompleted(true);
    expect(useAppStore.getState().wizardCompleted).toBe(true);
  });

  it('setAutoSave', () => {
    act(() => useAppStore.setState(initialState));
    useAppStore.getState().setAutoSave(false);
    expect(useAppStore.getState().autoSave).toBe(false);
  });

  it('setAutoSaveInterval', () => {
    act(() => useAppStore.setState(initialState));
    useAppStore.getState().setAutoSaveInterval(30);
    expect(useAppStore.getState().autoSaveInterval).toBe(30);
  });

  it('setStartupWindowMode', () => {
    act(() => useAppStore.setState(initialState));
    useAppStore.getState().setStartupWindowMode('fullscreen');
    expect(useAppStore.getState().startupWindowMode).toBe('fullscreen');
  });

  it('setSystemPrompt', () => {
    act(() => useAppStore.setState(initialState));
    useAppStore.getState().setSystemPrompt('new prompt');
    expect(useAppStore.getState().systemPrompt).toBe('new prompt');
  });

  it('setTemperature updates and calls invoke', () => {
    act(() => useAppStore.setState(initialState));
    vi.mocked(invoke).mockResolvedValue(undefined);
    useAppStore.getState().setTemperature(0.5);
    expect(useAppStore.getState().temperature).toBe(0.5);
    expect(vi.mocked(invoke)).toHaveBeenCalledWith('save_app_setting', {
      key: 'ai_temperature',
      value: '0.5',
    });
  });

  it('setMaxTokens updates and calls invoke', () => {
    act(() => useAppStore.setState(initialState));
    vi.mocked(invoke).mockResolvedValue(undefined);
    useAppStore.getState().setMaxTokens(4096);
    expect(useAppStore.getState().maxTokens).toBe(4096);
    expect(vi.mocked(invoke)).toHaveBeenCalledWith('save_app_setting', {
      key: 'ai_max_tokens',
      value: '4096',
    });
  });

  it('setMockAiMode', () => {
    act(() => useAppStore.setState(initialState));
    useAppStore.getState().setMockAiMode(true);
    expect(useAppStore.getState().mockAiMode).toBe(true);
  });

  it('reset restores initial state', () => {
    act(() => useAppStore.setState(initialState));
    useAppStore.getState().setTheme('light');
    useAppStore.getState().reset();
    expect(useAppStore.getState().theme).toBe('dark');
  });

  it('setAiProvider persists via invoke', async () => {
    act(() => useAppStore.setState(initialState));
    vi.mocked(invoke).mockResolvedValue(undefined);
    await useAppStore.getState().setAiProvider('openai');
    expect(useAppStore.getState().aiProvider).toBe('openai');
    expect(vi.mocked(invoke)).toHaveBeenCalledWith('save_app_setting', {
      key: 'ai_provider',
      value: 'openai',
    });
  });

  it('setAiProvider handles invoke error', async () => {
    act(() => useAppStore.setState(initialState));
    vi.mocked(invoke).mockRejectedValue(new Error('save fail'));
    await useAppStore.getState().setAiProvider('openai');
    expect(useAppStore.getState().aiProvider).toBe('openai');
  });

  it('setAiModel', async () => {
    act(() => useAppStore.setState(initialState));
    vi.mocked(invoke).mockResolvedValue(undefined);
    await useAppStore.getState().setAiModel('gpt-4');
    expect(useAppStore.getState().aiModel).toBe('gpt-4');
  });

  it('setOpenaiApiKey', async () => {
    act(() => useAppStore.setState(initialState));
    vi.mocked(invoke).mockResolvedValue(undefined);
    await useAppStore.getState().setOpenaiApiKey('sk-xxx');
    expect(useAppStore.getState().openaiApiKey).toBe('sk-xxx');
  });

  it('setOpenaiBaseUrl', async () => {
    act(() => useAppStore.setState(initialState));
    vi.mocked(invoke).mockResolvedValue(undefined);
    await useAppStore.getState().setOpenaiBaseUrl('https://custom.com/v1');
    expect(useAppStore.getState().openaiBaseUrl).toBe('https://custom.com/v1');
  });

  it('setGeminiApiKey', async () => {
    act(() => useAppStore.setState(initialState));
    vi.mocked(invoke).mockResolvedValue(undefined);
    await useAppStore.getState().setGeminiApiKey('sk-gem');
    expect(useAppStore.getState().geminiApiKey).toBe('sk-gem');
  });

  it('setAnthropicApiKey', async () => {
    act(() => useAppStore.setState(initialState));
    vi.mocked(invoke).mockResolvedValue(undefined);
    await useAppStore.getState().setAnthropicApiKey('sk-ant');
    expect(useAppStore.getState().anthropicApiKey).toBe('sk-ant');
  });

  it('setOllamaHost', async () => {
    act(() => useAppStore.setState(initialState));
    vi.mocked(invoke).mockResolvedValue(undefined);
    await useAppStore.getState().setOllamaHost('http://192.168.1.1:11434');
    expect(useAppStore.getState().ollamaHost).toBe('http://192.168.1.1:11434');
  });

  it('loadAiSettings loads all settings', async () => {
    act(() => useAppStore.setState(initialState));
    vi.mocked(invoke).mockReset();
    vi.mocked(invoke)
      .mockResolvedValueOnce('openai')
      .mockResolvedValueOnce('gpt-4')
      .mockResolvedValueOnce('sk-xxx')
      .mockResolvedValueOnce('https://api.openai.com/v1')
      .mockResolvedValueOnce('')
      .mockResolvedValueOnce('')
      .mockResolvedValueOnce('http://localhost:11434')
      .mockResolvedValueOnce('0.8')
      .mockResolvedValueOnce('4096');
    await useAppStore.getState().loadAiSettings();
    const s = useAppStore.getState();
    expect(s.aiProvider).toBe('openai');
    expect(s.aiModel).toBe('gpt-4');
    expect(s.openaiApiKey).toBe('sk-xxx');
    expect(s.temperature).toBe(0.8);
    expect(s.maxTokens).toBe(4096);
  });

  it('loadAiSettings uses defaults when values are null', async () => {
    act(() => useAppStore.setState(initialState));
    vi.mocked(invoke).mockReset();
    vi.mocked(invoke).mockResolvedValue(null);
    await useAppStore.getState().loadAiSettings();
    const s = useAppStore.getState();
    expect(s.aiProvider).toBe('local-gguf');
    expect(s.aiModel).toBe('phi-2-q4.gguf');
    expect(s.temperature).toBe(0.7);
    expect(s.maxTokens).toBe(2048);
  });

  it('loadAiSettings handles error', async () => {
    act(() => useAppStore.setState(initialState));
    vi.mocked(invoke).mockRejectedValue(new Error('load fail'));
    await useAppStore.getState().loadAiSettings();
    expect(useAppStore.getState().aiProvider).toBe('local-gguf');
  });
});
