import { describe, it, expect, vi, beforeEach } from 'vitest';
import {
  enumerateVideoDevices,
  getDeviceCapabilities,
  getSupportedResolutions,
  getSupportedFramerates,
  buildConstraints,
  testCamera,
  RESOLUTION_PRESETS,
  FRAMERATE_PRESETS,
} from './webcam';

function mockMediaDevices(overrides?: Partial<typeof navigator.mediaDevices>) {
  const defaults = {
    enumerateDevices: vi.fn(),
    getUserMedia: vi.fn(),
  };
  Object.assign(navigator, {
    mediaDevices: { ...defaults, ...overrides },
  });
}

describe('RESOLUTION_PRESETS', () => {
  it('contains expected resolutions', () => {
    expect(RESOLUTION_PRESETS[0]).toEqual([320, 240, '320 × 240 (QVGA)']);
    expect(RESOLUTION_PRESETS[3]).toEqual([1280, 720, '1280 × 720 (HD)']);
  });
});

describe('FRAMERATE_PRESETS', () => {
  it('contains common frame rates', () => {
    expect(FRAMERATE_PRESETS).toContain(30);
    expect(FRAMERATE_PRESETS).toContain(60);
  });
});

describe('enumerateVideoDevices', () => {
  beforeEach(() => {
    mockMediaDevices();
  });

  it('returns only video input devices', async () => {
    const devices = [
      { kind: 'videoinput', deviceId: 'cam1', label: 'Camera 1' },
      { kind: 'audioinput', deviceId: 'mic1', label: 'Mic 1' },
    ];
    navigator.mediaDevices.enumerateDevices = vi.fn().mockResolvedValue(devices);

    const result = await enumerateVideoDevices();
    expect(result).toHaveLength(1);
    expect(result[0].deviceId).toBe('cam1');
  });

  it('returns empty array when no video devices', async () => {
    navigator.mediaDevices.enumerateDevices = vi.fn().mockResolvedValue([]);
    const result = await enumerateVideoDevices();
    expect(result).toHaveLength(0);
  });
});

describe('getDeviceCapabilities', () => {
  beforeEach(() => {
    mockMediaDevices();
  });

  it('returns capabilities on success', async () => {
    const mockTrack = {
      getCapabilities: vi.fn().mockReturnValue({ width: { min: 320, max: 1920 } }),
      stop: vi.fn(),
    };
    navigator.mediaDevices.getUserMedia = vi.fn().mockResolvedValue({
      getVideoTracks: vi.fn().mockReturnValue([mockTrack]),
    });

    const caps = await getDeviceCapabilities('device1');
    expect(caps).toEqual({ width: { min: 320, max: 1920 } });
    expect(mockTrack.stop).toHaveBeenCalled();
  });

  it('returns null on error', async () => {
    navigator.mediaDevices.getUserMedia = vi.fn().mockRejectedValue(new Error('denied'));
    const caps = await getDeviceCapabilities('device1');
    expect(caps).toBeNull();
  });
});

describe('getSupportedResolutions', () => {
  it('returns all presets when capabilities lack width/height', () => {
    const result = getSupportedResolutions({});
    expect(result).toEqual(RESOLUTION_PRESETS);
  });

  it('filters based on capability range', () => {
    const caps = {
      width: { min: 640, max: 1280 },
      height: { min: 480, max: 720 },
    };
    const result = getSupportedResolutions(caps);
    expect(result).toEqual([
      [640, 480, '640 × 480 (VGA)'],
      [800, 600, '800 × 600 (SVGA)'],
      [1280, 720, '1280 × 720 (HD)'],
    ]);
  });
});

describe('getSupportedFramerates', () => {
  it('returns all presets when capabilities lack frameRate', () => {
    const result = getSupportedFramerates({});
    expect(result).toEqual(FRAMERATE_PRESETS);
  });

  it('filters based on frameRate range', () => {
    const caps = { frameRate: { min: 24, max: 48 } };
    const result = getSupportedFramerates(caps);
    expect(result).toEqual([24, 25, 30, 48]);
  });
});

describe('buildConstraints', () => {
  it('builds default constraint with no settings', () => {
    const result = buildConstraints({});
    expect(result).toEqual({ video: {} });
  });

  it('adds deviceId constraint', () => {
    const result = buildConstraints({ selectedDeviceId: 'cam1' });
    expect(result.video).toEqual({ deviceId: { exact: 'cam1' } });
  });

  it('sets exact resolution in custom mode', () => {
    const result = buildConstraints({
      resolutionMode: 'custom',
      customWidth: 1920,
      customHeight: 1080,
    });
    expect(result.video).toEqual({
      width: { exact: 1920 },
      height: { exact: 1080 },
    });
  });

  it('sets ideal resolution in best mode', () => {
    const result = buildConstraints({
      resolutionMode: 'best',
    });
    expect(result.video).toEqual({
      width: { ideal: 3840 },
      height: { ideal: 2160 },
    });
  });

  it('sets exact frameRate in custom mode', () => {
    const result = buildConstraints({
      framerateMode: 'custom',
      customFramerate: 60,
    });
    expect(result.video).toEqual({ frameRate: { exact: 60 } });
  });

  it('combines all constraints', () => {
    const result = buildConstraints({
      selectedDeviceId: 'cam1',
      resolutionMode: 'custom',
      customWidth: 1280,
      customHeight: 720,
      framerateMode: 'custom',
      customFramerate: 30,
    });
    expect(result.video).toEqual({
      deviceId: { exact: 'cam1' },
      width: { exact: 1280 },
      height: { exact: 720 },
      frameRate: { exact: 30 },
    });
  });
});

describe('testCamera', () => {
  beforeEach(() => {
    mockMediaDevices();
  });

  it('returns stream info on success', async () => {
    const mockTrack = {
      getSettings: vi.fn().mockReturnValue({ width: 640, height: 480, frameRate: 30 }),
      stop: vi.fn(),
    };
    const mockStream = {
      getVideoTracks: vi.fn().mockReturnValue([mockTrack]),
    };
    navigator.mediaDevices.getUserMedia = vi.fn().mockResolvedValue(mockStream);

    const result = await testCamera('cam1', { resolutionMode: 'best' });
    expect(result).not.toBeNull();
    expect(result!.width).toBe(640);
    expect(result!.height).toBe(480);
    expect(result!.frameRate).toBe(30);
  });

  it('returns null when getUserMedia fails', async () => {
    navigator.mediaDevices.getUserMedia = vi.fn().mockRejectedValue(new Error('not allowed'));
    const result = await testCamera('cam1', {});
    expect(result).toBeNull();
  });
});
