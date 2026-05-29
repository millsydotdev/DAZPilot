import { create } from 'zustand';

export interface ActionParam {
  key: string;
  value: string;
}

export interface StructuredAiAction {
  command: string;
  args: Record<string, unknown>;
  confidence: number;
  sdk_refs: string[];
  requires_confirmation: boolean;
  teach?: string;
}

export interface ChatMessage {
  id: string;
  role: 'user' | 'assistant' | 'system';
  content: string;
  timestamp: number;
  loading?: boolean;
  images?: string[];
  action?: StructuredAiAction;
  teach?: string;
  manualSteps?: string;
  feedback?: 'up' | 'down';
}

export interface ChatHistory {
  id: string;
  title: string;
  messages: ChatMessage[];
  createdAt: number;
  updatedAt: number;
}

export interface ChatState {
  messages: ChatMessage[];
  input: string;
  isLoading: boolean;
  history: ChatHistory[];
  currentHistoryId: string | null;
  error: string | null;
}

export interface ChatActions {
  setInput: (input: string) => void;
  addMessage: (message: Omit<ChatMessage, 'id' | 'timestamp'>) => void;
  updateMessage: (id: string, updates: Partial<ChatMessage>) => void;
  removeMessage: (id: string) => void;
  clearMessages: () => void;
  setLoading: (loading: boolean) => void;
  setError: (error: string | null) => void;
  sendMessage: (
    content: string,
    images?: string[],
    provider?: string,
    model?: string,
    forceConfirmation?: boolean
  ) => Promise<void>;
  createHistory: (title: string) => Promise<string>;
  loadHistory: (id: string) => Promise<void>;
  deleteHistory: (id: string) => Promise<void>;
  saveCurrentConversation: () => Promise<void>;
  loadHistoryList: () => Promise<void>;
  reset: () => void;
}

const initialState: ChatState = {
  messages: [],
  input: '',
  isLoading: false,
  history: [],
  currentHistoryId: null,
  error: null,
};

export const useChatStore = create<ChatState & ChatActions>((set, get) => ({
  ...initialState,

  setInput: (input) => set({ input }),

  addMessage: (message) =>
    set((state) => {
      const newMessages = [
        ...state.messages,
        {
          ...message,
          id: `msg-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`,
          timestamp: Date.now(),
        },
      ];
      // Auto-save to database after adding message
      setTimeout(() => get().saveCurrentConversation(), 100);
      return { messages: newMessages };
    }),

  updateMessage: (id, updates) =>
    set((state) => {
      const newMessages = state.messages.map((msg) =>
        msg.id === id ? { ...msg, ...updates } : msg
      );
      setTimeout(() => get().saveCurrentConversation(), 100);
      return { messages: newMessages };
    }),

  removeMessage: (id) =>
    set((state) => {
      const newMessages = state.messages.filter((msg) => msg.id !== id);
      setTimeout(() => get().saveCurrentConversation(), 100);
      return { messages: newMessages };
    }),

  clearMessages: () => {
    set({ messages: [] });
    setTimeout(() => get().saveCurrentConversation(), 100);
  },
  setLoading: (isLoading) => set({ isLoading }),
  setError: (error) => set({ error }),

  sendMessage: async (content, images, provider, model, forceConfirmation?: boolean) => {
    const { addMessage, setLoading, setError } = get();

    addMessage({ role: 'user', content, images, loading: false });
    setLoading(true);
    setError(null);

    try {
      const { invoke } = await import('@tauri-apps/api/core');
      const response = await invoke<{
        content: string;
        action: StructuredAiAction;
        teach?: string;
        manual_steps?: string;
      }>('process_chat_message', {
        message: content,
        images,
        provider: provider || null,
        model: model || null,
      });

      // Force confirmation if in Guide Me mode
      if (forceConfirmation && response.action) {
        response.action.requires_confirmation = true;
      }

      // If action is present and requires confirmation, we inject a marker that the UI recognizes
      let finalContent = response.content;
      if (response.action?.requires_confirmation) {
        finalContent += `\n\n[ACTION_REQUIRED] Planned action '${response.action.command}' needs confirmation before execution.`;
      }

      addMessage({
        role: 'assistant',
        content: finalContent,
        action: response.action,
        teach: response.teach,
        manualSteps: response.manual_steps,
        loading: false,
      });
    } catch (e) {
      setError(String(e));
      addMessage({
        role: 'system',
        content: `Error: ${String(e)}`,
        loading: false,
      });
    } finally {
      setLoading(false);
    }
  },

  saveCurrentConversation: async () => {
    const { currentHistoryId, messages, history } = get();
    if (!currentHistoryId) return;
    const title = history.find((h) => h.id === currentHistoryId)?.title || 'Conversation';
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      const now = Date.now();
      await invoke('save_conversation', {
        id: currentHistoryId,
        title,
        messages: JSON.stringify(messages),
        createdAt: now,
        updatedAt: now,
      });
    } catch (e) {
      console.error('Failed to save conversation:', e);
    }
  },

  createHistory: async (title) => {
    const id = `history-${Date.now()}`;
    const now = Date.now();
    const newHistory: ChatHistory = { id, title, messages: [], createdAt: now, updatedAt: now };
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('save_conversation', {
        id,
        title,
        messages: '[]',
        createdAt: now,
        updatedAt: now,
      }).catch((error) => {
        console.warn('Failed to save conversation history:', error);
      });
    } catch (error) {
      console.warn('Failed to create history:', error);
    }
    set((state) => ({
      history: [newHistory, ...state.history],
      currentHistoryId: id,
      messages: [],
    }));
    return id;
  },

  loadHistory: async (id) => {
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      const messagesJson = await invoke<string | null>('load_conversation', { id });
      if (messagesJson) {
        const messages = JSON.parse(messagesJson);
        set({ currentHistoryId: id, messages });
      }
    } catch (e) {
      console.error('Failed to load conversation:', e);
    }
  },

  deleteHistory: async (id) => {
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('delete_conversation', { id }).catch((error) => {
        console.warn('Failed to delete conversation:', error);
      });
    } catch (error) {
      console.warn('Failed to delete history:', error);
    }
    set((state) => ({
      history: state.history.filter((h) => h.id !== id),
      currentHistoryId: state.currentHistoryId === id ? null : state.currentHistoryId,
    }));
  },

  loadHistoryList: async () => {
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      const list =
        await invoke<Array<{ id: string; title: string; createdAt: number; updatedAt: number }>>(
          'list_conversations'
        );
      const history: ChatHistory[] = list.map((item) => ({
        id: item.id,
        title: item.title,
        messages: [],
        createdAt: item.createdAt,
        updatedAt: item.updatedAt,
      }));
      set({ history });
    } catch (e) {
      console.error('Failed to load conversation list:', e);
    }
  },

  reset: () => set(initialState),
}));
