import { useState, useEffect, useRef } from 'react';
import { User, Package, Lightbulb, Eye, EyeOff, Lock, Unlock, Trash2 } from 'lucide-react';
import { useSceneStore } from '../../store';
import styles from './ScenePanel.module.css';

type TabType = 'figures' | 'props' | 'lights';

export default function ScenePanel() {
  const {
    figures,
    props,
    lights,
    selectedItem,
    selectFigure,
    selectProp,
    toggleFigureVisibility,
    toggleFigureLock,
    removeFigure,
    togglePropVisibility,
    togglePropLock,
    removeProp,
    updateLight,
    removeLight,
    loadScene,
  } = useSceneStore();

  const [activeTab, setActiveTab] = useState<TabType>('figures');
  const loadSceneRef = useRef(loadScene);

  useEffect(() => {
    loadSceneRef.current();
  }, []);

  const renderItem = (
    item: { id: string; name: string },
    type: 'figure' | 'prop',
    onSelect: (id: string) => void
  ) => {
    const isSelected = selectedItem === item.id;
    const isFigure = type === 'figure';
    const data = isFigure
      ? figures.find((f) => f.id === item.id)
      : props.find((p) => p.id === item.id);

    return (
      <div
        key={item.id}
        className={`${styles.item} ${isSelected ? styles.selected : ''}`}
        onClick={() => onSelect(item.id)}
      >
        <div className={styles.itemIcon}>
          {isFigure ? <User size={16} /> : <Package size={16} />}
        </div>
        <div className={styles.itemInfo}>
          <div className={styles.itemName}>{item.name}</div>
          <div className={styles.itemType}>{isFigure ? 'Figure' : 'Prop'}</div>
        </div>
        <div className={styles.itemActions}>
          <button
            className={styles.actionButton}
            onClick={(e) => {
              e.stopPropagation();
              isFigure ? toggleFigureVisibility(item.id) : togglePropVisibility(item.id);
            }}
          >
            {data?.visible ? <Eye size={14} /> : <EyeOff size={14} />}
          </button>
          <button
            className={styles.actionButton}
            onClick={(e) => {
              e.stopPropagation();
              isFigure ? toggleFigureLock(item.id) : togglePropLock(item.id);
            }}
          >
            {data?.locked ? <Lock size={14} /> : <Unlock size={14} />}
          </button>
          <button
            className={styles.actionButton}
            onClick={(e) => {
              e.stopPropagation();
              isFigure ? removeFigure(item.id) : removeProp(item.id);
            }}
          >
            <Trash2 size={14} />
          </button>
        </div>
      </div>
    );
  };

  return (
    <div className={styles.container}>
      <div className={styles.sidebar}>
        <div className={styles.tabs}>
          <button
            className={`${styles.tab} ${activeTab === 'figures' ? styles.active : ''}`}
            onClick={() => setActiveTab('figures')}
          >
            Figures ({figures.length})
          </button>
          <button
            className={`${styles.tab} ${activeTab === 'props' ? styles.active : ''}`}
            onClick={() => setActiveTab('props')}
          >
            Props ({props.length})
          </button>
          <button
            className={`${styles.tab} ${activeTab === 'lights' ? styles.active : ''}`}
            onClick={() => setActiveTab('lights')}
          >
            Lights ({lights.length})
          </button>
        </div>

        <div className={styles.list}>
          {activeTab === 'figures' &&
            (figures.length === 0 ? (
              <p className={styles.emptyState}>No figures in scene</p>
            ) : (
              figures.map((f) => renderItem(f, 'figure', selectFigure))
            ))}
          {activeTab === 'props' &&
            (props.length === 0 ? (
              <p className={styles.emptyState}>No props in scene</p>
            ) : (
              props.map((p) => renderItem(p, 'prop', selectProp))
            ))}
          {activeTab === 'lights' &&
            (lights.length === 0 ? (
              <p className={styles.emptyState}>No lights in scene</p>
            ) : (
              lights.map((l) => (
                <div key={l.id} className={styles.item}>
                  <div className={styles.itemIcon}>
                    <Lightbulb size={16} />
                  </div>
                  <div className={styles.itemInfo}>
                    <div className={styles.itemName}>{l.name}</div>
                    <div className={styles.itemType}>{l.type}</div>
                  </div>
                  <div className={styles.itemActions}>
                    <button
                      className={styles.actionButton}
                      onClick={() => updateLight(l.id, { enabled: !l.enabled })}
                    >
                      {l.enabled ? <Eye size={14} /> : <EyeOff size={14} />}
                    </button>
                    <button className={styles.actionButton} onClick={() => removeLight(l.id)}>
                      <Trash2 size={14} />
                    </button>
                  </div>
                </div>
              ))
            ))}
        </div>
      </div>

      <div className={styles.content}>
        {selectedItem ? (
          <div className={styles.section}>
            <h3 className={styles.sectionTitle}>Properties</h3>
            <div className={styles.property}>
              <span className={styles.propertyLabel}>Selected</span>
              <span className={styles.propertyValue}>{selectedItem}</span>
            </div>
          </div>
        ) : (
          <div className={styles.section}>
            <h3 className={styles.sectionTitle}>Scene Info</h3>
            <div className={styles.property}>
              <span className={styles.propertyLabel}>Figures</span>
              <span className={styles.propertyValue}>{figures.length}</span>
            </div>
            <div className={styles.property}>
              <span className={styles.propertyLabel}>Props</span>
              <span className={styles.propertyValue}>{props.length}</span>
            </div>
            <div className={styles.property}>
              <span className={styles.propertyLabel}>Lights</span>
              <span className={styles.propertyValue}>{lights.length}</span>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
