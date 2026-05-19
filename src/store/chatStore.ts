import { create } from 'zustand';

export interface ActionParam {
  key: string;
  value: string;
}

export interface StructuredAiAction {
  command: string;
  args: Record<string, string>;
  confidence: number;
  sdk_refs: string[];
  requires_confirmation: boolean;
}

export interface ChatMessage {
  id: string;
  role: 'user' | 'assistant' | 'system';
  content: string;
  timestamp: number;
  loading?: boolean;
  images?: string[];
  action?: StructuredAiAction;
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
    model?: string
  ) => Promise<void>;
  createHistory: (title: string) => string;
  loadHistory: (id: string) => void;
  deleteHistory: (id: string) => void;
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
    set((state) => ({
      messages: [
        ...state.messages,
        {
          ...message,
          id: `msg-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`,
          timestamp: Date.now(),
        },
      ],
    })),

  updateMessage: (id, updates) =>
    set((state) => ({
      messages: state.messages.map((msg) => (msg.id === id ? { ...msg, ...updates } : msg)),
    })),

  removeMessage: (id) =>
    set((state) => ({
      messages: state.messages.filter((msg) => msg.id !== id),
    })),

  clearMessages: () => set({ messages: [] }),
  setLoading: (isLoading) => set({ isLoading }),
  setError: (error) => set({ error }),

  sendMessage: async (content, images, provider, model) => {
    const { addMessage, setLoading, setError } = get();

    addMessage({ role: 'user', content, images, loading: false });
    setLoading(true);
    setError(null);

    try {
      const { invoke } = await import('@tauri-apps/api/core');
      const response = await invoke<{ content: string; action: StructuredAiAction }>(
        'process_chat_message',
        {
          message: content,
          images,
          provider: provider || null,
          model: model || null,
        }
      );

      // If action is present and requires confirmation, we inject a marker that the UI recognizes
      let finalContent = response.content;
      if (response.action?.requires_confirmation) {
        finalContent += `\n\n[ACTION_REQUIRED] Planned action '${response.action.command}' needs confirmation before execution.`;
      }

      addMessage({
        role: 'assistant',
        content: finalContent,
        action: response.action,
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

  createHistory: (title) => {
    const id = `history-${Date.now()}`;
    const newHistory: ChatHistory = {
      id,
      title,
      messages: [],
      createdAt: Date.now(),
      updatedAt: Date.now(),
    };
    set((state) => ({
      history: [newHistory, ...state.history],
      currentHistoryId: id,
      messages: [],
    }));
    return id;
  },

  loadHistory: (id) => {
    const history = get().history.find((h) => h.id === id);
    if (history) {
      set({
        currentHistoryId: id,
        messages: history.messages,
      });
    }
  },

  deleteHistory: (id) =>
    set((state) => ({
      history: state.history.filter((h) => h.id !== id),
      currentHistoryId: state.currentHistoryId === id ? null : state.currentHistoryId,
    })),

  reset: () => set(initialState),
}));
