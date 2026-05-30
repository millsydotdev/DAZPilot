import { useState, useEffect, useRef } from 'react';
import { User, Package, Lightbulb, Eye, EyeOff, Lock, Unlock, Trash2 } from 'lucide-react';
import { useSceneStore } from '../../store';
import { PanelShell } from '../ui';
import ScenePresetsSection from './ScenePresetsSection';
import styles from './ScenePanel.module.css';

type TabType = 'figures' | 'props' | 'lights' | 'surfaces' | 'presets';

export default function ScenePanel() {
  const {
    figures,
    props,
    lights,
    selectedItem,
    nodeProperties,
    nodeMaterials,
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
    fetchNodeProperties,
    updateNodeProperty,
    fetchMaterialProperties,
    updateMaterialProperty,
    loadScene,
  } = useSceneStore();

  const [activeTab, setActiveTab] = useState<TabType>('figures');
  const loadSceneRef = useRef(loadScene);

  useEffect(() => {
    loadSceneRef.current();
  }, []);

  useEffect(() => {
    if (selectedItem) {
      fetchNodeProperties(selectedItem);
      fetchMaterialProperties(selectedItem);
    }
  }, [selectedItem, fetchNodeProperties, fetchMaterialProperties]);

  const properties = selectedItem ? nodeProperties[selectedItem] || [] : [];
  const morphs = properties.filter((p) => p.is_morph);
  const otherProps = properties.filter((p) => !p.is_morph);
  const materials = selectedItem ? nodeMaterials[selectedItem] || [] : [];

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
        role="treeitem"
        aria-selected={isSelected}
        className={`${styles.item} ${isSelected ? styles.selected : ''}`}
        onClick={() => onSelect(item.id)}
        onKeyDown={(e) => {
          if (e.key === 'Enter' || e.key === ' ') {
            onSelect(item.id);
          }
        }}
        tabIndex={0}
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
            aria-label={data?.visible ? `Hide ${item.name}` : `Show ${item.name}`}
            onClick={(e) => {
              e.stopPropagation();
              isFigure ? toggleFigureVisibility(item.id) : togglePropVisibility(item.id);
            }}
          >
            {data?.visible ? <Eye size={14} /> : <EyeOff size={14} />}
          </button>
          <button
            className={styles.actionButton}
            aria-label={data?.locked ? `Unlock ${item.name}` : `Lock ${item.name}`}
            onClick={(e) => {
              e.stopPropagation();
              isFigure ? toggleFigureLock(item.id) : togglePropLock(item.id);
            }}
          >
            {data?.locked ? <Lock size={14} /> : <Unlock size={14} />}
          </button>
          <button
            className={styles.actionButton}
            aria-label={`Delete ${item.name}`}
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
    <PanelShell title="Scene">
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
            <button
              className={`${styles.tab} ${activeTab === 'surfaces' ? styles.active : ''}`}
              onClick={() => setActiveTab('surfaces')}
            >
              Surfaces
            </button>
            <button
              className={`${styles.tab} ${activeTab === 'presets' ? styles.active : ''}`}
              onClick={() => setActiveTab('presets')}
            >
              Presets
            </button>
          </div>

          <div className={styles.list} role="tree" aria-label="Scene objects tree">
            {activeTab === 'presets' && <ScenePresetsSection />}
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
                  <div key={l.id} role="treeitem" aria-selected={false} className={styles.item}>
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
                        aria-label={l.enabled ? `Disable ${l.name}` : `Enable ${l.name}`}
                        onClick={() => updateLight(l.id, { enabled: !l.enabled })}
                      >
                        {l.enabled ? <Eye size={14} /> : <EyeOff size={14} />}
                      </button>
                      <button
                        className={styles.actionButton}
                        aria-label={`Delete ${l.name}`}
                        onClick={() => removeLight(l.id)}
                      >
                        <Trash2 size={14} />
                      </button>
                    </div>
                  </div>
                ))
              ))}
            {activeTab === 'surfaces' &&
              (!selectedItem ? (
                <p className={styles.emptyState}>Select a node to edit surfaces</p>
              ) : materials.length === 0 ? (
                <p className={styles.emptyState}>No surfaces found</p>
              ) : (
                <div className={styles.inspector}>
                  {materials.map((mat) => (
                    <div key={mat.name} className={styles.section}>
                      <h4 className={styles.subsectionTitle}>{mat.label}</h4>
                      <div className={styles.propertyList}>
                        {mat.properties.map((p) => (
                          <div key={p.name} className={styles.propertyItem}>
                            <div className={styles.propertyHeader}>
                              <span className={styles.propertyLabel}>{p.label}</span>
                              <span className={styles.propertyValue}>{p.value.toFixed(2)}</span>
                            </div>
                            <input
                              type="range"
                              className={styles.propertySlider}
                              min={p.min}
                              max={p.max}
                              step={0.01}
                              value={p.value}
                              onChange={(e) =>
                                updateMaterialProperty(
                                  selectedItem,
                                  mat.name,
                                  p.name,
                                  parseFloat(e.target.value)
                                )
                              }
                            />
                          </div>
                        ))}
                      </div>
                    </div>
                  ))}
                </div>
              ))}
          </div>
        </div>

        <div className={styles.content}>
          {selectedItem ? (
            <div className={styles.inspector}>
              <div className={styles.section}>
                <h3 className={styles.sectionTitle}>Inspector: {selectedItem}</h3>
              </div>

              {otherProps.length > 0 && (
                <div className={styles.section}>
                  <h4 className={styles.subsectionTitle}>Transforms & Properties</h4>
                  <div className={styles.propertyList}>
                    {otherProps.map((p) => (
                      <div key={p.name} className={styles.propertyItem}>
                        <div className={styles.propertyHeader}>
                          <span className={styles.propertyLabel}>{p.label}</span>
                          <span className={styles.propertyValue}>{p.value.toFixed(2)}</span>
                        </div>
                        <input
                          type="range"
                          className={styles.propertySlider}
                          min={p.min}
                          max={p.max}
                          step={0.01}
                          value={p.value}
                          onChange={(e) =>
                            updateNodeProperty(selectedItem, p.name, parseFloat(e.target.value))
                          }
                        />
                      </div>
                    ))}
                  </div>
                </div>
              )}

              {morphs.length > 0 && (
                <div className={styles.section}>
                  <h4 className={styles.subsectionTitle}>Morphs</h4>
                  <div className={styles.propertyList}>
                    {morphs.map((p) => (
                      <div key={p.name} className={styles.propertyItem}>
                        <div className={styles.propertyHeader}>
                          <span className={styles.propertyLabel}>{p.label}</span>
                          <span className={styles.propertyValue}>{p.value.toFixed(2)}</span>
                        </div>
                        <input
                          type="range"
                          className={styles.propertySlider}
                          min={p.min}
                          max={p.max}
                          step={0.01}
                          value={p.value}
                          onChange={(e) =>
                            updateNodeProperty(selectedItem, p.name, parseFloat(e.target.value))
                          }
                        />
                      </div>
                    ))}
                  </div>
                </div>
              )}
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
    </PanelShell>
  );
}
