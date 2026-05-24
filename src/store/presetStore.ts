import { create } from 'zustand';
import { useSceneStore } from './sceneStore';

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
    figures: any[];
    props: any[];
    lights: any[];
    cameras: any[];
    activeCamera: string | null;
    selectedItem: string | null;
    // We could store more detailed state like node properties, materials, etc.
  };
}

export interface PresetState {
  presets: ScenePreset[];
  selectedPreset: ScenePreset | null;
  isSaving: boolean;
  isLoading: boolean;
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
  error: null,
};

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
        category: category as any, // Type assertion for simplicity
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

      // Add to store
      get().addPreset(newPreset);

      // TODO: Persist to local storage or database
      // For now, we'll just keep it in memory

      // Show success toast
      const { useToastStore } = await import('./toastStore');
      useToastStore.getState().success(`Preset "${name}" saved successfully!`);
    } catch (e) {
      console.error('Failed to save preset:', e);
      set({ error: String(e) });

      const { useToastStore } = await import('./toastStore');
      useToastStore.getState().error(`Failed to save preset: ${String(e)}`);
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

      // Apply preset to current scene
      const sceneStore = useSceneStore.getState();

      // Clear current scene
      sceneStore.clearScene();

      // Apply preset data
      if (preset.sceneData.figures.length > 0) {
        for (const figure of preset.sceneData.figures) {
          sceneStore.addFigure(figure);
        }
      }

      if (preset.sceneData.props.length > 0) {
        for (const prop of preset.sceneData.props) {
          sceneStore.addProp(prop);
        }
      }

      if (preset.sceneData.lights.length > 0) {
        for (const light of preset.sceneData.lights) {
          sceneStore.addLight(light);
        }
      }

      if (preset.sceneData.cameras.length > 0) {
        for (const camera of preset.sceneData.cameras) {
          sceneStore.addCamera(camera);
        }
      }

      // Set active camera if specified
      if (preset.sceneData.activeCamera) {
        sceneStore.setActiveCamera(preset.sceneData.activeCamera);
      }

      // Set selected item if specified
      if (preset.sceneData.selectedItem) {
        sceneStore.selectItem(preset.sceneData.selectedItem);
      }

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

      // Confirm deletion
      const confirmed = window.confirm(
        `Are you sure you want to delete the preset "${preset.name}"?`
      );
      if (!confirmed) {
        set({ isSaving: false });
        return;
      }

      // Remove from store
      get().removePreset(id);

      // TODO: Remove from persistent storage

      // Show success toast
      const { useToastStore } = await import('./toastStore');
      useToastStore.getState().success(`Preset "${preset.name}" deleted successfully!`);
    } catch (e) {
      console.error('Failed to delete preset:', e);
      set({ error: String(e) });

      const { useToastStore } = await import('./toastStore');
      useToastStore.getState().error(`Failed to delete preset: ${String(e)}`);
    } finally {
      set({ isSaving: false });
    }
  },

  getPresetsByCategory: (category: string) => {
    return get().presets.filter((preset) => preset.category === category);
  },

  reset: () => set(initialState),
}));
