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
  reset: () => void;
}

const initialState: AppState = {
  theme: 'dark',
  logLevel: 'info',
  activePanel: 'chat',
  sidebarCollapsed: false,
  autoConnect: true,
  connectionTimeout: 30,
  wizardCompleted: false,
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
  reset: () => set(initialState),
}));
