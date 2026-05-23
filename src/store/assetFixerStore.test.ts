import { describe, it, expect, vi } from 'vitest';
import { act } from 'react';
import { useAssetFixerStore } from './assetFixerStore';
import { invoke } from '@tauri-apps/api/core';

const initialState = {
  isScanning: false,
  lastScanResult: null,
  lastFixResult: null,
  isFixing: false,
  selectedConflict: null,
};

describe('assetFixerStore', () => {
  it('scanConflicts returns result and updates state', async () => {
    act(() => useAssetFixerStore.setState(initialState));
    const result = { total_scanned: 10, conflicts: [], warnings: [] };
    vi.mocked(invoke).mockResolvedValue(result);
    const res = await useAssetFixerStore.getState().scanConflicts('/test');
    expect(res).toEqual(result);
    expect(useAssetFixerStore.getState().lastScanResult).toEqual(result);
    expect(useAssetFixerStore.getState().isScanning).toBe(false);
  });

  it('scanConflicts throws on error', async () => {
    act(() => useAssetFixerStore.setState(initialState));
    vi.mocked(invoke).mockRejectedValue(new Error('scan fail'));
    await expect(useAssetFixerStore.getState().scanConflicts('/test')).rejects.toThrow('scan fail');
    expect(useAssetFixerStore.getState().isScanning).toBe(false);
  });

  it('fixShellZones returns result and updates state', async () => {
    act(() => useAssetFixerStore.setState(initialState));
    const result = { success: true, fixed_files: ['a'], errors: [] };
    vi.mocked(invoke).mockResolvedValue(result);
    const res = await useAssetFixerStore.getState().fixShellZones('/shell', 'prefix');
    expect(res).toEqual(result);
    expect(useAssetFixerStore.getState().lastFixResult).toEqual(result);
    expect(useAssetFixerStore.getState().isFixing).toBe(false);
  });

  it('fixShellZones throws on error', async () => {
    act(() => useAssetFixerStore.setState(initialState));
    vi.mocked(invoke).mockRejectedValue(new Error('fix fail'));
    await expect(useAssetFixerStore.getState().fixShellZones('/shell', 'pre')).rejects.toThrow(
      'fix fail'
    );
    expect(useAssetFixerStore.getState().isFixing).toBe(false);
  });

  it('autoFixAll returns result and updates state', async () => {
    act(() => useAssetFixerStore.setState(initialState));
    const result = { success: true, fixed_files: ['a'], errors: [] };
    vi.mocked(invoke).mockResolvedValue(result);
    const res = await useAssetFixerStore.getState().autoFixAll('/root', '/out');
    expect(res).toEqual(result);
    expect(useAssetFixerStore.getState().lastFixResult).toEqual(result);
    expect(useAssetFixerStore.getState().isFixing).toBe(false);
  });

  it('autoFixAll throws on error', async () => {
    act(() => useAssetFixerStore.setState(initialState));
    vi.mocked(invoke).mockRejectedValue(new Error('auto fail'));
    await expect(useAssetFixerStore.getState().autoFixAll('/root', '/out')).rejects.toThrow(
      'auto fail'
    );
    expect(useAssetFixerStore.getState().isFixing).toBe(false);
  });

  it('analyzeShell returns shell info', async () => {
    act(() => useAssetFixerStore.setState(initialState));
    const info = { path: '/p', shell_type: 'genesis8', material_zones: [], uv_sets: [] };
    vi.mocked(invoke).mockResolvedValue(info);
    const res = await useAssetFixerStore.getState().analyzeShell('/p');
    expect(res).toEqual(info);
  });

  it('analyzeShell returns null on error', async () => {
    act(() => useAssetFixerStore.setState(initialState));
    vi.mocked(invoke).mockRejectedValue(new Error('analyze fail'));
    await expect(useAssetFixerStore.getState().analyzeShell('/p')).rejects.toThrow('analyze fail');
  });

  it('setSelectedConflict', () => {
    act(() => useAssetFixerStore.setState(initialState));
    const conflict = { conflict_type: 'zone', name: 'test', files: ['a'], severity: 'high' };
    useAssetFixerStore.getState().setSelectedConflict(conflict);
    expect(useAssetFixerStore.getState().selectedConflict).toEqual(conflict);
    useAssetFixerStore.getState().setSelectedConflict(null);
    expect(useAssetFixerStore.getState().selectedConflict).toBeNull();
  });

  it('clearResults resets results', () => {
    act(() => useAssetFixerStore.setState(initialState));
    useAssetFixerStore.getState().clearResults();
    expect(useAssetFixerStore.getState().lastScanResult).toBeNull();
    expect(useAssetFixerStore.getState().lastFixResult).toBeNull();
    expect(useAssetFixerStore.getState().selectedConflict).toBeNull();
  });
});
