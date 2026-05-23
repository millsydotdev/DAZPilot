import { describe, it, expect, vi } from 'vitest';
import { act } from 'react';
import { useScratchpadStore } from './scratchpadStore';

const initialState = {
  notes: [],
  todos: [],
  newNote: '',
  newTodo: '',
  loaded: false,
};

describe('scratchpadStore', () => {
  it('setNewNote updates draft', () => {
    act(() => useScratchpadStore.setState(initialState));
    useScratchpadStore.getState().setNewNote('hello');
    expect(useScratchpadStore.getState().newNote).toBe('hello');
  });

  it('setNewTodo updates draft', () => {
    act(() => useScratchpadStore.setState(initialState));
    useScratchpadStore.getState().setNewTodo('buy milk');
    expect(useScratchpadStore.getState().newTodo).toBe('buy milk');
  });

  it('addNote adds note and clears input', () => {
    act(() => useScratchpadStore.setState(initialState));
    useScratchpadStore.getState().setNewNote('my note');
    useScratchpadStore.getState().addNote();
    const s = useScratchpadStore.getState();
    expect(s.notes).toHaveLength(1);
    expect(s.notes[0].content).toBe('my note');
    expect(s.newNote).toBe('');
  });

  it('addNote ignores empty input', () => {
    act(() => useScratchpadStore.setState(initialState));
    useScratchpadStore.getState().setNewNote('   ');
    useScratchpadStore.getState().addNote();
    expect(useScratchpadStore.getState().notes).toHaveLength(0);
  });

  it('updateNote updates content', () => {
    act(() => useScratchpadStore.setState(initialState));
    useScratchpadStore.getState().setNewNote('original');
    useScratchpadStore.getState().addNote();
    const id = useScratchpadStore.getState().notes[0].id;
    useScratchpadStore.getState().updateNote(id, 'updated');
    expect(useScratchpadStore.getState().notes[0].content).toBe('updated');
  });

  it('deleteNote removes note', () => {
    act(() => useScratchpadStore.setState(initialState));
    useScratchpadStore.getState().setNewNote('note');
    useScratchpadStore.getState().addNote();
    const id = useScratchpadStore.getState().notes[0].id;
    useScratchpadStore.getState().deleteNote(id);
    expect(useScratchpadStore.getState().notes).toHaveLength(0);
  });

  it('addTodo adds todo and clears input', () => {
    act(() => useScratchpadStore.setState(initialState));
    useScratchpadStore.getState().setNewTodo('task');
    useScratchpadStore.getState().addTodo();
    const s = useScratchpadStore.getState();
    expect(s.todos).toHaveLength(1);
    expect(s.todos[0].content).toBe('task');
    expect(s.todos[0].completed).toBe(false);
    expect(s.todos[0].priority).toBe('medium');
    expect(s.newTodo).toBe('');
  });

  it('addTodo ignores empty input', () => {
    act(() => useScratchpadStore.setState(initialState));
    useScratchpadStore.getState().setNewTodo('');
    useScratchpadStore.getState().addTodo();
    expect(useScratchpadStore.getState().todos).toHaveLength(0);
  });

  it('toggleTodo toggles completed', () => {
    act(() => useScratchpadStore.setState(initialState));
    useScratchpadStore.getState().setNewTodo('task');
    useScratchpadStore.getState().addTodo();
    const id = useScratchpadStore.getState().todos[0].id;
    useScratchpadStore.getState().toggleTodo(id);
    expect(useScratchpadStore.getState().todos[0].completed).toBe(true);
    useScratchpadStore.getState().toggleTodo(id);
    expect(useScratchpadStore.getState().todos[0].completed).toBe(false);
  });

  it('deleteTodo removes todo', () => {
    act(() => useScratchpadStore.setState(initialState));
    useScratchpadStore.getState().setNewTodo('task');
    useScratchpadStore.getState().addTodo();
    const id = useScratchpadStore.getState().todos[0].id;
    useScratchpadStore.getState().deleteTodo(id);
    expect(useScratchpadStore.getState().todos).toHaveLength(0);
  });

  it('setTodoPriority changes priority', () => {
    act(() => useScratchpadStore.setState(initialState));
    useScratchpadStore.getState().setNewTodo('task');
    useScratchpadStore.getState().addTodo();
    const id = useScratchpadStore.getState().todos[0].id;
    useScratchpadStore.getState().setTodoPriority(id, 'high');
    expect(useScratchpadStore.getState().todos[0].priority).toBe('high');
  });

  it('clearCompleted removes completed todos', () => {
    act(() =>
      useScratchpadStore.setState({
        ...initialState,
        todos: [
          { id: 't1', content: 'completed task', completed: true, priority: 'medium' as const },
          { id: 't2', content: 'pending task', completed: false, priority: 'high' as const },
        ],
      })
    );
    useScratchpadStore.getState().clearCompleted();
    expect(useScratchpadStore.getState().todos).toHaveLength(1);
    expect(useScratchpadStore.getState().todos[0].content).toBe('pending task');
  });

  it('loadPersistedData loads data from invoke', async () => {
    act(() => useScratchpadStore.setState(initialState));
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockReset();
    vi.mocked(invoke).mockResolvedValueOnce([
      { id: 'n1', content: 'note1', tags: [], created_at: 100 },
    ]);
    vi.mocked(invoke).mockResolvedValueOnce([
      { id: 't1', content: 'todo1', completed: false, priority: 'high', created_at: 200 },
    ]);
    await useScratchpadStore.getState().loadPersistedData();
    const s = useScratchpadStore.getState();
    expect(s.loaded).toBe(true);
    expect(s.notes).toHaveLength(1);
    expect(s.notes[0].content).toBe('note1');
    expect(s.todos).toHaveLength(1);
    expect(s.todos[0].priority).toBe('high');
  });

  it('loadPersistedData no-ops if already loaded', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockReset();
    act(() => useScratchpadStore.setState({ ...initialState, loaded: true }));
    await useScratchpadStore.getState().loadPersistedData();
    expect(vi.mocked(invoke)).not.toHaveBeenCalled();
  });

  it('loadPersistedData handles error', async () => {
    act(() => useScratchpadStore.setState(initialState));
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockRejectedValue(new Error('fail'));
    await useScratchpadStore.getState().loadPersistedData();
    expect(useScratchpadStore.getState().loaded).toBe(true);
  });

  it('reset restores initial state', () => {
    act(() => useScratchpadStore.setState(initialState));
    useScratchpadStore.getState().setNewNote('x');
    useScratchpadStore.getState().addNote();
    useScratchpadStore.getState().reset();
    const s = useScratchpadStore.getState();
    expect(s.notes).toHaveLength(0);
    expect(s.newNote).toBe('');
  });
});
