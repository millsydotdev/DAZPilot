import { create } from 'zustand';
import { useToastStore } from './toastStore';

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
  setSettings: (settings: Partial<ConnectionSettings>) => Promise<void>;
  loadSettings: () => Promise<void>;
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
  setSettings: async (settings) => {
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      if (settings.host !== undefined) {
        await invoke('save_app_setting', { key: 'daz_bridge_host', value: settings.host });
      }
      if (settings.port !== undefined) {
        await invoke('save_app_setting', { key: 'daz_bridge_port', value: String(settings.port) });
      }
      if (settings.autoConnect !== undefined) {
        await invoke('save_app_setting', {
          key: 'daz_bridge_autoconnect',
          value: String(settings.autoConnect),
        });
      }
      if (settings.timeout !== undefined) {
        await invoke('save_app_setting', {
          key: 'daz_bridge_timeout',
          value: String(settings.timeout),
        });
      }
    } catch (e) {
      console.error('Failed to save connection setting:', e);
    }
    set((state) => ({ settings: { ...state.settings, ...settings } }));
  },
  loadSettings: async () => {
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      const host =
        (await invoke<string | null>('get_app_setting', { key: 'daz_bridge_host' })) || '127.0.0.1';
      const portStr =
        (await invoke<string | null>('get_app_setting', { key: 'daz_bridge_port' })) || '8765';
      const autoconnectStr =
        (await invoke<string | null>('get_app_setting', { key: 'daz_bridge_autoconnect' })) ||
        'true';
      const timeoutStr =
        (await invoke<string | null>('get_app_setting', { key: 'daz_bridge_timeout' })) || '30';

      set({
        settings: {
          host,
          port: parseInt(portStr, 10) || 8765,
          autoConnect: autoconnectStr === 'true',
          timeout: parseInt(timeoutStr, 10) || 30,
        },
      });
    } catch (e) {
      console.error('Failed to load connection settings:', e);
    }
  },
  setAiModel: (aiModel) => set({ aiModel }),
  setError: (error) => set({ error }),
  connect: async () => {
    set({ status: 'connecting', error: null });
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      const { host, port } = get().settings;
      await invoke('connect_to_daz3d', { host, port });
      set({ status: 'connected' });
      useToastStore.getState().success('Connected to Daz Studio Bridge!');
      get().checkStatus();
    } catch (e) {
      const errorMsg = String(e);
      set({ status: 'error', error: errorMsg });
      useToastStore.getState().error(`Bridge connection failed: ${errorMsg}`);
    }
  },
  disconnect: async () => {
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('disconnect_from_daz3d');
      set({ status: 'not_connected' });
      useToastStore.getState().info('Disconnected from Daz Studio Bridge.');
    } catch {
      set({ status: 'not_connected' });
      useToastStore.getState().info('Disconnected from Daz Studio Bridge.');
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
