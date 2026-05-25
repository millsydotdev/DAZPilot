import { beforeEach, describe, expect, it, vi } from 'vitest';
import { invoke } from '@tauri-apps/api/core';
import { usePresetStore, type ScenePreset } from './presetStore';
import { useSceneStore } from './sceneStore';

const emptyPresetState = {
  presets: [],
  selectedPreset: null,
  isSaving: false,
  isLoading: false,
  loaded: false,
  error: null,
};

const savedPreset: ScenePreset = {
  id: 'preset-1',
  name: 'Portrait',
  description: 'Camera and key light',
  category: 'scene',
  createdAt: 100,
  updatedAt: 200,
  sceneData: {
    figures: [],
    props: [],
    lights: [],
    cameras: [
      {
        id: 'camera-original',
        name: 'Portrait Camera',
        position: { x: 1, y: 2, z: 3 },
        target: { x: 0, y: 1, z: 0 },
        focalLength: 85,
        enabled: true,
      },
    ],
    activeCamera: 'camera-original',
    selectedItem: 'camera-original',
  },
};

describe('presetStore', () => {
  beforeEach(() => {
    vi.mocked(invoke).mockReset();
    usePresetStore.setState(emptyPresetState);
    useSceneStore.getState().clearScene();
  });

  it('hydrates persisted presets once', async () => {
    vi.mocked(invoke).mockResolvedValueOnce([
      {
        id: savedPreset.id,
        name: savedPreset.name,
        description: savedPreset.description,
        category: savedPreset.category,
        scene_data: savedPreset.sceneData,
        created_at: savedPreset.createdAt,
        updated_at: savedPreset.updatedAt,
      },
    ]);

    await usePresetStore.getState().loadPersistedPresets();
    await usePresetStore.getState().loadPersistedPresets();

    expect(invoke).toHaveBeenCalledOnce();
    expect(invoke).toHaveBeenCalledWith('load_scene_presets');
    expect(usePresetStore.getState().presets).toEqual([savedPreset]);
    expect(usePresetStore.getState().loaded).toBe(true);
  });

  it('persists a snapshot before adding it to the list', async () => {
    useSceneStore.setState(savedPreset.sceneData);
    vi.mocked(invoke).mockResolvedValueOnce(undefined);

    await usePresetStore.getState().saveCurrentSceneAsPreset('New Scene', 'Saved', 'scene');

    expect(invoke).toHaveBeenCalledWith(
      'save_scene_preset',
      expect.objectContaining({
        preset: expect.objectContaining({
          name: 'New Scene',
          description: 'Saved',
          category: 'scene',
          scene_data: savedPreset.sceneData,
        }),
      })
    );
    expect(usePresetStore.getState().presets).toHaveLength(1);
  });

  it('loads the exact saved identifiers for selection and active camera', async () => {
    usePresetStore.setState({ ...emptyPresetState, presets: [savedPreset] });

    await usePresetStore.getState().loadPreset(savedPreset.id);

    const scene = useSceneStore.getState();
    expect(scene.cameras[0].id).toBe('camera-original');
    expect(scene.activeCamera).toBe('camera-original');
    expect(scene.selectedItem).toBe('camera-original');
  });

  it('deletes a preset from persistence and local state', async () => {
    usePresetStore.setState({ ...emptyPresetState, presets: [savedPreset] });
    vi.mocked(invoke).mockResolvedValueOnce(undefined);

    await usePresetStore.getState().deletePreset(savedPreset.id);

    expect(invoke).toHaveBeenCalledWith('delete_scene_preset', { presetId: savedPreset.id });
    expect(usePresetStore.getState().presets).toEqual([]);
  });
});
