import { create } from 'zustand';

export interface LogEntry {
  id: string;
  timestamp: string;
  level: 'debug' | 'info' | 'warn' | 'error';
  category: 'system' | 'ai' | 'connection' | 'database' | 'viewport';
  message: string;
}

export interface LogState {
  logs: LogEntry[];
  maxLogs: number;
  autoScroll: boolean;
}

export interface LogActions {
  addLog: (level: LogEntry['level'], category: LogEntry['category'], message: string) => void;
  clearLogs: () => void;
  setAutoScroll: (autoScroll: boolean) => void;
  exportLogs: () => void;
}

const getLogCategory = (message: string): LogEntry['category'] => {
  const msg = message.toLowerCase();
  if (
    msg.includes('[ai]') ||
    msg.includes('ollama') ||
    msg.includes('llama') ||
    msg.includes('model') ||
    msg.includes('gguf')
  ) {
    return 'ai';
  }
  if (
    msg.includes('connection') ||
    msg.includes('daz3d') ||
    msg.includes('port') ||
    msg.includes('host') ||
    msg.includes('connect')
  ) {
    return 'connection';
  }
  if (
    msg.includes('database') ||
    msg.includes('sqlite') ||
    msg.includes('query') ||
    msg.includes('db')
  ) {
    return 'database';
  }
  if (
    msg.includes('viewport') ||
    msg.includes('live link') ||
    msg.includes('canvas') ||
    msg.includes('sync')
  ) {
    return 'viewport';
  }
  return 'system';
};

export const useLogStore = create<LogState & LogActions>((set, get) => {
  return {
    logs: [
      {
        id: 'initial',
        timestamp: new Date().toLocaleTimeString(),
        level: 'info',
        category: 'system',
        message: 'DazPilot System Logger initialized successfully.',
      },
    ],
    maxLogs: 1000,
    autoScroll: true,

    addLog: (level, category, message) => {
      const timestamp = new Date().toLocaleTimeString();
      const newEntry: LogEntry = {
        id: Math.random().toString(36).substring(2, 9),
        timestamp,
        level,
        category,
        message,
      };

      set((state) => {
        const nextLogs = [...state.logs, newEntry];
        if (nextLogs.length > state.maxLogs) {
          nextLogs.shift();
        }
        return { logs: nextLogs };
      });
    },

    clearLogs: () => {
      set({
        logs: [
          {
            id: `clear-${Date.now()}`,
            timestamp: new Date().toLocaleTimeString(),
            level: 'info',
            category: 'system',
            message: 'Log console buffer cleared.',
          },
        ],
      });
    },

    setAutoScroll: (autoScroll) => set({ autoScroll }),

    exportLogs: () => {
      const { logs } = get();
      const text = logs
        .map(
          (log) =>
            `[${log.timestamp}] [${log.level.toUpperCase()}] [${log.category.toUpperCase()}] ${log.message}`
        )
        .join('\n');

      const blob = new Blob([text], { type: 'text/plain;charset=utf-8' });
      const url = URL.createObjectURL(blob);
      const link = document.createElement('a');
      link.href = url;
      link.download = `dazpilot-system-logs-${Date.now()}.txt`;
      document.body.appendChild(link);
      link.click();
      document.body.removeChild(link);
      URL.revokeObjectURL(url);
    },
  };
});

// Graceful console interception to route logs directly into our store
export const initializeConsoleInterceptor = () => {
  if ((window as any).__console_intercepted) return;
  (window as any).__console_intercepted = true;

  const originalLog = console.log;
  const originalWarn = console.warn;
  const originalError = console.error;
  const originalDebug = console.debug;

  const logToStore = (level: LogEntry['level'], args: any[]) => {
    try {
      const message = args
        .map((arg) => (typeof arg === 'object' ? JSON.stringify(arg) : String(arg)))
        .join(' ');

      const category = getLogCategory(message);
      useLogStore.getState().addLog(level, category, message);
    } catch {
      // Avoid infinite cycles or crashes during serialisation
    }
  };

  console.log = (...args) => {
    originalLog.apply(console, args);
    logToStore('info', args);
  };

  console.warn = (...args) => {
    originalWarn.apply(console, args);
    logToStore('warn', args);
  };

  console.error = (...args) => {
    originalError.apply(console, args);
    logToStore('error', args);
  };

  console.debug = (...args) => {
    originalDebug.apply(console, args);
    logToStore('debug', args);
  };
};
