import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/core';

export interface AssetConflict {
  conflict_type: string;
  name: string;
  files: string[];
  severity: string;
}

export interface ConflictScanResult {
  total_scanned: number;
  conflicts: AssetConflict[];
  warnings: string[];
}

export interface AssetFixResult {
  success: boolean;
  fixed_files: string[];
  errors: string[];
}

export interface ShellInfo {
  path: string;
  shell_type: string;
  material_zones: string[];
  uv_sets: string[];
}

interface AssetFixerState {
  isScanning: boolean;
  lastScanResult: ConflictScanResult | null;
  lastFixResult: AssetFixResult | null;
  isFixing: boolean;
  selectedConflict: AssetConflict | null;
}

interface AssetFixerActions {
  scanConflicts: (rootPath: string) => Promise<ConflictScanResult>;
  fixShellZones: (shellPath: string, prefix: string) => Promise<AssetFixResult>;
  autoFixAll: (rootPath: string, outputDir: string) => Promise<AssetFixResult>;
  analyzeShell: (path: string) => Promise<ShellInfo | null>;
  setSelectedConflict: (conflict: AssetConflict | null) => void;
  clearResults: () => void;
}

export const useAssetFixerStore = create<AssetFixerState & AssetFixerActions>((set) => ({
  isScanning: false,
  lastScanResult: null,
  lastFixResult: null,
  isFixing: false,
  selectedConflict: null,

  scanConflicts: async (rootPath: string) => {
    set({ isScanning: true });
    try {
      const result = await invoke<ConflictScanResult>('scan_conflicts', { rootPath });
      set({ isScanning: false, lastScanResult: result });
      return result;
    } catch (e) {
      set({ isScanning: false });
      throw e;
    }
  },

  fixShellZones: async (shellPath: string, prefix: string) => {
    set({ isFixing: true });
    try {
      const result = await invoke<AssetFixResult>('fix_shell_zones', { shellPath, prefix });
      set({ isFixing: false, lastFixResult: result });
      return result;
    } catch (e) {
      set({ isFixing: false });
      throw e;
    }
  },

  autoFixAll: async (rootPath: string, outputDir: string) => {
    set({ isFixing: true });
    try {
      const result = await invoke<AssetFixResult>('auto_fix_all_conflicts', {
        rootPath,
        outputDir,
      });
      set({ isFixing: false, lastFixResult: result });
      return result;
    } catch (e) {
      set({ isFixing: false });
      throw e;
    }
  },

  analyzeShell: async (path: string) => {
    return await invoke<ShellInfo | null>('analyze_shell_file', { path });
  },

  setSelectedConflict: (conflict) => set({ selectedConflict: conflict }),

  clearResults: () => set({ lastScanResult: null, lastFixResult: null, selectedConflict: null }),
}));
