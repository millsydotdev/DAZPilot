export type ResolutionPreset = [number, number, string];

export const RESOLUTION_PRESETS: ResolutionPreset[] = [
  [320, 240, '320 × 240 (QVGA)'],
  [640, 480, '640 × 480 (VGA)'],
  [800, 600, '800 × 600 (SVGA)'],
  [1280, 720, '1280 × 720 (HD)'],
  [1280, 960, '1280 × 960'],
  [1600, 1200, '1600 × 1200 (UXGA)'],
  [1920, 1080, '1920 × 1080 (Full HD)'],
  [2560, 1440, '2560 × 1440 (QHD)'],
  [3840, 2160, '3840 × 2160 (4K UHD)'],
];

export const FRAMERATE_PRESETS = [15, 24, 25, 30, 48, 50, 60];

export async function enumerateVideoDevices(): Promise<MediaDeviceInfo[]> {
  const devices = await navigator.mediaDevices.enumerateDevices();
  return devices.filter((d) => d.kind === 'videoinput');
}

export async function getDeviceCapabilities(
  deviceId: string
): Promise<MediaTrackCapabilities | null> {
  try {
    const stream = await navigator.mediaDevices.getUserMedia({
      video: deviceId ? { deviceId: { exact: deviceId } } : true,
    });
    const track = stream.getVideoTracks()[0];
    const capabilities = track.getCapabilities();
    track.stop();
    return capabilities;
  } catch {
    return null;
  }
}

export function getSupportedResolutions(capabilities: MediaTrackCapabilities): ResolutionPreset[] {
  const supported: ResolutionPreset[] = [];

  if (!capabilities.width || !capabilities.height) return RESOLUTION_PRESETS;

  const minW = typeof capabilities.width.min === 'number' ? capabilities.width.min : 0;
  const maxW = typeof capabilities.width.max === 'number' ? capabilities.width.max : 3840;
  const minH = typeof capabilities.height.min === 'number' ? capabilities.height.min : 0;
  const maxH = typeof capabilities.height.max === 'number' ? capabilities.height.max : 2160;

  for (const [w, h, label] of RESOLUTION_PRESETS) {
    if (w >= minW && w <= maxW && h >= minH && h <= maxH) {
      supported.push([w, h, label]);
    }
  }

  return supported;
}

export function getSupportedFramerates(capabilities: MediaTrackCapabilities): number[] {
  if (!capabilities.frameRate) return FRAMERATE_PRESETS;

  const minFps = typeof capabilities.frameRate.min === 'number' ? capabilities.frameRate.min : 0;
  const maxFps = typeof capabilities.frameRate.max === 'number' ? capabilities.frameRate.max : 60;

  return FRAMERATE_PRESETS.filter((fps) => fps >= minFps && fps <= maxFps);
}

export interface WebcamConstraints {
  selectedDeviceId?: string;
  resolutionMode?: 'auto' | 'best' | 'custom';
  customWidth?: number;
  customHeight?: number;
  framerateMode?: 'auto' | 'custom';
  customFramerate?: number;
}

export function buildConstraints(settings: WebcamConstraints): MediaStreamConstraints {
  const video: MediaTrackConstraints = {};

  if (settings.selectedDeviceId) {
    video.deviceId = { exact: settings.selectedDeviceId };
  }

  if (settings.resolutionMode === 'custom' && settings.customWidth && settings.customHeight) {
    video.width = { exact: settings.customWidth };
    video.height = { exact: settings.customHeight };
  } else if (settings.resolutionMode === 'best') {
    video.width = { ideal: 3840 };
    video.height = { ideal: 2160 };
  }

  if (settings.framerateMode === 'custom' && settings.customFramerate) {
    video.frameRate = { exact: settings.customFramerate };
  }

  return { video };
}

export async function testCamera(
  deviceId: string,
  settings: WebcamConstraints
): Promise<{ stream: MediaStream; width: number; height: number; frameRate: number } | null> {
  try {
    const constraints = buildConstraints({ ...settings, selectedDeviceId: deviceId });
    const stream = await navigator.mediaDevices.getUserMedia(constraints);
    const track = stream.getVideoTracks()[0];
    const settings_ = track.getSettings();
    return {
      stream,
      width: settings_.width || 0,
      height: settings_.height || 0,
      frameRate: settings_.frameRate || 0,
    };
  } catch {
    return null;
  }
}
