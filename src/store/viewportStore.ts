import { create } from 'zustand';

export interface Keyframe {
  id: string;
  frame: number;
  properties: Record<string, unknown>;
}

export interface TimelineState {
  currentFrame: number;
  totalFrames: number;
  fps: number;
  duration: number;
}

export interface PlaybackState {
  isPlaying: boolean;
  isPaused: boolean;
  isLooping: boolean;
  playbackSpeed: number;
}

export interface Pose {
  id: string;
  name: string;
  category: string;
  thumbnail?: string;
  keyframes: Keyframe[];
}

export interface ViewportState {
  timeline: TimelineState;
  playback: PlaybackState;
  poses: Pose[];
  selectedPose: Pose | null;
  showPoseLibrary: boolean;
  selectedFigure: string | null;
  cameraPreset: string;
  error: string | null;
}

export interface ViewportActions {
  setTimeline: (timeline: Partial<TimelineState>) => void;
  setPlayback: (playback: Partial<PlaybackState>) => void;
  setPoses: (poses: Pose[]) => void;
  setSelectedPose: (pose: Pose | null) => void;
  togglePoseLibrary: () => void;
  setSelectedFigure: (figure: string | null) => void;
  setCameraPreset: (preset: string) => void;
  setError: (error: string | null) => void;
  play: () => void;
  pause: () => void;
  stop: () => void;
  seek: (frame: number) => void;
  toggleLoop: () => void;
  setPlaybackSpeed: (speed: number) => void;
  loadState: () => Promise<void>;
  reset: () => void;
}

const defaultTimeline: TimelineState = {
  currentFrame: 0,
  totalFrames: 300,
  fps: 30,
  duration: 10,
};

const defaultPlayback: PlaybackState = {
  isPlaying: false,
  isPaused: false,
  isLooping: true,
  playbackSpeed: 1,
};

const initialState: ViewportState = {
  timeline: defaultTimeline,
  playback: defaultPlayback,
  poses: [],
  selectedPose: null,
  showPoseLibrary: false,
  selectedFigure: null,
  cameraPreset: 'front',
  error: null,
};

export const useViewportStore = create<ViewportState & ViewportActions>((set, get) => {
  return {
    ...initialState,
    setTimeline: (timeline) => set((s) => ({ timeline: { ...s.timeline, ...timeline } })),
    setPlayback: (playback) => set((s) => ({ playback: { ...s.playback, ...playback } })),
    setPoses: (poses) => set({ poses }),
    setSelectedPose: (selectedPose) => set({ selectedPose }),
    togglePoseLibrary: () => set((s) => ({ showPoseLibrary: !s.showPoseLibrary })),
    setSelectedFigure: (selectedFigure) => set({ selectedFigure }),
    setCameraPreset: (cameraPreset) => set({ cameraPreset }),
    setError: (error) => set({ error }),
    play: async () => {
      const current = get();
      set({ playback: { ...current.playback, isPlaying: true, isPaused: false } });
      try {
        const { invoke } = await import('@tauri-apps/api/core');
        await invoke('play_animation');
      } catch (e) {
        set({ error: String(e) });
      }
    },
    pause: async () => {
      const current = get();
      set({ playback: { ...current.playback, isPlaying: false, isPaused: true } });
      try {
        const { invoke } = await import('@tauri-apps/api/core');
        await invoke('pause_animation');
      } catch (e) {
        set({ error: String(e) });
      }
    },
    stop: async () => {
      const current = get();
      set({
        playback: { ...current.playback, isPlaying: false, isPaused: false },
        timeline: { ...current.timeline, currentFrame: 0 },
      });
      try {
        const { invoke } = await import('@tauri-apps/api/core');
        await invoke('stop_animation');
      } catch (e) {
        set({ error: String(e) });
      }
    },
    seek: (frame) =>
      set((s) => ({
        timeline: {
          ...s.timeline,
          currentFrame: Math.max(0, Math.min(frame, s.timeline.totalFrames)),
        },
      })),
    toggleLoop: () =>
      set((s) => ({ playback: { ...s.playback, isLooping: !s.playback.isLooping } })),
    setPlaybackSpeed: (playbackSpeed) =>
      set((s) => ({ playback: { ...s.playback, playbackSpeed } })),
    loadState: async () => {
      try {
        const { invoke } = await import('@tauri-apps/api/core');
        const timeline = await invoke<TimelineState>('get_timeline_state');
        const playback = await invoke<PlaybackState>('get_playback_state');
        const poses = await invoke<Pose[]>('get_pose_library');
        set({ timeline, playback, poses });
      } catch (e) {
        set({ error: String(e) });
      }
    },
    reset: () => set(initialState),
  };
});
