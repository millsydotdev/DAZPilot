import { Wifi, WifiOff, Cpu, Gauge, User } from 'lucide-react';
import { useConnectionStore, useSceneStore, useViewportStore } from '../../../store';
import styles from './StatusBar.module.css';

export function StatusBar() {
  const status = useConnectionStore((s) => s.status);
  const aiModel = useConnectionStore((s) => s.aiModel);
  const activeFigure = useSceneStore((s) => s.figures.find((f) => f.selected) ?? null);
  const fps = useViewportStore((s) => s.timeline.fps);

  return (
    <div className={styles.statusBar}>
      <div className={styles.left}>
        <div className={styles.item}>
          {status === 'connected' ? (
            <Wifi size={14} className={styles.connected} />
          ) : (
            <WifiOff size={14} className={styles.disconnected} />
          )}
          <span>{status === 'connected' ? 'Connected' : 'Offline'}</span>
        </div>
        {aiModel.loaded && (
          <div className={styles.item}>
            <Cpu size={14} />
            <span>{aiModel.name}</span>
          </div>
        )}
      </div>
      <div className={styles.right}>
        {activeFigure && (
          <div className={styles.item}>
            <User size={14} />
            <span>{activeFigure.name}</span>
          </div>
        )}
        {fps > 0 && (
          <div className={styles.item}>
            <Gauge size={14} />
            <span>{fps} fps</span>
          </div>
        )}
      </div>
    </div>
  );
}
