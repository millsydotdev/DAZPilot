import { Play, Pause, SkipBack, Repeat, Rewind, FastForward } from 'lucide-react';
import { useViewportStore } from '../../../store';
import styles from './Timeline.module.css';

export function Timeline() {
  const { timeline, playback, seek, play, pause, stop, toggleLoop } = useViewportStore();
  const { currentFrame, totalFrames, fps, duration } = timeline;
  const { isPlaying, isLooping, playbackSpeed } = playback;

  const progress = (currentFrame / totalFrames) * 100;
  const currentTime = currentFrame / fps;

  const handleScrub = (e: React.MouseEvent<HTMLDivElement>) => {
    const rect = e.currentTarget.getBoundingClientRect();
    const x = e.clientX - rect.left;
    const percent = x / rect.width;
    const frame = Math.round(percent * totalFrames);
    seek(frame);
  };

  return (
    <div className={styles.timeline}>
      <div className={styles.transport}>
        <button className={styles.transportBtn} onClick={stop} title="Go to start">
          <SkipBack size={16} />
        </button>
        <button
          className={styles.transportBtn}
          onClick={() => seek(Math.max(0, currentFrame - 10))}
          title="Step back"
        >
          <Rewind size={16} />
        </button>
        {isPlaying ? (
          <button
            className={styles.transportBtn + ' ' + styles.playBtn}
            onClick={pause}
            title="Pause"
          >
            <Pause size={18} />
          </button>
        ) : (
          <button
            className={styles.transportBtn + ' ' + styles.playBtn}
            onClick={play}
            title="Play"
          >
            <Play size={18} />
          </button>
        )}
        <button
          className={styles.transportBtn}
          onClick={() => seek(Math.min(totalFrames, currentFrame + 10))}
          title="Step forward"
        >
          <FastForward size={16} />
        </button>
        <button
          className={styles.transportBtn}
          onClick={toggleLoop}
          title={isLooping ? 'Disable loop' : 'Enable loop'}
        >
          <Repeat size={16} className={isLooping ? styles.active : ''} />
        </button>
      </div>

      <div
        className={styles.scrubberContainer}
        onClick={handleScrub}
        role="slider"
        aria-label="Timeline scrubber"
        aria-valuemin={0}
        aria-valuemax={totalFrames}
        aria-valuenow={currentFrame}
        tabIndex={0}
        onKeyDown={(e) => {
          if (e.key === 'ArrowRight') seek(Math.min(totalFrames, currentFrame + 1));
          if (e.key === 'ArrowLeft') seek(Math.max(0, currentFrame - 1));
          if (e.key === 'Home') seek(0);
          if (e.key === 'End') seek(totalFrames);
        }}
      >
        <div className={styles.scrubberTrack}>
          <div className={styles.scrubberProgress} style={{ width: `${progress}%` }} />
          <div className={styles.scrubberHandle} style={{ left: `${progress}%` }} />
        </div>
        <div className={styles.frameMarkers}>
          {[0, 0.25, 0.5, 0.75, 1].map((pct) => (
            <span key={pct} className={styles.marker} style={{ left: `${pct * 100}%` }}>
              {Math.round(pct * totalFrames)}
            </span>
          ))}
        </div>
      </div>

      <div className={styles.timeDisplay}>
        <span className={styles.currentTime}>{currentTime.toFixed(2)}s</span>
        <span className={styles.separator}>/</span>
        <span className={styles.duration}>{duration.toFixed(2)}s</span>
        <span className={styles.fps}>{fps} fps</span>
        <select
          className={styles.speedSelect}
          value={playbackSpeed}
          onChange={(e) => useViewportStore.getState().setPlaybackSpeed(Number(e.target.value))}
        >
          <option value={0.25}>0.25x</option>
          <option value={0.5}>0.5x</option>
          <option value={1}>1x</option>
          <option value={2}>2x</option>
        </select>
      </div>
    </div>
  );
}
