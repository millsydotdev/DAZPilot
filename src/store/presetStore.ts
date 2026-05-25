import { create } from 'zustand';
import {
  useSceneStore,
  type SceneFigure,
  type SceneProp,
  type SceneLight,
  type SceneCamera,
} from './sceneStore';

export interface ScenePreset {
  id: string;
  name: string;
  description: string;
  category: 'lighting' | 'camera' | 'figure' | 'scene' | 'custom';
  createdAt: number;
  updatedAt: number;
  thumbnail?: string;
  // Store the actual scene configuration data
  sceneData: {
    figures: SceneFigure[];
    props: SceneProp[];
    lights: SceneLight[];
    cameras: SceneCamera[];
    activeCamera: string | null;
    selectedItem: string | null;
  };
}

export interface PresetState {
  presets: ScenePreset[];
  selectedPreset: ScenePreset | null;
  isSaving: boolean;
  isLoading: boolean;
  loaded: boolean;
  error: string | null;
}

export interface PresetActions {
  // Preset management
  setPresets: (presets: ScenePreset[]) => void;
  addPreset: (preset: ScenePreset) => void;
  updatePreset: (id: string, updates: Partial<ScenePreset>) => void;
  removePreset: (id: string) => void;
  setSelectedPreset: (preset: ScenePreset | null) => void;

  // Loading and saving
  setIsSaving: (isSaving: boolean) => void;
  setIsLoading: (isLoading: boolean) => void;
  setError: (error: string | null) => void;

  // Operations
  loadPersistedPresets: () => Promise<void>;
  saveCurrentSceneAsPreset: (name: string, description: string, category: string) => Promise<void>;
  loadPreset: (id: string) => Promise<void>;
  deletePreset: (id: string) => Promise<void>;

  // Utility
  getPresetsByCategory: (category: string) => ScenePreset[];
  reset: () => void;
}

const initialState: PresetState = {
  presets: [],
  selectedPreset: null,
  isSaving: false,
  isLoading: false,
  loaded: false,
  error: null,
};

interface DbScenePreset {
  id: string;
  name: string;
  description: string;
  category: ScenePreset['category'];
  thumbnail?: string;
  scene_data: ScenePreset['sceneData'];
  created_at: number;
  updated_at: number;
}

function toDbPreset(preset: ScenePreset): DbScenePreset {
  return {
    id: preset.id,
    name: preset.name,
    description: preset.description,
    category: preset.category,
    thumbnail: preset.thumbnail,
    scene_data: preset.sceneData,
    created_at: preset.createdAt,
    updated_at: preset.updatedAt,
  };
}

function fromDbPreset(preset: DbScenePreset): ScenePreset {
  return {
    id: preset.id,
    name: preset.name,
    description: preset.description,
    category: preset.category,
    thumbnail: preset.thumbnail,
    sceneData: preset.scene_data,
    createdAt: preset.created_at,
    updatedAt: preset.updated_at,
  };
}

export const usePresetStore = create<PresetState & PresetActions>((set, get) => ({
  ...initialState,

  setPresets: (presets) => set({ presets }),
  addPreset: (preset) =>
    set((state) => ({
      presets: [...state.presets, preset],
    })),
  updatePreset: (id, updates) =>
    set((state) => ({
      presets: state.presets.map((preset) =>
        preset.id === id ? { ...preset, ...updates, updatedAt: Date.now() } : preset
      ),
    })),
  removePreset: (id) =>
    set((state) => ({
      presets: state.presets.filter((preset) => preset.id !== id),
      selectedPreset: state.selectedPreset?.id === id ? null : state.selectedPreset,
    })),
  setSelectedPreset: (preset) => set({ selectedPreset: preset }),

  setIsSaving: (isSaving) => set({ isSaving }),
  setIsLoading: (isLoading) => set({ isLoading }),
  setError: (error) => set({ error }),

  loadPersistedPresets: async () => {
    if (get().loaded) return;
    set({ isLoading: true, error: null });
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      const persisted = await invoke<DbScenePreset[]>('load_scene_presets');
      set({ presets: persisted.map(fromDbPreset), loaded: true });
    } catch (e) {
      console.error('Failed to load presets:', e);
      set({ error: String(e), loaded: true });
    } finally {
      set({ isLoading: false });
    }
  },

  saveCurrentSceneAsPreset: async (name: string, description: string, category: string) => {
    set({ isSaving: true, error: null });
    try {
      // Get current scene state
      const sceneStore = useSceneStore.getState();
      const { figures, props, lights, cameras, activeCamera, selectedItem } = sceneStore;

      // Create preset ID
      const id = `preset-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;

      const newPreset: ScenePreset = {
        id,
        name,
        description,
        category: category as ScenePreset['category'],
        createdAt: Date.now(),
        updatedAt: Date.now(),
        sceneData: {
          figures: figures || [],
          props: props || [],
          lights: lights || [],
          cameras: cameras || [],
          activeCamera: activeCamera ?? null,
          selectedItem: selectedItem ?? null,
        },
      };

      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('save_scene_preset', { preset: toDbPreset(newPreset) });
      get().addPreset(newPreset);

      // Show success toast
      const { useToastStore } = await import('./toastStore');
      useToastStore.getState().success(`Preset "${name}" saved successfully!`);
    } catch (e) {
      console.error('Failed to save preset:', e);
      set({ error: String(e) });

      const { useToastStore } = await import('./toastStore');
      useToastStore.getState().error(`Failed to save preset: ${String(e)}`);
      throw e;
    } finally {
      set({ isSaving: false });
    }
  },

  loadPreset: async (id: string) => {
    set({ isLoading: true, error: null });
    try {
      const preset = get().presets.find((p) => p.id === id);
      if (!preset) {
        throw new Error(`Preset with ID ${id} not found`);
      }

      useSceneStore.setState({
        figures: preset.sceneData.figures,
        props: preset.sceneData.props,
        lights: preset.sceneData.lights,
        cameras: preset.sceneData.cameras,
        activeCamera: preset.sceneData.activeCamera,
        selectedItem: preset.sceneData.selectedItem,
      });

      // Update selected preset
      get().setSelectedPreset(preset);

      // Show success toast
      const { useToastStore } = await import('./toastStore');
      useToastStore.getState().success(`Preset "${preset.name}" loaded successfully!`);
    } catch (e) {
      console.error('Failed to load preset:', e);
      set({ error: String(e) });

      const { useToastStore } = await import('./toastStore');
      useToastStore.getState().error(`Failed to load preset: ${String(e)}`);
      throw e;
    } finally {
      set({ isLoading: false });
    }
  },

  deletePreset: async (id: string) => {
    set({ isSaving: true, error: null }); // Reuse isSaving for deletion feedback
    try {
      const preset = get().presets.find((p) => p.id === id);
      if (!preset) {
        throw new Error(`Preset with ID ${id} not found`);
      }

      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('delete_scene_preset', { presetId: id });
      get().removePreset(id);

      // Show success toast
      const { useToastStore } = await import('./toastStore');
      useToastStore.getState().success(`Preset "${preset.name}" deleted successfully!`);
    } catch (e) {
      console.error('Failed to delete preset:', e);
      set({ error: String(e) });

      const { useToastStore } = await import('./toastStore');
      useToastStore.getState().error(`Failed to delete preset: ${String(e)}`);
      throw e;
    } finally {
      set({ isSaving: false });
    }
  },

  getPresetsByCategory: (category: string) => {
    return get().presets.filter((preset) => preset.category === category);
  },

  reset: () => set(initialState),
}));
