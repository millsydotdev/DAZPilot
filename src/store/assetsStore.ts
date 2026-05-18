import { create } from 'zustand';

export interface AssetFile {
  id: string;
  name: string;
  path: string;
  type: string;
  size: number;
  modified: number;
  isFavourite?: boolean;
  metadata?: Record<string, unknown>;
}

export interface AssetFolder {
  id: string;
  name: string;
  path: string;
  children: (AssetFolder | string)[];
}

export interface ContentPath {
  id: string;
  name: string;
  path: string;
  enabled: boolean;
}

export interface ScanProgress {
  total: number;
  current: number;
  phase: string;
}

export type AssetFilter =
  | 'all'
  | 'figures'
  | 'poses'
  | 'morphs'
  | 'textures'
  | 'clothing'
  | 'hair'
  | 'materials'
  | 'environments'
  | 'lights'
  | 'cameras'
  | 'animations';

// Shape returned by Rust library_scanner::AssetInfo (via serde)
interface RustAssetInfo {
  path: string;
  name: string;
  file_type: string;
  size: number;
  category: string;
  subcategory?: string;
}

function rustAssetToFile(a: RustAssetInfo, favouritePaths: Set<string>): AssetFile {
  return {
    id: a.path,
    name: a.name,
    path: a.path,
    type: a.category || 'other',
    size: a.size,
    modified: 0,
    isFavourite: favouritePaths.has(a.path),
  };
}

export interface AssetsState {
  files: AssetFile[];
  folders: AssetFolder[];
  contentPaths: ContentPath[];
  filter: AssetFilter;
  search: string;
  isLoading: boolean;
  isScanning: boolean;
  scanProgress: ScanProgress | null;
  selectedFile: AssetFile | null;
  favouritePaths: Set<string>;
  loadingAsset: string | null;
  error: string | null;
}

export interface AssetsActions {
  setFiles: (files: AssetFile[]) => void;
  setFolders: (folders: AssetFolder[]) => void;
  setContentPaths: (paths: ContentPath[]) => void;
  toggleContentPath: (id: string) => void;
  setFilter: (filter: AssetFilter) => void;
  setSearch: (search: string) => void;
  setLoading: (loading: boolean) => void;
  setScanning: (scanning: boolean) => void;
  setScanProgress: (progress: ScanProgress | null) => void;
  setSelectedFile: (file: AssetFile | null) => void;
  setError: (error: string | null) => void;
  loadContentPaths: () => Promise<void>;
  scanLibrary: () => Promise<void>;
  searchAssets: (query: string, category: string) => Promise<void>;
  selectFile: (file: AssetFile) => void;
  loadAssetInDaz: (path: string) => Promise<string | null>;
  toggleFavourite: (path: string) => Promise<void>;
  getFavourites: () => Promise<void>;
  importModel: (path: string) => Promise<string | null>;
  exportScene: (nodeId: string, path: string) => Promise<string | null>;
  reset: () => void;
}

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
  favouritePaths: new Set(),
  loadingAsset: null,
  error: null,
};

export const useAssetsStore = create<AssetsState & AssetsActions>((set, get) => ({
  ...initialState,

  setFiles: (files) => set({ files }),
  setFolders: (folders) => set({ folders }),
  setContentPaths: (contentPaths) => set({ contentPaths }),
  setFilter: (filter) => set({ filter }),
  setSearch: (search) => set({ search }),
  setLoading: (isLoading) => set({ isLoading }),
  setScanning: (isScanning) => set({ isScanning }),
  setScanProgress: (scanProgress) => set({ scanProgress }),
  setSelectedFile: (selectedFile) => set({ selectedFile }),
  setError: (error) => set({ error }),

  toggleContentPath: (id) =>
    set((state) => ({
      contentPaths: state.contentPaths.map((p) =>
        p.id === id ? { ...p, enabled: !p.enabled } : p
      ),
    })),

  loadContentPaths: async () => {
    set({ isLoading: true, error: null });
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      const paths = await invoke<ContentPath[]>('get_content_paths');
      set({ contentPaths: paths });
    } catch (e) {
      set({ error: String(e) });
    } finally {
      set({ isLoading: false });
    }
  },

  scanLibrary: async () => {
    const { contentPaths, favouritePaths } = get();
    const enabledPaths = contentPaths.filter((p) => p.enabled).map((p) => p.path);
    set({
      isScanning: true,
      scanProgress: { total: 0, current: 0, phase: 'Scanning…' },
      error: null,
    });
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      const result = await invoke<{
        total_files: number;
        categorized: Record<string, RustAssetInfo[]>;
      }>('scan_library', { paths: enabledPaths });
      const allAssets: AssetFile[] = Object.values(result.categorized)
        .flat()
        .map((a) => rustAssetToFile(a, favouritePaths));
      set({ files: allAssets });
    } catch (e) {
      set({ error: String(e) });
    } finally {
      set({ isScanning: false, scanProgress: null });
    }
  },

  searchAssets: async (query: string, category: string) => {
    set({ isLoading: true, error: null });
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      const assets = await invoke<RustAssetInfo[]>('search_assets', { query, category });
      const { favouritePaths } = get();
      const files = assets.map((a) => rustAssetToFile(a, favouritePaths));
      set({ files });
    } catch (e) {
      set({ error: String(e) });
    } finally {
      set({ isLoading: false });
    }
  },

  selectFile: (file) => set({ selectedFile: file }),

  loadAssetInDaz: async (path: string) => {
    set({ loadingAsset: path, error: null });
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      return await invoke<string>('load_asset_in_daz', { path });
    } catch (e) {
      set({ error: String(e) });
      return null;
    } finally {
      set({ loadingAsset: null });
    }
  },

  toggleFavourite: async (path: string) => {
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      const isNowFav = await invoke<boolean>('toggle_favourite', { assetPath: path });
      set((state) => {
        const next = new Set(state.favouritePaths);
        if (isNowFav) next.add(path);
        else next.delete(path);
        return {
          favouritePaths: next,
          files: state.files.map((f) => (f.path === path ? { ...f, isFavourite: isNowFav } : f)),
        };
      });
    } catch (e) {
      set({ error: String(e) });
    }
  },

  getFavourites: async () => {
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      const paths = await invoke<string[]>('get_favourites');
      set({ favouritePaths: new Set(paths) });
    } catch {
      /* non-fatal */
    }
  },

  importModel: async (path: string) => {
    set({ isLoading: true, error: null });
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      const result = await invoke<{ success: boolean; message: string; asset_id?: string }>(
        'import_model',
        { path, settings: {} }
      );
      return result.success ? result.asset_id || null : result.message;
    } catch (e) {
      set({ error: String(e) });
      return null;
    } finally {
      set({ isLoading: false });
    }
  },

  exportScene: async (nodeId: string, path: string) => {
    set({ isLoading: true, error: null });
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      const result = await invoke<{ success: boolean; message: string; file_path?: string }>(
        'export_scene',
        { nodeId, path, settings: {} }
      );
      return result.success ? result.file_path || null : result.message;
    } catch (e) {
      set({ error: String(e) });
      return null;
    } finally {
      set({ isLoading: false });
    }
  },

  reset: () => set(initialState),
}));
