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
} from 'lucide-react';
import { useViewportStore } from '../../store';
import { Button } from '../ui';
import { listen } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';
import LiveLinkPanel from './LiveLinkPanel';
import styles from './ViewportCanvas.module.css';

export default function ViewportCanvas() {
  const {
    timeline,
    playback,
    poses,
    showPoseLibrary,
    togglePoseLibrary,
    play,
    stop,
    toggleLoop,
    loadState,
  } = useViewportStore();

  const [zoom, setZoom] = useState(100);
  const [activeTool, setActiveTool] = useState('select');
  const [viewportImage, setViewportImage] = useState<string | null>(null);
  const [isSyncing, setIsSyncing] = useState(false);
  const [showLiveLink, setShowLiveLink] = useState(false);
  const loadStateRef = useRef(loadState);

  useEffect(() => {
    loadStateRef.current();

    const unlisten = listen<{ image: string }>('viewport-update', (event) => {
      setViewportImage(`data:image/png;base64,${event.payload.image}`);
    });

    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  const toggleSync = async () => {
    const newState = !isSyncing;
    setIsSyncing(newState);
    await invoke('set_viewport_sync', { enabled: newState });
    if (newState) {
      await invoke('set_viewport_fps', { fps: 2 });
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

  const handleImageClick = async (e: React.MouseEvent<HTMLImageElement>) => {
    if (activeTool !== 'select') return;

    const rect = e.currentTarget.getBoundingClientRect();
    const x = e.clientX - rect.left;
    const y = e.clientY - rect.top;

    // Convert to actual image coordinates if it's scaled
    const scaleX = e.currentTarget.naturalWidth / rect.width;
    const scaleY = e.currentTarget.naturalHeight / rect.height;

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
          >
            <Move size={16} />
            Select
          </button>
          <button
            className={`${styles.toolButton} ${activeTool === 'rotate' ? styles.active : ''}`}
            onClick={() => setActiveTool('rotate')}
          >
            <RotateCw size={16} />
            Rotate
          </button>
          <button
            className={`${styles.toolButton} ${activeTool === 'zoom' ? styles.active : ''}`}
            onClick={() => setActiveTool('zoom')}
          >
            <ZoomIn size={16} />
            Zoom
          </button>
          <button
            className={`${styles.toolButton} ${activeTool === 'camera' ? styles.active : ''}`}
            onClick={() => setActiveTool('camera')}
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
          <img
            src={viewportImage}
            className={styles.viewportImage}
            style={{
              transform: `scale(${zoom / 100})`,
              cursor: activeTool === 'select' ? 'crosshair' : 'default',
            }}
            alt="Daz Viewport"
            onClick={handleImageClick}
          />
        ) : (
          <div className={styles.syncHudContainer}>
            <div className={styles.radarRing}>
              <div className={styles.radarPulse}></div>
              <FolderOpen size={36} className={styles.radarIcon} />
            </div>
            <h3 className={styles.hudTitle}>Viewport Streaming Offline</h3>
            <p className={styles.hudSubtitle}>
              Parity sync between Daz Studio and DazPilot is currently inactive.
            </p>
            <div className={styles.hudSteps}>
              <div className={styles.hudStep}>
                <span className={styles.hudStepNum}>1</span>
                <span>
                  Load <strong>VibeBridgePlugin</strong> inside Daz Studio
                </span>
              </div>
              <div className={styles.hudStep}>
                <span className={styles.hudStepNum}>2</span>
                <span>Verify Tauri port connection in Settings (Port 8765)</span>
              </div>
              <div className={styles.hudStep}>
                <span className={styles.hudStepNum}>3</span>
                <span>
                  Click the <strong>Sync Viewport</strong> overlay button below
                </span>
              </div>
            </div>
          </div>
        )}

        <div className={styles.syncOverlay}>
          <Button
            size="sm"
            variant={isSyncing ? 'primary' : 'secondary'}
            onClick={toggleSync}
            className={styles.syncButton}
          >
            {isSyncing ? 'Syncing...' : 'Sync Viewport'}
          </Button>
        </div>

        {showPoseLibrary && (
          <div className={styles.poseLibrary}>
            <div className={styles.poseHeader}>
              <span className={styles.poseTitle}>Pose Library</span>
              <button onClick={togglePoseLibrary}>×</button>
            </div>
            <div className={styles.poseList}>
              {poses.length === 0 ? (
                <p className={styles.emptyState}>No poses loaded</p>
              ) : (
                poses.map((pose) => (
                  <div key={pose.id} className={styles.poseItem}>
                    <div className={styles.poseName}>{pose.name}</div>
                    <div className={styles.poseCategory}>{pose.category}</div>
                  </div>
                ))
              )}
            </div>
          </div>
        )}

        {showLiveLink && <LiveLinkPanel onClose={() => setShowLiveLink(false)} />}
      </div>

      <div className={styles.timeline}>
        <div className={styles.controls}>
          <button className={styles.controlButton} onClick={stop}>
            <Square size={14} />
          </button>
          <button className={styles.controlButton} onClick={play}>
            <Play size={14} />
          </button>
          <button
            className={`${styles.controlButton} ${playback.isLooping ? styles.active : ''}`}
            onClick={toggleLoop}
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
