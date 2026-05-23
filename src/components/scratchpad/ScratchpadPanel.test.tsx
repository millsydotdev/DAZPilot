import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import '@testing-library/jest-dom';
import ScratchpadPanel from './ScratchpadPanel';
import { useScratchpadStore } from '../../store/scratchpadStore';

const initialState = {
  notes: [],
  todos: [],
  newNote: '',
  newTodo: '',
  loaded: false,
};

beforeEach(() => {
  useScratchpadStore.setState(initialState);
  vi.clearAllMocks();
});

describe('ScratchpadPanel', () => {
  it('renders the title', () => {
    render(<ScratchpadPanel />);
    expect(screen.getByText('Scratchpad')).toBeInTheDocument();
  });

  it('shows notes tab by default', () => {
    render(<ScratchpadPanel />);
    expect(screen.getByLabelText('Notes tab').className).toContain('active');
    expect(screen.getByPlaceholderText('Add a note...')).toBeInTheDocument();
  });

  it('shows empty state for notes', () => {
    render(<ScratchpadPanel />);
    expect(screen.getByText('No notes yet. Add one above!')).toBeInTheDocument();
  });

  it('renders notes list', () => {
    useScratchpadStore.setState({
      ...initialState,
      notes: [{ id: 'n1', content: 'Test note', timestamp: 1000, tags: [] }],
      loaded: true,
    });
    render(<ScratchpadPanel />);
    expect(screen.getByText('Test note')).toBeInTheDocument();
  });

  it('renders todos list', () => {
    useScratchpadStore.setState({
      ...initialState,
      todos: [{ id: 't1', content: 'Test todo', completed: false, priority: 'medium' }],
      loaded: true,
    });
    render(<ScratchpadPanel />);
    expect(screen.getByLabelText('Todos tab')).toBeInTheDocument();
  });

  it('switches to todos tab', () => {
    render(<ScratchpadPanel />);
    fireEvent.click(screen.getByLabelText('Todos tab'));
    expect(screen.getByLabelText('Todos tab').className).toContain('active');
    expect(screen.getByText('No todos yet. Add one above!')).toBeInTheDocument();
  });

  it('switches back to notes tab from todos', () => {
    render(<ScratchpadPanel />);
    fireEvent.click(screen.getByLabelText('Todos tab'));
    fireEvent.click(screen.getByLabelText('Notes tab'));
    expect(screen.getByPlaceholderText('Add a note...')).toBeInTheDocument();
  });

  it('shows empty state for todos', () => {
    render(<ScratchpadPanel />);
    fireEvent.click(screen.getByLabelText('Todos tab'));
    expect(screen.getByText('No todos yet. Add one above!')).toBeInTheDocument();
  });

  it('renders todo items with checkbox and priority', () => {
    useScratchpadStore.setState({
      ...initialState,
      todos: [{ id: 't1', content: 'Task A', completed: false, priority: 'high' }],
      loaded: true,
    });
    render(<ScratchpadPanel />);
    fireEvent.click(screen.getByLabelText('Todos tab'));
    expect(screen.getByText('Task A')).toBeInTheDocument();
    expect(screen.getByText('high')).toBeInTheDocument();
    const checkbox = screen.getByLabelText('Task A');
    expect(checkbox).toBeInTheDocument();
    expect(checkbox).not.toBeChecked();
  });

  it('renders completed todo with line-through', () => {
    useScratchpadStore.setState({
      ...initialState,
      todos: [{ id: 't1', content: 'Done task', completed: true, priority: 'low' }],
      loaded: true,
    });
    render(<ScratchpadPanel />);
    fireEvent.click(screen.getByLabelText('Todos tab'));
    const todoText = screen.getByText('Done task');
    expect(todoText.className).toContain('completed');
  });

  it('calls addNote when Enter is pressed in note input', () => {
    render(<ScratchpadPanel />);
    const input = screen.getByPlaceholderText('Add a note...');
    fireEvent.change(input, { target: { value: 'New note' } });
    fireEvent.keyDown(input, { key: 'Enter' });
    expect(useScratchpadStore.getState().notes).toHaveLength(1);
    expect(useScratchpadStore.getState().notes[0].content).toBe('New note');
  });

  it('calls addTodo when Enter is pressed in todo input', () => {
    render(<ScratchpadPanel />);
    fireEvent.click(screen.getByLabelText('Todos tab'));
    const input = screen.getByPlaceholderText('Add a todo...');
    fireEvent.change(input, { target: { value: 'New todo' } });
    fireEvent.keyDown(input, { key: 'Enter' });
    expect(useScratchpadStore.getState().todos).toHaveLength(1);
    expect(useScratchpadStore.getState().todos[0].content).toBe('New todo');
  });
});
