import { describe, it, expect, vi } from 'vitest';
import { act } from 'react';
import { useLogStore } from './logStore';

describe('logStore', () => {
  it('addLog appends entry', () => {
    act(() => useLogStore.setState({ logs: [], maxLogs: 1000, autoScroll: true } as never));
    useLogStore.getState().addLog('info', 'system', 'test msg');
    const logs = useLogStore.getState().logs;
    expect(logs).toHaveLength(1);
    expect(logs[0].level).toBe('info');
    expect(logs[0].category).toBe('system');
    expect(logs[0].message).toBe('test msg');
  });

  it('addLog respects maxLogs limit', () => {
    act(() => useLogStore.setState({ logs: [], maxLogs: 3, autoScroll: true } as never));
    useLogStore.getState().addLog('info', 'system', 'a');
    useLogStore.getState().addLog('info', 'system', 'b');
    useLogStore.getState().addLog('info', 'system', 'c');
    useLogStore.getState().addLog('info', 'system', 'd');
    expect(useLogStore.getState().logs).toHaveLength(3);
    expect(useLogStore.getState().logs[0].message).toBe('b');
  });

  it('addLog uses passed category directly', () => {
    act(() => useLogStore.setState({ logs: [], maxLogs: 1000, autoScroll: true } as never));
    useLogStore.getState().addLog('error', 'viewport', 'custom msg');
    expect(useLogStore.getState().logs[0].category).toBe('viewport');
  });

  it('clearLogs replaces logs with clear marker', () => {
    act(() => useLogStore.setState({ logs: [], maxLogs: 1000, autoScroll: true } as never));
    useLogStore.getState().addLog('info', 'system', 'old');
    useLogStore.getState().clearLogs();
    const c = useLogStore.getState();
    expect(c.logs).toHaveLength(1);
    expect(c.logs[0].message).toContain('cleared');
    expect(c.logs[0].level).toBe('info');
  });

  it('setAutoScroll', () => {
    act(() => useLogStore.setState({ logs: [], maxLogs: 1000, autoScroll: true } as never));
    useLogStore.getState().setAutoScroll(false);
    expect(useLogStore.getState().autoScroll).toBe(false);
  });

  it('exportLogs creates blob and triggers download', () => {
    const createObjectURL = vi.fn(() => 'blob:url');
    const revokeObjectURL = vi.fn();
    URL.createObjectURL = createObjectURL;
    URL.revokeObjectURL = revokeObjectURL;

    const clickMock = vi.fn();
    const appendChild = vi.fn();
    const removeChild = vi.fn();
    document.body.appendChild = appendChild;
    document.body.removeChild = removeChild;
    document.createElement = vi.fn(() => ({
      href: '',
      download: '',
      click: clickMock,
    })) as never;

    act(() => useLogStore.setState({ logs: [], maxLogs: 1000, autoScroll: true } as never));
    useLogStore.getState().addLog('info', 'system', 'export test');
    useLogStore.getState().exportLogs();

    expect(createObjectURL).toHaveBeenCalled();
    expect(clickMock).toHaveBeenCalled();
    expect(revokeObjectURL).toHaveBeenCalled();
  });
});
