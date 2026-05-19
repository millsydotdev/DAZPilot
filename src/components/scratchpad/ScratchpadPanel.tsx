import { useState, useEffect } from 'react';
import { Plus, Trash2 } from 'lucide-react';
import { useScratchpadStore } from '../../store';
import { Button, Input } from '../ui';
import styles from './ScratchpadPanel.module.css';

type TabType = 'notes' | 'todos';

export default function ScratchpadPanel() {
  const {
    notes,
    todos,
    newNote,
    newTodo,
    setNewNote,
    setNewTodo,
    addNote,
    deleteNote,
    addTodo,
    toggleTodo,
    deleteTodo,
    loadPersistedData,
  } = useScratchpadStore();

  useEffect(() => {
    loadPersistedData();
  }, [loadPersistedData]);

  const [activeTab, setActiveTab] = useState<TabType>('notes');

  const formatTime = (timestamp: number) => {
    return new Date(timestamp).toLocaleString();
  };

  return (
    <div className={styles.container}>
      <div className={styles.header}>
        <h2 className={styles.title}>Scratchpad</h2>
        <div className={styles.tabs}>
          <button
            className={`${styles.tab} ${activeTab === 'notes' ? styles.active : ''}`}
            onClick={() => setActiveTab('notes')}
            aria-label="Notes tab"
          >
            Notes
          </button>
          <button
            className={`${styles.tab} ${activeTab === 'todos' ? styles.active : ''}`}
            onClick={() => setActiveTab('todos')}
            aria-label="Todos tab"
          >
            Todos
          </button>
        </div>
      </div>

      {activeTab === 'notes' && (
        <>
          <div className={styles.inputRow}>
            <Input
              className={styles.input}
              placeholder="Add a note..."
              value={newNote}
              onChange={(e) => setNewNote(e.target.value)}
              onKeyDown={(e) => e.key === 'Enter' && addNote()}
              aria-label="Add a note"
            />
            <Button
              className={styles.addButton}
              onClick={addNote}
              icon={<Plus size={16} />}
              aria-label="Add note"
            >
              Add
            </Button>
          </div>

          <div className={styles.list}>
            {notes.length === 0 ? (
              <p className={styles.emptyState}>No notes yet. Add one above!</p>
            ) : (
              notes.map((note) => (
                <div key={note.id} className={styles.note}>
                  <div className={styles.noteContent}>{note.content}</div>
                  <div className={styles.noteTime}>{formatTime(note.timestamp)}</div>
                  <button
                    className={styles.deleteButton}
                    onClick={() => deleteNote(note.id)}
                    aria-label="Delete note"
                  >
                    <Trash2 size={14} />
                  </button>
                </div>
              ))
            )}
          </div>
        </>
      )}

      {activeTab === 'todos' && (
        <>
          <div className={styles.inputRow}>
            <Input
              className={styles.input}
              placeholder="Add a todo..."
              value={newTodo}
              onChange={(e) => setNewTodo(e.target.value)}
              onKeyDown={(e) => e.key === 'Enter' && addTodo()}
              aria-label="Add a todo"
            />
            <Button
              className={styles.addButton}
              onClick={addTodo}
              icon={<Plus size={16} />}
              aria-label="Add todo"
            >
              Add
            </Button>
          </div>

          <div className={styles.list}>
            {todos.length === 0 ? (
              <p className={styles.emptyState}>No todos yet. Add one above!</p>
            ) : (
              todos.map((todo) => (
                <div key={todo.id} className={styles.todoItem}>
                  <input
                    type="checkbox"
                    className={styles.todoCheckbox}
                    checked={todo.completed}
                    onChange={() => toggleTodo(todo.id)}
                    aria-label={todo.content}
                  />
                  <span
                    className={`${styles.todoContent} ${todo.completed ? styles.completed : ''}`}
                  >
                    {todo.content}
                  </span>
                  <span className={`${styles.priority} ${styles[todo.priority]}`}>
                    {todo.priority}
                  </span>
                  <button
                    className={styles.deleteButton}
                    onClick={() => deleteTodo(todo.id)}
                    aria-label="Delete todo"
                  >
                    <Trash2 size={14} />
                  </button>
                </div>
              ))
            )}
          </div>
        </>
      )}
    </div>
  );
}
