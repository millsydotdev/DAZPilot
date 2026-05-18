import { create } from 'zustand';

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
  setTemperature: (temperature) => set({ temperature }),
  setMaxTokens: (maxTokens) => set({ maxTokens }),
  setMockAiMode: (mockAiMode) => set({ mockAiMode }),
  reset: () => set(initialState),
}));
