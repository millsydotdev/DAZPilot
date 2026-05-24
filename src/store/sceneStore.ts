import { create } from 'zustand';

export interface SceneFigure {
  id: string;
  name: string;
  type: 'genesis8' | 'genesis9' | 'other';
  selected: boolean;
  visible: boolean;
  locked: boolean;
}

export interface SceneProp {
  id: string;
  name: string;
  type: string;
  selected: boolean;
  visible: boolean;
  locked: boolean;
}

export interface SceneLight {
  id: string;
  name: string;
  type: 'directional' | 'point' | 'spot' | 'ambient';
  enabled: boolean;
  intensity: number;
  color: string;
}

export interface SceneCamera {
  id: string;
  name: string;
  position: { x: number; y: number; z: number };
  target: { x: number; y: number; z: number };
  focalLength: number;
  enabled: boolean;
}

export interface NodeProperty {
  name: string;
  label: string;
  value: number;
  min: number;
  max: number;
  path: string;
  is_morph: boolean;
}

export interface MaterialProperty {
  name: string;
  label: string;
  value: number;
  min: number;
  max: number;
}

export interface Material {
  name: string;
  label: string;
  properties: MaterialProperty[];
}

export interface SceneState {
  figures: SceneFigure[];
  props: SceneProp[];
  lights: SceneLight[];
  cameras: SceneCamera[];
  nodeProperties: Record<string, NodeProperty[]>;
  nodeMaterials: Record<string, Material[]>;
  activeCamera: string | null;
  selectedItem: string | null;
  bridgeSynced: boolean;
}

export interface SceneActions {
  addFigure: (figure: Omit<SceneFigure, 'id' | 'selected' | 'visible' | 'locked'>) => void;
  removeFigure: (id: string) => void;
  selectFigure: (id: string) => void;
  toggleFigureVisibility: (id: string) => void;
  toggleFigureLock: (id: string) => void;
  addProp: (prop: Omit<SceneProp, 'id' | 'selected' | 'visible' | 'locked'>) => void;
  removeProp: (id: string) => void;
  selectProp: (id: string) => void;
  togglePropVisibility: (id: string) => void;
  togglePropLock: (id: string) => void;
  addLight: (light: Omit<SceneLight, 'id'>) => void;
  removeLight: (id: string) => void;
  updateLight: (id: string, updates: Partial<SceneLight>) => void;
  addCamera: (camera: Omit<SceneCamera, 'id'>) => void;
  removeCamera: (id: string) => void;
  selectCamera: (id: string) => void;
  toggleCameraEnabled: (id: string) => void;
  fetchNodeProperties: (nodeId: string) => Promise<void>;
  updateNodeProperty: (nodeId: string, propName: string, value: number) => Promise<void>;
  fetchMaterialProperties: (nodeId: string) => Promise<void>;
  updateMaterialProperty: (
    nodeId: string,
    matName: string,
    propName: string,
    value: number
  ) => Promise<void>;
  setActiveCamera: (id: string | null) => void;
  selectItem: (id: string | null) => void;
  clearScene: () => void;
  loadScene: () => Promise<void>;
}

const initialState: SceneState = {
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

async function bridgeConnected(): Promise<boolean> {
  try {
    const { invoke } = await import('@tauri-apps/api/core');
    const status = await invoke<string>('check_connection_status');
    return status === 'connected';
  } catch {
    return false;
  }
}

async function bridgeSelectNode(nodeId: string): Promise<void> {
  const { invoke } = await import('@tauri-apps/api/core');
  await invoke('execute_command', {
    command: 'select_node',
    args: { node_id: nodeId },
  });
}

async function bridgeDeleteNode(nodeId: string): Promise<void> {
  const { invoke } = await import('@tauri-apps/api/core');
  await invoke('execute_command', {
    command: 'delete_node',
    args: { node_id: nodeId },
  });
}

async function bridgeSetVisible(nodeId: string, visible: boolean): Promise<void> {
  const { invoke } = await import('@tauri-apps/api/core');
  await invoke('execute_command', {
    command: 'set_property',
    args: {
      node_id: nodeId,
      property: 'Visible',
      value: visible ? '1' : '0',
    },
  });
}

export const useSceneStore = create<SceneState & SceneActions>((set, get) => ({
  ...initialState,

  addFigure: (figure) =>
    set((state) => ({
      figures: [
        ...state.figures,
        {
          ...figure,
          id: `figure-${Date.now()}`,
          selected: false,
          visible: true,
          locked: false,
        },
      ],
    })),

  removeFigure: async (id) => {
    if (await bridgeConnected()) {
      try {
        await bridgeDeleteNode(id);
      } catch (e) {
        console.error('Bridge delete_node failed:', e);
      }
    }
    set((state) => ({
      figures: state.figures.filter((f) => f.id !== id),
    }));
  },

  selectFigure: async (id) => {
    if (await bridgeConnected()) {
      try {
        await bridgeSelectNode(id);
      } catch (e) {
        console.error('Bridge select_node failed:', e);
      }
    }
    set((state) => ({
      figures: state.figures.map((f) => ({ ...f, selected: f.id === id })),
      selectedItem: id,
    }));
  },

  toggleFigureVisibility: async (id) => {
    const figure = get().figures.find((f) => f.id === id);
    const nextVisible = figure ? !figure.visible : true;
    if (await bridgeConnected()) {
      try {
        await bridgeSetVisible(id, nextVisible);
      } catch (e) {
        console.error('Bridge set_property Visible failed:', e);
      }
    }
    set((state) => ({
      figures: state.figures.map((f) => (f.id === id ? { ...f, visible: nextVisible } : f)),
    }));
  },

  toggleFigureLock: async (id) => {
    const figure = get().figures.find((f) => f.id === id);
    const nextLocked = figure ? !figure.locked : true;
    if (await bridgeConnected()) {
      try {
        const { invoke } = await import('@tauri-apps/api/core');
        await invoke('execute_command', {
          command: 'set_property',
          args: {
            node_id: id,
            property: 'Locked',
            value: nextLocked ? '1' : '0',
          },
        });
      } catch (e) {
        console.error('Bridge set_property Locked failed:', e);
      }
    }
    set((state) => ({
      figures: state.figures.map((f) => (f.id === id ? { ...f, locked: nextLocked } : f)),
    }));
  },

  addProp: (prop) =>
    set((state) => ({
      props: [
        ...state.props,
        {
          ...prop,
          id: `prop-${Date.now()}`,
          selected: false,
          visible: true,
          locked: false,
        },
      ],
    })),

  removeProp: async (id) => {
    if (await bridgeConnected()) {
      try {
        await bridgeDeleteNode(id);
      } catch (e) {
        console.error('Bridge delete_node failed:', e);
      }
    }
    set((state) => ({
      props: state.props.filter((p) => p.id !== id),
    }));
  },

  selectProp: async (id) => {
    if (await bridgeConnected()) {
      try {
        await bridgeSelectNode(id);
      } catch (e) {
        console.error('Bridge select_node failed:', e);
      }
    }
    set((state) => ({
      props: state.props.map((p) => ({ ...p, selected: p.id === id })),
      selectedItem: id,
    }));
  },

  togglePropVisibility: async (id) => {
    const prop = get().props.find((p) => p.id === id);
    const nextVisible = prop ? !prop.visible : true;
    if (await bridgeConnected()) {
      try {
        await bridgeSetVisible(id, nextVisible);
      } catch (e) {
        console.error('Bridge set_property Visible failed:', e);
      }
    }
    set((state) => ({
      props: state.props.map((p) => (p.id === id ? { ...p, visible: nextVisible } : p)),
    }));
  },

  togglePropLock: async (id) => {
    const prop = get().props.find((p) => p.id === id);
    const nextLocked = prop ? !prop.locked : true;
    if (await bridgeConnected()) {
      try {
        const { invoke } = await import('@tauri-apps/api/core');
        await invoke('execute_command', {
          command: 'set_property',
          args: {
            node_id: id,
            property: 'Locked',
            value: nextLocked ? '1' : '0',
          },
        });
      } catch (e) {
        console.error('Bridge set_property Locked failed:', e);
      }
    }
    set((state) => ({
      props: state.props.map((p) => (p.id === id ? { ...p, locked: nextLocked } : p)),
    }));
  },

  addLight: (light) =>
    set((state) => ({
      lights: [...state.lights, { ...light, id: `light-${Date.now()}` }],
    })),

  removeLight: async (id) => {
    if (await bridgeConnected()) {
      try {
        await bridgeDeleteNode(id);
      } catch (e) {
        console.error('Bridge delete_node failed:', e);
      }
    }
    set((state) => ({
      lights: state.lights.filter((l) => l.id !== id),
    }));
  },

  updateLight: async (id, updates) => {
    if (await bridgeConnected()) {
      try {
        const { invoke } = await import('@tauri-apps/api/core');
        if (updates.enabled !== undefined) {
          await invoke('execute_command', {
            command: 'set_light',
            args: {
              node_id: id,
              property: 'Visible',
              value: updates.enabled ? '1' : '0',
            },
          });
        }
        if (updates.intensity !== undefined) {
          await invoke('execute_command', {
            command: 'set_light',
            args: {
              node_id: id,
              property: 'Intensity',
              value: updates.intensity.toString(),
            },
          });
        }
        if (updates.color !== undefined) {
          // Convert hex to RGB string expected by bridge
          const r = parseInt(updates.color.slice(1, 3), 16);
          const g = parseInt(updates.color.slice(3, 5), 16);
          const b = parseInt(updates.color.slice(5, 7), 16);
          await invoke('execute_command', {
            command: 'set_light',
            args: {
              node_id: id,
              property: 'Color',
              value: `${r},${g},${b}`,
            },
          });
        }
      } catch (e) {
        console.error('Bridge set_light failed:', e);
      }
    }
    set((state) => ({
      lights: state.lights.map((l) => (l.id === id ? { ...l, ...updates } : l)),
    }));
  },

  addCamera: (camera) =>
    set((state) => ({
      cameras: [...state.cameras, { ...camera, id: `camera-${Date.now()}` }],
    })),

  removeCamera: async (id) => {
    if (await bridgeConnected()) {
      try {
        await bridgeDeleteNode(id);
      } catch (e) {
        console.error('Bridge delete_node failed:', e);
      }
    }
    set((state) => ({
      cameras: state.cameras.filter((c) => c.id !== id),
      // If we're removing the active camera, clear it
      activeCamera: state.activeCamera === id ? null : state.activeCamera,
    }));
  },

  selectCamera: async (id) => {
    if (await bridgeConnected()) {
      try {
        await bridgeSelectNode(id);
      } catch (e) {
        console.error('Bridge select_node failed:', e);
      }
    }
    set(() => ({
      activeCamera: id,
      selectedItem: id,
    }));
  },

  toggleCameraEnabled: async (id) => {
    const camera = get().cameras.find((c) => c.id === id);
    const nextEnabled = camera ? !camera.enabled : true;
    if (await bridgeConnected()) {
      try {
        const { invoke } = await import('@tauri-apps/api/core');
        await invoke('execute_command', {
          command: 'set_property',
          args: {
            node_id: id,
            property: 'Visible',
            value: nextEnabled ? '1' : '0',
          },
        });
      } catch (e) {
        console.error('Bridge set_property Visible failed:', e);
      }
    }
    set((state) => ({
      cameras: state.cameras.map((c) => (c.id === id ? { ...c, enabled: nextEnabled } : c)),
    }));
  },

  fetchNodeProperties: async (nodeId) => {
    if (await bridgeConnected()) {
      try {
        const { invoke } = await import('@tauri-apps/api/core');
        const result = await invoke<{ properties: NodeProperty[] }>('execute_command', {
          command: 'get_node_properties',
          args: { node_id: nodeId },
        });
        set((state) => ({
          nodeProperties: {
            ...state.nodeProperties,
            [nodeId]: result.properties,
          },
        }));
      } catch (e) {
        console.error('Bridge get_node_properties failed:', e);
      }
    }
  },

  updateNodeProperty: async (nodeId, propName, value) => {
    // Optimistic update
    set((state) => ({
      nodeProperties: {
        ...state.nodeProperties,
        [nodeId]:
          state.nodeProperties[nodeId]?.map((p) => (p.name === propName ? { ...p, value } : p)) ||
          [],
      },
    }));

    if (await bridgeConnected()) {
      try {
        const { invoke } = await import('@tauri-apps/api/core');
        await invoke('execute_command', {
          command: 'set_property',
          args: {
            node_id: nodeId,
            property: propName,
            value: value.toString(),
          },
        });
      } catch (e) {
        console.error('Bridge set_property failed:', e);
      }
    }
  },

  fetchMaterialProperties: async (nodeId) => {
    if (await bridgeConnected()) {
      try {
        const { invoke } = await import('@tauri-apps/api/core');
        const result = await invoke<{ materials: Material[] }>('execute_command', {
          command: 'get_material_properties',
          args: { node_id: nodeId },
        });
        set((state) => ({
          nodeMaterials: {
            ...state.nodeMaterials,
            [nodeId]: result.materials,
          },
        }));
      } catch (e) {
        console.error('Bridge get_material_properties failed:', e);
      }
    }
  },

  updateMaterialProperty: async (nodeId, matName, propName, value) => {
    // Optimistic update
    set((state) => ({
      nodeMaterials: {
        ...state.nodeMaterials,
        [nodeId]:
          state.nodeMaterials[nodeId]?.map((m) =>
            m.name === matName
              ? {
                  ...m,
                  properties: m.properties.map((p) => (p.name === propName ? { ...p, value } : p)),
                }
              : m
          ) || [],
      },
    }));

    if (await bridgeConnected()) {
      try {
        const { invoke } = await import('@tauri-apps/api/core');
        await invoke('execute_command', {
          command: 'set_material_property',
          args: {
            node_id: nodeId,
            material: matName,
            property: propName,
            value: value.toString(),
          },
        });
      } catch (e) {
        console.error('Bridge set_material_property failed:', e);
      }
    }
  },

  setActiveCamera: (id) => set({ activeCamera: id }),
  selectItem: (id) => set({ selectedItem: id }),
  clearScene: () => set({ ...initialState }),
  loadScene: async () => {
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      const connected = await bridgeConnected();
      await invoke<{
        name: string;
        figure: string | null;
        node_count: number;
        light_count: number;
        camera_count: number;
      }>('get_scene_info');
      const nodes =
        await invoke<Array<{ name: string; node_type: string; id: string; selected: boolean }>>(
          'list_nodes'
        );

      const figures: SceneFigure[] = [];
      const props: SceneProp[] = [];
      const lights: SceneLight[] = [];
      const cameras: SceneCamera[] = [];

      for (const node of nodes) {
        const nodeType = node.node_type || (node as { type?: string }).type || 'Node';
        if (nodeType === 'Figure') {
          figures.push({
            id: node.id,
            name: node.name,
            type: node.name.toLowerCase().includes('genesis 9')
              ? 'genesis9'
              : node.name.toLowerCase().includes('genesis 8')
                ? 'genesis8'
                : 'other',
            selected: node.selected,
            visible: true,
            locked: false,
          });
        } else if (nodeType === 'Light') {
          lights.push({
            id: node.id,
            name: node.name,
            type: 'directional',
            enabled: true,
            intensity: 1,
            color: '#ffffff',
          });
        } else if (nodeType === 'Camera') {
          // For cameras, we need to get more detailed information
          // For now, we'll create a basic camera object
          // In a real implementation, we'd fetch the camera properties
          cameras.push({
            id: node.id,
            name: node.name,
            position: { x: 0, y: 0, z: 0 },
            target: { x: 0, y: 0, z: -1 },
            focalLength: 50,
            enabled: true,
          });
        } else {
          props.push({
            id: node.id,
            name: node.name,
            type: nodeType,
            selected: node.selected,
            visible: true,
            locked: false,
          });
        }
      }

      set({ figures, props, lights, cameras, bridgeSynced: connected });
    } catch (e) {
      console.error('Failed to load scene:', e);
      set({ bridgeSynced: false });
    }
  },
}));
