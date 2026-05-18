import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/core';

export type Theme = 'dark' | 'light';
export type LogLevel = 'debug' | 'info' | 'warn' | 'error';
export type ActivePanel = 'chat' | 'assets' | 'viewport' | 'scene' | 'settings' | 'scratchpad';

export interface AppState {
  theme: Theme;
  logLevel: LogLevel;
  activePanel: ActivePanel;
  sidebarCollapsed: boolean;
  autoConnect: boolean;
  connectionTimeout: number;
  wizardCompleted: boolean;
  autoSave: boolean;
  autoSaveInterval: number;
  startupWindowMode: 'windowed' | 'fullscreen';
  systemPrompt: string;
  temperature: number;
  maxTokens: number;
  mockAiMode: boolean;

  // AI settings
  aiProvider: string;
  aiModel: string;
  openaiApiKey: string;
  openaiBaseUrl: string;
  geminiApiKey: string;
  anthropicApiKey: string;
  ollamaHost: string;
}

export interface AppActions {
  setTheme: (theme: Theme) => void;
  setLogLevel: (level: LogLevel) => void;
  setActivePanel: (panel: ActivePanel) => void;
  toggleSidebar: () => void;
  setSidebarCollapsed: (collapsed: boolean) => void;
  setAutoConnect: (autoConnect: boolean) => void;
  setConnectionTimeout: (timeout: number) => void;
  setWizardCompleted: (completed: boolean) => void;
  setAutoSave: (enabled: boolean) => void;
  setAutoSaveInterval: (interval: number) => void;
  setStartupWindowMode: (mode: 'windowed' | 'fullscreen') => void;
  setSystemPrompt: (prompt: string) => void;
  setTemperature: (temp: number) => void;
  setMaxTokens: (tokens: number) => void;
  setMockAiMode: (enabled: boolean) => void;
  reset: () => void;

  // AI settings actions
  setAiProvider: (provider: string) => Promise<void>;
  setAiModel: (model: string) => Promise<void>;
  setOpenaiApiKey: (key: string) => Promise<void>;
  setOpenaiBaseUrl: (url: string) => Promise<void>;
  setGeminiApiKey: (key: string) => Promise<void>;
  setAnthropicApiKey: (key: string) => Promise<void>;
  setOllamaHost: (host: string) => Promise<void>;
  loadAiSettings: () => Promise<void>;
}

const defaultSystemPrompt = `You are DAZPilot Co-Pilot, an AI automation assistant for DAZ3D Studio.
Generate DAZ Script (JavaScript-based) to perform the requested animation, rigging, or scene modifications.
Always format script blocks using triple backticks and explain their functionality clearly.`;

const initialState: AppState = {
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
  systemPrompt: defaultSystemPrompt,
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
};

export const useAppStore = create<AppState & AppActions>((set) => ({
  ...initialState,

  setTheme: (theme) => set({ theme }),
  setLogLevel: (logLevel) => set({ logLevel }),
  setActivePanel: (activePanel) => set({ activePanel }),
  toggleSidebar: () => set((state) => ({ sidebarCollapsed: !state.sidebarCollapsed })),
  setSidebarCollapsed: (sidebarCollapsed) => set({ sidebarCollapsed }),
  setAutoConnect: (autoConnect) => set({ autoConnect }),
  setConnectionTimeout: (connectionTimeout) => set({ connectionTimeout }),
  setWizardCompleted: (wizardCompleted) => set({ wizardCompleted }),
  setAutoSave: (autoSave) => set({ autoSave }),
  setAutoSaveInterval: (autoSaveInterval) => set({ autoSaveInterval }),
  setStartupWindowMode: (startupWindowMode) => set({ startupWindowMode }),
  setSystemPrompt: (systemPrompt) => set({ systemPrompt }),
  setTemperature: (temperature) => {
    set({ temperature });
    invoke('save_app_setting', { key: 'ai_temperature', value: String(temperature) }).catch(
      console.error
    );
  },
  setMaxTokens: (maxTokens) => {
    set({ maxTokens });
    invoke('save_app_setting', { key: 'ai_max_tokens', value: String(maxTokens) }).catch(
      console.error
    );
  },
  setMockAiMode: (mockAiMode) => set({ mockAiMode }),
  reset: () => set(initialState),

  // AI settings persistence actions
  setAiProvider: async (aiProvider) => {
    set({ aiProvider });
    try {
      await invoke('save_app_setting', { key: 'ai_provider', value: aiProvider });
    } catch (e) {
      console.error('Failed to save ai_provider', e);
    }
  },
  setAiModel: async (aiModel) => {
    set({ aiModel });
    try {
      await invoke('save_app_setting', { key: 'ai_model', value: aiModel });
    } catch (e) {
      console.error('Failed to save ai_model', e);
    }
  },
  setOpenaiApiKey: async (openaiApiKey) => {
    set({ openaiApiKey });
    try {
      await invoke('save_app_setting', { key: 'openai_api_key', value: openaiApiKey });
    } catch (e) {
      console.error('Failed to save openai_api_key', e);
    }
  },
  setOpenaiBaseUrl: async (openaiBaseUrl) => {
    set({ openaiBaseUrl });
    try {
      await invoke('save_app_setting', { key: 'openai_base_url', value: openaiBaseUrl });
    } catch (e) {
      console.error('Failed to save openai_base_url', e);
    }
  },
  setGeminiApiKey: async (geminiApiKey) => {
    set({ geminiApiKey });
    try {
      await invoke('save_app_setting', { key: 'gemini_api_key', value: geminiApiKey });
    } catch (e) {
      console.error('Failed to save gemini_api_key', e);
    }
  },
  setAnthropicApiKey: async (anthropicApiKey) => {
    set({ anthropicApiKey });
    try {
      await invoke('save_app_setting', { key: 'anthropic_api_key', value: anthropicApiKey });
    } catch (e) {
      console.error('Failed to save anthropic_api_key', e);
    }
  },
  setOllamaHost: async (ollamaHost) => {
    set({ ollamaHost });
    try {
      await invoke('save_app_setting', { key: 'ollama_host', value: ollamaHost });
    } catch (e) {
      console.error('Failed to save ollama_host', e);
    }
  },

  loadAiSettings: async () => {
    try {
      const aiProvider = await invoke<string | null>('get_app_setting', { key: 'ai_provider' });
      const aiModel = await invoke<string | null>('get_app_setting', { key: 'ai_model' });
      const openaiApiKey = await invoke<string | null>('get_app_setting', {
        key: 'openai_api_key',
      });
      const openaiBaseUrl = await invoke<string | null>('get_app_setting', {
        key: 'openai_base_url',
      });
      const geminiApiKey = await invoke<string | null>('get_app_setting', {
        key: 'gemini_api_key',
      });
      const anthropicApiKey = await invoke<string | null>('get_app_setting', {
        key: 'anthropic_api_key',
      });
      const ollamaHost = await invoke<string | null>('get_app_setting', { key: 'ollama_host' });
      const temperature = await invoke<string | null>('get_app_setting', { key: 'ai_temperature' });
      const maxTokens = await invoke<string | null>('get_app_setting', { key: 'ai_max_tokens' });

      set({
        aiProvider: aiProvider || 'local-gguf',
        aiModel: aiModel || 'phi-2-q4.gguf',
        openaiApiKey: openaiApiKey || '',
        openaiBaseUrl: openaiBaseUrl || 'https://api.openai.com/v1',
        geminiApiKey: geminiApiKey || '',
        anthropicApiKey: anthropicApiKey || '',
        ollamaHost: ollamaHost || 'http://localhost:11434',
        temperature: temperature ? parseFloat(temperature) : 0.7,
        maxTokens: maxTokens ? parseInt(maxTokens) : 2048,
      });
    } catch (e) {
      console.error('Failed to load AI settings', e);
    }
  },
}));
