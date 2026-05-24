export type AppTab =
  | 'chat'
  | 'assets'
  | 'viewport'
  | 'scene'
  | 'scratchpad'
  | 'presets'
  | 'settings';

export interface LauncherCompleteOptions {
  tab?: AppTab;
}
