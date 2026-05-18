import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/core';
import { useToastStore } from './toastStore';

export interface PluginState {
  status: 'installed' | 'not_installed' | 'checking' | 'downloading' | 'error';
  customPath: string;
  downloadProgress: number;
  downloadedBytes: number;
  error: string | null;
}

export interface PluginActions {
  setStatus: (status: PluginState['status']) => void;
  setCustomPath: (path: string) => void;
  checkPluginStatus: () => Promise<void>;
  browseCustomPath: () => Promise<void>;
  downloadAndInstall: () => Promise<void>;
  installLocal: () => Promise<void>;
  setError: (err: string | null) => void;
  resetProgress: () => void;
}

const LOCAL_STORAGE_KEY = 'dazpilot_plugin_custom_path';

export const usePluginStore = create<PluginState & PluginActions>((set, get) => ({
  status: 'checking',
  customPath: localStorage.getItem(LOCAL_STORAGE_KEY) || '',
  downloadProgress: 0,
  downloadedBytes: 0,
  error: null,

  setStatus: (status) => set({ status }),

  setCustomPath: (customPath) => {
    localStorage.setItem(LOCAL_STORAGE_KEY, customPath);
    set({ customPath });
    get().checkPluginStatus();
  },

  checkPluginStatus: async () => {
    set({ status: 'checking', error: null });
    try {
      const path = get().customPath;
      const res = await invoke<string>('get_plugin_status', { customPath: path || null });
      set({ status: res as PluginState['status'] });
    } catch (e) {
      set({ status: 'error', error: String(e) });
    }
  },

  browseCustomPath: async () => {
    try {
      const path = await invoke<string | null>('select_plugins_directory');
      if (path) {
        get().setCustomPath(path);
      }
    } catch (e) {
      set({ error: String(e) });
    }
  },

  downloadAndInstall: async () => {
    const toast = useToastStore.getState();
    set({ status: 'downloading', downloadProgress: 0, downloadedBytes: 0, error: null });
    toast.info('Downloading bridge plugin from GitHub Releases...', 0, 'Plugin Download');
    try {
      const path = get().customPath;
      await invoke('download_and_install_plugin', { customPath: path || null });
      set({ status: 'installed', downloadProgress: 100 });
      toast.success('Bridge plugin installed successfully!', 5000, 'Plugin Installed');
    } catch (e) {
      set({ status: 'error', error: String(e) });
      toast.error(`Failed to download plugin: ${e}`, 8000, 'Download Failed');
    }
  },

  installLocal: async () => {
    const toast = useToastStore.getState();
    set({ status: 'checking', error: null });
    try {
      const path = get().customPath;
      await invoke('install_daz3d_plugin', { customPath: path || null });
      set({ status: 'installed' });
      toast.success('Local bridge plugin linked successfully!', 5000, 'Plugin Linked');
    } catch (e) {
      set({ status: 'error', error: String(e) });
      toast.error(`Failed to link local plugin: ${e}`, 8000, 'Link Failed');
    }
  },

  setError: (error) => set({ error }),
  resetProgress: () => set({ downloadProgress: 0, downloadedBytes: 0 }),
}));
