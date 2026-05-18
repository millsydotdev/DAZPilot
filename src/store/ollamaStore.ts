import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/core';

export interface OllamaModel {
  name: string;
  size: number;
  modified_at: string;
}

export interface ChatMessage {
  role: string;
  content: string;
}

export interface ChatResponse {
  message: ChatMessage;
  done: boolean;
}

interface OllamaState {
  isRunning: boolean;
  models: OllamaModel[];
  currentModel: string | null;
  isLoading: boolean;
  error: string | null;
  checkStatus: () => Promise<void>;
  loadModels: () => Promise<void>;
  pullModel: (modelName: string) => Promise<void>;
  chat: (messages: ChatMessage[], temperature?: number) => Promise<ChatResponse | null>;
  setCurrentModel: (model: string) => void;
}

export const useOllamaStore = create<OllamaState>((set, get) => ({
  isRunning: false,
  models: [],
  currentModel: null,
  isLoading: false,
  error: null,

  checkStatus: async () => {
    try {
      const status = await invoke<boolean>('check_ollama_status');
      set({ isRunning: status, error: null });
    } catch (e) {
      set({ isRunning: false, error: String(e) });
    }
  },

  loadModels: async () => {
    set({ isLoading: true, error: null });
    try {
      const models = await invoke<OllamaModel[]>('get_ollama_models');
      set({ models, isLoading: false });
    } catch (e) {
      set({ error: String(e), isLoading: false });
    }
  },

  pullModel: async (modelName: string) => {
    set({ isLoading: true, error: null });
    try {
      await invoke('pull_ollama_model', { modelName });
      await get().loadModels();
    } catch (e) {
      set({ error: String(e), isLoading: false });
    }
  },

  chat: async (messages: ChatMessage[], temperature = 0.7) => {
    const { currentModel } = get();
    if (!currentModel) {
      set({ error: 'No model selected' });
      return null;
    }

    set({ isLoading: true, error: null });
    try {
      const response = await invoke<ChatResponse>('ollama_chat', {
        model: currentModel,
        messages,
        temperature,
      });
      set({ isLoading: false });
      return response;
    } catch (e) {
      set({ error: String(e), isLoading: false });
      return null;
    }
  },

  setCurrentModel: (model: string) => {
    set({ currentModel: model });
  },
}));
