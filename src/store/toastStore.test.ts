import { describe, it, expect, vi } from 'vitest';
import { act } from 'react';
import { useToastStore } from './toastStore';

describe('toastStore', () => {
  it('addToast creates toast and returns id', () => {
    act(() => useToastStore.setState({ toasts: [] }));
    const id = useToastStore.getState().addToast('hello', 'info', 0);
    expect(typeof id).toBe('string');
    expect(useToastStore.getState().toasts).toHaveLength(1);
    expect(useToastStore.getState().toasts[0].message).toBe('hello');
    expect(useToastStore.getState().toasts[0].type).toBe('info');
  });

  it('addToast auto-removes after duration', () => {
    vi.useFakeTimers();
    act(() => useToastStore.setState({ toasts: [] }));
    useToastStore.getState().addToast('auto', 'info', 500);
    expect(useToastStore.getState().toasts).toHaveLength(1);
    vi.advanceTimersByTime(500);
    expect(useToastStore.getState().toasts).toHaveLength(0);
    vi.useRealTimers();
  });

  it('addToast with duration 0 does not auto-remove', () => {
    vi.useFakeTimers();
    act(() => useToastStore.setState({ toasts: [] }));
    useToastStore.getState().addToast('persistent', 'info', 0);
    vi.advanceTimersByTime(99999);
    expect(useToastStore.getState().toasts).toHaveLength(1);
    vi.useRealTimers();
  });

  it('removeToast removes by id', () => {
    act(() => useToastStore.setState({ toasts: [] }));
    const id = useToastStore.getState().addToast('x', 'info', 0);
    useToastStore.getState().removeToast(id);
    expect(useToastStore.getState().toasts).toHaveLength(0);
  });

  it('removeToast non-existent id is a no-op', () => {
    act(() => useToastStore.setState({ toasts: [] }));
    useToastStore.getState().removeToast('nope');
    expect(useToastStore.getState().toasts).toHaveLength(0);
  });

  it('success helper', () => {
    act(() => useToastStore.setState({ toasts: [] }));
    useToastStore.getState().success('done');
    expect(useToastStore.getState().toasts[0].type).toBe('success');
    expect(useToastStore.getState().toasts[0].message).toBe('done');
  });

  it('error helper defaults to 5000ms', () => {
    act(() => useToastStore.setState({ toasts: [] }));
    useToastStore.getState().error('fail');
    expect(useToastStore.getState().toasts[0].type).toBe('error');
    expect(useToastStore.getState().toasts[0].duration).toBe(5000);
  });

  it('info helper', () => {
    act(() => useToastStore.setState({ toasts: [] }));
    useToastStore.getState().info('info msg');
    expect(useToastStore.getState().toasts[0].type).toBe('info');
  });

  it('warning helper defaults to 4500ms', () => {
    act(() => useToastStore.setState({ toasts: [] }));
    useToastStore.getState().warning('warn');
    expect(useToastStore.getState().toasts[0].type).toBe('warning');
    expect(useToastStore.getState().toasts[0].duration).toBe(4500);
  });

  it('addToast with title', () => {
    act(() => useToastStore.setState({ toasts: [] }));
    useToastStore.getState().addToast('msg', 'success', 0, 'Title');
    expect(useToastStore.getState().toasts[0].title).toBe('Title');
  });
});
