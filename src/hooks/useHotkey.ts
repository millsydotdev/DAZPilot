import { useEffect, useCallback } from 'react';

type KeyHandler = () => void;

interface HotkeyConfig {
  key: string;
  ctrl?: boolean;
  shift?: boolean;
  alt?: boolean;
  handler: KeyHandler;
}

export function useHotkeys(hotkeys: HotkeyConfig[]) {
  const handleKeyDown = useCallback(
    (event: KeyboardEvent) => {
      for (const hotkey of hotkeys) {
        const ctrlMatch = hotkey.ctrl
          ? event.ctrlKey || event.metaKey
          : !event.ctrlKey && !event.metaKey;
        const shiftMatch = hotkey.shift ? event.shiftKey : !event.shiftKey;
        const altMatch = hotkey.alt ? event.altKey : !event.altKey;
        const keyMatch = event.key.toLowerCase() === hotkey.key.toLowerCase();

        if (keyMatch && ctrlMatch && shiftMatch && altMatch) {
          event.preventDefault();
          hotkey.handler();
          break;
        }
      }
    },
    [hotkeys]
  );

  useEffect(() => {
    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [handleKeyDown]);
}
