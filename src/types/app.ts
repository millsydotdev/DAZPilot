export type AppTab =
  | 'chat'
  | 'assets'
  | 'viewport'
  | 'scene'
  | 'scratchpad'
  | 'presets'
  | 'settings'
  | 'tutorial';

export interface LauncherCompleteOptions {
  tab?: AppTab;
}
