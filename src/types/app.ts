export type AppTab =
  | 'chat'
  | 'assets'
  | 'viewport'
  | 'scene'
  | 'scratchpad'
  | 'presets'
  | 'compose'
  | 'settings'
  | 'tutorial';

export interface LauncherCompleteOptions {
  tab?: AppTab;
}
