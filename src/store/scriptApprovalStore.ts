import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/core';
import { useToastStore } from './toastStore';

export interface ScriptSuggestion {
  id: string;
  script: string;
  context: string;
  timestamp: string;
  status: 'pending' | 'approved' | 'rejected' | 'draft';
}

export interface ScriptApprovalState {
  suggestions: ScriptSuggestion[];
  isOpen: boolean;
  activeScriptId: string | null;
  isLoading: boolean;
}

export interface ScriptApprovalActions {
  addSuggestion: (suggestion: Omit<ScriptSuggestion, 'status'>) => void;
  approveScript: (id: string) => Promise<void>;
  rejectScript: (id: string) => void;
  updateScript: (id: string, script: string) => void;
  setActiveScript: (id: string | null) => void;
  clearHistory: () => void;
  togglePanel: () => void;
  setOpen: (open: boolean) => void;
  executeScript: (id: string) => Promise<void>;
}

export const useScriptApprovalStore = create<ScriptApprovalState & ScriptApprovalActions>(
  (set, get) => ({
    suggestions: [],
    isOpen: false,
    activeScriptId: null,
    isLoading: false,

    addSuggestion: (suggestion) => {
      const toast = useToastStore.getState();
      set((state) => ({
        suggestions: [
          { ...suggestion, status: 'pending', id: Date.now().toString() },
          ...state.suggestions,
        ],
        isOpen: true,
      }));
      toast.info('New DazScript suggestion requires your approval.', 5000, 'Script Approval');
    },

    approveScript: async (id: string) => {
      const toast = useToastStore.getState();
      const suggestion = get().suggestions.find((s) => s.id === id);
      if (!suggestion) return;

      try {
        await invoke('execute_approved_script', { script: suggestion.script });
        set((state) => ({
          suggestions: state.suggestions.map((s) =>
            s.id === id ? { ...s, status: 'approved' as const } : s
          ),
        }));
        toast.success('DazScript approved and executed successfully!', 4000, 'Script Executed');
      } catch (e) {
        toast.error(`Script execution failed: ${e}`, 6000);
      }
    },

    rejectScript: (id: string) => {
      const toast = useToastStore.getState();
      set((state) => ({
        suggestions: state.suggestions.map((s) =>
          s.id === id ? { ...s, status: 'rejected' as const } : s
        ),
      }));
      toast.info('DazScript suggestion rejected.', 3000, 'Script Rejected');
    },

    updateScript: (id: string, script: string) => {
      set((state) => ({
        suggestions: state.suggestions.map((s) => (s.id === id ? { ...s, script } : s)),
      }));
    },

    setActiveScript: (id: string | null) => {
      set({ activeScriptId: id });
    },

    executeScript: async (id: string) => {
      const toast = useToastStore.getState();
      const suggestion = get().suggestions.find((s) => s.id === id);
      if (!suggestion) return;

      set(() => ({ isLoading: true }));
      try {
        await invoke('execute_approved_script', { script: suggestion.script });
        set((_state) => ({
          suggestions: _state.suggestions.map((s) =>
            s.id === id ? { ...s, status: 'approved' as const } : s
          ),
          isLoading: false,
        }));
        toast.success('DazScript executed successfully!', 4000, 'Script Executed');
      } catch (e) {
        set(() => ({ isLoading: false }));
        toast.error(`Script execution failed: ${e}`, 6000);
      }
    },

    clearHistory: () => {
      set({ suggestions: [] });
    },

    togglePanel: () => {
      set((state) => ({ isOpen: !state.isOpen }));
    },

    setOpen: (open: boolean) => {
      set({ isOpen: open });
    },
  })
);
