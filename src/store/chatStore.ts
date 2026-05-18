import { create } from 'zustand';

export interface ChatMessage {
  id: string;
  role: 'user' | 'assistant' | 'system';
  content: string;
  timestamp: number;
  loading?: boolean;
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
  sendMessage: (content: string) => Promise<void>;
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

  sendMessage: async (content) => {
    const { addMessage, setLoading, setError } = get();

    addMessage({ role: 'user', content, loading: false });
    setLoading(true);
    setError(null);

    try {
      const { invoke } = await import('@tauri-apps/api/core');
      const response = await invoke<string>('process_chat_message', { message: content });

      addMessage({ role: 'assistant', content: response, loading: false });
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
