import { describe, it, expect, vi, beforeEach } from 'vitest';
import { act } from 'react';
import { useWebcamStore } from './webcamStore';

const initialState = {
  selectedDeviceId: '',
  resolutionMode: 'auto' as const,
  customWidth: 640,
  customHeight: 480,
  framerateMode: 'auto' as const,
  customFramerate: 30,
  mirrorEnabled: true,
  autoStartLiveLink: false,
  availableDevices: [],
  actualWidth: 0,
  actualHeight: 0,
  actualFramerate: 0,
};

describe('webcamStore', () => {
  beforeEach(async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockResolvedValue(undefined);
  });

  it('setSelectedDeviceId', () => {
    act(() => useWebcamStore.setState(initialState));
    useWebcamStore.getState().setSelectedDeviceId('cam1');
    expect(useWebcamStore.getState().selectedDeviceId).toBe('cam1');
  });

  it('setResolutionMode', () => {
    act(() => useWebcamStore.setState(initialState));
    useWebcamStore.getState().setResolutionMode('best');
    expect(useWebcamStore.getState().resolutionMode).toBe('best');
    useWebcamStore.getState().setResolutionMode('custom');
    expect(useWebcamStore.getState().resolutionMode).toBe('custom');
  });

  it('setCustomResolution', () => {
    act(() => useWebcamStore.setState(initialState));
    useWebcamStore.getState().setCustomResolution(1920, 1080);
    expect(useWebcamStore.getState().customWidth).toBe(1920);
    expect(useWebcamStore.getState().customHeight).toBe(1080);
  });

  it('setFramerateMode', () => {
    act(() => useWebcamStore.setState(initialState));
    useWebcamStore.getState().setFramerateMode('custom');
    expect(useWebcamStore.getState().framerateMode).toBe('custom');
  });

  it('setCustomFramerate', () => {
    act(() => useWebcamStore.setState(initialState));
    useWebcamStore.getState().setCustomFramerate(60);
    expect(useWebcamStore.getState().customFramerate).toBe(60);
  });

  it('setMirrorEnabled', () => {
    act(() => useWebcamStore.setState(initialState));
    useWebcamStore.getState().setMirrorEnabled(false);
    expect(useWebcamStore.getState().mirrorEnabled).toBe(false);
  });

  it('setAutoStartLiveLink', () => {
    act(() => useWebcamStore.setState(initialState));
    useWebcamStore.getState().setAutoStartLiveLink(true);
    expect(useWebcamStore.getState().autoStartLiveLink).toBe(true);
  });

  it('setAvailableDevices', () => {
    act(() => useWebcamStore.setState(initialState));
    const devices = [
      { deviceId: 'cam1', kind: 'videoinput', label: 'Cam 1', groupId: 'g1' },
    ] as MediaDeviceInfo[];
    useWebcamStore.getState().setAvailableDevices(devices);
    expect(useWebcamStore.getState().availableDevices).toHaveLength(1);
  });

  it('setActualResolution', () => {
    act(() => useWebcamStore.setState(initialState));
    useWebcamStore.getState().setActualResolution(1280, 720);
    expect(useWebcamStore.getState().actualWidth).toBe(1280);
    expect(useWebcamStore.getState().actualHeight).toBe(720);
  });

  it('setActualFramerate', () => {
    act(() => useWebcamStore.setState(initialState));
    useWebcamStore.getState().setActualFramerate(30);
    expect(useWebcamStore.getState().actualFramerate).toBe(30);
  });

  it('loadSettings populates from invoke', async () => {
    act(() => useWebcamStore.setState(initialState));
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockReset();
    vi.mocked(invoke).mockResolvedValueOnce('cam1');
    vi.mocked(invoke).mockResolvedValueOnce('true');
    vi.mocked(invoke).mockResolvedValueOnce('true');
    await useWebcamStore.getState().loadSettings();
    expect(useWebcamStore.getState().selectedDeviceId).toBe('cam1');
    expect(useWebcamStore.getState().mirrorEnabled).toBe(true);
    expect(useWebcamStore.getState().autoStartLiveLink).toBe(true);
  });

  it('loadSettings handles null values', async () => {
    act(() => useWebcamStore.setState(initialState));
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockReset();
    vi.mocked(invoke).mockResolvedValueOnce(null);
    vi.mocked(invoke).mockResolvedValueOnce(null);
    vi.mocked(invoke).mockResolvedValueOnce(null);
    await useWebcamStore.getState().loadSettings();
    expect(useWebcamStore.getState().selectedDeviceId).toBe('');
    expect(useWebcamStore.getState().mirrorEnabled).toBe(true);
    expect(useWebcamStore.getState().autoStartLiveLink).toBe(false);
  });

  it('loadSettings handles error', async () => {
    act(() => useWebcamStore.setState(initialState));
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockRejectedValue(new Error('fail'));
    await useWebcamStore.getState().loadSettings();
    expect(useWebcamStore.getState().selectedDeviceId).toBe('');
  });

  it('reset restores initial state', () => {
    act(() => useWebcamStore.setState(initialState));
    useWebcamStore.getState().setCustomResolution(1920, 1080);
    useWebcamStore.getState().reset();
    expect(useWebcamStore.getState().customWidth).toBe(640);
    expect(useWebcamStore.getState().customHeight).toBe(480);
  });
});
