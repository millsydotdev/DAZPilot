import { create } from 'zustand';
import { useChatStore } from './chatStore';
import type { TutorialCategory } from '../data/tutorials';

export interface ConceptDef {
  id: string;
  name: string;
  command: string;
  category: TutorialCategory;
}

export const ALL_CONCEPTS: ConceptDef[] = [
  { id: 'figure', name: 'Figure Loading', command: 'add_figure', category: 'basics' },
  { id: 'asset', name: 'Asset Loading', command: 'load_asset', category: 'basics' },
  { id: 'scene-tree', name: 'Scene Hierarchy', command: 'add_node', category: 'basics' },
  { id: 'selection', name: 'Selection', command: 'select_node', category: 'basics' },
  { id: 'property', name: 'Property Editing', command: 'set_property', category: 'basics' },
  { id: 'pose', name: 'Posing', command: 'apply_pose', category: 'posing' },
  { id: 'morph', name: 'Morphs', command: 'set_morph', category: 'posing' },
  { id: 'light', name: 'Lighting', command: 'set_light', category: 'lighting' },
  { id: 'material', name: 'Materials', command: 'set_material', category: 'materials' },
  { id: 'camera', name: 'Camera', command: 'set_camera', category: 'animation' },
  { id: 'animation', name: 'Animation', command: 'set_keyframe', category: 'animation' },
  {
    id: 'physics',
    name: 'Physics (dForce)',
    command: 'run_dforce_simulation',
    category: 'animation',
  },
  { id: 'render', name: 'Rendering', command: 'render_preview', category: 'lighting' },
  { id: 'export', name: 'Exporting', command: 'export_scene', category: 'basics' },
];

export const TUTORIAL_SUGGESTIONS: Record<string, string[]> = {
  basics: ['scene-setup'],
  lighting: ['three-point-lighting'],
  posing: ['posing-basics'],
  materials: ['materials-101'],
  animation: ['camera-basics'],
};

export interface ConceptProgress {
  concept: ConceptDef;
  discovered: boolean;
  count: number;
}

export interface LearningState {
  discoveredConcepts: Record<string, ConceptProgress>;
  totalConcepts: number;
  discoveredCount: number;
  categoryBreakdown: Record<TutorialCategory, { total: number; discovered: number }>;
  suggestedTutorials: string[];
  refresh: () => void;
}

function buildCategoryBreakdown(
  discovered: Record<string, ConceptProgress>
): Record<TutorialCategory, { total: number; discovered: number }> {
  const breakdown: Record<string, { total: number; discovered: number }> = {};
  for (const c of ALL_CONCEPTS) {
    if (!breakdown[c.category]) {
      breakdown[c.category] = { total: 0, discovered: 0 };
    }
    breakdown[c.category].total++;
    if (discovered[c.id]?.discovered) {
      breakdown[c.category].discovered++;
    }
  }
  return breakdown as Record<TutorialCategory, { total: number; discovered: number }>;
}

function computeSuggestions(discovered: Record<string, ConceptProgress>): string[] {
  const suggested = new Set<string>();
  const missingByCategory: Record<string, number> = {};

  for (const c of ALL_CONCEPTS) {
    if (!discovered[c.id]?.discovered) {
      missingByCategory[c.category] = (missingByCategory[c.category] || 0) + 1;
    }
  }

  const sorted = Object.entries(missingByCategory).sort((a, b) => b[1] - a[1]);

  for (const [cat] of sorted.slice(0, 3)) {
    const tutorials = TUTORIAL_SUGGESTIONS[cat] || [];
    for (const t of tutorials) {
      suggested.add(t);
    }
  }

  return Array.from(suggested);
}

export const useLearningStore = create<LearningState>((set) => ({
  discoveredConcepts: {},
  totalConcepts: ALL_CONCEPTS.length,
  discoveredCount: 0,
  categoryBreakdown: buildCategoryBreakdown({}),
  suggestedTutorials: [],

  refresh: () => {
    const messages = useChatStore.getState().messages;
    const commandCounts = new Map<string, number>();

    for (const msg of messages) {
      if (msg.action?.command) {
        const cmd = msg.action.command;
        commandCounts.set(cmd, (commandCounts.get(cmd) || 0) + 1);
      }
    }

    const discovered: Record<string, ConceptProgress> = {};
    for (const concept of ALL_CONCEPTS) {
      const count = commandCounts.get(concept.command) || 0;
      // also check alternative commands
      let altCount = 0;
      if (concept.command === 'set_camera') {
        altCount = commandCounts.get('create_camera') || 0;
      }
      if (concept.command === 'set_material') {
        altCount = commandCounts.get('set_surface') || 0;
      }
      if (concept.command === 'render_preview') {
        altCount = commandCounts.get('render') || 0;
      }
      discovered[concept.id] = {
        concept,
        discovered: count > 0 || altCount > 0,
        count: count + altCount,
      };
    }

    const discoveredCount = Object.values(discovered).filter((d) => d.discovered).length;

    set({
      discoveredConcepts: discovered,
      discoveredCount,
      categoryBreakdown: buildCategoryBreakdown(discovered),
      suggestedTutorials: computeSuggestions(discovered),
    });
  },
}));
