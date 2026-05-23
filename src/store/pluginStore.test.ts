import { describe, it, expect, vi, beforeEach } from 'vitest';
import { act } from 'react';
import { usePluginStore } from './pluginStore';
import { invoke } from '@tauri-apps/api/core';
import { useToastStore } from './toastStore';

const initialState = {
  status: 'checking' as const,
  customPath: '',
  downloadProgress: 0,
  downloadedBytes: 0,
  error: null,
};

describe('pluginStore', () => {
  beforeEach(() => {
    localStorage.clear();
    vi.mocked(invoke).mockReset();
    act(() => useToastStore.setState({ toasts: [] }));
  });

  it('setStatus updates status', () => {
    act(() => usePluginStore.setState(initialState));
    usePluginStore.getState().setStatus('installed');
    expect(usePluginStore.getState().status).toBe('installed');
  });

  it('setCustomPath saves to localStorage and checks status', () => {
    act(() => usePluginStore.setState(initialState));
    vi.mocked(invoke).mockResolvedValue('not_installed');
    usePluginStore.getState().setCustomPath('/custom/path');
    expect(localStorage.getItem('dazpilot_plugin_custom_path')).toBe('/custom/path');
    expect(usePluginStore.getState().customPath).toBe('/custom/path');
  });

  it('checkPluginStatus sets status from invoke', async () => {
    act(() => usePluginStore.setState(initialState));
    vi.mocked(invoke).mockResolvedValue('installed');
    await usePluginStore.getState().checkPluginStatus();
    expect(usePluginStore.getState().status).toBe('installed');
  });

  it('checkPluginStatus sets error on failure', async () => {
    act(() => usePluginStore.setState(initialState));
    vi.mocked(invoke).mockRejectedValue(new Error('check fail'));
    await usePluginStore.getState().checkPluginStatus();
    expect(usePluginStore.getState().status).toBe('error');
    expect(usePluginStore.getState().error).toBe('Error: check fail');
  });

  it('browseCustomPath sets path and calls setCustomPath', async () => {
    act(() => usePluginStore.setState(initialState));
    vi.mocked(invoke).mockReset();
    vi.mocked(invoke).mockResolvedValueOnce('/selected/path');
    vi.mocked(invoke).mockResolvedValueOnce('not_installed');
    await usePluginStore.getState().browseCustomPath();
    expect(usePluginStore.getState().customPath).toBe('/selected/path');
  });

  it('browseCustomPath handles null path', async () => {
    act(() => usePluginStore.setState(initialState));
    vi.mocked(invoke).mockResolvedValue(null);
    await usePluginStore.getState().browseCustomPath();
    expect(usePluginStore.getState().customPath).toBe('');
  });

  it('browseCustomPath sets error on failure', async () => {
    act(() => usePluginStore.setState(initialState));
    vi.mocked(invoke).mockRejectedValue(new Error('browse fail'));
    await usePluginStore.getState().browseCustomPath();
    expect(usePluginStore.getState().error).toBe('Error: browse fail');
  });

  it('downloadAndInstall succeeds', async () => {
    act(() => usePluginStore.setState(initialState));
    vi.mocked(invoke).mockResolvedValue(undefined);
    await usePluginStore.getState().downloadAndInstall();
    expect(usePluginStore.getState().status).toBe('installed');
    expect(usePluginStore.getState().downloadProgress).toBe(100);
  });

  it('downloadAndInstall sets error on failure', async () => {
    act(() => usePluginStore.setState(initialState));
    vi.mocked(invoke).mockRejectedValue(new Error('dl fail'));
    await usePluginStore.getState().downloadAndInstall();
    expect(usePluginStore.getState().status).toBe('error');
    expect(usePluginStore.getState().error).toBe('Error: dl fail');
  });

  it('installLocal succeeds', async () => {
    act(() => usePluginStore.setState(initialState));
    vi.mocked(invoke).mockResolvedValue(undefined);
    await usePluginStore.getState().installLocal();
    expect(usePluginStore.getState().status).toBe('installed');
  });

  it('installLocal sets error on failure', async () => {
    act(() => usePluginStore.setState(initialState));
    vi.mocked(invoke).mockRejectedValue(new Error('install fail'));
    await usePluginStore.getState().installLocal();
    expect(usePluginStore.getState().status).toBe('error');
    expect(usePluginStore.getState().error).toBe('Error: install fail');
  });

  it('setError', () => {
    act(() => usePluginStore.setState(initialState));
    usePluginStore.getState().setError('custom error');
    expect(usePluginStore.getState().error).toBe('custom error');
  });

  it('resetProgress zeros progress', () => {
    act(() =>
      usePluginStore.setState({ ...initialState, downloadProgress: 50, downloadedBytes: 100 })
    );
    usePluginStore.getState().resetProgress();
    expect(usePluginStore.getState().downloadProgress).toBe(0);
    expect(usePluginStore.getState().downloadedBytes).toBe(0);
  });
});
