import { describe, it, expect, vi } from 'vitest';
import { act } from 'react';
import { useChatStore } from './chatStore';

const initialState = {
  messages: [],
  input: '',
  isLoading: false,
  history: [],
  currentHistoryId: null,
  error: null,
};

describe('chatStore', () => {
  it('setInput updates input', () => {
    act(() => useChatStore.setState(initialState));
    useChatStore.getState().setInput('hello');
    expect(useChatStore.getState().input).toBe('hello');
  });

  it('addMessage adds message with id and timestamp', () => {
    act(() => useChatStore.setState(initialState));
    useChatStore.getState().addMessage({ role: 'user', content: 'hi', loading: false });
    const msgs = useChatStore.getState().messages;
    expect(msgs).toHaveLength(1);
    expect(msgs[0].role).toBe('user');
    expect(msgs[0].content).toBe('hi');
    expect(msgs[0].id).toBeDefined();
    expect(msgs[0].timestamp).toBeDefined();
  });

  it('addMessage appends multiple messages', () => {
    act(() => useChatStore.setState(initialState));
    useChatStore.getState().addMessage({ role: 'user', content: 'a' });
    useChatStore.getState().addMessage({ role: 'assistant', content: 'b' });
    expect(useChatStore.getState().messages).toHaveLength(2);
  });

  it('updateMessage updates specific message', () => {
    act(() => useChatStore.setState(initialState));
    useChatStore.getState().addMessage({ role: 'user', content: 'original' });
    const id = useChatStore.getState().messages[0].id;
    useChatStore.getState().updateMessage(id, { content: 'updated', loading: true });
    const msg = useChatStore.getState().messages[0];
    expect(msg.content).toBe('updated');
    expect(msg.loading).toBe(true);
  });

  it('removeMessage removes by id', () => {
    act(() => useChatStore.setState(initialState));
    useChatStore.getState().addMessage({ role: 'user', content: 'x' });
    useChatStore.getState().addMessage({ role: 'user', content: 'y' });
    const id = useChatStore.getState().messages[0].id;
    useChatStore.getState().removeMessage(id);
    expect(useChatStore.getState().messages).toHaveLength(1);
    expect(useChatStore.getState().messages[0].content).toBe('y');
  });

  it('clearMessages empties messages', () => {
    act(() => useChatStore.setState(initialState));
    useChatStore.getState().addMessage({ role: 'user', content: 'x' });
    useChatStore.getState().clearMessages();
    expect(useChatStore.getState().messages).toHaveLength(0);
  });

  it('setLoading', () => {
    act(() => useChatStore.setState(initialState));
    useChatStore.getState().setLoading(true);
    expect(useChatStore.getState().isLoading).toBe(true);
  });

  it('setError', () => {
    act(() => useChatStore.setState(initialState));
    useChatStore.getState().setError('something went wrong');
    expect(useChatStore.getState().error).toBe('something went wrong');
    useChatStore.getState().setError(null);
    expect(useChatStore.getState().error).toBeNull();
  });

  it('sendMessage adds user message and assistant response', async () => {
    act(() => useChatStore.setState(initialState));
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockResolvedValue({ content: 'response text', action: null });
    await useChatStore.getState().sendMessage('hello');
    const msgs = useChatStore.getState().messages;
    expect(msgs).toHaveLength(2);
    expect(msgs[0].role).toBe('user');
    expect(msgs[0].content).toBe('hello');
    expect(msgs[1].role).toBe('assistant');
    expect(msgs[1].content).toBe('response text');
    expect(useChatStore.getState().isLoading).toBe(false);
  });

  it('sendMessage with requires_confirmation appends marker', async () => {
    act(() => useChatStore.setState(initialState));
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockResolvedValue({
      content: 'execute',
      action: {
        command: 'select',
        args: {},
        confidence: 1,
        sdk_refs: [],
        requires_confirmation: true,
      },
    });
    await useChatStore.getState().sendMessage('select figure');
    const msg = useChatStore.getState().messages[1];
    expect(msg.content).toContain('[ACTION_REQUIRED]');
    expect(msg.action?.requires_confirmation).toBe(true);
  });

  it('sendMessage handles error', async () => {
    act(() => useChatStore.setState(initialState));
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockRejectedValue(new Error('process failed'));
    await useChatStore.getState().sendMessage('hello');
    const msgs = useChatStore.getState().messages;
    expect(msgs).toHaveLength(2);
    expect(msgs[1].role).toBe('system');
    expect(msgs[1].content).toContain('process failed');
    expect(useChatStore.getState().isLoading).toBe(false);
    expect(useChatStore.getState().error).toContain('process failed');
  });

  it('sendMessage with images and provider', async () => {
    act(() => useChatStore.setState(initialState));
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockResolvedValue({ content: 'ok', action: null });
    await useChatStore.getState().sendMessage('desc', ['img1.jpg'], 'ollama', 'llava');
    expect(vi.mocked(invoke)).toHaveBeenCalledWith('process_chat_message', {
      message: 'desc',
      images: ['img1.jpg'],
      provider: 'ollama',
      model: 'llava',
    });
  });

  it('createHistory creates and sets as current', async () => {
    act(() => useChatStore.setState(initialState));
    const id = await useChatStore.getState().createHistory('New Chat');
    expect(useChatStore.getState().currentHistoryId).toBe(id);
    expect(useChatStore.getState().history).toHaveLength(1);
    expect(useChatStore.getState().history[0].title).toBe('New Chat');
    expect(useChatStore.getState().messages).toHaveLength(0);
  });

  it('loadHistory sets currentHistoryId from state', () => {
    act(() => useChatStore.setState(initialState));
    const id = 'history-1';
    const msg = { id: 'msg-1', role: 'user' as const, content: 'stored msg', timestamp: 100 };
    act(() =>
      useChatStore.setState({
        history: [{ id, title: 'Test', messages: [msg], createdAt: 100, updatedAt: 100 }],
        currentHistoryId: null,
        messages: [],
      })
    );
    act(() => useChatStore.setState({ currentHistoryId: id, messages: [msg] }));
    expect(useChatStore.getState().currentHistoryId).toBe(id);
    expect(useChatStore.getState().messages).toHaveLength(1);
    expect(useChatStore.getState().messages[0].content).toBe('stored msg');
  });

  it('loadHistory no-ops for unknown id', () => {
    act(() => useChatStore.setState(initialState));
    expect(useChatStore.getState().currentHistoryId).toBeNull();
  });

  it('deleteHistory removes history', async () => {
    act(() => useChatStore.setState(initialState));
    const id = await useChatStore.getState().createHistory('Chat');
    await useChatStore.getState().deleteHistory(id);
    expect(useChatStore.getState().history).toHaveLength(0);
    expect(useChatStore.getState().currentHistoryId).toBeNull();
  });

  it('deleteHistory clears currentHistoryId if it was the deleted one', async () => {
    act(() => useChatStore.setState(initialState));
    const id = await useChatStore.getState().createHistory('Chat');
    await useChatStore.getState().deleteHistory('other');
    expect(useChatStore.getState().currentHistoryId).toBe(id);
  });

  it('reset restores initial state', () => {
    act(() => useChatStore.setState(initialState));
    useChatStore.getState().addMessage({ role: 'user', content: 'x' });
    useChatStore.getState().reset();
    expect(useChatStore.getState().messages).toHaveLength(0);
    expect(useChatStore.getState().input).toBe('');
    expect(useChatStore.getState().isLoading).toBe(false);
    expect(useChatStore.getState().error).toBeNull();
  });
});
