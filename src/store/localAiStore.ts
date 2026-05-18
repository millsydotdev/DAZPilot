import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/core';

export interface LocalModelInfo {
  name: string;
  size_mb: number;
  loaded: boolean;
}

interface LocalAiState {
  isRunning: boolean;
  models: LocalModelInfo[];
  currentModel: string | null;
  isLoading: boolean;
  error: string | null;
  modelsDir: string;
  checkServerStatus: () => Promise<void>;
  startServer: (modelPath: string, port?: number) => Promise<void>;
  stopServer: () => Promise<void>;
  loadModels: () => Promise<void>;
  getModelsDir: () => Promise<void>;
  downloadModel: (url: string, filename: string) => Promise<void>;
  chat: (prompt: string, model?: string) => Promise<string | null>;
}

const DEFAULT_PORT = 8080;

export const useLocalAiStore = create<LocalAiState>((set, get) => ({
  isRunning: false,
  models: [],
  currentModel: null,
  isLoading: false,
  error: null,
  modelsDir: '',

  getModelsDir: async () => {
    try {
      const dir = await invoke<string>('get_models_dir');
      set({ modelsDir: dir });
    } catch (e) {
      set({ error: String(e) });
    }
  },

  checkServerStatus: async () => {
    try {
      const status = await invoke<boolean>('is_local_server_running');
      set({ isRunning: status, error: null });
    } catch (e) {
      set({ isRunning: false, error: String(e) });
    }
  },

  startServer: async (modelPath: string, port: number = DEFAULT_PORT) => {
    set({ isLoading: true, error: null });
    try {
      await invoke('start_local_server', { modelPath, port });
      set({ isRunning: true, isLoading: false });
    } catch (e) {
      set({ error: String(e), isLoading: false });
    }
  },

  stopServer: async () => {
    try {
      await invoke('stop_local_server');
      set({ isRunning: false });
    } catch (e) {
      set({ error: String(e) });
    }
  },

  loadModels: async () => {
    set({ isLoading: true, error: null });
    try {
      const models = await invoke<LocalModelInfo[]>('list_local_models');
      set({ models, isLoading: false });
    } catch (e) {
      set({ error: String(e), isLoading: false });
    }
  },

  downloadModel: async (url: string, filename: string) => {
    set({ isLoading: true, error: null });
    try {
      await invoke<string>('download_gguf_model', { url, filename });
      await get().loadModels();
    } catch (e) {
      set({ error: String(e), isLoading: false });
    }
  },

  chat: async (prompt: string, model?: string) => {
    const modelName = model || get().currentModel || 'model';
    try {
      const response = await invoke<string>('chat_with_local', {
        prompt,
        model: modelName,
      });
      return response;
    } catch (e) {
      set({ error: String(e) });
      return null;
    }
  },
}));
