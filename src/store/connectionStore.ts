import { create } from 'zustand';

export type ConnectionStatus = 'not_connected' | 'connecting' | 'connected' | 'error';

export interface ModelInfo {
  name: string;
  size: number;
  loaded: boolean;
}

export interface ConnectionSettings {
  host: string;
  port: number;
  autoConnect: boolean;
  timeout: number;
}

export interface ConnectionState {
  status: ConnectionStatus;
  isConnecting: boolean;
  settings: ConnectionSettings;
  aiModel: ModelInfo;
  error: string | null;
}

export interface ConnectionActions {
  setStatus: (status: ConnectionStatus) => void;
  setSettings: (settings: Partial<ConnectionSettings>) => void;
  setAiModel: (model: ModelInfo) => void;
  setError: (error: string | null) => void;
  connect: () => Promise<void>;
  disconnect: () => Promise<void>;
  checkStatus: () => Promise<void>;
  reset: () => void;
}

const initialState: ConnectionState = {
  status: 'not_connected',
  isConnecting: false,
  settings: {
    host: 'localhost',
    port: 8765,
    autoConnect: true,
    timeout: 30,
  },
  aiModel: { name: 'ollama', size: 0, loaded: false },
  error: null,
};

export const useConnectionStore = create<ConnectionState & ConnectionActions>((set, get) => ({
  ...initialState,

  setStatus: (status) => set({ status }),
  setSettings: (settings) => set((state) => ({ settings: { ...state.settings, ...settings } })),
  setAiModel: (aiModel) => set({ aiModel }),
  setError: (error) => set({ error }),
  connect: async () => {
    set({ status: 'connecting', error: null });
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      const { host, port } = get().settings;
      await invoke('connect_to_daz3d', { host, port });
      set({ status: 'connected' });
      get().checkStatus();
    } catch (e) {
      set({ status: 'error', error: String(e) });
    }
  },
  disconnect: async () => {
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('disconnect_from_daz3d');
      set({ status: 'not_connected' });
    } catch {
      set({ status: 'not_connected' });
    }
  },
  checkStatus: async () => {
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      const status = await invoke<string>('check_connection_status');
      set({ status: status as ConnectionStatus });

      const modelInfo = await invoke<ModelInfo>('get_ai_status');
      set({ aiModel: modelInfo });
    } catch (e) {
      set({ error: String(e) });
    }
  },
  reset: () => set(initialState),
}));
