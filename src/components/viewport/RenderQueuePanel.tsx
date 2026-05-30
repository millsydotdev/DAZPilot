import { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { ListOrdered, Play, X, Loader2, Trash2 } from 'lucide-react';
import styles from './RenderQueuePanel.module.css';

interface QueuedRender {
  id: string;
  passName: string;
  cameraName?: string;
  status: 'queued' | 'rendering' | 'done' | 'failed';
}

export default function RenderQueuePanel() {
  const [queue, setQueue] = useState<QueuedRender[]>([]);
  const [passName, setPassName] = useState('');
  const [cameraName, setCameraName] = useState('');
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const addToQueue = async () => {
    const name = passName.trim() || `Pass ${queue.length + 1}`;
    setLoading(true);
    setError(null);
    try {
      await invoke('send_daz3d_command', {
        command: 'queue_render',
        args: {
          pass_name: name,
          ...(cameraName.trim() ? { camera_name: cameraName.trim() } : {}),
        },
      });
      setQueue((prev) => [
        ...prev,
        {
          id: `${Date.now()}-${prev.length}`,
          passName: name,
          cameraName: cameraName.trim() || undefined,
          status: 'queued',
        },
      ]);
      setPassName('');
    } catch (err) {
      setError(String(err));
    } finally {
      setLoading(false);
    }
  };

  const cancelRender = async (clearQueue = false) => {
    setLoading(true);
    setError(null);
    try {
      await invoke('send_daz3d_command', {
        command: 'cancel_render',
        args: { clear_queue: clearQueue ? 'true' : 'false' },
      });
      if (clearQueue) {
        setQueue([]);
      } else {
        setQueue((prev) => prev.slice(1));
      }
    } catch (err) {
      setError(String(err));
    } finally {
      setLoading(false);
    }
  };

  const removeItem = (id: string) => {
    setQueue((prev) => prev.filter((item) => item.id !== id));
  };

  return (
    <div className={styles.panel}>
      <div className={styles.header}>
        <ListOrdered size={16} />
        <span>Render Queue</span>
        <span className={styles.count}>{queue.length}</span>
      </div>

      <div className={styles.form}>
        <input
          className={styles.input}
          placeholder="Pass name"
          value={passName}
          onChange={(e) => setPassName(e.target.value)}
        />
        <input
          className={styles.input}
          placeholder="Camera (optional)"
          value={cameraName}
          onChange={(e) => setCameraName(e.target.value)}
        />
        <button type="button" className={styles.addBtn} onClick={addToQueue} disabled={loading}>
          {loading ? <Loader2 size={14} className={styles.spin} /> : <Play size={14} />}
          Queue
        </button>
      </div>

      {error && <p className={styles.error}>{error}</p>}

      {queue.length > 0 && (
        <div className={styles.actions}>
          <button type="button" className={styles.actionBtn} onClick={() => cancelRender(false)}>
            <X size={14} />
            Cancel Current
          </button>
          <button type="button" className={styles.actionBtn} onClick={() => cancelRender(true)}>
            <Trash2 size={14} />
            Clear Queue
          </button>
        </div>
      )}

      <ul className={styles.list}>
        {queue.length === 0 ? (
          <li className={styles.empty}>No renders queued</li>
        ) : (
          queue.map((item, index) => (
            <li key={item.id} className={styles.item}>
              <span className={styles.position}>{index + 1}</span>
              <div className={styles.itemBody}>
                <span className={styles.passName}>{item.passName}</span>
                {item.cameraName && <span className={styles.camera}>{item.cameraName}</span>}
              </div>
              <button
                type="button"
                className={styles.removeBtn}
                aria-label={`Remove ${item.passName}`}
                onClick={() => removeItem(item.id)}
              >
                <X size={12} />
              </button>
            </li>
          ))
        )}
      </ul>
    </div>
  );
}
