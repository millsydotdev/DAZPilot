import { describe, it, expect, vi } from 'vitest';
import { act } from 'react';
import { useViewportStore } from './viewportStore';
import type { Pose, TimelineState, PlaybackState } from './viewportStore';

const defaultTimeline: TimelineState = { currentFrame: 0, totalFrames: 300, fps: 30, duration: 10 };
const defaultPlayback: PlaybackState = {
  isPlaying: false,
  isPaused: false,
  isLooping: true,
  playbackSpeed: 1,
};

const initialState = {
  timeline: defaultTimeline,
  playback: defaultPlayback,
  poses: [],
  selectedPose: null,
  showPoseLibrary: false,
  selectedFigure: null,
  cameraPreset: 'front',
  syncFps: 5,
  error: null,
};

describe('viewportStore', () => {
  it('setTimeline merges partial', () => {
    act(() => useViewportStore.setState(initialState));
    useViewportStore.getState().setTimeline({ currentFrame: 50 });
    expect(useViewportStore.getState().timeline.currentFrame).toBe(50);
    expect(useViewportStore.getState().timeline.totalFrames).toBe(300);
  });

  it('setPlayback merges partial', () => {
    act(() => useViewportStore.setState(initialState));
    useViewportStore.getState().setPlayback({ isPlaying: true });
    expect(useViewportStore.getState().playback.isPlaying).toBe(true);
    expect(useViewportStore.getState().playback.isLooping).toBe(true);
  });

  it('setPoses replaces poses', () => {
    act(() => useViewportStore.setState(initialState));
    const pose: Pose = { id: 'p1', name: 'test', category: 'standing', keyframes: [] };
    useViewportStore.getState().setPoses([pose]);
    expect(useViewportStore.getState().poses).toHaveLength(1);
    expect(useViewportStore.getState().poses[0].name).toBe('test');
  });

  it('setSelectedPose', () => {
    act(() => useViewportStore.setState(initialState));
    const pose: Pose = { id: 'p1', name: 'test', category: 'standing', keyframes: [] };
    useViewportStore.getState().setSelectedPose(pose);
    expect(useViewportStore.getState().selectedPose?.id).toBe('p1');
    useViewportStore.getState().setSelectedPose(null);
    expect(useViewportStore.getState().selectedPose).toBeNull();
  });

  it('togglePoseLibrary toggles', () => {
    act(() => useViewportStore.setState(initialState));
    useViewportStore.getState().togglePoseLibrary();
    expect(useViewportStore.getState().showPoseLibrary).toBe(true);
    useViewportStore.getState().togglePoseLibrary();
    expect(useViewportStore.getState().showPoseLibrary).toBe(false);
  });

  it('setSelectedFigure', () => {
    act(() => useViewportStore.setState(initialState));
    useViewportStore.getState().setSelectedFigure('fig1');
    expect(useViewportStore.getState().selectedFigure).toBe('fig1');
  });

  it('setCameraPreset', () => {
    act(() => useViewportStore.setState(initialState));
    useViewportStore.getState().setCameraPreset('top');
    expect(useViewportStore.getState().cameraPreset).toBe('top');
  });

  it('setError', () => {
    act(() => useViewportStore.setState(initialState));
    useViewportStore.getState().setError('oops');
    expect(useViewportStore.getState().error).toBe('oops');
    useViewportStore.getState().setError(null);
    expect(useViewportStore.getState().error).toBeNull();
  });

  it('play sets playing and calls invoke', async () => {
    act(() => useViewportStore.setState(initialState));
    await useViewportStore.getState().play();
    expect(useViewportStore.getState().playback.isPlaying).toBe(true);
    expect(useViewportStore.getState().playback.isPaused).toBe(false);
  });

  it('pause sets paused and calls invoke', async () => {
    act(() =>
      useViewportStore.setState({
        ...initialState,
        playback: { ...defaultPlayback, isPlaying: true },
      })
    );
    await useViewportStore.getState().pause();
    expect(useViewportStore.getState().playback.isPlaying).toBe(false);
    expect(useViewportStore.getState().playback.isPaused).toBe(true);
  });

  it('stop resets frame and playback', async () => {
    act(() =>
      useViewportStore.setState({
        ...initialState,
        timeline: { ...defaultTimeline, currentFrame: 150 },
        playback: { ...defaultPlayback, isPlaying: true },
      })
    );
    await useViewportStore.getState().stop();
    expect(useViewportStore.getState().playback.isPlaying).toBe(false);
    expect(useViewportStore.getState().playback.isPaused).toBe(false);
    expect(useViewportStore.getState().timeline.currentFrame).toBe(0);
  });

  it('seek clamps to 0..totalFrames', () => {
    act(() => useViewportStore.setState(initialState));
    useViewportStore.getState().seek(-10);
    expect(useViewportStore.getState().timeline.currentFrame).toBe(0);
    useViewportStore.getState().seek(9999);
    expect(useViewportStore.getState().timeline.currentFrame).toBe(300);
    useViewportStore.getState().seek(100);
    expect(useViewportStore.getState().timeline.currentFrame).toBe(100);
  });

  it('toggleLoop toggles', () => {
    act(() => useViewportStore.setState(initialState));
    useViewportStore.getState().toggleLoop();
    expect(useViewportStore.getState().playback.isLooping).toBe(false);
    useViewportStore.getState().toggleLoop();
    expect(useViewportStore.getState().playback.isLooping).toBe(true);
  });

  it('setPlaybackSpeed', () => {
    act(() => useViewportStore.setState(initialState));
    useViewportStore.getState().setPlaybackSpeed(2);
    expect(useViewportStore.getState().playback.playbackSpeed).toBe(2);
  });

  it('loadState populates from invoke', async () => {
    act(() => useViewportStore.setState(initialState));
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke)
      .mockResolvedValueOnce({
        current_frame: 10,
        total_frames: 100,
        fps: 24,
        is_playing: false,
        active_figure: null,
      })
      .mockResolvedValueOnce({
        playing: true,
        current_time: 5,
        duration: 12,
        loop_enabled: false,
        speed: 1.5,
      })
      .mockResolvedValueOnce([
        { name: 'Pose One', file_path: '/a.dsf', compatible_figures: [], category: 'test' },
      ]);
    await useViewportStore.getState().loadState();
    const s = useViewportStore.getState();
    expect(s.timeline.currentFrame).toBe(10);
    expect(s.timeline.totalFrames).toBe(100);
    expect(s.timeline.fps).toBe(24);
    expect(s.timeline.duration).toBe(12);
    expect(s.playback.isPlaying).toBe(true);
    expect(s.playback.isLooping).toBe(false);
    expect(s.playback.playbackSpeed).toBe(1.5);
    expect(s.poses).toHaveLength(1);
    expect(s.poses[0].name).toBe('Pose One');
  });

  it('loadState sets error on failure', async () => {
    act(() => useViewportStore.setState(initialState));
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockRejectedValue(new Error('timeline fail'));
    await useViewportStore.getState().loadState();
    expect(useViewportStore.getState().error).toBe('Error: timeline fail');
  });

  it('reset restores initial state', () => {
    act(() => useViewportStore.setState({ ...initialState, cameraPreset: 'top' }));
    useViewportStore.getState().reset();
    expect(useViewportStore.getState().cameraPreset).toBe('front');
  });
});
