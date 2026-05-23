import { describe, it, expect, vi } from 'vitest';
import { act } from 'react';
import { useSceneStore } from './sceneStore';

const initialState = {
  figures: [],
  props: [],
  lights: [],
  cameras: [],
  nodeProperties: {},
  nodeMaterials: {},
  activeCamera: null,
  selectedItem: null,
  bridgeSynced: false,
};

describe('sceneStore', () => {
  it('addFigure adds with defaults', () => {
    act(() => useSceneStore.setState(initialState));
    useSceneStore.getState().addFigure({ name: 'Genesis 9', type: 'genesis9' });
    const fig = useSceneStore.getState().figures[0];
    expect(fig.name).toBe('Genesis 9');
    expect(fig.selected).toBe(false);
    expect(fig.visible).toBe(true);
    expect(fig.locked).toBe(false);
    expect(fig.id).toBeDefined();
  });

  it('removeFigure removes figure', async () => {
    act(() => useSceneStore.setState(initialState));
    useSceneStore.getState().addFigure({ name: 'F', type: 'genesis8' });
    const id = useSceneStore.getState().figures[0].id;
    await useSceneStore.getState().removeFigure(id);
    expect(useSceneStore.getState().figures).toHaveLength(0);
  });

  it('selectFigure selects and updates selectedItem', async () => {
    act(() =>
      useSceneStore.setState({
        ...initialState,
        figures: [
          {
            id: 'fig1',
            name: 'F1',
            type: 'genesis9' as const,
            selected: false,
            visible: true,
            locked: false,
          },
          {
            id: 'fig2',
            name: 'F2',
            type: 'genesis9' as const,
            selected: false,
            visible: true,
            locked: false,
          },
        ],
      })
    );
    await useSceneStore.getState().selectFigure('fig1');
    const s = useSceneStore.getState();
    expect(s.figures[0].selected).toBe(true);
    expect(s.figures[1].selected).toBe(false);
    expect(s.selectedItem).toBe('fig1');
  });

  it('toggleFigureVisibility toggles', async () => {
    act(() => useSceneStore.setState(initialState));
    useSceneStore.getState().addFigure({ name: 'F', type: 'genesis8' });
    const id = useSceneStore.getState().figures[0].id;
    await useSceneStore.getState().toggleFigureVisibility(id);
    expect(useSceneStore.getState().figures[0].visible).toBe(false);
  });

  it('toggleFigureLock toggles', async () => {
    act(() => useSceneStore.setState(initialState));
    useSceneStore.getState().addFigure({ name: 'F', type: 'genesis8' });
    const id = useSceneStore.getState().figures[0].id;
    await useSceneStore.getState().toggleFigureLock(id);
    expect(useSceneStore.getState().figures[0].locked).toBe(true);
  });

  it('addProp adds with defaults', () => {
    act(() => useSceneStore.setState(initialState));
    useSceneStore.getState().addProp({ name: 'Prop1', type: 'Prop' });
    const p = useSceneStore.getState().props[0];
    expect(p.name).toBe('Prop1');
    expect(p.selected).toBe(false);
    expect(p.visible).toBe(true);
    expect(p.locked).toBe(false);
  });

  it('removeProp removes prop', async () => {
    act(() => useSceneStore.setState(initialState));
    useSceneStore.getState().addProp({ name: 'P', type: 'Prop' });
    const id = useSceneStore.getState().props[0].id;
    await useSceneStore.getState().removeProp(id);
    expect(useSceneStore.getState().props).toHaveLength(0);
  });

  it('selectProp selects and updates selectedItem', async () => {
    act(() =>
      useSceneStore.setState({
        ...initialState,
        props: [
          { id: 'p1', name: 'P1', type: 'Prop', selected: false, visible: true, locked: false },
          { id: 'p2', name: 'P2', type: 'Prop', selected: false, visible: true, locked: false },
        ],
      })
    );
    await useSceneStore.getState().selectProp('p1');
    const s = useSceneStore.getState();
    expect(s.props[0].selected).toBe(true);
    expect(s.props[1].selected).toBe(false);
    expect(s.selectedItem).toBe('p1');
  });

  it('togglePropVisibility toggles', async () => {
    act(() => useSceneStore.setState(initialState));
    useSceneStore.getState().addProp({ name: 'P', type: 'Prop' });
    const id = useSceneStore.getState().props[0].id;
    await useSceneStore.getState().togglePropVisibility(id);
    expect(useSceneStore.getState().props[0].visible).toBe(false);
  });

  it('togglePropLock toggles', async () => {
    act(() => useSceneStore.setState(initialState));
    useSceneStore.getState().addProp({ name: 'P', type: 'Prop' });
    const id = useSceneStore.getState().props[0].id;
    await useSceneStore.getState().togglePropLock(id);
    expect(useSceneStore.getState().props[0].locked).toBe(true);
  });

  it('addLight adds with defaults', () => {
    act(() => useSceneStore.setState(initialState));
    useSceneStore.getState().addLight({
      name: 'Light1',
      type: 'directional',
      enabled: true,
      intensity: 1,
      color: '#ffffff',
    });
    const l = useSceneStore.getState().lights[0];
    expect(l.name).toBe('Light1');
    expect(l.id).toBeDefined();
  });

  it('removeLight removes light', async () => {
    act(() => useSceneStore.setState(initialState));
    useSceneStore
      .getState()
      .addLight({ name: 'L', type: 'point', enabled: true, intensity: 1, color: '#fff' });
    const id = useSceneStore.getState().lights[0].id;
    await useSceneStore.getState().removeLight(id);
    expect(useSceneStore.getState().lights).toHaveLength(0);
  });

  it('updateLight updates fields', async () => {
    act(() => useSceneStore.setState(initialState));
    useSceneStore
      .getState()
      .addLight({ name: 'L', type: 'directional', enabled: true, intensity: 1, color: '#ffffff' });
    const id = useSceneStore.getState().lights[0].id;
    await useSceneStore
      .getState()
      .updateLight(id, { enabled: false, intensity: 0.5, color: '#ff0000' });
    const l = useSceneStore.getState().lights[0];
    expect(l.enabled).toBe(false);
    expect(l.intensity).toBe(0.5);
    expect(l.color).toBe('#ff0000');
  });

  it('fetchNodeProperties fetches and stores', async () => {
    act(() => useSceneStore.setState(initialState));
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockReset();
    vi.mocked(invoke).mockResolvedValueOnce('connected');
    vi.mocked(invoke).mockResolvedValueOnce({
      properties: [
        { name: 'prop1', label: 'Prop 1', value: 0.5, min: 0, max: 1, path: '/p', is_morph: false },
      ],
    });
    await useSceneStore.getState().fetchNodeProperties('n1');
    expect(useSceneStore.getState().nodeProperties['n1']).toHaveLength(1);
    expect(useSceneStore.getState().nodeProperties['n1'][0].name).toBe('prop1');
  });

  it('updateNodeProperty optimistically updates', async () => {
    act(() => useSceneStore.setState(initialState));
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockResolvedValue(undefined);
    useSceneStore.getState().fetchNodeProperties = vi.fn().mockResolvedValue(undefined);
    useSceneStore.setState({
      nodeProperties: {
        n1: [{ name: 'prop1', label: 'P', value: 0, min: 0, max: 1, path: '/p', is_morph: false }],
      },
    });
    await useSceneStore.getState().updateNodeProperty('n1', 'prop1', 0.8);
    expect(useSceneStore.getState().nodeProperties['n1'][0].value).toBe(0.8);
  });

  it('fetchMaterialProperties fetches and stores', async () => {
    act(() => useSceneStore.setState(initialState));
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockReset();
    vi.mocked(invoke).mockResolvedValueOnce('connected');
    vi.mocked(invoke).mockResolvedValueOnce({
      materials: [
        {
          name: 'mat1',
          label: 'Mat 1',
          properties: [{ name: 'prop1', label: 'P', value: 0.5, min: 0, max: 1 }],
        },
      ],
    });
    await useSceneStore.getState().fetchMaterialProperties('n1');
    expect(useSceneStore.getState().nodeMaterials['n1']).toHaveLength(1);
    expect(useSceneStore.getState().nodeMaterials['n1'][0].name).toBe('mat1');
  });

  it('updateMaterialProperty optimistically updates', async () => {
    act(() => useSceneStore.setState(initialState));
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockResolvedValue(undefined);
    useSceneStore.setState({
      nodeMaterials: {
        n1: [
          {
            name: 'mat1',
            label: 'M',
            properties: [{ name: 'roughness', label: 'R', value: 0, min: 0, max: 1 }],
          },
        ],
      },
    });
    await useSceneStore.getState().updateMaterialProperty('n1', 'mat1', 'roughness', 0.9);
    expect(useSceneStore.getState().nodeMaterials['n1'][0].properties[0].value).toBe(0.9);
  });

  it('setActiveCamera', () => {
    act(() => useSceneStore.setState(initialState));
    useSceneStore.getState().setActiveCamera('cam1');
    expect(useSceneStore.getState().activeCamera).toBe('cam1');
    useSceneStore.getState().setActiveCamera(null);
    expect(useSceneStore.getState().activeCamera).toBeNull();
  });

  it('selectItem', () => {
    act(() => useSceneStore.setState(initialState));
    useSceneStore.getState().selectItem('item1');
    expect(useSceneStore.getState().selectedItem).toBe('item1');
  });

  it('clearScene resets to initial state', () => {
    act(() => useSceneStore.setState(initialState));
    useSceneStore.getState().addFigure({ name: 'F', type: 'genesis8' });
    useSceneStore.getState().clearScene();
    expect(useSceneStore.getState().figures).toHaveLength(0);
    expect(useSceneStore.getState().props).toHaveLength(0);
  });

  it('loadScene populates from invoke', async () => {
    act(() => useSceneStore.setState(initialState));
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockReset();
    vi.mocked(invoke).mockResolvedValueOnce('connected');
    vi.mocked(invoke).mockResolvedValueOnce({
      name: 'Scene',
      figure: null,
      node_count: 0,
      light_count: 0,
      camera_count: 0,
    });
    vi.mocked(invoke).mockResolvedValueOnce([
      { id: 'fig1', name: 'Genesis 9', node_type: 'Figure', selected: true },
      { id: 'light1', name: 'Sun', node_type: 'Light', selected: false },
      { id: 'cam1', name: 'Camera', node_type: 'Camera', selected: false },
    ]);
    await useSceneStore.getState().loadScene();
    const s = useSceneStore.getState();
    expect(s.bridgeSynced).toBe(true);
    expect(s.figures).toHaveLength(1);
    expect(s.figures[0].name).toBe('Genesis 9');
    expect(s.figures[0].type).toBe('genesis9');
    expect(s.lights).toHaveLength(1);
    expect(s.lights[0].name).toBe('Sun');
  });

  it('loadScene handles error and sets bridgeSynced false', async () => {
    act(() => useSceneStore.setState(initialState));
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockRejectedValue(new Error('fail'));
    await useSceneStore.getState().loadScene();
    expect(useSceneStore.getState().bridgeSynced).toBe(false);
  });
});
