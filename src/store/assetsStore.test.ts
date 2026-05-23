import { describe, it, expect, vi } from 'vitest';
import { act } from 'react';
import { useAssetsStore } from './assetsStore';
import type { AssetFile, AssetFolder } from './assetsStore';
import type { AssetsState } from './assetsStore';

const initialState: AssetsState = {
  files: [],
  folders: [],
  contentPaths: [],
  filter: 'all',
  search: '',
  isLoading: false,
  isScanning: false,
  scanProgress: null,
  selectedFile: null,
  favouritePaths: new Set<string>(),
  loadingAsset: null,
  error: null,
  sortBy: 'none',
  sortDirection: 'asc',
  advancedFilters: { fileTypes: null, dateRange: null, sizeRange: null },
};

describe('assetsStore', () => {
  it('setFiles', () => {
    act(() => useAssetsStore.setState(initialState));
    const files: AssetFile[] = [
      { id: '1', name: 'a', path: '/a', type: 'figure', size: 100, modified: 0 },
    ];
    useAssetsStore.getState().setFiles(files);
    expect(useAssetsStore.getState().files).toHaveLength(1);
  });

  it('setFolders', () => {
    act(() => useAssetsStore.setState(initialState));
    const folders: AssetFolder[] = [{ id: '1', name: 'root', path: '/', children: [] }];
    useAssetsStore.getState().setFolders(folders);
    expect(useAssetsStore.getState().folders).toHaveLength(1);
  });

  it('setContentPaths', () => {
    act(() => useAssetsStore.setState(initialState));
    useAssetsStore
      .getState()
      .setContentPaths([{ id: '1', name: 'Content', path: '/content', enabled: true }]);
    expect(useAssetsStore.getState().contentPaths).toHaveLength(1);
  });

  it('setFilter', () => {
    act(() => useAssetsStore.setState(initialState));
    useAssetsStore.getState().setFilter('figures');
    expect(useAssetsStore.getState().filter).toBe('figures');
  });

  it('setSearch', () => {
    act(() => useAssetsStore.setState(initialState));
    useAssetsStore.getState().setSearch('test');
    expect(useAssetsStore.getState().search).toBe('test');
  });

  it('setLoading', () => {
    act(() => useAssetsStore.setState(initialState));
    useAssetsStore.getState().setLoading(true);
    expect(useAssetsStore.getState().isLoading).toBe(true);
  });

  it('setScanning', () => {
    act(() => useAssetsStore.setState(initialState));
    useAssetsStore.getState().setScanning(true);
    expect(useAssetsStore.getState().isScanning).toBe(true);
  });

  it('setScanProgress', () => {
    act(() => useAssetsStore.setState(initialState));
    useAssetsStore.getState().setScanProgress({ total: 100, current: 50, phase: 'Scanning' });
    expect(useAssetsStore.getState().scanProgress?.total).toBe(100);
  });

  it('setSelectedFile', () => {
    act(() => useAssetsStore.setState(initialState));
    const file: AssetFile = {
      id: '1',
      name: 'a',
      path: '/a',
      type: 'figure',
      size: 100,
      modified: 0,
    };
    useAssetsStore.getState().setSelectedFile(file);
    expect(useAssetsStore.getState().selectedFile?.id).toBe('1');
    useAssetsStore.getState().setSelectedFile(null);
    expect(useAssetsStore.getState().selectedFile).toBeNull();
  });

  it('setError', () => {
    act(() => useAssetsStore.setState(initialState));
    useAssetsStore.getState().setError('err');
    expect(useAssetsStore.getState().error).toBe('err');
  });

  it('setSortBy', () => {
    act(() => useAssetsStore.setState(initialState));
    useAssetsStore.getState().setSortBy('name');
    expect(useAssetsStore.getState().sortBy).toBe('name');
  });

  it('setSortDirection', () => {
    act(() => useAssetsStore.setState(initialState));
    useAssetsStore.getState().setSortDirection('desc');
    expect(useAssetsStore.getState().sortDirection).toBe('desc');
  });

  it('setAdvancedFilters', () => {
    act(() => useAssetsStore.setState(initialState));
    useAssetsStore
      .getState()
      .setAdvancedFilters({ fileTypes: ['duf'], dateRange: null, sizeRange: null });
    expect(useAssetsStore.getState().advancedFilters.fileTypes).toEqual(['duf']);
  });

  it('toggleContentPath toggles enabled', () => {
    act(() => useAssetsStore.setState(initialState));
    useAssetsStore.getState().setContentPaths([{ id: '1', name: 'C', path: '/c', enabled: true }]);
    useAssetsStore.getState().toggleContentPath('1');
    expect(useAssetsStore.getState().contentPaths[0].enabled).toBe(false);
    useAssetsStore.getState().toggleContentPath('1');
    expect(useAssetsStore.getState().contentPaths[0].enabled).toBe(true);
  });

  it('loadContentPaths loads from invoke', async () => {
    act(() => useAssetsStore.setState(initialState));
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockResolvedValue([
      { id: '1', name: 'Content', path: '/content', is_default: true },
    ]);
    await useAssetsStore.getState().loadContentPaths();
    expect(useAssetsStore.getState().contentPaths).toHaveLength(1);
    expect(useAssetsStore.getState().contentPaths[0].id).toBe('1');
    expect(useAssetsStore.getState().isLoading).toBe(false);
  });

  it('loadContentPaths handles error', async () => {
    act(() => useAssetsStore.setState(initialState));
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockRejectedValue(new Error('fail'));
    await useAssetsStore.getState().loadContentPaths();
    expect(useAssetsStore.getState().error).toBe('Error: fail');
    expect(useAssetsStore.getState().isLoading).toBe(false);
  });

  it('addCustomPath adds and reloads', async () => {
    act(() => useAssetsStore.setState(initialState));
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockReset();
    vi.mocked(invoke).mockResolvedValueOnce(undefined);
    vi.mocked(invoke).mockResolvedValueOnce([]);
    await useAssetsStore.getState().addCustomPath('/custom', 'Custom');
    expect(useAssetsStore.getState().isLoading).toBe(false);
  });

  it('addCustomPath handles error', async () => {
    act(() => useAssetsStore.setState(initialState));
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockRejectedValue(new Error('add fail'));
    await useAssetsStore.getState().addCustomPath('/bad', 'Bad');
    expect(useAssetsStore.getState().error).toBe('Error: add fail');
  });

  it('removeCustomPath removes and reloads', async () => {
    act(() => useAssetsStore.setState(initialState));
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockReset();
    vi.mocked(invoke).mockResolvedValueOnce(undefined);
    vi.mocked(invoke).mockResolvedValueOnce([]);
    await useAssetsStore.getState().removeCustomPath('1');
    expect(useAssetsStore.getState().isLoading).toBe(false);
  });

  it('removeCustomPath handles error', async () => {
    act(() => useAssetsStore.setState(initialState));
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockRejectedValue(new Error('remove fail'));
    await useAssetsStore.getState().removeCustomPath('1');
    expect(useAssetsStore.getState().error).toBe('Error: remove fail');
  });

  it('scanLibrary scans enabled content paths', async () => {
    act(() => useAssetsStore.setState(initialState));
    useAssetsStore.getState().setContentPaths([{ id: '1', name: 'C', path: '/c', enabled: true }]);
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockResolvedValue({
      total_files: 1,
      categorized: {
        figures: [
          { path: '/c/f.duf', name: 'F', file_type: 'duf', size: 100, category: 'figures' },
        ],
      },
    });
    await useAssetsStore.getState().scanLibrary();
    expect(useAssetsStore.getState().files).toHaveLength(1);
    expect(useAssetsStore.getState().isScanning).toBe(false);
    expect(useAssetsStore.getState().scanProgress).toBeNull();
  });

  it('scanLibrary handles error', async () => {
    act(() => useAssetsStore.setState(initialState));
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockRejectedValue(new Error('scan fail'));
    await useAssetsStore.getState().scanLibrary();
    expect(useAssetsStore.getState().error).toBe('Error: scan fail');
    expect(useAssetsStore.getState().isScanning).toBe(false);
  });

  it('searchAssets searches and sets files', async () => {
    act(() => useAssetsStore.setState(initialState));
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockResolvedValue([
      { path: '/r', name: 'Result', file_type: 'duf', size: 50, category: 'figures' },
    ]);
    await useAssetsStore.getState().searchAssets('query', 'figures');
    expect(useAssetsStore.getState().files).toHaveLength(1);
    expect(useAssetsStore.getState().isLoading).toBe(false);
  });

  it('searchAssets handles error', async () => {
    act(() => useAssetsStore.setState(initialState));
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockRejectedValue(new Error('search fail'));
    await useAssetsStore.getState().searchAssets('q', 'all');
    expect(useAssetsStore.getState().error).toBe('Error: search fail');
  });

  it('selectFile sets selected file', () => {
    act(() => useAssetsStore.setState(initialState));
    const file: AssetFile = {
      id: '1',
      name: 'a',
      path: '/a',
      type: 'figure',
      size: 100,
      modified: 0,
    };
    useAssetsStore.getState().selectFile(file);
    expect(useAssetsStore.getState().selectedFile?.id).toBe('1');
  });

  it('loadAssetInDaz loads asset', async () => {
    act(() => useAssetsStore.setState(initialState));
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockResolvedValue('loaded');
    const result = await useAssetsStore.getState().loadAssetInDaz('/a.duf');
    expect(result).toBe('loaded');
    expect(useAssetsStore.getState().loadingAsset).toBeNull();
  });

  it('loadAssetInDaz returns null on error', async () => {
    act(() => useAssetsStore.setState(initialState));
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockRejectedValue(new Error('load fail'));
    const result = await useAssetsStore.getState().loadAssetInDaz('/a.duf');
    expect(result).toBeNull();
    expect(useAssetsStore.getState().error).toBe('Error: load fail');
  });

  it('toggleFavourite adds to set', async () => {
    act(() => useAssetsStore.setState(initialState));
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockResolvedValue(true);
    await useAssetsStore.getState().toggleFavourite('/a');
    expect(useAssetsStore.getState().favouritePaths.has('/a')).toBe(true);
  });

  it('toggleFavourite removes from set', async () => {
    act(() => useAssetsStore.setState({ ...initialState, favouritePaths: new Set(['/a']) }));
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockResolvedValue(false);
    await useAssetsStore.getState().toggleFavourite('/a');
    expect(useAssetsStore.getState().favouritePaths.has('/a')).toBe(false);
  });

  it('toggleFavourite sets error on failure', async () => {
    act(() => useAssetsStore.setState(initialState));
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockRejectedValue(new Error('fav fail'));
    await useAssetsStore.getState().toggleFavourite('/a');
    expect(useAssetsStore.getState().error).toBe('Error: fav fail');
  });

  it('getFavourites loads favourites', async () => {
    act(() => useAssetsStore.setState(initialState));
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockResolvedValue(['/a', '/b']);
    await useAssetsStore.getState().getFavourites();
    expect(useAssetsStore.getState().favouritePaths.has('/a')).toBe(true);
    expect(useAssetsStore.getState().favouritePaths.has('/b')).toBe(true);
  });

  it('importModel imports and returns asset id', async () => {
    act(() => useAssetsStore.setState(initialState));
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockResolvedValue({ success: true, message: 'ok', asset_id: 'aid' });
    const result = await useAssetsStore.getState().importModel('/m.duf');
    expect(result).toBe('aid');
    expect(useAssetsStore.getState().isLoading).toBe(false);
  });

  it('importModel returns error message on failure result', async () => {
    act(() => useAssetsStore.setState(initialState));
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockResolvedValue({ success: false, message: 'import failed' });
    const result = await useAssetsStore.getState().importModel('/m.duf');
    expect(result).toBe('import failed');
  });

  it('importModel handles invoke error', async () => {
    act(() => useAssetsStore.setState(initialState));
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockRejectedValue(new Error('fail'));
    const result = await useAssetsStore.getState().importModel('/m.duf');
    expect(result).toBeNull();
  });

  it('exportScene exports and returns file path', async () => {
    act(() => useAssetsStore.setState(initialState));
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockResolvedValue({ success: true, message: 'ok', file_path: '/out.dsf' });
    const result = await useAssetsStore.getState().exportScene('node1', '/out.dsf');
    expect(result).toBe('/out.dsf');
    expect(useAssetsStore.getState().isLoading).toBe(false);
  });

  it('exportScene returns null on error', async () => {
    act(() => useAssetsStore.setState(initialState));
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockRejectedValue(new Error('fail'));
    const result = await useAssetsStore.getState().exportScene('n1', '/o');
    expect(result).toBeNull();
  });

  it('reset restores initial state', () => {
    act(() => useAssetsStore.setState(initialState));
    useAssetsStore.getState().setFilter('figures');
    useAssetsStore.getState().reset();
    expect(useAssetsStore.getState().filter).toBe('all');
  });
});
