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
  loaded: boolean;
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
  loadPersistedData: () => Promise<void>;
  reset: () => void;
}

const initialState: ScratchpadState = {
  notes: [],
  todos: [],
  newNote: '',
  newTodo: '',
  loaded: false,
};

async function persistNote(note: Note): Promise<void> {
  try {
    const { invoke } = await import('@tauri-apps/api/core');
    await invoke('save_scratchpad_note', {
      note: {
        id: note.id,
        content: note.content,
        tags: note.tags,
        created_at: note.timestamp,
        updated_at: Date.now(),
      },
    });
  } catch (e) {
    console.error('Failed to persist note:', e);
  }
}

async function removeNote(id: string): Promise<void> {
  try {
    const { invoke } = await import('@tauri-apps/api/core');
    await invoke('delete_scratchpad_note', { noteId: id });
  } catch (e) {
    console.error('Failed to delete note:', e);
  }
}

async function persistTodo(todo: Todo): Promise<void> {
  try {
    const { invoke } = await import('@tauri-apps/api/core');
    await invoke('save_scratchpad_todo', {
      todo: {
        id: todo.id,
        content: todo.content,
        completed: todo.completed,
        priority: todo.priority,
        created_at: Date.now(),
        updated_at: Date.now(),
      },
    });
  } catch (e) {
    console.error('Failed to persist todo:', e);
  }
}

async function removeTodo(id: string): Promise<void> {
  try {
    const { invoke } = await import('@tauri-apps/api/core');
    await invoke('delete_scratchpad_todo', { todoId: id });
  } catch (e) {
    console.error('Failed to delete todo:', e);
  }
}

export const useScratchpadStore = create<ScratchpadState & ScratchpadActions>((set, get) => ({
  ...initialState,

  setNewNote: (newNote) => set({ newNote }),
  setNewTodo: (newTodo) => set({ newTodo }),

  loadPersistedData: async () => {
    if (get().loaded) return;
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      const [dbNotes, dbTodos] = await Promise.all([
        invoke<Array<{ id: string; content: string; tags: string[]; created_at: number }>>(
          'load_scratchpad_notes'
        ),
        invoke<
          Array<{
            id: string;
            content: string;
            completed: boolean;
            priority: string;
            created_at: number;
          }>
        >('load_scratchpad_todos'),
      ]);

      const notes: Note[] = dbNotes.map((n) => ({
        id: n.id,
        content: n.content,
        timestamp: n.created_at,
        tags: n.tags,
      }));

      const todos: Todo[] = dbTodos.map((t) => ({
        id: t.id,
        content: t.content,
        completed: t.completed,
        priority: (t.priority as TodoPriority) || 'medium',
      }));

      set({ notes, todos, loaded: true });
    } catch (e) {
      console.error('Failed to load scratchpad data:', e);
      set({ loaded: true });
    }
  },

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
    persistNote(note);
  },

  updateNote: (id, content) => {
    set((state) => ({
      notes: state.notes.map((n) => (n.id === id ? { ...n, content } : n)),
    }));
    const note = get().notes.find((n) => n.id === id);
    if (note) persistNote({ ...note, content });
  },

  deleteNote: (id) => {
    set((state) => ({
      notes: state.notes.filter((n) => n.id !== id),
    }));
    removeNote(id);
  },

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
    persistTodo(todo);
  },

  toggleTodo: (id) => {
    set((state) => ({
      todos: state.todos.map((t) => (t.id === id ? { ...t, completed: !t.completed } : t)),
    }));
    const todo = get().todos.find((t) => t.id === id);
    if (todo) persistTodo(todo);
  },

  deleteTodo: (id) => {
    set((state) => ({
      todos: state.todos.filter((t) => t.id !== id),
    }));
    removeTodo(id);
  },

  setTodoPriority: (id, priority) => {
    set((state) => ({
      todos: state.todos.map((t) => (t.id === id ? { ...t, priority } : t)),
    }));
    const todo = get().todos.find((t) => t.id === id);
    if (todo) persistTodo(todo);
  },

  clearCompleted: () => {
    set((state) => ({
      todos: state.todos.filter((t) => !t.completed),
    }));
    (async () => {
      try {
        const { invoke } = await import('@tauri-apps/api/core');
        await invoke('clear_completed_scratchpad_todos');
      } catch (e) {
        console.error('Failed to clear completed todos:', e);
      }
    })();
  },

  reset: () => set(initialState),
}));
