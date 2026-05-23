import { useState, useEffect, useRef } from 'react';
import {
  Play,
  Square,
  Repeat,
  ZoomIn,
  ZoomOut,
  RotateCw,
  Move,
  Camera,
  FolderOpen,
  RefreshCw,
  AlertCircle,
} from 'lucide-react';
import { useViewportStore } from '../../store';
import { Button } from '../ui';
import { listen } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';
import LiveLinkPanel from './LiveLinkPanel';
import { KeyframeEditor } from '../animation/KeyframeEditor';
import { PoseLibrary } from '../animation/PoseLibrary';
import { PhysicsControls } from '../physics/PhysicsControls';
import styles from './ViewportCanvas.module.css';

export default function ViewportCanvas() {
  const {
    timeline,
    playback,
    showPoseLibrary,
    togglePoseLibrary,
    play,
    stop,
    toggleLoop,
    loadState,
    syncFps,
  } = useViewportStore();

  const [zoom, setZoom] = useState(100);
  const [activeTool, setActiveTool] = useState('select');
  const [viewportImage, setViewportImage] = useState<string | null>(null);
  const [isSyncing, setIsSyncing] = useState(false);
  const [showLiveLink, setShowLiveLink] = useState(false);
  const [syncStatus, setSyncStatus] = useState<'idle' | 'connected' | 'error'>('idle');
  const [syncError, setSyncError] = useState<string | null>(null);
  const [isCapturingFrame, setIsCapturingFrame] = useState(false);
  const loadStateRef = useRef(loadState);

  useEffect(() => {
    loadStateRef.current();

    const unlisten1 = listen<{ image: string }>('viewport-update', (event) => {
      setViewportImage(`data:image/png;base64,${event.payload.image}`);
      setSyncStatus('connected');
      setSyncError(null);
    });

    const unlisten2 = listen<{ error: string; failures: number }>('viewport-error', (event) => {
      setSyncStatus('error');
      setSyncError(event.payload.error);
    });

    return () => {
      unlisten1.then((fn) => fn());
      unlisten2.then((fn) => fn());
    };
  }, []);

  const toggleSync = async () => {
    const newState = !isSyncing;
    setIsSyncing(newState);
    if (newState) {
      setSyncStatus('idle');
      await invoke('set_viewport_fps', { fps: syncFps });
    } else {
      setSyncStatus('idle');
    }
    await invoke('set_viewport_sync', { enabled: newState });
  };

  const captureFrame = async () => {
    setIsCapturingFrame(true);
    setSyncError(null);
    try {
      const base64 = await invoke<string>('capture_viewport_single');
      setViewportImage(`data:image/png;base64,${base64}`);
      setSyncStatus('connected');
    } catch (err) {
      setSyncStatus('error');
      setSyncError(String(err));
    } finally {
      setIsCapturingFrame(false);
    }
  };

  const formatTime = (frame: number) => {
    const seconds = Math.floor(frame / timeline.fps);
    const minutes = Math.floor(seconds / 60);
    const remainingSeconds = seconds % 60;
    const remainingFrames = frame % timeline.fps;
    return `${minutes.toString().padStart(2, '0')}:${remainingSeconds.toString().padStart(2, '0')}:${remainingFrames.toString().padStart(2, '0')}`;
  };

  const progress = (timeline.currentFrame / timeline.totalFrames) * 100;

  const handleImageClick = async (e: React.MouseEvent<HTMLDivElement>) => {
    if (activeTool !== 'select') return;

    const img = e.currentTarget.querySelector('img');
    if (!img) return;

    const rect = img.getBoundingClientRect();
    const x = e.clientX - rect.left;
    const y = e.clientY - rect.top;

    const scaleX = img.naturalWidth / rect.width;
    const scaleY = img.naturalHeight / rect.height;

    const imageX = Math.round(x * scaleX);
    const imageY = Math.round(y * scaleY);

    try {
      await invoke('send_daz3d_command', {
        command: 'viewport_click',
        args: { x: imageX.toString(), y: imageY.toString() },
      });
    } catch (err) {
      console.error('Viewport click failed:', err);
    }
  };

  return (
    <div className={styles.container}>
      <div className={styles.toolbar}>
        <div className={styles.toolGroup}>
          <button
            className={`${styles.toolButton} ${activeTool === 'select' ? styles.active : ''}`}
            onClick={() => setActiveTool('select')}
            aria-label="Select"
          >
            <Move size={16} />
            Select
          </button>
          <button
            className={`${styles.toolButton} ${activeTool === 'rotate' ? styles.active : ''}`}
            onClick={() => setActiveTool('rotate')}
            aria-label="Rotate"
          >
            <RotateCw size={16} />
            Rotate
          </button>
          <button
            className={`${styles.toolButton} ${activeTool === 'zoom' ? styles.active : ''}`}
            onClick={() => setActiveTool('zoom')}
            aria-label="Zoom"
          >
            <ZoomIn size={16} />
            Zoom
          </button>
          <button
            className={`${styles.toolButton} ${activeTool === 'camera' ? styles.active : ''}`}
            onClick={() => setActiveTool('camera')}
            aria-label="Camera"
          >
            <Camera size={16} />
            Camera
          </button>
        </div>

        <div className={styles.toolGroup}>
          <button
            className={`${styles.toolButton} ${showLiveLink ? styles.active : ''}`}
            onClick={() => setShowLiveLink(!showLiveLink)}
            title="Live Link: Face Tracking"
            aria-label="Live Link"
          >
            <Camera size={16} />
            Live Link
          </button>
        </div>

        <div className={styles.toolGroup}>
          <button
            className={styles.toolButton}
            onClick={() => setZoom((z) => Math.max(10, z - 10))}
          >
            <ZoomOut size={16} />
          </button>
          <span className={styles.zoomLabel}>{zoom}%</span>
          <button
            className={styles.toolButton}
            onClick={() => setZoom((z) => Math.min(200, z + 10))}
          >
            <ZoomIn size={16} />
          </button>
        </div>
      </div>

      <div className={styles.viewport}>
        {viewportImage ? (
          <div className={styles.viewportImageContainer}>
            <div
              role="button"
              tabIndex={0}
              onClick={handleImageClick}
              onKeyDown={(e) => {
                if (e.key === 'Enter' || e.key === ' ') {
                  const rect = e.currentTarget.getBoundingClientRect();
                  handleImageClick({
                    ...e,
                    clientX: rect.left + rect.width / 2,
                    clientY: rect.top + rect.height / 2,
                  } as unknown as React.MouseEvent<HTMLDivElement>);
                }
              }}
            >
              <img
                src={viewportImage}
                className={styles.viewportImage}
                style={{
                  transform: `scale(${zoom / 100})`,
                  cursor: activeTool === 'select' ? 'crosshair' : 'default',
                }}
                alt="Daz Viewport"
                aria-label="Daz Studio viewport"
              />
            </div>
            {isSyncing && syncStatus === 'connected' && (
              <div className={styles.viewportStatusBadge}>
                <span className={styles.statusLiveDot} />
                Live
              </div>
            )}
            {isSyncing && syncStatus === 'error' && (
              <div className={styles.viewportStatusBadgeError}>
                <AlertCircle size={10} />
                Sync Error
              </div>
            )}
          </div>
        ) : (
          <div className={styles.syncHudContainer}>
            <div className={styles.radarRing}>
              <div className={styles.radarPulse}></div>
              <FolderOpen size={36} className={styles.radarIcon} />
            </div>
            <h3 className={styles.hudTitle}>Viewport Streaming Offline</h3>
            <p className={styles.hudSubtitle}>
              {syncStatus === 'error'
                ? 'Bridge connection error. Check Daz Studio is running with the plugin loaded.'
                : 'Parity sync between Daz Studio and DazPilot is currently inactive.'}
            </p>

            {syncStatus === 'error' && syncError && (
              <div className={styles.hudError}>
                <AlertCircle size={14} />
                {syncError}
              </div>
            )}

            <div className={styles.hudSteps}>
              <div className={styles.hudStep}>
                <span className={styles.hudStepNum}>1</span>
                <span>
                  Load <strong>VibeBridgePlugin</strong> inside Daz Studio
                </span>
              </div>
              <div className={styles.hudStep}>
                <span className={styles.hudStepNum}>2</span>
                <span>Verify port connection in Settings (Default: localhost:8765)</span>
              </div>
              <div className={styles.hudStep}>
                <span className={styles.hudStepNum}>3</span>
                <span>
                  Click <strong>Capture Frame</strong> to test, or <strong>Sync Viewport</strong>{' '}
                  for live
                </span>
              </div>
            </div>
          </div>
        )}

        <div className={styles.syncOverlay}>
          <Button
            size="sm"
            variant="ghost"
            onClick={captureFrame}
            disabled={isCapturingFrame}
            aria-disabled={isCapturingFrame}
            aria-label="Capture frame"
            className={styles.captureButton}
          >
            <RefreshCw size={12} className={isCapturingFrame ? styles.spin : ''} />
            {isCapturingFrame ? 'Capturing...' : 'Capture Frame'}
          </Button>
          <Button
            size="sm"
            variant={isSyncing ? 'primary' : 'secondary'}
            onClick={toggleSync}
            aria-label="Sync viewport"
            className={styles.syncButton}
          >
            {isSyncing ? 'Syncing...' : 'Sync Viewport'}
          </Button>
        </div>

        {showPoseLibrary && <PoseLibrary />}

        {showLiveLink && <LiveLinkPanel onClose={() => setShowLiveLink(false)} />}
      </div>

      {/* Physics Controls Panel */}
      <div className={styles.physicsControlsPanel}>
        <PhysicsControls />
      </div>

      {/* Keyframe Editor Panel */}
      <div className={styles.keyframeEditorPanel}>
        <KeyframeEditor />
      </div>

      <div className={styles.timeline}>
        <div className={styles.controls}>
          <button className={styles.controlButton} onClick={stop} aria-label="Stop">
            <Square size={14} />
          </button>
          <button className={styles.controlButton} onClick={play} aria-label="Play">
            <Play size={14} />
          </button>
          <button
            className={`${styles.controlButton} ${playback.isLooping ? styles.active : ''}`}
            onClick={toggleLoop}
            aria-label="Loop"
          >
            <Repeat size={14} />
          </button>
        </div>

        <div className={styles.scrubber}>
          <div className={styles.scrubberProgress} style={{ width: `${progress}%` }} />
          <div className={styles.scrubberHandle} style={{ left: `${progress}%` }} />
        </div>

        <div className={styles.timeDisplay}>
          {formatTime(timeline.currentFrame)} / {formatTime(timeline.totalFrames)}
        </div>

        <Button size="sm" onClick={togglePoseLibrary}>
          Poses
        </Button>
      </div>
    </div>
  );
}
