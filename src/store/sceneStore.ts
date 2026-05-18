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
}

export interface SceneState {
  figures: SceneFigure[];
  props: SceneProp[];
  lights: SceneLight[];
  cameras: SceneCamera[];
  activeCamera: string | null;
  selectedItem: string | null;
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
  setActiveCamera: (id: string | null) => void;
  selectItem: (id: string | null) => void;
  clearScene: () => void;
  loadScene: () => Promise<void>;
}

const initialState: SceneState = {
  figures: [],
  props: [],
  lights: [
    {
      id: 'default-light',
      name: 'Main Light',
      type: 'directional',
      enabled: true,
      intensity: 1,
      color: '#ffffff',
    },
  ],
  cameras: [],
  activeCamera: null,
  selectedItem: null,
};

export const useSceneStore = create<SceneState & SceneActions>((set) => ({
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

  removeFigure: (id) =>
    set((state) => ({
      figures: state.figures.filter((f) => f.id !== id),
    })),

  selectFigure: (id) =>
    set((state) => ({
      figures: state.figures.map((f) => ({ ...f, selected: f.id === id })),
      selectedItem: id,
    })),

  toggleFigureVisibility: (id) =>
    set((state) => ({
      figures: state.figures.map((f) => (f.id === id ? { ...f, visible: !f.visible } : f)),
    })),

  toggleFigureLock: (id) =>
    set((state) => ({
      figures: state.figures.map((f) => (f.id === id ? { ...f, locked: !f.locked } : f)),
    })),

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

  removeProp: (id) =>
    set((state) => ({
      props: state.props.filter((p) => p.id !== id),
    })),

  selectProp: (id) =>
    set((state) => ({
      props: state.props.map((p) => ({ ...p, selected: p.id === id })),
      selectedItem: id,
    })),

  togglePropVisibility: (id) =>
    set((state) => ({
      props: state.props.map((p) => (p.id === id ? { ...p, visible: !p.visible } : p)),
    })),

  togglePropLock: (id) =>
    set((state) => ({
      props: state.props.map((p) => (p.id === id ? { ...p, locked: !p.locked } : p)),
    })),

  addLight: (light) =>
    set((state) => ({
      lights: [...state.lights, { ...light, id: `light-${Date.now()}` }],
    })),

  removeLight: (id) =>
    set((state) => ({
      lights: state.lights.filter((l) => l.id !== id),
    })),

  updateLight: (id, updates) =>
    set((state) => ({
      lights: state.lights.map((l) => (l.id === id ? { ...l, ...updates } : l)),
    })),

  setActiveCamera: (id) => set({ activeCamera: id }),
  selectItem: (id) => set({ selectedItem: id }),
  clearScene: () => set(initialState),
  loadScene: async () => {
    try {
      const { invoke } = await import('@tauri-apps/api/core');
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

      for (const node of nodes) {
        if (node.node_type === 'Figure') {
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
        } else if (node.node_type === 'Light') {
          lights.push({
            id: node.id,
            name: node.name,
            type: 'directional',
            enabled: true,
            intensity: 1,
            color: '#ffffff',
          });
        } else if (node.node_type !== 'Camera') {
          props.push({
            id: node.id,
            name: node.name,
            type: node.node_type,
            selected: node.selected,
            visible: true,
            locked: false,
          });
        }
      }

      set({ figures, props, lights });
    } catch (e) {
      console.error('Failed to load scene:', e);
    }
  },
}));
