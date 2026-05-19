import { create } from 'zustand';

export interface WebcamSettings {
  selectedDeviceId: string;
  resolutionMode: 'auto' | 'best' | 'custom';
  customWidth: number;
  customHeight: number;
  framerateMode: 'auto' | 'custom';
  customFramerate: number;
  mirrorEnabled: boolean;
  autoStartLiveLink: boolean;
}

export interface WebcamState extends WebcamSettings {
  availableDevices: MediaDeviceInfo[];
  actualWidth: number;
  actualHeight: number;
  actualFramerate: number;
}

export interface WebcamActions {
  setSelectedDeviceId: (deviceId: string) => void;
  setResolutionMode: (mode: 'auto' | 'best' | 'custom') => void;
  setCustomResolution: (width: number, height: number) => void;
  setFramerateMode: (mode: 'auto' | 'custom') => void;
  setCustomFramerate: (fps: number) => void;
  setMirrorEnabled: (enabled: boolean) => void;
  setAutoStartLiveLink: (enabled: boolean) => void;
  setAvailableDevices: (devices: MediaDeviceInfo[]) => void;
  setActualResolution: (width: number, height: number) => void;
  setActualFramerate: (fps: number) => void;
  loadSettings: () => Promise<void>;
  reset: () => void;
}

const initialState: WebcamState = {
  selectedDeviceId: '',
  resolutionMode: 'auto',
  customWidth: 640,
  customHeight: 480,
  framerateMode: 'auto',
  customFramerate: 30,
  mirrorEnabled: true,
  autoStartLiveLink: false,
  availableDevices: [],
  actualWidth: 0,
  actualHeight: 0,
  actualFramerate: 0,
};

export const useWebcamStore = create<WebcamState & WebcamActions>((set) => ({
  ...initialState,

  setSelectedDeviceId: (selectedDeviceId) => {
    set({ selectedDeviceId });
    import('@tauri-apps/api/core').then(({ invoke }) => {
      invoke('save_app_setting', { key: 'webcam_device_id', value: selectedDeviceId }).catch(
        console.error
      );
    });
  },

  setResolutionMode: (resolutionMode) => set({ resolutionMode }),

  setCustomResolution: (customWidth, customHeight) => set({ customWidth, customHeight }),

  setFramerateMode: (framerateMode) => set({ framerateMode }),

  setCustomFramerate: (customFramerate) => set({ customFramerate }),

  setMirrorEnabled: (mirrorEnabled) => {
    set({ mirrorEnabled });
    import('@tauri-apps/api/core').then(({ invoke }) => {
      invoke('save_app_setting', { key: 'webcam_mirror', value: String(mirrorEnabled) }).catch(
        console.error
      );
    });
  },

  setAutoStartLiveLink: (autoStartLiveLink) => {
    set({ autoStartLiveLink });
    import('@tauri-apps/api/core').then(({ invoke }) => {
      invoke('save_app_setting', {
        key: 'webcam_autostart',
        value: String(autoStartLiveLink),
      }).catch(console.error);
    });
  },

  setAvailableDevices: (availableDevices) => set({ availableDevices }),

  setActualResolution: (width, height) => set({ actualWidth: width, actualHeight: height }),

  setActualFramerate: (actualFramerate) => set({ actualFramerate }),

  loadSettings: async () => {
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      const deviceId = await invoke<string | null>('get_app_setting', { key: 'webcam_device_id' });
      const mirror = await invoke<string | null>('get_app_setting', { key: 'webcam_mirror' });
      const autoStart = await invoke<string | null>('get_app_setting', { key: 'webcam_autostart' });
      set({
        selectedDeviceId: deviceId || '',
        mirrorEnabled: mirror !== 'false',
        autoStartLiveLink: autoStart === 'true',
      });
    } catch (e) {
      console.error('Failed to load webcam settings', e);
    }
  },

  reset: () => set(initialState),
}));
