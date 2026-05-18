import { create } from 'zustand';

export interface Note {
  id: string;
  content: string;
  timestamp: number;
  tags: string[];
}

export type TodoPriority = 'low' | 'medium' | 'high';

export interface Todo {
  id: string;
  content: string;
  completed: boolean;
  priority: TodoPriority;
}

export interface ScratchpadState {
  notes: Note[];
  todos: Todo[];
  newNote: string;
  newTodo: string;
}

export interface ScratchpadActions {
  setNewNote: (note: string) => void;
  setNewTodo: (todo: string) => void;
  addNote: () => void;
  updateNote: (id: string, content: string) => void;
  deleteNote: (id: string) => void;
  addTodo: () => void;
  toggleTodo: (id: string) => void;
  deleteTodo: (id: string) => void;
  setTodoPriority: (id: string, priority: TodoPriority) => void;
  clearCompleted: () => void;
  reset: () => void;
}

const initialState: ScratchpadState = {
  notes: [],
  todos: [],
  newNote: '',
  newTodo: '',
};

export const useScratchpadStore = create<ScratchpadState & ScratchpadActions>((set, get) => ({
  ...initialState,

  setNewNote: (newNote) => set({ newNote }),
  setNewTodo: (newTodo) => set({ newTodo }),

  addNote: () => {
    const { newNote } = get();
    if (!newNote.trim()) return;

    const note: Note = {
      id: `note-${Date.now()}`,
      content: newNote,
      timestamp: Date.now(),
      tags: [],
    };

    set((state) => ({
      notes: [note, ...state.notes],
      newNote: '',
    }));
  },

  updateNote: (id, content) =>
    set((state) => ({
      notes: state.notes.map((n) => (n.id === id ? { ...n, content } : n)),
    })),

  deleteNote: (id) =>
    set((state) => ({
      notes: state.notes.filter((n) => n.id !== id),
    })),

  addTodo: () => {
    const { newTodo } = get();
    if (!newTodo.trim()) return;

    const todo: Todo = {
      id: `todo-${Date.now()}`,
      content: newTodo,
      completed: false,
      priority: 'medium',
    };

    set((state) => ({
      todos: [todo, ...state.todos],
      newTodo: '',
    }));
  },

  toggleTodo: (id) =>
    set((state) => ({
      todos: state.todos.map((t) => (t.id === id ? { ...t, completed: !t.completed } : t)),
    })),

  deleteTodo: (id) =>
    set((state) => ({
      todos: state.todos.filter((t) => t.id !== id),
    })),

  setTodoPriority: (id, priority) =>
    set((state) => ({
      todos: state.todos.map((t) => (t.id === id ? { ...t, priority } : t)),
    })),

  clearCompleted: () =>
    set((state) => ({
      todos: state.todos.filter((t) => !t.completed),
    })),

  reset: () => set(initialState),
}));
