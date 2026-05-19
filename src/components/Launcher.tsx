import { useState, useEffect, useCallback, type ReactNode } from 'react';
import { check } from '@tauri-apps/plugin-updater';
import { relaunch } from '@tauri-apps/plugin-process';
import { getVersion } from '@tauri-apps/api/app';
import {
  Settings,
  FileText,
  Github,
  RefreshCw,
  ArrowRight,
  Cpu,
  Radio,
  Zap,
  CheckCircle,
  XCircle,
} from 'lucide-react';
import { DazPilotLogo } from './brand/DazPilotLogo';
import { cn } from '../utils/cn';
import type { AppTab, LauncherCompleteOptions } from '../types/app';

interface LauncherProps {
  onComplete: (options?: LauncherCompleteOptions) => void;
  aiServerRunning: boolean;
  pluginInstalled: boolean;
  dazStudioConnected: boolean;
}

type UpdateStatus =
  | 'initializing'
  | 'checking'
  | 'downloading'
  | 'installing'
  | 'ready'
  | 'error'
  | 'relaunching';

function openExternal(url: string) {
  window.open(url, '_blank', 'noopener,noreferrer');
}

function StatusPill({ label, ok, icon }: { label: string; ok: boolean; icon: ReactNode }) {
  return (
    <div className="flex items-center justify-between gap-3 rounded-md border border-border-subtle bg-surface/80 px-3 py-2">
      <div className="flex items-center gap-2 text-xs text-zinc-400">
        {icon}
        <span>{label}</span>
      </div>
      <div className="flex items-center gap-1.5 text-xs font-medium">
        {ok ? (
          <>
            <CheckCircle size={14} className="text-cyan" />
            <span className="text-zinc-300">Online</span>
          </>
        ) : (
          <>
            <XCircle size={14} className="text-accent" />
            <span className="text-zinc-500">Offline</span>
          </>
        )}
      </div>
    </div>
  );
}

export function Launcher({
  onComplete,
  aiServerRunning,
  pluginInstalled,
  dazStudioConnected,
}: LauncherProps) {
  const [status, setStatus] = useState<UpdateStatus>('initializing');
  const [statusMessage, setStatusMessage] = useState('Initializing DazPilot...');
  const [progress, setProgress] = useState(0);
  const [version, setVersion] = useState('');
  const [isRechecking, setIsRechecking] = useState(false);

  const enterWorkspace = useCallback(
    (tab?: AppTab) => {
      onComplete(tab ? { tab } : undefined);
    },
    [onComplete]
  );

  const runUpdateCheck = useCallback(async (manual = false) => {
    try {
      if (manual) setIsRechecking(true);
      setStatus('checking');
      setStatusMessage('Checking for updates...');
      setProgress(15);

      const update = await check();

      if (update) {
        setStatusMessage(`Update available: v${update.version}`);
        setProgress(30);
        setStatus('downloading');
        setStatusMessage('Downloading update...');

        let downloaded = 0;
        let contentLength = 0;

        await update.downloadAndInstall((event) => {
          switch (event.event) {
            case 'Started':
              contentLength = event.data.contentLength ?? 0;
              break;
            case 'Progress':
              downloaded += event.data.chunkLength;
              if (contentLength > 0) {
                const p = Math.round((downloaded / contentLength) * 100);
                setProgress(30 + p * 0.5);
                setStatusMessage(`Downloading update... ${p}%`);
              }
              break;
            case 'Finished':
              setProgress(85);
              setStatus('installing');
              setStatusMessage('Installing update...');
              break;
          }
        });

        setProgress(100);
        setStatus('relaunching');
        setStatusMessage('Update installed. Relaunching...');
        setTimeout(() => {
          void relaunch();
        }, 1500);
        return;
      }

      setStatus('ready');
      setStatusMessage('You are up to date');
      setProgress(100);
    } catch (error) {
      console.error('Update error:', error);
      setStatus('error');
      setStatusMessage('Could not check for updates');
      setProgress(100);
    } finally {
      if (manual) setIsRechecking(false);
    }
  }, []);

  useEffect(() => {
    const init = async () => {
      try {
        const v = await getVersion();
        setVersion(v);
      } catch {
        setVersion('0.1.0');
      }
      const timer = setTimeout(() => {
        void runUpdateCheck();
      }, 1800);
      return () => clearTimeout(timer);
    };
    void init();
  }, [runUpdateCheck]);

  const showProgress = status !== 'ready' && status !== 'relaunching';
  const showReady = status === 'ready' || status === 'error';

  return (
    <div className="fixed inset-0 z-[9999] flex flex-col items-center justify-center overflow-hidden bg-void text-white select-none">
      <div
        className="pointer-events-none absolute inset-0 opacity-[0.03]"
        style={{
          backgroundImage: `
            linear-gradient(rgba(255,255,255,0.5) 1px, transparent 1px),
            linear-gradient(90deg, rgba(255,255,255,0.5) 1px, transparent 1px)
          `,
          backgroundSize: '48px 48px',
        }}
      />
      <div className="pointer-events-none absolute top-1/2 left-1/2 h-[420px] w-[420px] -translate-x-1/2 -translate-y-1/2 rounded-full bg-accent/10 blur-[100px]" />

      <div className="relative z-10 flex animate-fade-in flex-col items-center px-6">
        <DazPilotLogo size={72} showWordmark />
        <p className="mt-3 font-mono text-[11px] uppercase tracking-[0.25em] text-zinc-500">
          AI Scene Control System
        </p>
      </div>

      <div
        className={cn(
          'relative z-10 mt-14 w-full max-w-sm px-4 transition-all duration-500',
          showProgress ? 'opacity-100' : 'pointer-events-none h-0 opacity-0'
        )}
      >
        <div className="mb-2 flex items-end justify-between">
          <span
            className={cn(
              'text-xs font-medium transition-colors duration-300',
              status === 'error' ? 'text-amber-400' : 'text-zinc-400'
            )}
          >
            {statusMessage}
          </span>
          {showProgress && (
            <span className="font-mono text-[10px] text-zinc-600">{Math.round(progress)}%</span>
          )}
        </div>
        <div className="dp-progress-track">
          <div className="dp-progress-fill" style={{ width: `${progress}%` }} />
        </div>
      </div>

      <div
        className={cn(
          'relative z-10 mt-10 flex w-full max-w-md flex-col items-center gap-6 px-4 transition-all duration-500',
          showReady ? 'translate-y-0 opacity-100' : 'pointer-events-none translate-y-4 opacity-0'
        )}
      >
        <button
          type="button"
          className="dp-btn-primary group flex w-full max-w-xs items-center justify-center gap-2 py-3"
          onClick={() => enterWorkspace()}
        >
          Enter Workspace
          <ArrowRight size={18} className="transition-transform group-hover:translate-x-0.5" />
        </button>

        <div className="flex flex-wrap items-center justify-center gap-2">
          <button
            type="button"
            className="dp-btn-ghost gap-2 px-3 py-2 text-xs"
            onClick={() => enterWorkspace('settings')}
          >
            <Settings size={14} />
            Settings
          </button>
          <button
            type="button"
            className="dp-btn-ghost gap-2 px-3 py-2 text-xs"
            onClick={() => enterWorkspace('settings')}
          >
            <FileText size={14} />
            Logs
          </button>
          <button
            type="button"
            className="dp-btn-ghost gap-2 px-3 py-2 text-xs"
            onClick={() => openExternal('https://github.com/millsydotdev/DazPilot')}
          >
            <Github size={14} />
            GitHub
          </button>
          <button
            type="button"
            className="dp-btn-ghost gap-2 px-3 py-2 text-xs"
            disabled={isRechecking}
            onClick={() => void runUpdateCheck(true)}
          >
            <RefreshCw size={14} className={cn(isRechecking && 'animate-spin')} />
            Check updates
          </button>
        </div>

        <div className="w-full space-y-2 border-t border-border-subtle pt-4">
          <p className="mb-2 text-center font-mono text-[10px] uppercase tracking-widest text-zinc-600">
            System Status
          </p>
          <StatusPill label="AI Server" ok={aiServerRunning} icon={<Cpu size={14} />} />
          <StatusPill label="Bridge Plugin" ok={pluginInstalled} icon={<Zap size={14} />} />
          <StatusPill label="Daz Studio" ok={dazStudioConnected} icon={<Radio size={14} />} />
        </div>
      </div>

      <div className="absolute bottom-8 flex flex-col items-center gap-1 font-mono text-[10px] font-medium uppercase tracking-widest text-zinc-600">
        <p className="text-[9px] text-zinc-700">
          DazPilot is an independent third-party project and is not endorsed by Daz 3D.
        </p>
        <span>Version {version || '—'}</span>
      </div>
    </div>
  );
}
